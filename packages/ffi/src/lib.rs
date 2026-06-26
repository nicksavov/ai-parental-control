//! UniFFI binding layer.
//!
//! Exposes the security-critical core (identity, the X3DH-style handshake, and
//! alert sealing) to Swift and Kotlin through a flat, string-based surface. All
//! key material crosses the boundary as base64 so the agents can store it in the
//! platform keystore. No crypto types leak across FFI; this crate only wraps
//! apc-pairing and apc-alert.
//!
//! Generate bindings (after installing a matching uniffi-bindgen) with, for
//! example:
//!   uniffi-bindgen generate --library target/debug/apc_ffi.dll --language kotlin --out-dir gen
//!   uniffi-bindgen generate --library target/debug/libapc_ffi.dylib --language swift --out-dir gen

use base64::Engine as _;

uniffi::setup_scaffolding!();

const B64: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD;

#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum FfiError {
    #[error("decode error: {0}")]
    Decode(String),
    #[error("crypto error: {0}")]
    Crypto(String),
    #[error("alert error: {0}")]
    Alert(String),
}

/// A device's full key material, base64 encoded. The two secrets go in the
/// platform keystore; the publics are shared during pairing.
#[derive(uniffi::Record)]
pub struct IdentityMaterial {
    pub dh_secret: String,
    pub signing_secret: String,
    pub dh_public: String,
    pub verifying_key: String,
}

/// A parent's prekey: a secret to keep until pairing completes, and the public
/// to put in the bundle.
#[derive(uniffi::Record)]
pub struct PrekeyMaterial {
    pub secret: String,
    pub public: String,
}

/// The signed prekey bundle a parent shows during pairing, base64 encoded.
#[derive(uniffi::Record)]
pub struct BundleWire {
    pub identity_dh: String,
    pub identity_vk: String,
    pub signed_prekey: String,
    pub signature: String,
}

/// The initiator's output: the shared secret plus the public values the
/// responder needs to derive the same secret.
#[derive(uniffi::Record)]
pub struct HandshakeResult {
    pub shared_secret: String,
    pub initiator_dh_public: String,
    pub ephemeral_public: String,
}

fn b64_to_array<const N: usize>(s: &str) -> Result<[u8; N], FfiError> {
    let v = B64.decode(s).map_err(|e| FfiError::Decode(e.to_string()))?;
    v.try_into()
        .map_err(|_| FfiError::Decode(format!("expected {N} bytes")))
}

fn identity_from(m: &IdentityMaterial) -> Result<apc_pairing::DeviceIdentity, FfiError> {
    Ok(apc_pairing::DeviceIdentity::from_secret_bytes(
        b64_to_array::<32>(&m.dh_secret)?,
        b64_to_array::<32>(&m.signing_secret)?,
    ))
}

/// Generate a fresh device identity.
#[uniffi::export]
pub fn generate_identity() -> IdentityMaterial {
    let id = apc_pairing::DeviceIdentity::generate();
    let (dh, sign) = id.secret_bytes();
    IdentityMaterial {
        dh_secret: B64.encode(dh),
        signing_secret: B64.encode(sign),
        dh_public: B64.encode(id.dh_public_bytes()),
        verifying_key: B64.encode(id.verifying_key_bytes()),
    }
}

/// Generate a parent prekey.
#[uniffi::export]
pub fn generate_prekey() -> PrekeyMaterial {
    let pk = apc_pairing::ResponderPrekey::generate();
    PrekeyMaterial {
        secret: B64.encode(pk.secret_bytes()),
        public: B64.encode(pk.public_bytes()),
    }
}

/// Parent: build the signed prekey bundle to show during pairing.
#[uniffi::export]
pub fn build_bundle(
    identity: IdentityMaterial,
    prekey: PrekeyMaterial,
) -> Result<BundleWire, FfiError> {
    let id = identity_from(&identity)?;
    let pk = apc_pairing::ResponderPrekey::from_secret_bytes(b64_to_array::<32>(&prekey.secret)?);
    let (idh, ivk, spk, sig) = apc_pairing::build_prekey_bundle(&id, &pk).raw_parts();
    Ok(BundleWire {
        identity_dh: B64.encode(idh),
        identity_vk: B64.encode(ivk),
        signed_prekey: B64.encode(spk),
        signature: B64.encode(sig),
    })
}

/// Child: run the initiator side of the handshake against a parent's bundle.
#[uniffi::export]
pub fn initiator_handshake(
    initiator: IdentityMaterial,
    bundle: BundleWire,
) -> Result<HandshakeResult, FfiError> {
    let id = identity_from(&initiator)?;
    let b = apc_pairing::PrekeyBundle::from_raw(
        b64_to_array::<32>(&bundle.identity_dh)?,
        b64_to_array::<32>(&bundle.identity_vk)?,
        b64_to_array::<32>(&bundle.signed_prekey)?,
        b64_to_array::<64>(&bundle.signature)?,
    )
    .map_err(|e| FfiError::Crypto(e.to_string()))?;
    let h = apc_pairing::initiator_handshake(&id, &b).map_err(|e| FfiError::Crypto(e.to_string()))?;
    Ok(HandshakeResult {
        shared_secret: B64.encode(h.shared_secret),
        initiator_dh_public: B64.encode(h.initiator_dh_public_bytes()),
        ephemeral_public: B64.encode(h.ephemeral_public_bytes()),
    })
}

