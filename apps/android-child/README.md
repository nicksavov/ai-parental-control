# android-child

The child agent for Android. Kotlin. Highest-feasibility platform, so it leads v0.

> This is a v0 sketch: real, Android Studio-ready files, but not built in this repo's CI (no Android SDK here). Open in Android Studio, run `gradle wrapper`, and build a flavor.

## Two build flavors (same source, gated)

- `store`: Play-compliant. UsageStatsManager time limits plus overlay blocking, VpnService DNS filtering (the Play-blessed mechanism), FusedLocation, NotificationListener message previews, on-device image AI. No AccessibilityService for monitoring, no READ_SMS.
- `sideload`: adds AccessibilityService full on-screen text (for the text AI) and optionally READ_SMS, for Bark-level coverage. Distributed via F-Droid or direct APK with its own updater.

Both flavors are overt: visible icon plus a persistent monitoring notification that cannot be disabled. The store-submission checklist applies to both.

## What is in the sketch

| File | Role |
|---|---|
| `app/src/main/AndroidManifest.xml` | Permissions, the overt foreground service, the DNS VpnService |
| `app/src/sideload/AndroidManifest.xml` | Flavor overlay adding Accessibility and optional SMS |
| `MonitoringService.kt` | The overt-only invariant: persistent, non-dismissable monitoring notification |
| `ChildApp.kt` | Notification channel setup |
| `dns/DnsFilterVpnService.kt` | Local VPN DNS filter skeleton |
| `usage/UsageReporter.kt` | UsageStatsManager screen-time reporting and foreground detection |
| `pairing/PairingManager.kt` | Pairing + alert relay using the shared Rust core (packages/ffi) and the backend |
| `ui/SetupActivity.kt` | Overt setup, consent, and permission grants |
| `accessibility/TextMonitorAccessibilityService.kt` | Sideload-only on-screen text source for the text AI |

## v0 scope

Pairing (claim a code, register device, X3DH via the Rust core), VpnService DNS filter with SafeSearch and category blocklists, UsageStatsManager reporting, daily-total plus bedtime overlay lock.

## Wiring to the rest of the repo

- Crypto and pairing call the generated UniFFI bindings from [../../packages/ffi](../../packages/ffi) (`uniffi.apc_ffi.*`) over a bundled native `.so`. The generated Kotlin is checked in at `app/src/main/kotlin/uniffi/apc_ffi/apc_ffi.kt`; regenerate it with the command in the packages/ffi README when the Rust surface changes.
- The backend API is [../../packages/proto/openapi.yaml](../../packages/proto/openapi.yaml).
- Capability and policy limits per platform: [../../docs/architecture.md](../../docs/architecture.md).

## Not committed (binary placeholders)

App icons (`mipmap/ic_launcher`) and the notification icon (`drawable/ic_monitoring`) are binary assets; add them in Android Studio. The Gradle wrapper jar is generated with `gradle wrapper`.
