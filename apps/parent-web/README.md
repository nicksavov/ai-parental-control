# parent-web

The parent dashboard. React plus TypeScript plus Vite, shipped as a PWA. The same code is wrapped in a Capacitor shell for mobile (native push, biometric unlock) and installs as a desktop PWA. The parent side needs no privileged device APIs, so a web core maximizes reuse and self-hostability.

## v0 scope

Sign in, create a child, generate a pairing QR/code, view usage and filter status, set daily-total and bedtime limits, see device pairing state.

## Later

Alert feed (decrypts envelopes locally, renders category/severity/time/app and an optional snippet the parent chooses to view), location map, per-app limits, geofences, teen transparency view.

Talks to the backend via [../../packages/proto/openapi.yaml](../../packages/proto/openapi.yaml). Alerts are decrypted client-side; the backend only relays ciphertext.

## Status (scaffolded)

Vite + React + TypeScript. Builds and tests offline.

- `src/api/client.ts`: typed client for the backend (register, login, create child, pairing code, set policy, alert stream). Attaches the bearer token and refreshes once on a 401.
- `src/api/types.ts`: token, pairing, envelope, and alert shapes (mirroring the policy model and proto).
- `src/ui/Login.tsx`, `src/ui/Dashboard.tsx`: sign in, add a child, generate a pairing code, and an alert feed.
- `src/api/crypto.ts` + `src/wasm/`: the shared core compiled to WebAssembly (packages/wasm). Alerts are decrypted in the browser; the Dashboard includes an in-browser crypto self-test that pairs, seals, and opens a demo alert client side.

```
npm install
npm test          # vitest: 5 client tests (auth, refresh-on-401, errors, long-poll)
npm run build     # tsc + vite production build
npm run dev       # local dev server; set VITE_API_URL to point at the backend
```

Next: a policy editor (limits, schedules, filtering) and persisting the parent's per-device shared secret so the live alert feed decrypts real envelopes (the crypto path is already wired via `src/api/crypto.ts`).