/// Parent: run the responder side and return the shared secret (base64).
#[uniffi::export]
pub fn responder_handshake(
    responder: IdentityMaterial,
    prekey_secret: String,
    initiator_dh_public: String,
    ephemeral_public: String,
) -> Result<String, FfiError> {
    let id = identity_from(&responder)?;
    let pk = apc_pairing::ResponderPrekey::from_secret_bytes(b64_to_array::<32>(&prekey_secret)?);
    let ss = apc_pairing::responder_handshake_raw(
        &id,
        &pk,
        b64_to_array::<32>(&initiator_dh_public)?,
        b64_to_array::<32>(&ephemeral_public)?,
    );
    Ok(B64.encode(ss))
}

/// Child: seal a validated alert (as JSON) into an envelope (as JSON).
#[uniffi::export]
pub fn seal_alert(
    shared_secret: String,
    alert_json: String,
    recipient_id: String,
) -> Result<String, FfiError> {
    let ss = b64_to_array::<32>(&shared_secret)?;
    let alert = apc_alert::Alert::from_json(&alert_json).map_err(|e| FfiError::Alert(e.to_string()))?;
    let envelope =
        apc_pairing::seal_alert(&ss, &alert, &recipient_id).map_err(|e| FfiError::Crypto(e.to_string()))?;
    serde_json::to_string(&envelope).map_err(|e| FfiError::Alert(e.to_string()))
}

/// Parent: open an envelope (as JSON) back into a validated alert (as JSON).
#[uniffi::export]
pub fn open_alert(shared_secret: String, envelope_json: String) -> Result<String, FfiError> {
    let ss = b64_to_array::<32>(&shared_secret)?;
    let envelope: apc_pairing::AlertEnvelope =
        serde_json::from_str(&envelope_json).map_err(|e| FfiError::Decode(e.to_string()))?;
    let alert = apc_pairing::open_alert(&ss, &envelope).map_err(|e| FfiError::Crypto(e.to_string()))?;
    alert.to_json().map_err(|e| FfiError::Alert(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALERT_JSON: &str = r#"{"v":1,"id":"a1","childDeviceId":"d1","category":"grooming","severity":"severe","createdAt":"2026-06-26T00:00:00Z","source":"instagram","modality":"text","snippet":"don't tell"}"#;

    #[test]
    fn full_ffi_flow_pairs_seals_and_opens() {
        // Everything the mobile agents do, through the flat FFI surface only.
        let child = generate_identity();
        let parent = generate_identity();
        let prekey = generate_prekey();

        let bundle = build_bundle(
            IdentityMaterial {
                dh_secret: parent.dh_secret.clone(),
                signing_secret: parent.signing_secret.clone(),
                dh_public: parent.dh_public.clone(),
                verifying_key: parent.verifying_key.clone(),
            },
            PrekeyMaterial {
                secret: prekey.secret.clone(),
                public: prekey.public.clone(),
            },
        )
        .unwrap();

        let init = initiator_handshake(
            IdentityMaterial {
                dh_secret: child.dh_secret.clone(),
                signing_secret: child.signing_secret.clone(),
                dh_public: child.dh_public.clone(),
                verifying_key: child.verifying_key.clone(),
            },
            bundle,
        )
        .unwrap();

        let parent_secret = responder_handshake(
            parent,
            prekey.secret,
            init.initiator_dh_public.clone(),
            init.ephemeral_public.clone(),
        )
        .unwrap();

        assert_eq!(init.shared_secret, parent_secret);

        let envelope = seal_alert(init.shared_secret, ALERT_JSON.to_string(), "parent-1".to_string()).unwrap();
        // The relayed envelope must not leak the snippet.
        assert!(!envelope.contains("don't tell"));

        let opened = open_alert(parent_secret, envelope).unwrap();
        let a: serde_json::Value = serde_json::from_str(ALERT_JSON).unwrap();
        let b: serde_json::Value = serde_json::from_str(&opened).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn stranger_secret_cannot_open() {
        let child = generate_identity();
        let parent = generate_identity();
        let prekey = generate_prekey();
        let bundle = build_bundle(
            IdentityMaterial {
                dh_secret: parent.dh_secret.clone(),
                signing_secret: parent.signing_secret.clone(),
                dh_public: parent.dh_public.clone(),
                verifying_key: parent.verifying_key.clone(),
            },
            PrekeyMaterial { secret: prekey.secret.clone(), public: prekey.public.clone() },
        )
        .unwrap();
        let init = initiator_handshake(child, bundle).unwrap();
        let envelope = seal_alert(init.shared_secret, ALERT_JSON.to_string(), "parent-1".to_string()).unwrap();
        let stranger = B64.encode([7u8; 32]);
        assert!(open_alert(stranger, envelope).is_err());
    }
}
