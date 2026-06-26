# apc-ffi

The UniFFI binding layer. Exposes the security-critical core (device identity, the X3DH-style handshake, and alert sealing) to Swift and Kotlin so the iOS, macOS, and Android agents call one audited Rust implementation instead of reimplementing crypto per platform.

## Surface

A flat, string-based API ([src/lib.rs](src/lib.rs)). All key material crosses the boundary as base64 so agents can store secrets in the platform keystore. No crypto types leak across FFI.

- `generate_identity()`, `generate_prekey()`
- `build_bundle(identity, prekey)` (parent shows this at pairing)
- `initiator_handshake(identity, bundle)` (child)
- `responder_handshake(identity, prekey_secret, initiator_dh_public, ephemeral_public)` (parent)
- `seal_alert(shared_secret, alert_json, recipient_id)` (child)
- `open_alert(shared_secret, envelope_json)` (parent)

`alert_json` follows [../alert-schema](../alert-schema); the envelope follows [../proto/openapi.yaml](../proto/openapi.yaml).

## Build and test

```
cargo test -p apc-ffi
```

The crate builds `cdylib` and `staticlib` for mobile linking plus an `rlib` for tests. Two tests prove the full pair-seal-open flow works through the FFI surface only, and that a stranger secret cannot open an envelope.

## Generating Kotlin and Swift bindings

This crate ships its own `uniffi-bindgen` binary ([src/bin/uniffi-bindgen.rs](src/bin/uniffi-bindgen.rs)), so no separate tool install is needed. Build the library, then generate against it:

```
cargo build -p apc-ffi
# Kotlin (checked in to the Android app):
cargo run -p apc-ffi --bin uniffi-bindgen -- generate \
  --library target/debug/apc_ffi.dll --language kotlin \
  --out-dir ../../apps/android-child/app/src/main/kotlin
# Swift (for the iOS app, generated when that module is built):
cargo run -p apc-ffi --bin uniffi-bindgen -- generate \
  --library target/debug/libapc_ffi.dylib --language swift --out-dir gen
```

The generated Kotlin lands at `uniffi/apc_ffi/apc_ffi.kt` (package `uniffi.apc_ffi`) and is checked into the Android module. Regenerate it whenever this crate's exported surface changes. The Android side links the native `apc_ffi` library as a `jniLibs` `.so` and depends on JNA (see the app `build.gradle.kts`).
