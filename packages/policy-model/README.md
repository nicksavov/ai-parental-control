# policy-model

The rules a parent sets for one child: filtering, screen time, monitoring tracks, and location. One model for every platform.

See [policy.schema.json](policy.schema.json) for the full shape.

## Principle: enforce what the OS allows, report the rest

Platform capability is uneven (see the matrix in [docs/architecture.md](../../docs/architecture.md)). A rule that an OS cannot enforce is **reported back** to the parent as "not enforceable on this device", never silently dropped. Example: a per-app time limit applies cleanly on iOS via FamilyControls and on Android via UsageStats plus overlay, but on macOS it depends on an MDM profile and may be coarser. The parent should see that, not assume full coverage.

## Sections

- `filtering`: DNS category blocklists, SafeSearch, allow/block domain lists. Enforced via the local VPN DNS filter on the device and/or a self-hosted network filter.
- `screenTime`: daily total, per-app budgets, and named schedule windows (bedtime, school, free time) with block/allow modes.
- `monitoring`: which on-device AI tracks are on plus per-category sensitivity (`none`/`all`/`severe`, mirroring Bark's tiers). This never enables cloud routing; all inference stays on the device.
- `location`: opt-in, with a capped history window and geofences.

## Notes

- `monitoring.textAnalysis` defaults to off. It only does anything where the OS and build permit reading text (Android notification previews on the store build, full on-screen text on the sideload build, nothing on iOS).
- `location.historyDays` is capped at 90 and defaults to 30, to keep retention short by default (a COPPA expectation).
- This model lives in the shared Rust core so all agents and the backend serialize it identically.
