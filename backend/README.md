# backend

The thin coordination service. Go plus Postgres plus Redis. It does pairing, auth, rule distribution, encrypted alert relay, and push fan-out. It never sees raw content.

Licensed AGPL-3.0-or-later ([LICENSE](LICENSE)) so any hosted modified backend must publish its source.

## Layout (planned)

```
/backend
  /api        Go service: auth, family graph, pairing, policy, alert relay
  /push       APNs and FCM fan-out
  docker-compose.yml
  .env.example
  LICENSE
```

The API surface is defined in [../packages/proto/openapi.yaml](../packages/proto/openapi.yaml). The policy and alert shapes come from [../packages/policy-model](../packages/policy-model) and [../packages/alert-schema](../packages/alert-schema).

## Self-host quick start

```
cp .env.example .env   # edit secrets
docker compose up -d
```

This brings up Postgres, Redis, and the API. It fits a 512 MB to 1 GB VPS or a home Raspberry Pi because it only relays small encrypted envelopes. Heavy compute (AI) runs on the child devices, not here.

## What the backend stores

Family graph, accounts, device public keys and pairing state, rules, push tokens, and ciphertext alert envelopes. Never a derived secret, never plaintext alerts, never raw media. Retention limits with auto-delete are enforced here.
