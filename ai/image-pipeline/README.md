# image-pipeline

On-device nudity detection: NudeNet v3 detector plus a Falconsai ViT confirmer, sampled frames for video. Generic nudity only, never a CSAM classifier. Alert plus delete prompt; the image is never persisted or transmitted. See [../README.md](../README.md).

The orchestration lives in Rust in [../../packages/ai-core](../../packages/ai-core) (crate `apc-ai`): the `NudityDetector` trait and `ImagePipeline`, which produce an image `AlertDraft` that carries no part of the image. The model that implements `NudityDetector` (NudeNet plus ViT via ONNX / Core ML) lives here.
