//! WebAssembly bindings to the core.
//!
//! Lets the parent PWA run the same audited crypto as the native agents: pair,
//! and open alert envelopes in the browser. The backend never sees plaintext, so
//! decryption has to happen on the parent's device, including in a browser.
//!
//! The surface mirrors packages/ffi but uses JSON strings so it is trivial to
//! call from TypeScript.

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

const B64: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Identity {
    dh_secret: String,
    signing_secret: String,
    dh_public: String,
    verifying_key: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Prekey {
    secret: String,
    public: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Bundle {
    identity_dh: String,
    identity_vk: String,
    signed_prekey: String,
    signature: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Handshake {
    shared_secret: String,
    initiator_dh_public: String,
    ephemeral_public: String,
}

fn err(e: impl ToString) -> JsError {
    JsError::new(&e.to_string())
}

fn arr<const N: usize>(s: &str) -> Result<[u8; N], JsError> {
    let v = B64.decode(s).map_err(err)?;
    v.try_into().map_err(|_| err(format!("expected {N} bytes")))
}

fn from_json<T: for<'de> Deserialize<'de>>(s: &str) -> Result<T, JsError> {
    serde_json::from_str(s).map_err(err)
}

fn to_json<T: Serialize>(v: &T) -> Result<String, JsError> {
    serde_json::to_string(v).map_err(err)
}

fn identity_of(i: &Identity) -> Result<apc_pairing::DeviceIdentity, JsError> {
    Ok(apc_pairing::DeviceIdentity::from_secret_bytes(
        arr::<32>(&i.dh_secret)?,
        arr::<32>(&i.signing_secret)?,
    ))
}

#[wasm_bindgen(js_name = generateIdentity)]
pub fn generate_identity() -> Result<String, JsError> {
    let id = apc_pairing::DeviceIdentity::generate();
    let (dh, sign) = id.secret_bytes();
    to_json(&Identity {
        dh_secret: B64.encode(dh),
        signing_secret: B64.encode(sign),
        dh_public: B64.encode(id.dh_public_bytes()),
        verifying_key: B64.encode(id.verifying_key_bytes()),
    })
}

#[wasm_bindgen(js_name = generatePrekey)]
pub fn generate_prekey() -> Result<String, JsError> {
    let pk = apc_pairing::ResponderPrekey::generate();
    to_json(&Prekey {
        secret: B64.encode(pk.secret_bytes()),
        public: B64.encode(pk.public_bytes()),
    })
}

#[wasm_bindgen(js_name = buildBundle)]
pub fn build_bundle(identity_json: &str, prekey_json: &str) -> Result<String, JsError> {
    let id = identity_of(&from_json::<Identity>(identity_json)?)?;
    let pk = apc_pairing::ResponderPrekey::from_secret_bytes(arr::<32>(&from_json::<Prekey>(prekey_json)?.secret)?);
    let (idh, ivk, spk, sig) = apc_pairing::build_prekey_bundle(&id, &pk).raw_parts();
    to_json(&Bundle {
        identity_dh: B64.encode(idh),
        identity_vk: B64.encode(ivk),
        signed_prekey: B64.encode(spk),
        signature: B64.encode(sig),
    })
}

#[wasm_bindgen(js_name = initiatorHandshake)]
pub fn initiator_handshake(identity_json: &str, bundle_json: &str) -> Result<String, JsError> {
    let id = identity_of(&from_json::<Identity>(identity_json)?)?;
    let b = from_json::<Bundle>(bundle_json)?;
    let bundle = apc_pairing::PrekeyBundle::from_raw(
        arr::<32>(&b.identity_dh)?,
        arr::<32>(&b.identity_vk)?,
        arr::<32>(&b.signed_prekey)?,
        arr::<64>(&b.signature)?,
    )
    .map_err(err)?;
    let h = apc_pairing::initiator_handshake(&id, &bundle).map_err(err)?;
    to_json(&Handshake {
        shared_secret: B64.encode(h.shared_secret),
        initiator_dh_public: B64.encode(h.initiator_dh_public_bytes()),
        ephemeral_public: B64.encode(h.ephemeral_public_bytes()),
    })
}

#[wasm_bindgen(js_name = responderHandshake)]
pub fn responder_handshake(
    identity_json: &str,
    prekey_secret: &str,
    initiator_dh_public: &str,
    ephemeral_public: &str,
) -> Result<String, JsError> {
    let id = identity_of(&from_json::<Identity>(identity_json)?)?;
    let pk = apc_pairing::ResponderPrekey::from_secret_bytes(arr::<32>(prekey_secret)?);
    let ss = apc_pairing::responder_handshake_raw(&id, &pk, arr::<32>(initiator_dh_public)?, arr::<32>(ephemeral_public)?);
    Ok(B64.encode(ss))
}

#[wasm_bindgen(js_name = sealAlert)]
pub fn seal_alert(shared_secret: &str, alert_json: &str, recipient_id: &str) -> Result<String, JsError> {
    let ss = arr::<32>(shared_secret)?;
    let alert = apc_alert::Alert::from_json(alert_json).map_err(|e| err(e.to_string()))?;
    let envelope = apc_pairing::seal_alert(&ss, &alert, recipient_id).map_err(err)?;
    to_json(&envelope)
}

/// Open an alert envelope back into a validated alert (JSON). This is what the
/// parent dashboard calls to read an alert in the browser.
#[wasm_bindgen(js_name = openAlert)]
pub fn open_alert(shared_secret: &str, envelope_json: &str) -> Result<String, JsError> {
    let ss = arr::<32>(shared_secret)?;
    let envelope: apc_pairing::AlertEnvelope = from_json(envelope_json)?;
    let alert = apc_pairing::open_alert(&ss, &envelope).map_err(err)?;
    alert.to_json().map_err(|e| err(e.to_string()))
}
