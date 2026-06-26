# Threat model

The system holds extremely sensitive data about minors: location, alerts derived from messages, image-flagged events. It is a high-value breach and subpoena target. Security here is a compliance obligation (the COPPA written security program).

## Assets

- A child's content (messages, images) on the device.
- Alert metadata (category, severity, time, app, short snippet).
- Location history.
- Family graph, accounts, device keys.

## The stalkerware boundary (primary design constraint)

The same capabilities that protect a child could stalk a person. We stay on the right side by construction:

- Overt agent plus persistent notification, always, on every build.
- Child-only targeting; no adult-target mode.
- No stealth/hide mode exists in any build.
- Tamper and uninstall are parent-notified events, not hidden, undefeatable blocks.

## Data-flow defenses

- AI inference runs on the child device. Raw content is analyzed in memory and discarded. Only structured alerts leave the device.
- Alerts are end-to-end encrypted from the child device to the parent. The backend relays ciphertext and cannot read it.
- The alert schema cannot carry raw media or full message bodies (enforced by `additionalProperties: false` and CI tests).
- No children's content is sent to any third-party cloud by default.

## Storage and retention

- Alert metadata encrypted at rest via the platform keystore (iOS Keychain/Secure Enclave, Android Keystore, Windows DPAPI).
- All sync traffic over TLS; certificate pinning where feasible.
- Retention limits with automatic deletion. Short defaults (for example alerts 30 to 90 days, location history 30 days), parent-configurable downward.
- The backend is self-hostable so a family can avoid any operator-held data.

## Adversaries and limits

- A curious child: handled by overt enforcement plus tamper alerts.
- A determined child with admin rights, physical access, or dual-boot: can bypass device-level enforcement. We do not pretend otherwise; the parent is notified when enforcement is not active.
- A network attacker: mitigated by TLS and E2E encryption.
- A breach of the backend: limited blast radius because the backend holds only ciphertext envelopes and metadata, never raw content.
- Legal process against the operator: the operator cannot produce content it never held.

## Out of scope (by deliberate decision)

- CSAM detection, hashing, or reporting infrastructure (see legal-notes.md).
- Deep HTTPS content inspection via TLS interception (wiretap and store risk; we stay at DNS/SNI level).
- Reading two-party communication content.
