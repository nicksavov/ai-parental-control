# android-child

The child agent for Android. Kotlin. Highest-feasibility platform, so it leads v0.

## Two build flavors (same source, gated)

- `store`: Play-compliant. UsageStatsManager time limits plus overlay blocking, VpnService DNS filtering (the Play-blessed mechanism), FusedLocation, NotificationListener message previews, on-device image AI. No AccessibilityService for monitoring, no READ_SMS.
- `sideload`: adds AccessibilityService full on-screen text (for the text AI) and optionally READ_SMS, for Bark-level coverage. Distributed via F-Droid or direct APK with its own updater.

Both flavors are overt: visible icon plus a persistent monitoring notification that cannot be disabled. The store-submission checklist applies to both.

## v0 scope

Pairing (claim a code, register device, X3DH via the Rust core), VpnService DNS filter with SafeSearch and category blocklists, UsageStatsManager reporting, daily-total plus bedtime overlay lock.

## Key APIs

UsageStatsManager, VpnService, SYSTEM_ALERT_WINDOW overlay, FusedLocationProvider, NotificationListenerService, ActivityRecognition, ONNX Runtime / ExecuTorch. See the capability matrix in [../../docs/architecture.md](../../docs/architecture.md).
