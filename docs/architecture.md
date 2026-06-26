# Architecture

## Philosophy: smart edge, thin cloud

Three research findings drive every decision:

1. Store policy is the binding constraint, not the OS. Both Apple and Google Play require overt, child-only, persistent-notification, non-covert, no-data-sale monitoring. iOS forbids reading other apps' message content entirely.
2. Privacy is the product. On-device AI uploads only alerts, never content. The CSAM legal rule makes "never store or transmit flagged media" a hard architectural invariant.
3. Free and self-hostable means the backend is a thin relay; all heavy compute runs on the device.

So: fat native agents do all sensing, enforcement, and AI locally; a small backend only does pairing, auth, encrypted alert relay, rule distribution, and push fan-out.

## Topology

```
 PARENT APP  <--HTTPS/WSS-->  COORDINATION BACKEND  <--HTTPS/WSS-->  CHILD AGENT (per OS)
 PWA + mobile shells          Go + Postgres + Redis                 native, fat edge:
 dashboard, alerts,           - auth / family graph                 - sensing
 rules, map                   - device pairing                      - enforcement
                              - E2E alert relay (ciphertext only)    - ON-DEVICE AI
                              - rule distribution                    - location
                              - APNs/FCM fan-out
                                     |
                              optional: self-hosted DNS filter (AdGuard Home / Pi-hole)
```

The backend is dumb about content. It relays end-to-end-encrypted envelopes (see [packages/pairing-protocol](../packages/pairing-protocol)) and never holds raw media or message bodies.

## Components and stack

- Backend: Go (chi + nhooyr/websocket) + Postgres + Redis. One static binary, one `docker compose up`. Auth: self-issued JWT plus rotating refresh tokens. Push: APNs and FCM, data-only payloads carrying an alert id. AGPL-3.0.
- Parent app: React + TypeScript PWA core, wrapped in a Capacitor shell for mobile (native push, biometric unlock). Desktop is the installable PWA. GPL-3.0.
- Child agents: native per OS (table below). GPL-3.0.
- Shared Rust core ([packages](../packages)): pairing, policy model, alert construction, E2E crypto, sync client. Compiled to each platform via UniFFI. One audited implementation of the security-critical code.

| OS | Language | Key frameworks |
|---|---|---|
| iOS / iPadOS | Swift + SwiftUI | FamilyControls, DeviceActivity, ManagedSettings, CoreLocation, CoreMotion, Core ML, Sensitive Content Analysis |
| Android | Kotlin | UsageStatsManager, VpnService, FusedLocationProvider, NotificationListenerService, ActivityRecognition, overlay, ONNX/ExecuTorch |
| Windows 11 | C# / .NET 8 (WinUI 3) | Windows Service, GetForegroundWindow + ETW + Job Objects, AppLocker, WFP/DNS, Assigned Access, ONNX Runtime + DirectML |
| macOS | Swift | NanoMDM config profiles + helper, Network Extension content filter |

## Capability matrix

Legend: High = clean public-API path; Med = works but reactive/battery/policy-fragile; Sideload = Android deep build only; Blocked = no sanctioned path.

| Feature | iOS/iPadOS | Android (store) | Android (sideload) | Windows 11 | macOS |
|---|---|---|---|---|---|
| App time limits / shielding | FamilyControls.shield (High, entitlement) | UsageStats + overlay (Med) | + Accessibility (High) | Job Objects + AppLocker (High) | MDM profile (Med) |
| Schedule / bedtime | DeviceActivitySchedule (High) | foreground svc + overlay (Med) | + Accessibility (High) | service + lock (High) | MDM Downtime (Med) |
| Web / content filtering | NEFilterDataProvider or DNS (High) | VpnService DNS/SNI (High) | same | WFP/DNS (High; deep HTTPS Low) | Network Extension (High) |
| Network filter (no agent) | AdGuard Home / Pi-hole at router (at-home only) | same | same | same | same |
| On-device image (nudity) AI | Core ML + Sensitive Content Analysis (High) | NudeNet + ViT (High) | same | ONNX + DirectML (High) | Core ML (High) |
| On-device text AI | Blocked (no API) | NotificationListener previews (Med) | Accessibility full text (High) | UIAutomation (Med) | Accessibility w/ consent (Med) |
| SMS / call-log | Blocked | Blocked (policy) | READ_SMS (Sideload) | n/a | n/a |
| Location + history | CoreLocation (High, lower freq) | FusedLocation (High) | same | coarse/IP (Med) | CoreLocation (High) |
| Distracted-walking | CoreMotion (Med) | ActivityRecognition (Med) | same | n/a | n/a |
| Anti-tamper | MDM removal lock (Med) | Device Admin FORCE_LOCK (Low) | + Accessibility guard (Med) | Service + AppLocker (High) | MDM lock (Med) |

Takeaways: image AI and DNS filtering are uniformly High, so they lead the roadmap. Text monitoring is the great divider, which is why Android ships in two builds.

## On-device AI pipeline

See [ai/README.md](../ai/README.md). Stage 0 is a cheap classifier on every message; only a small fraction escalates to an on-device LLM. Images run a nudity detector plus a confirmer. Output is a structured alert; raw content is discarded.

## Data and retention

The backend persists only the family graph, device keys and pairing state, rules, and encrypted alert envelopes. No raw media, message bodies, or AI inputs. Retention limits with auto-delete; short defaults. See [compliance/threat-model.md](../compliance/threat-model.md).
