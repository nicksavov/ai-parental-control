package main

import (
	"context"
	_ "embed"
	"errors"
	"time"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/jackc/pgx/v5/pgxpool"
)

//go:embed migrations/0001_init.sql
var initSQL string

// pgStore is the Postgres-backed Store for real deployments.
type pgStore struct {
	pool *pgxpool.Pool
}

func newPgStore(ctx context.Context, url string) (*pgStore, error) {
	pool, err := pgxpool.New(ctx, url)
	if err != nil {
		return nil, err
	}
	if _, err := pool.Exec(ctx, initSQL); err != nil {
		pool.Close()
		return nil, err
	}
	return &pgStore{pool: pool}, nil
}

func (s *pgStore) Close() { s.pool.Close() }

func isUniqueViolation(err error) bool {
	var pgErr *pgconn.PgError
	return errors.As(err, &pgErr) && pgErr.Code == "23505"
}

func (s *pgStore) CreateUser(ctx context.Context, email, passwordHash string) (string, error) {
	id := randID("user-")
	_, err := s.pool.Exec(ctx,
		`insert into users (id, email, password_hash) values ($1, $2, $3)`,
		id, email, passwordHash)
	if isUniqueViolation(err) {
		return "", errConflict
	}
	if err != nil {
		return "", err
	}
	return id, nil
}

func (s *pgStore) UserByEmail(ctx context.Context, email string) (*user, error) {
	var u user
	err := s.pool.QueryRow(ctx,
		`select id, email, password_hash from users where email = $1`, email).
		Scan(&u.ID, &u.Email, &u.PasswordHash)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, errNotFound
	}
	if err != nil {
		return nil, err
	}
	return &u, nil
}

func (s *pgStore) CreateChild(ctx context.Context, parentID string) (string, error) {
	id := randID("child-")
	_, err := s.pool.Exec(ctx, `insert into children (id, parent_id) values ($1, $2)`, id, parentID)
	return id, err
}

func (s *pgStore) CreatePairingCode(ctx context.Context, parentID, childID string, expiresAt time.Time) (*pairingCode, error) {
	var owner string
	err := s.pool.QueryRow(ctx, `select parent_id from children where id = $1`, childID).Scan(&owner)
	if errors.Is(err, pgx.ErrNoRows) || owner != parentID {
		return nil, errForbidden
	}
	if err != nil {
		return nil, err
	}
	pc := &pairingCode{Code: shortCode(), ChildID: childID, RecipientID: parentID, ExpiresAt: expiresAt}
	_, err = s.pool.Exec(ctx,
		`insert into pairing_codes (code, child_id, recipient_id, expires_at) values ($1, $2, $3, $4)`,
		pc.Code, pc.ChildID, pc.RecipientID, pc.ExpiresAt)
	if err != nil {
		return nil, err
	}
	return pc, nil
}

func (s *pgStore) ClaimCode(ctx context.Context, code, publicKey, platform string, capabilities []string, now time.Time) (*device, error) {
	tx, err := s.pool.Begin(ctx)
	if err != nil {
		return nil, err
	}
	defer tx.Rollback(ctx)

	var pc pairingCode
	err = tx.QueryRow(ctx,
		`select code, child_id, recipient_id, expires_at, used from pairing_codes where code = $1 for update`, code).
		Scan(&pc.Code, &pc.ChildID, &pc.RecipientID, &pc.ExpiresAt, &pc.Used)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, errBadCode
	}
	if err != nil {
		return nil, err
	}
	if pc.Used || now.After(pc.ExpiresAt) {
		return nil, errBadCode
	}
	if _, err := tx.Exec(ctx, `update pairing_codes set used = true where code = $1`, code); err != nil {
		return nil, err
	}
	d := &device{
		ChildDeviceID: randID("device-"),
		ChildID:       pc.ChildID,
		RecipientID:   pc.RecipientID,
		Platform:      platform,
		PublicKey:     publicKey,
		Capabilities:  capabilities,
	}
	_, err = tx.Exec(ctx,
		`insert into devices (child_device_id, child_id, recipient_id, platform, public_key, capabilities)
		 values ($1, $2, $3, $4, $5, $6)`,
		d.ChildDeviceID, d.ChildID, d.RecipientID, d.Platform, d.PublicKey, d.Capabilities)
	if err != nil {
		return nil, err
	}
	if err := tx.Commit(ctx); err != nil {
		return nil, err
	}
	return d, nil
}

func (s *pgStore) DeviceByID(ctx context.Context, childDeviceID string) (*device, error) {
	var d device
	err := s.pool.QueryRow(ctx,
		`select child_device_id, child_id, recipient_id, platform, public_key, capabilities
		 from devices where child_device_id = $1`, childDeviceID).
		Scan(&d.ChildDeviceID, &d.ChildID, &d.RecipientID, &d.Platform, &d.PublicKey, &d.Capabilities)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, errNotFound
	}
	if err != nil {
		return nil, err
	}
	return &d, nil
}

func (s *pgStore) SetPolicy(ctx context.Context, parentID, childDeviceID string, policy []byte) error {
	var owner string
	err := s.pool.QueryRow(ctx,
		`select c.parent_id from devices d join children c on c.id = d.child_id
		 where d.child_device_id = $1`, childDeviceID).Scan(&owner)
	if errors.Is(err, pgx.ErrNoRows) {
		return errNotFound
	}
	if err != nil {
		return err
	}
	if owner != parentID {
		return errForbidden
	}
	_, err = s.pool.Exec(ctx,
		`insert into policies (child_device_id, policy, updated_at) values ($1, $2, now())
		 on conflict (child_device_id) do update set policy = excluded.policy, updated_at = now()`,
		childDeviceID, policy)
	return err
}

func (s *pgStore) GetPolicy(ctx context.Context, childDeviceID string) ([]byte, error) {
	var policy []byte
	err := s.pool.QueryRow(ctx, `select policy from policies where child_device_id = $1`, childDeviceID).Scan(&policy)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, errNotFound
	}
	if err != nil {
		return nil, err
	}
	return policy, nil
}

func (s *pgStore) EnqueueEnvelope(ctx context.Context, recipientID string, raw []byte) error {
	_, err := s.pool.Exec(ctx,
		`insert into alert_envelopes (recipient_id, envelope) values ($1, $2)`, recipientID, raw)
	return err
}

func (s *pgStore) DrainInbox(ctx context.Context, recipientID string) ([][]byte, error) {
	rows, err := s.pool.Query(ctx,
		`with drained as (
			delete from alert_envelopes where recipient_id = $1 returning id, envelope
		 )
		 select envelope from drained order by id`, recipientID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var out [][]byte
	for rows.Next() {
		var e []byte
		if err := rows.Scan(&e); err != nil {
			return nil, err
		}
		out = append(out, e)
	}
	return out, rows.Err()
}
