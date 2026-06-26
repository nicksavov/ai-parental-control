# api

The Go coordination service: auth, family graph, pairing, policy distribution, encrypted alert relay. Implements [../../packages/proto/openapi.yaml](../../packages/proto/openapi.yaml). Stores ciphertext envelopes and metadata only, never raw content.

## v0 status

Stdlib-only, in-memory store, runs and tests offline. Postgres and Redis replace the in-memory store later behind the same interface.

- `main.go`: server, routing (Go 1.22+ pattern mux), entrypoint.
- `auth.go`: HMAC-signed bearer tokens (scaffold; real JWT plus refresh tokens later).
- `store.go`: in-memory family graph, pairing, policies, and a per-recipient inbox of opaque envelopes.
- `handlers.go`: the endpoints. The alert handler reads only the envelope routing fields and relays the body byte-for-byte; it never parses the ciphertext.

Run it:

```
JWT_SIGNING_KEY=dev-secret go run ./...
go test ./...
```

Key property under test: an alert submitted by a child is relayed to the paired parent unchanged, a device cannot address an envelope to a parent it is not paired with, and forged or missing tokens are rejected.
