# macos-agent

The child agent for macOS. Swift. Ships in v3. Honestly the weakest platform for screen time.

## Hard constraints

- macOS has no Screen Time / FamilyControls API (those frameworks are iOS/iPadOS only). Enforcement goes through MDM / Configuration Profiles (NanoMDM backend) plus session accounting.
- The macOS app and installer must be notarized.
- Network Extension content filter is available for DNS/web filtering.

## v3 scope

MDM-profile install for restrictions, usage/session accounting, DNS/web filtering via Network Extension, on-device nudity AI via Core ML. Pairing and alert construction via the shared Rust core.
