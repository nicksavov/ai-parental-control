package main

import (
	"bytes"
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"
)

func newTestServer(t *testing.T) *httptest.Server {
	t.Helper()
	return httptest.NewServer(newServer([]byte("test-secret"), newMemStore(), newMemNotifier()).routes())
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

type paired struct {
	parentAccess  string
	deviceAccess  string
	recipientID   string
	childDeviceID string
}

// setup registers a parent, creates a child, issues and claims a pairing code,
// and exchanges the device credential for a device access token.
func setup(t *testing.T, srv *httptest.Server) paired {
	t.Helper()
	_, data := do(t, "POST", srv.URL+"/v1/auth/register", "", map[string]any{
		"email": "mom@example.com", "password": "correct horse",
	})
	parentAccess := decodeMap(t, data)["accessToken"].(string)

	_, data = do(t, "POST", srv.URL+"/v1/family/children", parentAccess, nil)
	childID := decodeMap(t, data)["childId"].(string)

	_, data = do(t, "POST", srv.URL+"/v1/pairing/codes", parentAccess, map[string]any{"childId": childID})
	codeResp := decodeMap(t, data)
	code := codeResp["code"].(string)
	recipientID := codeResp["recipientId"].(string)

	_, data = do(t, "POST", srv.URL+"/v1/pairing/claim", "", map[string]any{
		"code": code, "devicePublicKey": "pk", "platform": "android", "capabilities": []string{"filtering"},
	})
	claim := decodeMap(t, data)
	childDeviceID := claim["childDeviceId"].(string)
	deviceCredential := claim["deviceCredential"].(string)

	// Exchange the device refresh credential for an access token.
	_, data = do(t, "POST", srv.URL+"/v1/auth/token", "", map[string]any{
		"grantType": "refresh", "refreshToken": deviceCredential,
	})
	deviceAccess := decodeMap(t, data)["accessToken"].(string)

	return paired{parentAccess, deviceAccess, recipientID, childDeviceID}
}

func sampleEnvelope(p paired) []byte {
	raw, _ := json.Marshal(map[string]any{
		"v": 1, "id": "alert-1", "childDeviceId": p.childDeviceID, "recipientId": p.recipientID,
		"createdAt": "2026-06-26T00:00:00Z", "ciphertext": "Y2lwaGVydGV4dA", "nonce": "bm9uY2U",
	})
	return raw
}

func TestPairingAndAlertRelay(t *testing.T) {
	srv := newTestServer(t)
	defer srv.Close()
	p := setup(t, srv)

	resp, _ := do(t, "POST", srv.URL+"/v1/alerts", p.deviceAccess, sampleEnvelope(p))
	if resp.StatusCode != http.StatusAccepted {
		t.Fatalf("submit alert: want 202, got %d", resp.StatusCode)
	}

	resp, data := do(t, "GET", srv.URL+"/v1/alerts/stream", p.parentAccess, nil)
	if resp.StatusCode != 200 {
		t.Fatalf("stream: %d %s", resp.StatusCode, data)
	}
	var pulled []map[string]any
	if err := json.Unmarshal(data, &pulled); err != nil {
		t.Fatal(err)
	}
	if len(pulled) != 1 || pulled[0]["ciphertext"] != "Y2lwaGVydGV4dA" || pulled[0]["id"] != "alert-1" {
		t.Fatalf("relayed envelope was altered or missing: %v", pulled)
	}

	// Drained on the second pull.
	_, data = do(t, "GET", srv.URL+"/v1/alerts/stream", p.parentAccess, nil)
	if strings.TrimSpace(string(data)) != "[]" {
		t.Fatalf("inbox should be drained, got %s", data)
	}
}

func TestDeviceCannotSpoofRecipient(t *testing.T) {
	srv := newTestServer(t)
	defer srv.Close()
	p := setup(t, srv)

	raw, _ := json.Marshal(map[string]any{
		"v": 1, "id": "x", "childDeviceId": p.childDeviceID, "recipientId": "someone-else",
		"createdAt": "2026-06-26T00:00:00Z", "ciphertext": "x", "nonce": "y",
	})
	resp, _ := do(t, "POST", srv.URL+"/v1/alerts", p.deviceAccess, raw)
	if resp.StatusCode != http.StatusForbidden {
		t.Fatalf("want 403 for spoofed recipient, got %d", resp.StatusCode)
	}
}

func TestAuthRejectsForgedAndMissingTokens(t *testing.T) {
	srv := newTestServer(t)
	defer srv.Close()
	for _, tok := range []string{"", "forged.token.here", "a.b.c"} {
		resp, _ := do(t, "POST", srv.URL+"/v1/family/children", tok, nil)
		if resp.StatusCode != http.StatusUnauthorized {
			t.Fatalf("want 401 for token %q, got %d", tok, resp.StatusCode)
		}
	}
}

func TestPasswordLoginAndRefresh(t *testing.T) {
	srv := newTestServer(t)
	defer srv.Close()

	do(t, "POST", srv.URL+"/v1/auth/register", "", map[string]any{"email": "dad@example.com", "password": "hunter2hunter"})

	// Wrong password is rejected.
	resp, _ := do(t, "POST", srv.URL+"/v1/auth/token", "", map[string]any{
		"grantType": "password", "email": "dad@example.com", "password": "wrong",
	})
	if resp.StatusCode != http.StatusUnauthorized {
		t.Fatalf("want 401 for wrong password, got %d", resp.StatusCode)
	}

	// Correct password yields tokens; the refresh token mints an access token.
	_, data := do(t, "POST", srv.URL+"/v1/auth/token", "", map[string]any{
		"grantType": "password", "email": "dad@example.com", "password": "hunter2hunter",
	})
	refresh := decodeMap(t, data)["refreshToken"].(string)

	_, data = do(t, "POST", srv.URL+"/v1/auth/token", "", map[string]any{
		"grantType": "refresh", "refreshToken": refresh,
	})
	access := decodeMap(t, data)["accessToken"].(string)
	resp, _ = do(t, "POST", srv.URL+"/v1/family/children", access, nil)
	if resp.StatusCode != http.StatusCreated {
		t.Fatalf("refreshed access token should work, got %d", resp.StatusCode)
	}
}

func TestLongPollDeliversAlert(t *testing.T) {
	srv := newTestServer(t)
	defer srv.Close()
	p := setup(t, srv)

	ch := make(chan []byte, 1)
	go func() {
		req, _ := http.NewRequest("GET", srv.URL+"/v1/alerts/stream?wait=1", nil)
		req.Header.Set("Authorization", "Bearer "+p.parentAccess)
		resp, err := http.DefaultClient.Do(req)
		if err != nil {
			ch <- nil
			return
		}
		defer resp.Body.Close()
		data, _ := io.ReadAll(resp.Body)
		ch <- data
	}()

	time.Sleep(200 * time.Millisecond) // let the long poll start waiting
	resp, _ := do(t, "POST", srv.URL+"/v1/alerts", p.deviceAccess, sampleEnvelope(p))
	if resp.StatusCode != http.StatusAccepted {
		t.Fatalf("submit: %d", resp.StatusCode)
	}

	select {
	case data := <-ch:
		if !strings.Contains(string(data), "alert-1") {
			t.Fatalf("long poll did not deliver the alert: %s", data)
		}
	case <-time.After(5 * time.Second):
		t.Fatal("long poll timed out")
	}
}
