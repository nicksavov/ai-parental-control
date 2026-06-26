# ios-child

The child agent for iOS and iPadOS. Swift plus SwiftUI. Ships in v3 because it is entitlement-gated and the most constrained platform.

## Hard constraints

- Screen time and app blocking must use FamilyControls, ManagedSettings, and DeviceActivity. No MDM/VPN/private-API workarounds (auto-rejected).
- Requires the Apple Family Controls entitlement, which is special-grant and slow. Apply on day one of the project.
- There is no API to read other apps' message content. Bark/Adora-style text monitoring is not possible on iOS. Do not promise it.

## v3 scope

Screen-time limits and app shielding (FamilyControls), nudity detection (Sensitive Content Analysis or a bundled Core ML model, Photos permission), location (CoreLocation), distracted-walking (CoreMotion). Pairing and alert construction via the shared Rust core.
