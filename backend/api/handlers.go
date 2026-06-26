package main

import (
	"context"
	"encoding/json"
	"errors"
	"io"
	"net/http"
	"time"
)

const (
	pairingCodeTTL = 10 * time.Minute
	streamWait     = 25 * time.Second
)

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

// POST /v1/auth/register
func (s *server) handleRegister(w http.ResponseWriter, r *http.Request) {
	var req struct {
		Email    string `json:"email"`
		Password string `json:"password"`
	}
	if err := decode(r, &req); err != nil || req.Email == "" {
		writeError(w, http.StatusBadRequest, "email and password required")
		return
	}
	if len(req.Password) < 8 {
		writeError(w, http.StatusBadRequest, errWeakPassword.Error())
		return
	}
	hash, err := hashPassword(req.Password)
	if err != nil {
		writeError(w, http.StatusInternalServerError, "could not hash password")
		return
	}
	id, err := s.store.CreateUser(r.Context(), req.Email, hash)
	if errors.Is(err, errConflict) {
		writeError(w, http.StatusConflict, "email already registered")
		return
	}
	if err != nil {
		writeError(w, http.StatusInternalServerError, "could not create user")
		return
	}
	s.writeTokens(w, id, "parent")
}

// POST /v1/auth/token (password or refresh grant)
func (s *server) handleToken(w http.ResponseWriter, r *http.Request) {
	var req struct {
		GrantType    string `json:"grantType"`
		Email        string `json:"email"`
		Password     string `json:"password"`
		RefreshToken string `json:"refreshToken"`
	}
	if err := decode(r, &req); err != nil {
		writeError(w, http.StatusBadRequest, "bad request")
		return
	}
	switch req.GrantType {
	case "password":
		u, err := s.store.UserByEmail(r.Context(), req.Email)
		if err != nil || !verifyPassword(req.Password, u.PasswordHash) {
			writeError(w, http.StatusUnauthorized, "invalid credentials")
			return
		}
		s.writeTokens(w, u.ID, "parent")
	case "refresh":
		c, err := s.parse(req.RefreshToken)
		if err != nil || c.Typ != "refresh" {
			writeError(w, http.StatusUnauthorized, "invalid refresh token")
			return
		}
		writeJSON(w, http.StatusOK, map[string]any{
			"accessToken": s.issueAccess(c.Sub, c.Role),
			"tokenType":   "Bearer",
			"expiresIn":   int(accessTTL.Seconds()),
		})
	default:
		writeError(w, http.StatusBadRequest, "unsupported grantType")
	}
}

func (s *server) writeTokens(w http.ResponseWriter, sub, role string) {
	writeJSON(w, http.StatusOK, map[string]any{
		"accessToken":  s.issueAccess(sub, role),
		"refreshToken": s.issueRefresh(sub, role),
		"tokenType":    "Bearer",
		"expiresIn":    int(accessTTL.Seconds()),
	})
}

// POST /v1/family/children
func (s *server) handleCreateChild(w http.ResponseWriter, r *http.Request, parentID string) {
	id, err := s.store.CreateChild(r.Context(), parentID)
	if err != nil {
		writeError(w, http.StatusInternalServerError, "could not create child")
		return
	}
	writeJSON(w, http.StatusCreated, map[string]string{"childId": id})
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
	pc, err := s.store.CreatePairingCode(r.Context(), parentID, req.ChildID, time.Now().Add(pairingCodeTTL))
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
	d, err := s.store.ClaimCode(r.Context(), req.Code, req.DevicePubKey, req.Platform, req.Capabilities, time.Now())
	if err != nil {
		writeError(w, http.StatusBadRequest, "invalid or expired code")
		return
	}
	// The device credential is a long-lived refresh token; the agent exchanges
	// it for short-lived access tokens via /v1/auth/token.
	writeJSON(w, http.StatusOK, map[string]string{
		"childDeviceId":    d.ChildDeviceID,
		"deviceCredential": s.issueRefresh(d.ChildDeviceID, "device"),
	})
}

// PUT /v1/devices/{id}/policy
func (s *server) handlePutPolicy(w http.ResponseWriter, r *http.Request, parentID string) {
	id := r.PathValue("id")
	body, err := io.ReadAll(io.LimitReader(r.Body, 1<<20))
	if err != nil || !json.Valid(body) {
		writeError(w, http.StatusBadRequest, "policy must be JSON")
		return
	}
	switch err := s.store.SetPolicy(r.Context(), parentID, id, body); {
	case err == nil:
		w.WriteHeader(http.StatusNoContent)
	case errors.Is(err, errForbidden):
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
	policy, err := s.store.GetPolicy(r.Context(), childDeviceID)
	if errors.Is(err, errNotFound) {
		writeJSON(w, http.StatusOK, map[string]any{})
		return
	}
	if err != nil {
		writeError(w, http.StatusInternalServerError, "could not load policy")
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
	if route.ChildDeviceID != childDeviceID {
		writeError(w, http.StatusForbidden, "envelope childDeviceId does not match device")
		return
	}
	d, err := s.store.DeviceByID(r.Context(), childDeviceID)
	if err != nil || route.RecipientID != d.RecipientID {
		writeError(w, http.StatusForbidden, "envelope recipient is not the paired parent")
		return
	}
	if err := s.store.EnqueueEnvelope(r.Context(), route.RecipientID, body); err != nil {
		writeError(w, http.StatusInternalServerError, "could not queue")
		return
	}
	s.notifier.Notify(r.Context(), route.RecipientID)
	w.WriteHeader(http.StatusAccepted)
}

// GET /v1/alerts/stream (parent pulls queued envelopes; ?wait=1 long-polls)
func (s *server) handleStreamAlerts(w http.ResponseWriter, r *http.Request, parentID string) {
	ctx := r.Context()
	raw, err := s.store.DrainInbox(ctx, parentID)
	if err != nil {
		writeError(w, http.StatusInternalServerError, "could not read inbox")
		return
	}
	if len(raw) == 0 && r.URL.Query().Get("wait") == "1" {
		wctx, cancel := context.WithTimeout(ctx, streamWait)
		defer cancel()
		s.notifier.Wait(wctx, parentID)
		raw, _ = s.store.DrainInbox(ctx, parentID)
	}
	out := make([]json.RawMessage, 0, len(raw))
	for _, e := range raw {
		out = append(out, json.RawMessage(e))
	}
	writeJSON(w, http.StatusOK, out)
}
