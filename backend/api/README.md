# api

The Go coordination service: auth, family graph, pairing, policy distribution, encrypted alert relay. Implements [../../packages/proto/openapi.yaml](../../packages/proto/openapi.yaml). Stores ciphertext envelopes and metadata only, never raw content.

## Status

- `main.go`: server, routing (Go 1.22+ pattern mux), env-based wiring.
- `jwt.go`: HS256 JWTs with expiry; short-lived access tokens, long-lived refresh tokens.
- `password.go`: PBKDF2-HMAC-SHA256 password hashing (stdlib `crypto/pbkdf2`).
- `auth.go`: bearer middleware requiring a valid access token of the right role.
- `store.go` / `memstore.go` / `pgstore.go`: the `Store` interface with an in-memory and a Postgres (pgx) implementation; `migrations/0001_init.sql` is the schema.
- `notifier.go` / `redis.go`: alert-arrival fan-out so the stream endpoint can long-poll; in-process by default, Redis pub/sub when configured.
- `handlers.go`: the endpoints. The alert handler reads only the envelope routing fields and relays the body byte-for-byte; it never parses the ciphertext.

## Configuration

| Env | Default | Effect |
|---|---|---|
| `JWT_SIGNING_KEY` | required | HMAC key for signing tokens |
| `DATABASE_URL` | unset | Postgres (pgx). Unset uses the in-memory store |
| `REDIS_URL` | unset | Redis pub/sub fan-out. Unset uses in-process |
| `API_PORT` | 8080 | listen port |

## Run and test

```
JWT_SIGNING_KEY=dev-secret go run ./...
go test ./...                                  # offline; Postgres test skips
DATABASE_URL=postgres://apc:apc@localhost:5432/apc?sslmode=disable go test ./...  # runs the pg integration test
```

Auth flow: register or password-grant returns an access token plus a refresh token; the refresh grant mints new access tokens. A paired device gets a refresh credential and exchanges it for access tokens the same way.

Properties under test: an alert submitted by a child is relayed to the paired parent unchanged; a device cannot address an envelope to a parent it is not paired with; forged, missing, or wrong-password requests are rejected; the refresh grant works; and a long-poll stream is woken by a new alert.
