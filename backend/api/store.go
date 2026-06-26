package main

import (
	"crypto/rand"
	"encoding/hex"
	"errors"
	"sync"
	"time"
)

// The store is intentionally dumb about content. It holds the family graph,
// pairing state, policies, and a per-recipient inbox of OPAQUE alert envelopes
// (raw bytes it never decrypts). Postgres and Redis replace this in-memory map
// later; the interface stays the same so the backend never gains the ability to
// read an alert.
//
// This scaffold keeps everything in memory and dependency-free so the package
// builds and tests offline. It is not production storage.

var (
	errNotFound  = errors.New("not found")
	errForbidden = errors.New("forbidden")
	errBadCode   = errors.New("invalid or expired pairing code")
)

type device struct {
	ChildDeviceID string
	ChildID       string
	RecipientID   string // the parent that receives this device's alerts
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

type store struct {
	mu       sync.Mutex
	children map[string]string       // childID -> owning parentID
	codes    map[string]*pairingCode // code -> pairing code
	devices  map[string]*device      // childDeviceID -> device
	policies map[string][]byte       // childDeviceID -> policy JSON
	inbox    map[string][][]byte     // recipientID -> queued opaque envelopes
	now      func() time.Time
}

func newStore() *store {
	return &store{
		children: map[string]string{},
		codes:    map[string]*pairingCode{},
		devices:  map[string]*device{},
		policies: map[string][]byte{},
		inbox:    map[string][][]byte{},
		now:      time.Now,
	}
}

func randID(prefix string) string {
	b := make([]byte, 16)
	_, _ = rand.Read(b)
	return prefix + hex.EncodeToString(b)
}

func (s *store) createChild(parentID string) string {
	s.mu.Lock()
	defer s.mu.Unlock()
	id := randID("child-")
	s.children[id] = parentID
	return id
}

func (s *store) createPairingCode(parentID, childID string, ttl time.Duration) (*pairingCode, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	if owner, ok := s.children[childID]; !ok || owner != parentID {
		return nil, errForbidden
	}
	pc := &pairingCode{
		Code:        randID("")[:8],
		ChildID:     childID,
		RecipientID: parentID, // the parent device that will decrypt; parentID in this scaffold
		ExpiresAt:   s.now().Add(ttl),
	}
	s.codes[pc.Code] = pc
	return pc, nil
}

func (s *store) claimCode(code, publicKey, platform string, capabilities []string) (*device, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	pc, ok := s.codes[code]
	if !ok || pc.Used || s.now().After(pc.ExpiresAt) {
		return nil, errBadCode
	}
	pc.Used = true
	d := &device{
		ChildDeviceID: randID("device-"),
		ChildID:       pc.ChildID,
		RecipientID:   pc.RecipientID,
		Platform:      platform,
		PublicKey:     publicKey,
		Capabilities:  capabilities,
	}
	s.devices[d.ChildDeviceID] = d
	return d, nil
}

func (s *store) deviceByID(childDeviceID string) (*device, bool) {
	s.mu.Lock()
	defer s.mu.Unlock()
	d, ok := s.devices[childDeviceID]
	return d, ok
}

func (s *store) setPolicy(parentID, childDeviceID string, policy []byte) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	d, ok := s.devices[childDeviceID]
	if !ok {
		return errNotFound
	}
	if s.children[d.ChildID] != parentID {
		return errForbidden
	}
	s.policies[childDeviceID] = policy
	return nil
}

func (s *store) getPolicy(childDeviceID string) ([]byte, bool) {
	s.mu.Lock()
	defer s.mu.Unlock()
	p, ok := s.policies[childDeviceID]
	return p, ok
}

// enqueueEnvelope stores an opaque envelope for its recipient. The backend reads
// only the routing fields (passed in) and never the ciphertext.
func (s *store) enqueueEnvelope(recipientID string, raw []byte) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.inbox[recipientID] = append(s.inbox[recipientID], raw)
}

// drainInbox returns and clears the queued envelopes for a recipient.
func (s *store) drainInbox(recipientID string) [][]byte {
	s.mu.Lock()
	defer s.mu.Unlock()
	out := s.inbox[recipientID]
	delete(s.inbox, recipientID)
	return out
}
