import { useState } from "react";
import type { ApiClient } from "../api/client";

export function Login({ client, onAuthed }: { client: ApiClient; onAuthed: () => void }) {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function run(action: "login" | "register") {
    setBusy(true);
    setError(null);
    try {
      if (action === "register") await client.register(email, password);
      else await client.login(email, password);
      onAuthed();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Something went wrong");
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="card">
      <h2>Parent sign in</h2>
      <label>
        Email
        <input type="email" value={email} onChange={(e) => setEmail(e.target.value)} autoComplete="username" />
      </label>
      <label>
        Password
        <input
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          autoComplete="current-password"
        />
      </label>
      {error && <p className="error">{error}</p>}
      <div className="row">
        <button disabled={busy} onClick={() => run("login")}>Sign in</button>
        <button disabled={busy} className="secondary" onClick={() => run("register")}>Create account</button>
      </div>
    </section>
  );
}
