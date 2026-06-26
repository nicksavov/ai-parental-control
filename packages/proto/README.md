# proto

The agent and parent to backend API, defined in [openapi.yaml](openapi.yaml). Covers auth, child creation, pairing (code issue and claim), policy get/put, and encrypted alert submit/stream. The alert envelope carries ciphertext only; the plaintext inside follows [../alert-schema](../alert-schema).
