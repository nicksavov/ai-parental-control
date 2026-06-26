//! Device pairing and end-to-end alert encryption.
//!
//! A child device (the initiator) pairs with a parent (the responder). They run
//! an X3DH-style handshake to derive a shared secret. The backend only ever
//! relays ciphertext, so alerts are sealed on the child device and opened on the
//! parent device. See README.md in this directory for the protocol notes.
//!
//! Crypto choices: X25519 for Diffie-Hellman, Ed25519 for the prekey signature,
//! HKDF-SHA256 for key derivation, XChaCha20-Poly1305 for authenticated
//! encryption. This is a clear, auditable implementation, not a from-scratch
//! primitive.

use base64::Engine as _;
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use hkdf::Hkdf;
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use x25519_dalek::{PublicKey, StaticSecret};

const B64: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD;
const ENVELOPE_VERSION: u32 = 1;

/// Errors from pairing and sealing.
#[derive(Debug)]
pub enum PairingError {
    BadPrekeySignature,
    BadKeyLength,
    Decode(String),
    Aead,
    Alert(String),
    IdentityMismatch,
}

impl std::fmt::Display for PairingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PairingError::BadPrekeySignature => write!(f, "prekey signature did not verify"),
            PairingError::BadKeyLength => write!(f, "key or nonce had the wrong length"),
            PairingError::Decode(e) => write!(f, "decode error: {e}"),
            PairingError::Aead => write!(f, "authenticated decryption failed"),
            PairingError::Alert(e) => write!(f, "alert error: {e}"),
            PairingError::IdentityMismatch => {
                write!(f, "envelope routing fields did not match the sealed alert")
            }
        }
    }
}

impl std::error::Error for PairingError {}

/// A device's long-lived keys: an X25519 key for Diffie-Hellman and an Ed25519
/// key for signing. Generate once at first run and store in the platform keystore.
pub struct DeviceIdentity {
    dh_secret: StaticSecret,
    signing: SigningKey,
}

impl DeviceIdentity {
    /// Generate a fresh identity using the OS CSPRNG.
    pub fn generate() -> DeviceIdentity {
        DeviceIdentity {
            dh_secret: StaticSecret::random_from_rng(OsRng),
            signing: SigningKey::generate(&mut OsRng),
        }
    }

    pub fn dh_public(&self) -> PublicKey {
        PublicKey::from(&self.dh_secret)
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing.verifying_key()
    }
}

/// What a parent (responder) publishes so a child (initiator) can pair. The
/// signed prekey is an X25519 public key signed by the parent's Ed25519 key,
/// which proves it belongs to this parent.
pub struct PrekeyBundle {
    pub identity_dh_public: PublicKey,
    pub identity_verifying_key: VerifyingKey,
    pub signed_prekey_public: PublicKey,
    pub prekey_signature: Signature,
}

/// A parent's ephemeral-but-stored prekey, kept until the handshake completes.
pub struct ResponderPrekey {
    secret: StaticSecret,
}

impl ResponderPrekey {
    pub fn generate() -> ResponderPrekey {
        ResponderPrekey {
            secret: StaticSecret::random_from_rng(OsRng),
        }
    }

    pub fn public(&self) -> PublicKey {
        PublicKey::from(&self.secret)
    }
}

/// Build the bundle the parent shows during pairing (as a QR or code link).
pub fn build_prekey_bundle(identity: &DeviceIdentity, prekey: &ResponderPrekey) -> PrekeyBundle {
    let prekey_public = prekey.public();
    let signature = identity.signing.sign(prekey_public.as_bytes());
    PrekeyBundle {
        identity_dh_public: identity.dh_public(),
        identity_verifying_key: identity.verifying_key(),
        signed_prekey_public: prekey_public,
        prekey_signature: signature,
    }
}

/// Result of the initiator side: the shared secret plus the public values the
/// responder needs to derive the same secret.
pub struct InitiatorHandshake {
    pub shared_secret: [u8; 32],
    pub initiator_dh_public: PublicKey,
    pub ephemeral_public: PublicKey,
}

