// First run creates the app password. Later runs ask for it.
import { useState, type FormEvent } from "react";
import { api, errorText } from "./api";
import { Button } from "./ui";

const MIN_LEN = 8; // Matches MIN_PASSWORD_LEN in src-tauri/src/db/mod.rs.

export function LockScreen({ firstRun, onDone }: { firstRun: boolean; onDone: () => void }) {
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  async function submit(e: FormEvent) {
    e.preventDefault();
    setError("");
    if (firstRun) {
      if (password.length < MIN_LEN) return setError(`Use at least ${MIN_LEN} characters.`);
      if (password !== confirm) return setError("Passwords don't match.");
    }
    setBusy(true);
    try {
      await (firstRun ? api.createPassword(password) : api.unlock(password));
      setPassword("");
      setConfirm("");
      onDone();
    } catch (err) {
      setError(errorText(err));
    } finally {
      setBusy(false);
    }
  }

  const input = "w-full rounded-lg bg-slate-900 px-4 py-3 text-base outline-none ring-1 ring-slate-700 focus:ring-brand";

  return (
    <div className="flex h-full items-center justify-center p-8">
      <form onSubmit={submit} className="w-full max-w-sm space-y-4">
        <h1 className="text-2xl font-semibold">{firstRun ? "Welcome to MyLife" : "Unlock MyLife"}</h1>
        {firstRun && (
          <p className="text-sm text-slate-400">
            Pick an app password. It encrypts all your data. If you lose it, the data can't be recovered.
          </p>
        )}
        <input
          className={input}
          type="password"
          autoFocus
          placeholder="App password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
        />
        {firstRun && (
          <input
            className={input}
            type="password"
            placeholder="Type it again"
            value={confirm}
            onChange={(e) => setConfirm(e.target.value)}
          />
        )}
        {error && <p className="text-sm text-accent">{error}</p>}
        <Button kind="primary" type="submit" disabled={busy || !password}>
          {firstRun ? "Create password" : "Unlock"}
        </Button>
      </form>
    </div>
  );
}
