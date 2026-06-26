# parent-web

The parent dashboard. React plus TypeScript plus Vite, shipped as a PWA. The same code is wrapped in a Capacitor shell for mobile (native push, biometric unlock) and installs as a desktop PWA. The parent side needs no privileged device APIs, so a web core maximizes reuse and self-hostability.

## v0 scope

Sign in, create a child, generate a pairing QR/code, view usage and filter status, set daily-total and bedtime limits, see device pairing state.

## Later

Alert feed (decrypts envelopes locally, renders category/severity/time/app and an optional snippet the parent chooses to view), location map, per-app limits, geofences, teen transparency view.

Talks to the backend via [../../packages/proto/openapi.yaml](../../packages/proto/openapi.yaml). Alerts are decrypted client-side; the backend only relays ciphertext.
