package main

import (
	"crypto/hmac"
	"crypto/sha256"
	"encoding/base64"
	"encoding/json"
	"errors"
	"strings"
	"time"
)

// Compact HS256 JWTs, stdlib only. Real signed tokens with expiry and a refresh
// flow. The payload is "role:subject" semantics carried in typed claims.

const (
	accessTTL  = 15 * time.Minute
	refreshTTL = 30 * 24 * time.Hour
)

var (
	errInvalidToken = errors.New("invalid token")
	errExpiredToken = errors.New("expired token")
)

type claims struct {
	Sub  string `json:"sub"`
	Role string `json:"role"` // "parent" or "device"
	Typ  string `json:"typ"`  // "access" or "refresh"
	Iat  int64  `json:"iat"`
	Exp  int64  `json:"exp"`
}

func b64(b []byte) string { return base64.RawURLEncoding.EncodeToString(b) }

func (s *server) mac(signing string) []byte {
	m := hmac.New(sha256.New, s.secret)
	m.Write([]byte(signing))
	return m.Sum(nil)
}

func (s *server) issue(sub, role, typ string, ttl time.Duration, now time.Time) string {
	c := claims{Sub: sub, Role: role, Typ: typ, Iat: now.Unix(), Exp: now.Add(ttl).Unix()}
	payload, _ := json.Marshal(c)
	header := b64([]byte(`{"alg":"HS256","typ":"JWT"}`))
	signing := header + "." + b64(payload)
	return signing + "." + b64(s.mac(signing))
}

func (s *server) issueAccess(sub, role string) string {
	return s.issue(sub, role, "access", accessTTL, time.Now())
}

func (s *server) issueRefresh(sub, role string) string {
	return s.issue(sub, role, "refresh", refreshTTL, time.Now())
}

func (s *server) parse(token string) (*claims, error) {
	parts := strings.Split(token, ".")
	if len(parts) != 3 {
		return nil, errInvalidToken
	}
	signing := parts[0] + "." + parts[1]
	sig, err := base64.RawURLEncoding.DecodeString(parts[2])
	if err != nil || !hmac.Equal(sig, s.mac(signing)) {
		return nil, errInvalidToken
	}
	payload, err := base64.RawURLEncoding.DecodeString(parts[1])
	if err != nil {
		return nil, errInvalidToken
	}
	var c claims
	if err := json.Unmarshal(payload, &c); err != nil {
		return nil, errInvalidToken
	}
	if time.Now().Unix() >= c.Exp {
		return nil, errExpiredToken
	}
	return &c, nil
}
