// Shapes shared with the backend. The policy mirrors packages/policy-model and
// the envelope mirrors packages/proto/openapi.yaml.

export interface Tokens {
  accessToken: string;
  refreshToken?: string;
}

export interface PairingCode {
  code: string;
  recipientId: string;
  expiresAt: string;
}

// The relayed alert envelope. Ciphertext only; the parent decrypts it locally
// with the shared core (a WASM build of packages/ffi). The backend never sees
// the plaintext, and neither does this type.
export interface AlertEnvelope {
  v: number;
  id: string;
  childDeviceId: string;
  recipientId: string;
  createdAt: string;
  ciphertext: string;
  nonce: string;
  ephemeralPublicKey?: string;
}

// The decrypted alert (after local decryption). Matches packages/alert-schema.
export interface Alert {
  v: number;
  id: string;
  childDeviceId: string;
  category: string;
  severity: "info" | "all" | "severe";
  createdAt: string;
  source: string;
  modality?: string;
  confidence?: number;
  rationale?: string;
  snippet?: string;
}
