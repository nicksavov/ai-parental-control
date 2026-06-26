# text-pipeline

Stage-0 cheap classifier (Detoxify-ONNX plus lexicons) on every message, escalating only a small fraction to the Stage-1 on-device LLM with a structured rubric. Sliding conversation window for grooming and self-harm. Output is a structured alert; raw text is discarded. See [../README.md](../README.md).

The cross-platform Stage-0 rule gate and the pipeline orchestration are implemented in Rust in [../../packages/ai-core](../../packages/ai-core) (crate `apc-ai`): `TextGate::evaluate` and `evaluate_conversation`, producing an `AlertDraft` that becomes an `apc_alert::Alert`. This directory holds the model-backed Stage-1 (Detoxify ONNX plus the on-device LLM) that the gate escalates to.
