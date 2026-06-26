import { useMemo, useState } from "react";
import { ApiClient } from "./api/client";
import { Login } from "./ui/Login";
import { Dashboard } from "./ui/Dashboard";

const API_URL = (import.meta.env.VITE_API_URL as string | undefined) ?? "http://localhost:8080";

export function App() {
  const client = useMemo(() => new ApiClient(API_URL), []);
  const [authed, setAuthed] = useState(false);

  return (
    <main className="app">
      <header>
        <h1>Parental Control</h1>
        <p className="tagline">Free, open-source, on-device. Overt by design.</p>
      </header>
      {authed ? (
        <Dashboard client={client} onLogout={() => { client.logout(); setAuthed(false); }} />
      ) : (
        <Login client={client} onAuthed={() => setAuthed(true)} />
      )}
    </main>
  );
}
