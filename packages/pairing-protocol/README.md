# pairing-protocol

How a parent links a child device, and how the end-to-end encryption keys are established so the backend can relay alerts it cannot read.

This is a design spec for the scaffold. The implementation lives in the shared Rust core and is the security-critical part of the system, so it gets one audited implementation rather than one per platform.

## Goals

1. Pair a child device to a family with a short, in-person, overt flow.
2. Establish a shared secret between the child device and the parent device(s) so alert payloads are end-to-end encrypted. The backend never holds plaintext.
3. Survive the parent having multiple devices and re-pairing.

## Entities

```
Family -> Parent(s) -> Child -> Device(s)
```

The parent is the consenting party (COPPA). Each device has a long-lived device credential issued at pairing and an identity key pair.

## Pairing flow (overt by design)

1. Parent signs in to the dashboard and creates a child, then taps "Pair device". The backend issues a short-lived pairing code and the parent's identity public key, shown as a QR code and a typeable code.
2. On the child device, the user installs the agent. Setup is visible and explains what will be monitored (no silent or hidden install). The agent scans the QR or enters the code.
3. The agent and the parent perform an X3DH-style key agreement (identity keys plus an ephemeral key) to derive a shared root key. The backend only ever sees public keys and the pairing state, never the derived secret.
4. The agent registers its device credential and push token, shows the persistent monitoring notification, and reports its platform capability set (which policy rules it can enforce).
5. Pairing code expires after a short TTL and one successful use.

## Encryption

- Identity keys: X25519 for agreement, Ed25519 for signatures.
- Session: an X3DH-style handshake derives a root key; messages use an AEAD (for example XChaCha20-Poly1305) with per-message nonces. A double-ratchet is not required for one-directional alert relay but may be added for rule channels later.
- Multi-parent: each parent device is a separate recipient; the child encrypts to each parent's key (small fan-out) or to a shared family key distributed at pairing. The scaffold assumes per-recipient encryption for simplicity.

## What the backend stores

Public keys, device ids, pairing state, push tokens, and ciphertext envelopes. Never a derived secret, never plaintext alerts. See [../proto](../proto) for the endpoints.

## Tests

- Round-trip: an alert encrypted on a simulated child device decrypts on a simulated parent device, and a backend that only sees ciphertext cannot recover the plaintext.
- Expiry: a reused or expired pairing code is rejected.