fn derive_shared_secret(dh1: &[u8], dh2: &[u8], dh3: &[u8]) -> [u8; 32] {
    let mut ikm = Vec::with_capacity(96);
    ikm.extend_from_slice(dh1);
    ikm.extend_from_slice(dh2);
    ikm.extend_from_slice(dh3);
    let hk = Hkdf::<Sha256>::new(None, &ikm);
    let mut okm = [0u8; 32];
    hk.expand(b"apc/x3dh/sk/v1", &mut okm)
        .expect("32 is a valid HKDF length");
    okm
}

/// Child side of the handshake. Verifies the parent's signed prekey, then
/// derives the shared secret.
pub fn initiator_handshake(
    initiator: &DeviceIdentity,
    bundle: &PrekeyBundle,
) -> Result<InitiatorHandshake, PairingError> {
    bundle
        .identity_verifying_key
        .verify(bundle.signed_prekey_public.as_bytes(), &bundle.prekey_signature)
        .map_err(|_| PairingError::BadPrekeySignature)?;

    let ephemeral = StaticSecret::random_from_rng(OsRng);
    let ephemeral_public = PublicKey::from(&ephemeral);

    let dh1 = initiator.dh_secret.diffie_hellman(&bundle.signed_prekey_public);
    let dh2 = ephemeral.diffie_hellman(&bundle.identity_dh_public);
    let dh3 = ephemeral.diffie_hellman(&bundle.signed_prekey_public);

    let shared_secret = derive_shared_secret(dh1.as_bytes(), dh2.as_bytes(), dh3.as_bytes());

    Ok(InitiatorHandshake {
        shared_secret,
        initiator_dh_public: initiator.dh_public(),
        ephemeral_public,
    })
}

/// Parent side of the handshake. Uses the child's identity and ephemeral public
/// keys plus the parent's own private keys to derive the same shared secret.
pub fn responder_handshake(
    responder: &DeviceIdentity,
    prekey: &ResponderPrekey,
    initiator_dh_public: &PublicKey,
    ephemeral_public: &PublicKey,
) -> [u8; 32] {
    let dh1 = prekey.secret.diffie_hellman(initiator_dh_public);
    let dh2 = responder.dh_secret.diffie_hellman(ephemeral_public);
    let dh3 = prekey.secret.diffie_hellman(ephemeral_public);
    derive_shared_secret(dh1.as_bytes(), dh2.as_bytes(), dh3.as_bytes())
}

// --- Byte-oriented API for FFI and storage ---------------------------------
//
// The mobile and desktop agents store keys in the platform keystore and cross an
// FFI boundary, so they need to export and import key material as plain bytes.
// These helpers keep the crypto types contained in this crate; callers (the
// apc-ffi layer) deal only in fixed-size byte arrays.

impl DeviceIdentity {
    /// Reconstruct an identity from stored secret bytes.
    pub fn from_secret_bytes(dh_secret: [u8; 32], signing_secret: [u8; 32]) -> DeviceIdentity {
        DeviceIdentity {
            dh_secret: StaticSecret::from(dh_secret),
            signing: SigningKey::from_bytes(&signing_secret),
        }
    }

    /// Export the secret bytes for storage in the platform keystore.
    pub fn secret_bytes(&self) -> ([u8; 32], [u8; 32]) {
        (self.dh_secret.to_bytes(), self.signing.to_bytes())
    }

    pub fn dh_public_bytes(&self) -> [u8; 32] {
        self.dh_public().to_bytes()
    }

    pub fn verifying_key_bytes(&self) -> [u8; 32] {
        self.verifying_key().to_bytes()
    }
}

impl ResponderPrekey {
    pub fn from_secret_bytes(secret: [u8; 32]) -> ResponderPrekey {
        ResponderPrekey {
            secret: StaticSecret::from(secret),
        }
    }

