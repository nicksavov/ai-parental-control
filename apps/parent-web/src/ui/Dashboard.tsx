import { useState } from "react";
import type { ApiClient } from "../api/client";
import type { AlertEnvelope, PairingCode } from "../api/types";

export function Dashboard({ client, onLogout }: { client: ApiClient; onLogout: () => void }) {
  const [childId, setChildId] = useState<string | null>(null);
  const [code, setCode] = useState<PairingCode | null>(null);
  const [envelopes, setEnvelopes] = useState<AlertEnvelope[]>([]);
  const [error, setError] = useState<string | null>(null);

  async function guard(fn: () => Promise<void>) {
    setError(null);
    try {
      await fn();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Something went wrong");
    }
  }

  return (
    <section className="card">
      <div className="row between">
        <h2>Family</h2>
        <button className="secondary" onClick={onLogout}>Sign out</button>
      </div>
      {error && <p className="error">{error}</p>}

      <div className="block">
        <button onClick={() => guard(async () => setChildId(await client.createChild()))}>Add a child</button>
        {childId && <p className="mono">Child id: {childId}</p>}
      </div>

      {childId && (
        <div className="block">
          <button onClick={() => guard(async () => setCode(await client.createPairingCode(childId)))}>
            Generate a pairing code
          </button>
          {code && (
            <div className="pairing">
              <p>Enter this on the child device. It expires at {new Date(code.expiresAt).toLocaleString()}.</p>
              <p className="code">{code.code}</p>
            </div>
          )}
        </div>
      )}

      <div className="block">
        <button onClick={() => guard(async () => setEnvelopes(await client.streamAlerts()))}>Check for alerts</button>
        {envelopes.length === 0 ? (
          <p className="muted">No new alerts.</p>
        ) : (
          <ul className="alerts">
            {envelopes.map((e) => (
              <li key={e.id}>
                <span className="badge">encrypted</span>
                <span className="mono">{e.id}</span>
                <span className="muted"> {new Date(e.createdAt).toLocaleString()}</span>
              </li>
            ))}
          </ul>
        )}
        <p className="muted small">
          Alerts arrive end-to-end encrypted. Decrypting them in the browser uses a WASM build of the shared core
          (packages/ffi); that wiring is the next step.
        </p>
      </div>
    </section>
  );
}
