package main

import (
	"crypto/hmac"
	"crypto/sha256"
	"encoding/base64"
	"net/http"
	"strings"
)

// Minimal HMAC-signed bearer tokens, stdlib only. This is real integrity (a
// client cannot forge a token without the secret) but it is a scaffold: it has
// no expiry, rotation, or password verification. Real auth (JWT access plus
// rotating refresh tokens, hashed passwords) replaces this. The token payload is
// "role:subject", for example "parent:mom@example.com" or "device:device-abc".

func (s *server) sign(role, subject string) string {
	payload := role + ":" + subject
	mac := hmac.New(sha256.New, s.secret)
	mac.Write([]byte(payload))
	sig := mac.Sum(nil)
	return base64.RawURLEncoding.EncodeToString([]byte(payload)) + "." +
		base64.RawURLEncoding.EncodeToString(sig)
}

func (s *server) verify(token string) (role, subject string, ok bool) {
	parts := strings.SplitN(token, ".", 2)
	if len(parts) != 2 {
		return "", "", false
	}
	payload, err := base64.RawURLEncoding.DecodeString(parts[0])
	if err != nil {
		return "", "", false
	}
	sig, err := base64.RawURLEncoding.DecodeString(parts[1])
	if err != nil {
		return "", "", false
	}
	mac := hmac.New(sha256.New, s.secret)
	mac.Write(payload)
	if !hmac.Equal(sig, mac.Sum(nil)) {
		return "", "", false
	}
	pp := strings.SplitN(string(payload), ":", 2)
	if len(pp) != 2 {
		return "", "", false
	}
	return pp[0], pp[1], true
}

func bearer(r *http.Request) string {
	h := r.Header.Get("Authorization")
	if after, found := strings.CutPrefix(h, "Bearer "); found {
		return after
	}
	return ""
}

// parentHandler is a handler that requires a valid parent token.
type parentHandler func(w http.ResponseWriter, r *http.Request, parentID string)

func (s *server) requireParent(h parentHandler) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		role, subject, ok := s.verify(bearer(r))
		if !ok || role != "parent" {
			writeError(w, http.StatusUnauthorized, "unauthorized")
			return
		}
		h(w, r, subject)
	}
}

// deviceHandler is a handler that requires a valid device credential.
type deviceHandler func(w http.ResponseWriter, r *http.Request, childDeviceID string)

func (s *server) requireDevice(h deviceHandler) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		role, subject, ok := s.verify(bearer(r))
		if !ok || role != "device" {
			writeError(w, http.StatusUnauthorized, "unauthorized")
			return
		}
		h(w, r, subject)
	}
}
