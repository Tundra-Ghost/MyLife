// Settings: backups and restore.
import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { open } from "@tauri-apps/plugin-dialog";
import { api, errorText } from "./api";
import { formatDay, formatTime } from "./time";
import { localDate } from "./capture/parse";
import { Button, Section, ShortList, useRefresh } from "./ui";

function when(iso: string) {
  return `${formatDay(localDate(new Date(iso)))} ${formatTime(iso)}`;
}

/** "mylife-20261006T051500Z-daily.db" to a readable label. */
function label(name: string) {
  const m = name.match(/^mylife-(\d{4})(\d{2})(\d{2})T(\d{2})(\d{2})(\d{2})Z-(.+)\.db$/);
  if (!m) return name;
  const iso = `${m[1]}-${m[2]}-${m[3]}T${m[4]}:${m[5]}:${m[6]}Z`;
  const kind = { daily: "Daily", manual: "Manual", prerestore: "Before a restore" }[m[7]] ?? (m[7].startsWith("premigrate") ? "Before an update" : m[7]);
  return `${when(iso)} · ${kind}`;
}

function RestoreRow({ name, bytes }: { name: string; bytes: number }) {
  const [asking, setAsking] = useState(false);
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const qc = useQueryClient();

  async function restore() {
    setError("");
    setBusy(true);
    try {
      await api.restoreBackup(name, password);
      setPassword("");
      setAsking(false);
      // Everything changed, so reload all data.
      qc.invalidateQueries();
      window.alert("Restored. Your data before the restore was saved as a backup too.");
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="space-y-2 rounded-lg bg-slate-900 px-3 py-2">
      <div className="flex items-center gap-2">
        <div className="min-w-0 flex-1 text-sm">
          <div className="truncate">{label(name)}</div>
          <div className="text-xs text-slate-500">{Math.max(1, Math.round(bytes / 1024))} KB</div>
        </div>
        <Button onClick={() => setAsking(!asking)}>Restore</Button>
      </div>
      {asking && (
        <div className="space-y-2">
          <p className="text-xs text-slate-400">
            This replaces all your current data with this backup. Your current data is backed up first. Enter your app password to go ahead.
          </p>
          <div className="flex gap-2">
            <input
              type="password"
              className="flex-1 rounded-lg bg-slate-950 px-3 py-2 text-sm outline-none ring-1 ring-slate-700 focus:ring-brand"
              placeholder="App password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
            />
            <Button kind="action" onClick={restore} disabled={!password || busy}>
              Restore this backup
            </Button>
          </div>
          {error && <p className="text-sm text-accent">{error}</p>}
        </div>
      )}
    </div>
  );
}

export function SettingsView() {
  const info = useQuery({ queryKey: ["backups"], queryFn: api.backupInfo });
  const refresh = useRefresh();
  const [error, setError] = useState("");

  async function pickFolder() {
    setError("");
    const dir = await open({ directory: true, title: "Pick a folder for backups" });
    if (typeof dir !== "string") return;
    try {
      await api.setBackupDir(dir);
      refresh();
    } catch (e) {
      setError(errorText(e));
    }
  }

  async function now() {
    setError("");
    try {
      await api.backupNow();
      refresh();
    } catch (e) {
      setError(errorText(e));
    }
  }

  const d = info.data;
  return (
    <div className="mx-auto max-w-2xl space-y-8">
      <h1 className="text-2xl font-semibold">Settings</h1>
      <Section title="Backups">
        <p className="text-sm text-slate-400">
          MyLife backs up once a day. Backups stay encrypted with your app password. A cloud-synced folder like OneDrive keeps a copy off this PC.
        </p>
        {d && (
          <div className="space-y-1 text-sm">
            <div className="truncate">
              <span className="text-slate-500">Folder: </span>
              {d.dir}
            </div>
            <div>
              <span className="text-slate-500">Last backup: </span>
              {d.last_backup_at ? when(d.last_backup_at) : "Not yet"}
            </div>
          </div>
        )}
        <div className="flex gap-2">
          <Button onClick={pickFolder}>Change folder</Button>
          <Button kind="primary" onClick={now}>
            Back up now
          </Button>
        </div>
        {error && <p className="text-sm text-accent">{error}</p>}
        <ShortList items={d?.files ?? []} empty="No backups in this folder yet." render={(f) => <RestoreRow key={f.name} {...f} />} />
      </Section>
    </div>
  );
}
