# ai

The on-device content safety layer. All inference runs on the child device. Raw content is analyzed in memory and discarded. Only structured alerts ([packages/alert-schema](../packages/alert-schema)) leave the device. This is the privacy promise and the legal safe harbor.

## Pipeline

```
TEXT  -> Stage 0: regex/lexicon + Detoxify-ONNX (toxicity/profanity)   [every msg, cheap, in-RAM]
           | below threshold and no risk-lexicon hit -> drop, nothing stored
           v escalate (small fraction of messages)
         Stage 1: on-device small LLM (Llama Guard 3 INT4 or ShieldGemma 2B)
                  via ExecuTorch (mobile) / llama.cpp (desktop)
                  structured rubric -> {category, severity, rationale}
                  sliding conversation window for grooming and self-harm
IMAGE -> NudeNet v3 (ONNX, ~7MB) -> Falconsai ViT confirmer   [in-RAM, sampled frames for video]
           v
       build alert {category, severity, ts, source, optional short snippet}
       raw media/text discarded; never written to disk or network
```

The Stage-0 gate is what makes an on-device LLM viable on a phone battery: only a small fraction of messages reach Stage 1.

## Folders

- [text-pipeline](text-pipeline): Stage-0 classifier plus Stage-1 LLM rubric, sliding-window grooming.
- [image-pipeline](image-pipeline): NudeNet plus ViT confirmer, frame sampling for video.
- [runtime](runtime): llama.cpp / ExecuTorch / ONNX Runtime / Core ML integration glue.
- [models](models): bundled quantized models (downloaded as assets, not committed).

## Hard rule: no CSAM pipeline

The image track is a generic nudity classifier, explicitly not a CSAM classifier. No perceptual-hash matching, no media persistence, no cloud routing. See [compliance/legal-notes.md](../compliance/legal-notes.md).
