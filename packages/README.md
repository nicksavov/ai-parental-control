# packages

Shared, cross-platform contracts and the security-critical core. Build these first: every agent and the backend depend on them, and they are cheap to define now but expensive to retrofit later.

- [alert-schema](alert-schema): the alert contract and the no-raw-content invariant.
- [policy-model](policy-model): the rules a parent sets for a child.
- [pairing-protocol](pairing-protocol): device pairing and end-to-end encryption.
- [proto](proto): the agent and parent to backend API (OpenAPI).

## Implementation plan

The runtime implementation of pairing, the policy model, alert construction, the E2E crypto, and the sync client lives in one **Rust core** compiled to each platform via UniFFI (Swift and Kotlin bindings) and native linkage on Windows and macOS. Keeping the crypto and the alert "no content" invariant in a single audited library, rather than four copies, is the whole point.

AI model inference is the exception: it stays platform-native (Core ML, ExecuTorch, ONNX Runtime) because the runtimes differ. The Rust core orchestrates the pipeline and builds alerts from its output, but does not run the models itself.
