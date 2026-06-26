package main

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"errors"
	"time"
)

// Store is the persistence boundary. The in-memory implementation (memStore)
// backs unit tests and single-node dev; the Postgres implementation (pgStore)
// backs real deployments. Neither can read an alert: envelopes are stored and
// returned as opaque bytes.

var (
	errNotFound  = errors.New("not found")
	errForbidden = errors.New("forbidden")
	errBadCode   = errors.New("invalid or expired pairing code")
	errConflict  = errors.New("already exists")
)

type user struct {
	ID           string
	Email        string
	PasswordHash string
}

type device struct {
	ChildDeviceID string
	ChildID       string
	RecipientID   string
	Platform      string
	PublicKey     string
	Capabilities  []string
}

type pairingCode struct {
	Code        string
	ChildID     string
	RecipientID string
	ExpiresAt   time.Time
	Used        bool
}

type Store interface {
	CreateUser(ctx context.Context, email, passwordHash string) (string, error)
	UserByEmail(ctx context.Context, email string) (*user, error)

	CreateChild(ctx context.Context, parentID string) (string, error)
	CreatePairingCode(ctx context.Context, parentID, childID string, expiresAt time.Time) (*pairingCode, error)
	ClaimCode(ctx context.Context, code, publicKey, platform string, capabilities []string, now time.Time) (*device, error)
	DeviceByID(ctx context.Context, childDeviceID string) (*device, error)

	SetPolicy(ctx context.Context, parentID, childDeviceID string, policy []byte) error
	GetPolicy(ctx context.Context, childDeviceID string) ([]byte, error)

	EnqueueEnvelope(ctx context.Context, recipientID string, raw []byte) error
	DrainInbox(ctx context.Context, recipientID string) ([][]byte, error)
}

func randID(prefix string) string {
	b := make([]byte, 16)
	_, _ = rand.Read(b)
	return prefix + hex.EncodeToString(b)
}

func shortCode() string {
	b := make([]byte, 4)
	_, _ = rand.Read(b)
	return hex.EncodeToString(b)
}
