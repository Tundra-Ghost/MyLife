// Top level: lock screen until the app password is entered, then the main layout.
import { useEffect } from "react";
import { useQuery } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import { api } from "./api";
import { LockScreen } from "./LockScreen";
import { Layout } from "./Layout";
import { useUi } from "./store";

export default function App() {
  const status = useQuery({ queryKey: ["status"], queryFn: api.appStatus });
  const setCaptureOpen = useUi((s) => s.setCaptureOpen);

  // The Ctrl+Shift+Space hotkey (handled in Rust) sends this event.
  useEffect(() => {
    const off = listen("quick-capture", () => setCaptureOpen(true));
    return () => {
      off.then((f) => f());
    };
  }, [setCaptureOpen]);

  if (status.isPending) return null;
  if (status.isError) return <p className="p-8 text-slate-300">MyLife could not start. Restart the app.</p>;
  if (!status.data.unlocked) return <LockScreen firstRun={!status.data.created} onDone={() => status.refetch()} />;
  return <Layout />;
}
