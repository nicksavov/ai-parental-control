# api

The Go coordination service: auth, family graph, pairing, policy distribution, encrypted alert relay. Implements [../../packages/proto/openapi.yaml](../../packages/proto/openapi.yaml). A Dockerfile is added with the v0 implementation (the compose file builds this directory). Stores ciphertext envelopes and metadata only, never raw content.
