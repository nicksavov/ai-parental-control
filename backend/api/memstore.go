package main

import (
	"context"
	"sync"
	"time"
)

// memStore is the in-memory Store. It backs unit tests and single-node dev.
type memStore struct {
	mu        sync.Mutex
	users     map[string]*user        // email -> user
	children  map[string]string       // childID -> owning parentID
	codes     map[string]*pairingCode // code -> pairing code
	devices   map[string]*device      // childDeviceID -> device
	policies  map[string][]byte       // childDeviceID -> policy JSON
	inbox     map[string][][]byte     // recipientID -> queued opaque envelopes
}

func newMemStore() *memStore {
	return &memStore{
		users:    map[string]*user{},
		children: map[string]string{},
		codes:    map[string]*pairingCode{},
		devices:  map[string]*device{},
		policies: map[string][]byte{},
		inbox:    map[string][][]byte{},
	}
}

func (s *memStore) CreateUser(_ context.Context, email, passwordHash string) (string, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	if _, ok := s.users[email]; ok {
		return "", errConflict
	}
	u := &user{ID: randID("user-"), Email: email, PasswordHash: passwordHash}
	s.users[email] = u
	return u.ID, nil
}

func (s *memStore) UserByEmail(_ context.Context, email string) (*user, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	u, ok := s.users[email]
	if !ok {
		return nil, errNotFound
	}
	return u, nil
}

func (s *memStore) CreateChild(_ context.Context, parentID string) (string, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	id := randID("child-")
	s.children[id] = parentID
	return id, nil
}

func (s *memStore) CreatePairingCode(_ context.Context, parentID, childID string, expiresAt time.Time) (*pairingCode, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	if owner, ok := s.children[childID]; !ok || owner != parentID {
		return nil, errForbidden
	}
	pc := &pairingCode{Code: shortCode(), ChildID: childID, RecipientID: parentID, ExpiresAt: expiresAt}
	s.codes[pc.Code] = pc
	return pc, nil
}

func (s *memStore) ClaimCode(_ context.Context, code, publicKey, platform string, capabilities []string, now time.Time) (*device, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	pc, ok := s.codes[code]
	if !ok || pc.Used || now.After(pc.ExpiresAt) {
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

func (s *memStore) DeviceByID(_ context.Context, childDeviceID string) (*device, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	d, ok := s.devices[childDeviceID]
	if !ok {
		return nil, errNotFound
	}
	return d, nil
}

func (s *memStore) SetPolicy(_ context.Context, parentID, childDeviceID string, policy []byte) error {
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

func (s *memStore) GetPolicy(_ context.Context, childDeviceID string) ([]byte, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	p, ok := s.policies[childDeviceID]
	if !ok {
		return nil, errNotFound
	}
	return p, nil
}

func (s *memStore) EnqueueEnvelope(_ context.Context, recipientID string, raw []byte) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.inbox[recipientID] = append(s.inbox[recipientID], raw)
	return nil
}

func (s *memStore) DrainInbox(_ context.Context, recipientID string) ([][]byte, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	out := s.inbox[recipientID]
	delete(s.inbox, recipientID)
	return out, nil
}
