# Roadmap

Platform feasibility is very uneven, so we ship where it is highest first. Order of feasibility: Android (sideload) > Android (store) > Windows/macOS desktop > DNS-for-all > iOS/iPadOS.

Apply for the Apple Family Controls entitlement on day one, in parallel with v0. It is the longest pole and gates everything on iOS.

## v0 MVP: pairing + DNS filtering + screen-time core (Android, Windows)

Smallest useful, store-clean slice, no exotic entitlements.

- Device pairing (QR/code), parent-child link, persistent monitoring notification.
- DNS content filtering: self-hostable filtering DNS plus on-device local VPN (Android) / DNS hook (Windows), SafeSearch, category blocklists, DoH/DoT bypass detection.
- Screen-time reporting: per-app and total usage (UsageStatsManager / ETW).
- Basic limits: daily total plus bedtime window; overlay lock (Android) / session lock (Windows).
- Parent dashboard (PWA): usage, filter status, set limits.

Build order to de-risk: (1) pairing plus the shared contracts in [packages](../packages), (2) DNS filtering, (3) usage reporting, (4) enforcement.

## v1: location + richer limits + alerts (Android, Windows)

- Location: real-time plus history (FusedLocationProvider) plus geofencing; foreground-service fallback to avoid the strict background-location review.
- Per-app daily timers and per-app time-window blocking.
- Anti-tamper: Device Admin FORCE_LOCK (Android), AppLocker block of uninstall tools (Windows); "monitoring not active" is a first-class parent alert. Battery-exemption onboarding.
- Push alerts via [packages/alert-schema](../packages/alert-schema).

## v2: on-device AI moderation (Android-primary; sideload for deep text)

- Image/nudity (store and sideload): NudeNet plus ViT, alert plus delete prompt, never persist media.
- Text: Stage-0 Detoxify gate then Stage-1 small LLM. Source split:
  - Store build: image AI plus NotificationListener previews. No Accessibility, no SMS.
  - Sideload/F-Droid deep build: adds Accessibility full text and optional SMS; own updater; still overt.
- Smombie detection (ActivityRecognition).

## v3: iOS/iPadOS and macOS (entitlement-gated, reduced scope)

- iOS: FamilyControls plus ManagedSettings shield plus DeviceActivity; Sensitive Content Analysis or Core ML nudity; CoreLocation; CoreMotion.
- macOS: MDM/Configuration Profiles (NanoMDM) plus session accounting. No Screen Time API on macOS.
- Honest cuts vs Bark: no third-party chat-text reading on iOS, no SMS/iMessage scanning, lower-frequency location.

## v4+ backlog

Positive-reinforcement layer (Habitica-style), app-install approval workflow, optional hosted tier, EMM/Device-Owner route for deeper control, localization.

## Features cut or degraded vs Bark (set expectations)

- iOS text/chat monitoring: cut (no API).
- SMS/Call-log and Accessibility scraping: cut on stores, sideload-only.
- macOS screen-time enforcement: degraded (MDM only).
- Force-kill another app on Android: degraded (reactive overlay only).
- Bark-style cloud AI over 30+ apps/email/web: cut by design (on-device only).
- CSAM detection / PhotoDNA: cut (legal landmine).
- Covert/stealth mode: cut (forbidden).
- Deep HTTPS filtering: degraded (DNS/SNI level only).
