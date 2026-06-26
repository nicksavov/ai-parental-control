# infra

Self-hosting, CI, and the sideload updater.

- Self-host: the backend stack is [backend/docker-compose.yml](../backend/docker-compose.yml). Add reverse-proxy and TLS examples here.
- CI: build and test each app, run the shared-core tests, and enforce the invariants that must never regress: the overt-only notification test and the alert no-raw-content test (see [compliance/store-submission-checklist.md](../compliance/store-submission-checklist.md)).
- Sideload updater: the Android deep build is distributed outside Play, so it ships its own signed update channel. The updater lives here.