    pub fn secret_bytes(&self) -> [u8; 32] {
        self.secret.to_bytes()
    }

    pub fn public_bytes(&self) -> [u8; 32] {
        self.public().to_bytes()
    }
}

impl PrekeyBundle {
    /// (identity_dh_public, identity_verifying_key, signed_prekey_public, prekey_signature)
    pub fn raw_parts(&self) -> ([u8; 32], [u8; 32], [u8; 32], [u8; 64]) {
        (
            self.identity_dh_public.to_bytes(),
            self.identity_verifying_key.to_bytes(),
            self.signed_prekey_public.to_bytes(),
            self.prekey_signature.to_bytes(),
        )
    }

    pub fn from_raw(
        identity_dh: [u8; 32],
        identity_vk: [u8; 32],
        signed_prekey: [u8; 32],
        signature: [u8; 64],
    ) -> Result<PrekeyBundle, PairingError> {
        Ok(PrekeyBundle {
            identity_dh_public: PublicKey::from(identity_dh),
            identity_verifying_key: VerifyingKey::from_bytes(&identity_vk)
                .map_err(|_| PairingError::BadKeyLength)?,
            signed_prekey_public: PublicKey::from(signed_prekey),
            prekey_signature: Signature::from_bytes(&signature),
        })
    }
}

impl InitiatorHandshake {
    pub fn initiator_dh_public_bytes(&self) -> [u8; 32] {
        self.initiator_dh_public.to_bytes()
    }

    pub fn ephemeral_public_bytes(&self) -> [u8; 32] {
        self.ephemeral_public.to_bytes()
    }
}

/// Responder handshake from raw public-key bytes (the FFI-friendly variant).
pub fn responder_handshake_raw(
    responder: &DeviceIdentity,
    prekey: &ResponderPrekey,
    initiator_dh_public: [u8; 32],
    ephemeral_public: [u8; 32],
) -> [u8; 32] {
    responder_handshake(
        responder,
        prekey,
        &PublicKey::from(initiator_dh_public),
        &PublicKey::from(ephemeral_public),
    )
}

fn message_key(shared_secret: &[u8; 32]) -> Key {
    let hk = Hkdf::<Sha256>::new(None, shared_secret);
    let mut okm = [0u8; 32];
    hk.expand(b"apc/alert/key/v1", &mut okm)
        .expect("32 is a valid HKDF length");
    *Key::from_slice(&okm)
}

/// Seal a plaintext with the channel key. The associated data is authenticated
/// but not encrypted, so it binds the routing fields to the ciphertext.
fn seal(shared_secret: &[u8; 32], plaintext: &[u8], aad: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let cipher = XChaCha20Poly1305::new(&message_key(shared_secret));
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, Payload { msg: plaintext, aad })
        .expect("encryption does not fail with a valid key and nonce");
    (nonce.to_vec(), ciphertext)
}

fn open(
    shared_secret: &[u8; 32],
    nonce: &[u8],
    ciphertext: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, PairingError> {
    if nonce.len() != 24 {
        return Err(PairingError::BadKeyLength);
    }
    let cipher = XChaCha20Poly1305::new(&message_key(shared_secret));
    cipher
        .decrypt(XNonce::from_slice(nonce), Payload { msg: ciphertext, aad })
        .map_err(|_| PairingError::Aead)
}

/// The transport wrapper that travels through the backend. Carries ciphertext
/// only. Matches the AlertEnvelope in packages/proto/openapi.yaml.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AlertEnvelope {
    pub v: u32,
    pub id: String,
    pub child_device_id: String,
    pub recipient_id: String,
    pub created_at: String,
    pub ciphertext: String,
    pub nonce: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ephemeral_public_key: Option<String>,
}

fn envelope_aad(
    v: u32,
    id: &str,
    child_device_id: &str,
    recipient_id: &str,
    created_at: &str,
) -> Vec<u8> {
    format!("{v}|{id}|{child_device_id}|{recipient_id}|{created_at}").into_bytes()
}

