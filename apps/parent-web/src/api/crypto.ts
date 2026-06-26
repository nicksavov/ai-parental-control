// Browser-side cryptography via the shared core compiled to WebAssembly
// (packages/wasm). The backend only relays ciphertext, so the parent decrypts
// alerts here, on their own device, with the same audited code the native agents
// use.

import init, {
  buildBundle,
  generateIdentity,
  generatePrekey,
  initiatorHandshake,
  openAlert,
  responderHandshake,
  sealAlert,
} from "../wasm/apc_wasm.js";

let ready: Promise<unknown> | null = null;

function ensureReady(): Promise<unknown> {
  if (!ready) ready = init();
  return ready;
}

/** Decrypt an alert envelope (JSON) into an alert (JSON) with the shared secret
 *  established at pairing. */
export async function decryptAlert(sharedSecret: string, envelopeJson: string): Promise<string> {
  await ensureReady();
  return openAlert(sharedSecret, envelopeJson);
}

interface Prekey {
  secret: string;
  public: string;
}
interface Handshake {
  sharedSecret: string;
  initiatorDhPublic: string;
  ephemeralPublic: string;
}

/**
 * Proof that decryption runs in this browser: generate two identities, pair them,
 * seal a demo alert, and open it, entirely client side. Returns the decrypted
 * alert JSON.
 */
export async function cryptoSelfTest(): Promise<string> {
  await ensureReady();
  const child = generateIdentity();
  const parent = generateIdentity();
  const prekey = JSON.parse(generatePrekey()) as Prekey;

  const bundle = buildBundle(parent, JSON.stringify(prekey));
  const handshake = JSON.parse(initiatorHandshake(child, bundle)) as Handshake;
  const parentSecret = responderHandshake(
    parent,
    prekey.secret,
    handshake.initiatorDhPublic,
    handshake.ephemeralPublic,
  );

  const alert = JSON.stringify({
    v: 1,
    id: "demo",
    childDeviceId: "demo-device",
    category: "profanity",
    severity: "all",
    createdAt: "2026-06-26T00:00:00Z",
    source: "selftest",
    modality: "text",
    snippet: "hello from wasm",
  });

  const envelope = sealAlert(handshake.sharedSecret, alert, "parent-demo");
  return openAlert(parentSecret, envelope);
}
