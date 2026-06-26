package main

import (
	"bytes"
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

func newTestServer(t *testing.T) *httptest.Server {
	t.Helper()
	return httptest.NewServer(newServer([]byte("test-secret"), newStore()).routes())
}

func do(t *testing.T, method, url, token string, body any) (*http.Response, []byte) {
	t.Helper()
	var r io.Reader
	switch b := body.(type) {
	case nil:
		r = nil
	case []byte:
		r = bytes.NewReader(b)
	default:
		raw, err := json.Marshal(b)
		if err != nil {
			t.Fatal(err)
		}
		r = bytes.NewReader(raw)
	}
	req, err := http.NewRequest(method, url, r)
	if err != nil {
		t.Fatal(err)
	}
	if token != "" {
		req.Header.Set("Authorization", "Bearer "+token)
	}
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	data, _ := io.ReadAll(resp.Body)
	return resp, data
}

func decodeMap(t *testing.T, data []byte) map[string]any {
	t.Helper()
	var m map[string]any
	if err := json.Unmarshal(data, &m); err != nil {
		t.Fatalf("decode %q: %v", data, err)
	}
	return m
}

// Walks the whole flow: parent signs in, creates a child, issues a pairing code,
// the child claims it, submits an opaque encrypted envelope, and the parent
// pulls it back unchanged.
func TestPairingAndAlertRelay(t *testing.T) {
	srv := newTestServer(t)
	defer srv.Close()

	resp, data := do(t, "POST", srv.URL+"/v1/auth/token", "", map[string]any{
		"grantType": "password", "email": "mom@example.com", "password": "x",
	})
	if resp.StatusCode != 200 {
		t.Fatalf("token: %d %s", resp.StatusCode, data)
	}
	parentTok := decodeMap(t, data)["accessToken"].(string)

	_, data = do(t, "POST", srv.URL+"/v1/family/children", parentTok, nil)
	childID := decodeMap(t, data)["childId"].(string)

	_, data = do(t, "POST", srv.URL+"/v1/pairing/codes", parentTok, map[string]any{"childId": childID})
	codeResp := decodeMap(t, data)
	code := codeResp["code"].(string)
	recipientID := codeResp["recipientId"].(string)

	_, data = do(t, "POST", srv.URL+"/v1/pairing/claim", "", map[string]any{
		"code": code, "devicePublicKey": "pk", "platform": "android", "capabilities": []string{"filtering"},
	})
	claim := decodeMap(t, data)
	childDeviceID := claim["childDeviceId"].(string)
	deviceCred := claim["deviceCredential"].(string)

	// The child encrypts an alert locally; the backend only ever sees this.
	envelope := map[string]any{
		"v": 1, "id": "alert-1", "childDeviceId": childDeviceID, "recipientId": recipientID,
		"createdAt": "2026-06-26T00:00:00Z",
		"ciphertext": "Y2lwaGVydGV4dA", "nonce": "bm9uY2U",
	}
	envBytes, _ := json.Marshal(envelope)

	resp, _ = do(t, "POST", srv.URL+"/v1/alerts", deviceCred, envBytes)
	if resp.StatusCode != http.StatusAccepted {
		t.Fatalf("submit alert: want 202, got %d", resp.StatusCode)
	}

	resp, data = do(t, "GET", srv.URL+"/v1/alerts/stream", parentTok, nil)
	if resp.StatusCode != 200 {
		t.Fatalf("stream: %d %s", resp.StatusCode, data)
	}
	var pulled []map[string]any
	if err := json.Unmarshal(data, &pulled); err != nil {
		t.Fatal(err)
	}
	if len(pulled) != 1 {
		t.Fatalf("want 1 relayed envelope, got %d", len(pulled))
	}
	if pulled[0]["ciphertext"] != "Y2lwaGVydGV4dA" || pulled[0]["id"] != "alert-1" {
		t.Fatalf("relayed envelope was altered: %v", pulled[0])
	}

	// The relay must be faithful and content-blind: no plaintext exists, and the
	// ciphertext came through byte-identical.
	if !strings.Contains(string(data), "Y2lwaGVydGV4dA") {
		t.Fatal("ciphertext did not round-trip through the relay")
	}

	// Second pull is empty (drained).
	_, data = do(t, "GET", srv.URL+"/v1/alerts/stream", parentTok, nil)
	if strings.TrimSpace(string(data)) != "[]" {
		t.Fatalf("inbox should be drained, got %s", data)
	}
}

func TestDeviceCannotSpoofRecipient(t *testing.T) {
	srv := newTestServer(t)
	defer srv.Close()

	_, data := do(t, "POST", srv.URL+"/v1/auth/token", "", map[string]any{"email": "dad@example.com"})
	parentTok := decodeMap(t, data)["accessToken"].(string)
	_, data = do(t, "POST", srv.URL+"/v1/family/children", parentTok, nil)
	childID := decodeMap(t, data)["childId"].(string)
	_, data = do(t, "POST", srv.URL+"/v1/pairing/codes", parentTok, map[string]any{"childId": childID})
	code := decodeMap(t, data)["code"].(string)
	_, data = do(t, "POST", srv.URL+"/v1/pairing/claim", "", map[string]any{"code": code, "platform": "android"})
	claim := decodeMap(t, data)
	childDeviceID := claim["childDeviceId"].(string)
	deviceCred := claim["deviceCredential"].(string)

	// Address the envelope to a parent the device is not paired with.
	envelope := map[string]any{
		"v": 1, "id": "x", "childDeviceId": childDeviceID, "recipientId": "someone-else",
		"createdAt": "2026-06-26T00:00:00Z", "ciphertext": "x", "nonce": "y",
	}
	envBytes, _ := json.Marshal(envelope)
	resp, _ := do(t, "POST", srv.URL+"/v1/alerts", deviceCred, envBytes)
	if resp.StatusCode != http.StatusForbidden {
		t.Fatalf("want 403 for spoofed recipient, got %d", resp.StatusCode)
	}
}

func TestUnauthenticatedIsRejected(t *testing.T) {
	srv := newTestServer(t)
	defer srv.Close()
	resp, _ := do(t, "POST", srv.URL+"/v1/family/children", "", nil)
	if resp.StatusCode != http.StatusUnauthorized {
		t.Fatalf("want 401, got %d", resp.StatusCode)
	}
	resp, _ = do(t, "POST", srv.URL+"/v1/family/children", "forged.token", nil)
	if resp.StatusCode != http.StatusUnauthorized {
		t.Fatalf("want 401 for forged token, got %d", resp.StatusCode)
	}
}