/// Seal a validated alert into an envelope addressed to a parent device.
pub fn seal_alert(
    shared_secret: &[u8; 32],
    alert: &apc_alert::Alert,
    recipient_id: &str,
) -> Result<AlertEnvelope, PairingError> {
    alert.validate().map_err(|e| PairingError::Alert(e.to_string()))?;
    let plaintext = alert
        .to_json()
        .map_err(|e| PairingError::Alert(e.to_string()))?;
    let aad = envelope_aad(
        ENVELOPE_VERSION,
        &alert.id,
        &alert.child_device_id,
        recipient_id,
        &alert.created_at,
    );
    let (nonce, ciphertext) = seal(shared_secret, plaintext.as_bytes(), &aad);
    Ok(AlertEnvelope {
        v: ENVELOPE_VERSION,
        id: alert.id.clone(),
        child_device_id: alert.child_device_id.clone(),
        recipient_id: recipient_id.to_string(),
        created_at: alert.created_at.clone(),
        ciphertext: B64.encode(ciphertext),
        nonce: B64.encode(nonce),
        ephemeral_public_key: None,
    })
}

/// Open an envelope back into a validated alert. Fails if the ciphertext was
/// tampered with, the key is wrong, or the routing fields do not match the
/// sealed alert.
pub fn open_alert(
    shared_secret: &[u8; 32],
    envelope: &AlertEnvelope,
) -> Result<apc_alert::Alert, PairingError> {
    let nonce = B64
        .decode(&envelope.nonce)
        .map_err(|e| PairingError::Decode(e.to_string()))?;
    let ciphertext = B64
        .decode(&envelope.ciphertext)
        .map_err(|e| PairingError::Decode(e.to_string()))?;
    let aad = envelope_aad(
        envelope.v,
        &envelope.id,
        &envelope.child_device_id,
        &envelope.recipient_id,
        &envelope.created_at,
    );
    let plaintext = open(shared_secret, &nonce, &ciphertext, &aad)?;
    let json = String::from_utf8(plaintext).map_err(|e| PairingError::Decode(e.to_string()))?;
    let alert = apc_alert::Alert::from_json(&json).map_err(|e| PairingError::Alert(e.to_string()))?;
    if alert.id != envelope.id || alert.child_device_id != envelope.child_device_id {
        return Err(PairingError::IdentityMismatch);
    }
    Ok(alert)
}

#[cfg(test)]
mod tests {
    use super::*;
    use apc_alert::{Alert, Category, Modality, Severity, ALERT_SCHEMA_VERSION};

    fn pair() -> ([u8; 32], [u8; 32]) {
        let child = DeviceIdentity::generate();
        let parent = DeviceIdentity::generate();
        let parent_prekey = ResponderPrekey::generate();
        let bundle = build_prekey_bundle(&parent, &parent_prekey);

        let init = initiator_handshake(&child, &bundle).expect("handshake should verify");
        let parent_secret = responder_handshake(
            &parent,
            &parent_prekey,
            &init.initiator_dh_public,
            &init.ephemeral_public,
        );
        (init.shared_secret, parent_secret)
    }

    fn sample_alert() -> Alert {
        Alert {
            v: ALERT_SCHEMA_VERSION,
            id: "11111111-1111-4111-8111-111111111111".to_string(),
            child_device_id: "device-abc".to_string(),
            category: Category::Grooming,
            severity: Severity::Severe,
            created_at: "2026-06-26T00:00:00Z".to_string(),
            source: "instagram".to_string(),
            modality: Some(Modality::Text),
            confidence: Some(0.81),
            rationale: Some("escalating secrecy and gift offers".to_string()),
            snippet: Some("don't tell your...".to_string()),
        }
    }

    #[test]
    fn both_sides_derive_the_same_secret() {
        let (child_secret, parent_secret) = pair();
        assert_eq!(child_secret, parent_secret);
    }

