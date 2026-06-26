package org.aiparentalcontrol.child.pairing

import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import org.json.JSONArray
import org.json.JSONObject

/**
 * Drives pairing and alert relay against the backend, using the shared Rust core
 * (packages/ffi) for all crypto. The backend only ever sees opaque ciphertext.
 *
 * The generated UniFFI bindings are referenced as `uniffi.apc_ffi.*`:
 *   generateIdentity(), buildBundle(), initiatorHandshake(),
 *   responderHandshake(), sealAlert(), openAlert().
 * They are not imported here yet because the bindings are generated from
 * packages/ffi during the native build (see packages/ffi/README.md).
 */
class PairingManager(
    private val backendBaseUrl: String,
    private val secrets: SecretStore,
    private val http: OkHttpClient = OkHttpClient(),
) {

    private val json = "application/json".toMediaType()

    /**
     * Pair this device using a code scanned from the parent. The parent's signed
     * prekey bundle is delivered alongside the code (in the QR payload).
     *
     * Returns the childDeviceId on success.
     */
    fun pair(code: String, parentBundleJson: String): String {
        // 1. Generate or load our identity, persist secrets in the keystore.
        val identity = secrets.loadOrCreateIdentity() // calls uniffi.apc_ffi.generateIdentity() once

        // 2. Claim the code so the backend registers this device.
        val claim = postJson(
            "$backendBaseUrl/v1/pairing/claim",
            JSONObject()
                .put("code", code)
                .put("devicePublicKey", identity.dhPublic)
                .put("platform", "android")
                .put("capabilities", JSONArray(listOf("filtering", "screenTime", "imageNudity"))),
            auth = null,
        )
        val childDeviceId = claim.getString("childDeviceId")
        val deviceCredential = claim.getString("deviceCredential")

        // 3. Run the initiator handshake against the parent's bundle. The shared
        //    secret never leaves the device.
        // val hs = uniffi.apc_ffi.initiatorHandshake(identity, parseBundle(parentBundleJson))
        // secrets.saveSharedSecret(hs.sharedSecret)

        // 4. Persist the device credential and start overt monitoring.
        secrets.saveDeviceCredential(deviceCredential)
        return childDeviceId
    }

    /** Seal and submit an alert. The plaintext is built and encrypted on-device. */
    fun submitAlert(alertJson: String, recipientId: String) {
        // val envelope = uniffi.apc_ffi.sealAlert(secrets.sharedSecret(), alertJson, recipientId)
        // postRaw("$backendBaseUrl/v1/alerts", envelope, auth = secrets.deviceCredential())
    }

    private fun postJson(url: String, body: JSONObject, auth: String?): JSONObject {
        val builder = Request.Builder().url(url).post(body.toString().toRequestBody(json))
        if (auth != null) builder.header("Authorization", "Bearer $auth")
        http.newCall(builder.build()).execute().use { resp ->
            val text = resp.body?.string().orEmpty()
            require(resp.isSuccessful) { "pairing request failed: ${resp.code} $text" }
            return JSONObject(text)
        }
    }
}

/** Persists identity secrets, the shared secret, and the device credential in
 *  the Android Keystore. Implementation TODO in v0. */
interface SecretStore {
    data class Identity(val dhSecret: String, val signingSecret: String, val dhPublic: String, val verifyingKey: String)

    fun loadOrCreateIdentity(): Identity
    fun saveSharedSecret(b64: String)
    fun sharedSecret(): String
    fun saveDeviceCredential(token: String)
    fun deviceCredential(): String
}
