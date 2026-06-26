# runtime

Integration glue for the on-device inference engines: llama.cpp / ExecuTorch on mobile, ONNX Runtime plus DirectML on Windows, Core ML on Apple platforms. The shared Rust core orchestrates the pipeline and builds alerts; the models run here, natively per platform. See [../README.md](../README.md).
