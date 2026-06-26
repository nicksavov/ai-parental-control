package main

import (
	"encoding/json"
	"io"
	"net/http"
	"time"
)

const pairingCodeTTL = 10 * time.Minute

// envelopeRouting is the ONLY part of an alert envelope the backend reads. The
// ciphertext and nonce are never parsed here; the full body is relayed as-is.
type envelopeRouting struct {
	V             int    `json:"v"`
	ID            string `json:"id"`
	ChildDeviceID string `json:"childDeviceId"`
	RecipientID   string `json:"recipientId"`
	CreatedAt     string `json:"createdAt"`
}

func decode(r *http.Request, v any) error {
	return json.NewDecoder(io.LimitReader(r.Body, 1<<20)).Decode(v)
}

// POST /v1/auth/token
// Scaffold: password grant accepts any credentials and treats the email as the
// parent id. Real auth verifies a hashed password and issues refresh tokens.
func (s *server) handleToken(w http.ResponseWriter, r *http.Request) {
	var req struct {
		GrantType string `json:"grantType"`
		Email     string `json:"email"`
		Password  string `json:"password"`
	}
	if err := decode(r, &req); err != nil || req.Email == "" {
		writeError(w, http.StatusBadRequest, "email required")
		return
	}
	writeJSON(w, http.StatusOK, map[string]any{
		"accessToken": s.sign("parent", req.Email),
		"tokenType":   "Bearer",
	})
}

// POST /v1/family/children
func (s *server) handleCreateChild(w http.ResponseWriter, _ *http.Request, parentID string) {
	writeJSON(w, http.StatusCreated, map[string]string{"childId": s.store.createChild(parentID)})
}

// POST /v1/pairing/codes
func (s *server) handleCreateCode(w http.ResponseWriter, r *http.Request, parentID string) {
	var req struct {
		ChildID string `json:"childId"`
	}
	if err := decode(r, &req); err != nil || req.ChildID == "" {
		writeError(w, http.StatusBadRequest, "childId required")
		return
	}
	pc, err := s.store.createPairingCode(parentID, req.ChildID, pairingCodeTTL)
	if err != nil {
		writeError(w, http.StatusForbidden, "not your child")
		return
	}
	writeJSON(w, http.StatusCreated, map[string]any{
		"code":        pc.Code,
		"recipientId": pc.RecipientID,
		"expiresAt":   pc.ExpiresAt.UTC().Format(time.RFC3339),
	})
}

// POST /v1/pairing/claim (unauthenticated; the code is the credential)
func (s *server) handleClaim(w http.ResponseWriter, r *http.Request) {
	var req struct {
		Code         string   `json:"code"`
		DevicePubKey string   `json:"devicePublicKey"`
		Platform     string   `json:"platform"`
		Capabilities []string `json:"capabilities"`
		PushToken    string   `json:"pushToken"`
	}
	if err := decode(r, &req); err != nil || req.Code == "" {
		writeError(w, http.StatusBadRequest, "code required")
		return
	}
	d, err := s.store.claimCode(req.Code, req.DevicePubKey, req.Platform, req.Capabilities)
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid or expired code")
		return
	}
	writeJSON(w, http.StatusOK, map[string]string{
		"childDeviceId":    d.ChildDeviceID,
		"deviceCredential": s.sign("device", d.ChildDeviceID),
	})
}

// PUT /v1/devices/{id}/policy
func (s *server) handlePutPolicy(w http.ResponseWriter, r *http.Request, parentID string) {
	id := r.PathValue("id")
	body, err := io.ReadAll(io.LimitReader(r.Body, 1<<20))
	if err != nil {
		writeError(w, http.StatusBadRequest, "bad body")
		return
	}
	if !json.Valid(body) {
		writeError(w, http.StatusBadRequest, "policy must be JSON")
		return
	}
	switch err := s.store.setPolicy(parentID, id, body); err {
	case nil:
		w.WriteHeader(http.StatusNoContent)
	case errForbidden:
		writeError(w, http.StatusForbidden, "not your device")
	default:
		writeError(w, http.StatusNotFound, "device not found")
	}
}

// GET /v1/devices/{id}/policy (device fetches its own policy)
func (s *server) handleGetPolicy(w http.ResponseWriter, r *http.Request, childDeviceID string) {
	if r.PathValue("id") != childDeviceID {
		writeError(w, http.StatusForbidden, "a device may only read its own policy")
		return
	}
	policy, ok := s.store.getPolicy(childDeviceID)
	if !ok {
		writeJSON(w, http.StatusOK, map[string]any{})
		return
	}
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write(policy)
}

// POST /v1/alerts (device submits an opaque encrypted envelope)
func (s *server) handleSubmitAlert(w http.ResponseWriter, r *http.Request, childDeviceID string) {
	body, err := io.ReadAll(io.LimitReader(r.Body, 1<<20))
	if err != nil {
		writeError(w, http.StatusBadRequest, "bad body")
		return
	}
	var route envelopeRouting
	if err := json.Unmarshal(body, &route); err != nil {
		writeError(w, http.StatusBadRequest, "malformed envelope")
		return
	}
	// A device may only submit envelopes for itself, addressed to the parent it
	// is paired with. This prevents a compromised device from spoofing others or
	// fanning out to arbitrary recipients.
	if route.ChildDeviceID != childDeviceID {
		writeError(w, http.StatusForbidden, "envelope childDeviceId does not match device")
		return
	}
	d, ok := s.store.deviceByID(childDeviceID)
	if !ok || route.RecipientID != d.RecipientID {
		writeError(w, http.StatusForbidden, "envelope recipient is not the paired parent")
		return
	}
	s.store.enqueueEnvelope(route.RecipientID, body)
	w.WriteHeader(http.StatusAccepted)
}

// GET /v1/alerts/stream (parent pulls queued envelopes; WebSocket later)
func (s *server) handleStreamAlerts(w http.ResponseWriter, _ *http.Request, parentID string) {
	raw := s.store.drainInbox(parentID)
	out := make([]json.RawMessage, 0, len(raw))
	for _, e := range raw {
		out = append(out, json.RawMessage(e))
	}
	writeJSON(w, http.StatusOK, out)
}
