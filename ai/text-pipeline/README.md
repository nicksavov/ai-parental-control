# text-pipeline

Stage-0 cheap classifier (Detoxify-ONNX plus lexicons) on every message, escalating only a small fraction to the Stage-1 on-device LLM with a structured rubric. Sliding conversation window for grooming and self-harm. Output is a structured alert; raw text is discarded. See [../README.md](../README.md).
