package main

import (
	"context"
	"os"
	"testing"
	"time"
)

// Runs the Store contract against a real Postgres. Skipped unless DATABASE_URL
// is set, so the default test run stays offline.
//
//	DATABASE_URL=postgres://apc:apc@localhost:5432/apc?sslmode=disable go test ./...
func TestPgStoreContract(t *testing.T) {
	url := os.Getenv("DATABASE_URL")
	if url == "" {
		t.Skip("set DATABASE_URL to run the Postgres integration test")
	}
	ctx := context.Background()
	st, err := newPgStore(ctx, url)
	if err != nil {
		t.Fatalf("connect: %v", err)
	}
	defer st.Close()

	email := "pg-" + shortCode() + "@example.com"
	parentID, err := st.CreateUser(ctx, email, "hash")
	if err != nil {
		t.Fatalf("create user: %v", err)
	}
	if _, err := st.CreateUser(ctx, email, "hash"); err != errConflict {
		t.Fatalf("want errConflict on duplicate email, got %v", err)
	}

	childID, err := st.CreateChild(ctx, parentID)
	if err != nil {
		t.Fatalf("create child: %v", err)
	}

	pc, err := st.CreatePairingCode(ctx, parentID, childID, time.Now().Add(time.Minute))
	if err != nil {
		t.Fatalf("create code: %v", err)
	}
	d, err := st.ClaimCode(ctx, pc.Code, "pk", "android", []string{"filtering"}, time.Now())
	if err != nil {
		t.Fatalf("claim: %v", err)
	}
	if _, err := st.ClaimCode(ctx, pc.Code, "pk", "android", nil, time.Now()); err != errBadCode {
		t.Fatalf("want errBadCode on reused code, got %v", err)
	}

	if err := st.SetPolicy(ctx, parentID, d.ChildDeviceID, []byte(`{"v":1}`)); err != nil {
		t.Fatalf("set policy: %v", err)
	}
	if p, err := st.GetPolicy(ctx, d.ChildDeviceID); err != nil || string(p) != `{"v":1}` {
		t.Fatalf("get policy: %q %v", p, err)
	}

	if err := st.EnqueueEnvelope(ctx, d.RecipientID, []byte(`{"id":"a"}`)); err != nil {
		t.Fatalf("enqueue: %v", err)
	}
	out, err := st.DrainInbox(ctx, d.RecipientID)
	if err != nil || len(out) != 1 || string(out[0]) != `{"id":"a"}` {
		t.Fatalf("drain: %v %v", out, err)
	}
	if out, _ := st.DrainInbox(ctx, d.RecipientID); len(out) != 0 {
		t.Fatalf("inbox should be empty after drain, got %d", len(out))
	}
}