    #[test]
    fn tampered_prekey_signature_is_rejected() {
        let parent = DeviceIdentity::generate();
        let parent_prekey = ResponderPrekey::generate();
        let mut bundle = build_prekey_bundle(&parent, &parent_prekey);
        // Swap in a different prekey that the signature does not cover.
        bundle.signed_prekey_public = ResponderPrekey::generate().public();
        let child = DeviceIdentity::generate();
        assert!(initiator_handshake(&child, &bundle).is_err());
    }

    #[test]
    fn alert_seals_and_opens_for_the_paired_parent() {
        let (child_secret, parent_secret) = pair();
        let alert = sample_alert();
        let envelope = seal_alert(&child_secret, &alert, "parent-device-1").unwrap();
        let opened = open_alert(&parent_secret, &envelope).unwrap();
        assert_eq!(alert, opened);
    }

    #[test]
    fn envelope_carries_no_plaintext() {
        let (child_secret, _) = pair();
        let alert = sample_alert();
        let envelope = seal_alert(&child_secret, &alert, "parent-device-1").unwrap();
        let wire = serde_json::to_string(&envelope).unwrap();
        // The snippet and rationale must not be readable in the relayed envelope.
        assert!(!wire.contains("don't tell your"));
        assert!(!wire.contains("escalating secrecy"));
        assert!(!wire.contains("instagram"));
    }

    #[test]
    fn a_stranger_secret_cannot_open() {
        let (child_secret, _) = pair();
        let alert = sample_alert();
        let envelope = seal_alert(&child_secret, &alert, "parent-device-1").unwrap();
        let stranger = [9u8; 32];
        assert!(open_alert(&stranger, &envelope).is_err());
    }

    #[test]
    fn tampering_with_routing_fields_breaks_decryption() {
        let (child_secret, parent_secret) = pair();
        let alert = sample_alert();
        let mut envelope = seal_alert(&child_secret, &alert, "parent-device-1").unwrap();
        // A relay that rewrites the recipient breaks the AEAD because the
        // routing fields are authenticated as associated data.
        envelope.recipient_id = "attacker-device".to_string();
        assert!(open_alert(&parent_secret, &envelope).is_err());
    }

    #[test]
    fn envelope_round_trips_as_json() {
        let (child_secret, _) = pair();
        let envelope = seal_alert(&child_secret, &sample_alert(), "parent-device-1").unwrap();
        let wire = serde_json::to_string(&envelope).unwrap();
        let back: AlertEnvelope = serde_json::from_str(&wire).unwrap();
        assert_eq!(envelope, back);
    }

    #[test]
    fn byte_export_import_preserves_the_handshake() {
        // Exercise the FFI-facing byte API end to end: export every key, rebuild
        // from bytes on the "other side", and confirm both sides still agree.
        let child = DeviceIdentity::generate();
        let parent = DeviceIdentity::generate();
        let parent_prekey = ResponderPrekey::generate();
        let bundle = build_prekey_bundle(&parent, &parent_prekey);

        let (cd, cs) = child.secret_bytes();
        let child2 = DeviceIdentity::from_secret_bytes(cd, cs);
        let (idh, ivk, spk, sig) = bundle.raw_parts();
        let bundle2 = PrekeyBundle::from_raw(idh, ivk, spk, sig).unwrap();

        let init = initiator_handshake(&child2, &bundle2).unwrap();

        let (pd, ps) = parent.secret_bytes();
        let parent2 = DeviceIdentity::from_secret_bytes(pd, ps);
        let prekey2 = ResponderPrekey::from_secret_bytes(parent_prekey.secret_bytes());
        let parent_secret = responder_handshake_raw(
            &parent2,
            &prekey2,
            init.initiator_dh_public_bytes(),
            init.ephemeral_public_bytes(),
        );

        assert_eq!(init.shared_secret, parent_secret);
    }
}
