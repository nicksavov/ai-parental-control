# apc-wasm

WebAssembly bindings to the core so the parent PWA can decrypt alerts in the browser. The backend only relays ciphertext, so the parent has to decrypt on their own device, including in a browser, with the same audited code the native agents use.

## Surface

JSON-string functions mirroring [../ffi](../ffi) ([src/lib.rs](src/lib.rs)): `generateIdentity`, `generatePrekey`, `buildBundle`, `initiatorHandshake`, `responderHandshake`, `sealAlert`, `openAlert`. Randomness uses the Web Crypto API via the `getrandom` `js` feature.

## Build and regenerate the bindings

```
# 1. Compile the crate to wasm
cargo build -p apc-wasm --target wasm32-unknown-unknown --release

# 2. Generate the web bindings into the PWA (checked in)
wasm-bindgen packages/target/wasm32-unknown-unknown/release/apc_wasm.wasm \
  --out-dir apps/parent-web/src/wasm --target web
```

Regenerate whenever the exported surface changes. The PWA imports the result from `src/wasm/apc_wasm.js` (see `apps/parent-web/src/api/crypto.ts`); Vite emits the `.wasm` as an asset.

## Verified

A Node round-trip (nodejs-target bindings) pairs two identities, seals an alert, opens it, checks the envelope leaks no plaintext, and confirms a stranger secret cannot open it. The PWA's in-browser self-test does the same client side.
