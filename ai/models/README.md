# models

Model binaries are downloaded as release assets, not committed (see the repo `.gitignore`). Each keeps its own open license; record it here and in the project NOTICE file.

| Model | Use | Format | License |
|---|---|---|---|
| Detoxify (unbiased-toxic-roberta) | Stage-0 text gate: toxicity, profanity, threats | ONNX | Apache-2.0 |
| Llama Guard 3 (INT4) | Stage-1 text context: self-harm, sexual, violence | GGUF / ExecuTorch | Llama license (open weights) |
| ShieldGemma 2B | Stage-1 alternative | GGUF | Gemma license (open weights) |
| NudeNet v3 | Image nudity detection | ONNX | open (verify per release) |
| Falconsai/nsfw_image_detection | Image confirmer (ViT) | ONNX | open (verify per release) |

When adding or swapping a model, confirm its license permits redistribution and on-device use, and update this table plus NOTICE.
