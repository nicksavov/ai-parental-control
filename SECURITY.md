# Security and Safety

This project holds sensitive data about minors. Security is a compliance obligation, not just good practice.

## Reporting a vulnerability

Email the security contact (TODO: set a real address, for example security@your-domain) with details and steps to reproduce. Please do not open a public issue for an exploitable vulnerability. We will acknowledge within a few days.

## Child safety contact

A named child safety point of contact is required by Google Play's Families policy. TODO: set a real address (for example child-safety@your-domain).

## Security posture (summary)

See [compliance/threat-model.md](compliance/threat-model.md) for the full model. Key points:

- AI inference runs on the child device. Raw content never leaves the device; only structured alerts do.
- The backend relays end-to-end-encrypted alert envelopes and never stores raw media or message bodies.
- Alert metadata is encrypted at rest using the platform keystore (iOS Keychain / Secure Enclave, Android Keystore, Windows DPAPI). All sync traffic is TLS, with certificate pinning where feasible.
- Retention limits with automatic deletion are enforced (configurable, short defaults).
- The backend is self-hostable so privacy-conscious families can keep all data on their own infrastructure.

## Scope note

A determined child with physical access, an admin account, or the ability to dual-boot can bypass device-level enforcement. The app treats tamper and uninstall as parent-notified events rather than pretending to be unbreakable.
