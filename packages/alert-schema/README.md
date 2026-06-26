# alert-schema

The contract for an alert: what the child device sends the parent when something fires. This is the most important invariant in the project.

## The no-content invariant

An alert carries **structured metadata only**: category, severity, timestamp, source app, modality, confidence, and an optional short rationale. The single exception is `snippet`, which:

- exists for `text` modality only,
- is capped at 280 characters,
- is never the full message,
- is never present for `image` modality (we never include any part of a flagged image), and
- is shown only when the parent chooses to view it.

An alert MUST NOT carry raw media, full message bodies, file paths, base64 blobs, screenshots, thumbnails, or URLs pointing to a child's content. The JSON Schema sets `additionalProperties: false` everywhere to enforce this structurally.

Why this matters: if alerts carried content, the relay backend would become a store of minors' messages and images. That crosses the CSAM (strict liability), wiretap, and COPPA lines described in [/compliance](../../compliance). On-device analysis plus metadata-only alerts is what keeps the operator off that surface.

## Transport

The object defined by [alert.schema.json](alert.schema.json) is the **plaintext** payload. Before it leaves the device it is end-to-end encrypted to the parent using the keys established at pairing (see [../pairing-protocol](../pairing-protocol)). The backend relays the ciphertext and cannot read it.

## Required tests (must stay green in CI)

1. A forbidden-keys test: assert an alert can never serialize any of `rawText`, `body`, `fullMessage`, `image`, `imageData`, `base64`, `mediaPath`, `filePath`, `mediaUrl`, `contentUrl`, `attachment`, `thumbnail`, `screenshot`.
2. An image-modality test: assert `snippet` is rejected when `modality` is `image`.
3. A length test: assert `snippet` and `rationale` over 280 characters are rejected.

These belong to the shared Rust core that owns alert construction, so all four agents inherit them.
