# Store submission checklist

Run this before any App Store, Play, or Microsoft Store submission, and before publishing the sideload build. A failed item blocks release.

## Both stores (overt-only invariant)

- [ ] The child agent shows a visible app icon and a persistent, non-dismissable monitoring notification whenever it runs.
- [ ] There is no build flag, debug mode, or setting that hides the agent. A grep for stealth/hide affordances returns nothing.
- [ ] The overt-only invariant is covered by an automated test that fails CI if the notification can be suppressed.
- [ ] Marketing copy (store listing, README, site, screenshots) describes parental and family safety only. No "spy", "secret", "catch a cheater", or covert language anywhere.
- [ ] The app targets children only. No adult-target mode exists.
- [ ] A privacy policy is published and linked. It states that data is not sold or shared.
- [ ] A named child safety contact is published (Play Families requirement).

## Apple (App Store, iOS/iPadOS/macOS)

- [ ] Screen time and app blocking use Family Controls, ManagedSettings, and DeviceActivity only. No MDM, VPN, or private-API workarounds for these.
- [ ] The Family Controls entitlement has been granted. (Apply early; this is the long pole.)
- [ ] No claim or attempt to read other apps' message content on iOS.
- [ ] Data collection is disclosed on-screen before use (Guideline 5.1.1).
- [ ] macOS agent and installer are notarized.

## Google Play (Android store build)

- [ ] No READ_SMS or READ_CALL_LOG permission in the store build.
- [ ] No AccessibilityService used for monitoring in the store build.
- [ ] VpnService is used for DNS filtering only, with the permitted parental-control declaration; the persistent VPN icon and consent dialog are present.
- [ ] Background location uses a Play Console declaration, or a visible foreground service is used instead.
- [ ] NotificationListenerService use is disclosed, core, and consented.
- [ ] Spyware/Stalkerware policy compliance confirmed: persistent notification, identifying icon, child-only.

## Android sideload / F-Droid "deep" build

- [ ] Still overt: visible icon and persistent notification. No stealth, even though sideloading removes the store gate.
- [ ] Accessibility and any SMS use are disclosed at setup with explicit consent.
- [ ] Ships its own signed updater.
- [ ] Users are told Play Protect may warn on sideloaded monitoring apps.

## Windows / Microsoft Store

- [ ] Binary is code-signed (OV certificate or Store re-signing via MSIX).
- [ ] The agent is transparent (visible UI, clear uninstall path for the parent) to avoid a PUA flag.
- [ ] If flagged as PUA, a Microsoft review request is filed explaining the legitimate parental-control use.
