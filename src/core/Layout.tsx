// Three zones: module sidebar, center work area, collapsible agenda panel.
import { useEffect } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { getVersion } from "@tauri-apps/api/app";
import { api } from "./api";
import { AgendaPanel } from "./AgendaPanel";
import { QuickCapture } from "./QuickCapture";
import { TodayView } from "./TodayView";
import { useUi, type Screen } from "./store";
import { TasksView } from "../modules/tasks/TasksView";
import { CalendarView } from "../modules/calendar/CalendarView";
import { RulesView } from "./RulesView";
import { SettingsView } from "./SettingsView";

const NAV: { id: Screen; label: string; key: string }[] = [
  { id: "today", label: "Today", key: "1" },
  { id: "calendar", label: "Calendar", key: "2" },
  { id: "tasks", label: "Tasks", key: "3" },
  { id: "rules", label: "Reminders", key: "4" },
];

// Modules that are planned but not built yet. Shown dimmed so the map is clear.
const COMING = ["Chores", "Gym", "Money"];

export function Layout() {
  const { screen, setScreen, setCaptureOpen, agendaOpen, toggleAgenda } = useUi();
  const qc = useQueryClient();
  // Shown in the sidebar so you can tell which build is installed.
  const version = useQuery({ queryKey: ["version"], queryFn: getVersion, staleTime: Infinity });

  // Keyboard: Alt+1 to Alt+4 switch screens, Ctrl+N opens quick capture.
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.altKey) {
        const hit = NAV.find((n) => n.key === e.key);
        if (hit) {
          e.preventDefault();
          setScreen(hit.id);
        }
      }
      if (e.ctrlKey && e.key.toLowerCase() === "n") {
        e.preventDefault();
        setCaptureOpen(true);
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [setScreen, setCaptureOpen]);

  async function lock() {
    await api.lock();
    qc.clear();
    qc.invalidateQueries({ queryKey: ["status"] });
  }

  return (
    <div className="flex h-full">
      <nav className="flex w-48 shrink-0 flex-col gap-1 border-r border-slate-800 p-3">
        <div className="mb-4 px-3 text-lg font-semibold text-brand">MyLife</div>
        {NAV.map((n) => (
          <button
            key={n.id}
            onClick={() => setScreen(n.id)}
            title={`Alt+${n.key}`}
            className={`rounded-lg px-3 py-2 text-left text-sm ${
              screen === n.id ? "bg-slate-800 text-slate-100" : "text-slate-400 hover:bg-slate-900"
            }`}
          >
            {n.label}
          </button>
        ))}
        <div className="mt-4 px-3 text-xs uppercase tracking-wide text-slate-600">Coming next</div>
        {COMING.map((c) => (
          <div key={c} className="px-3 py-1 text-sm text-slate-700">
            {c}
          </div>
        ))}
        <div className="mt-auto space-y-1">
          <button
            onClick={() => setCaptureOpen(true)}
            className="w-full rounded-lg px-3 py-2 text-left text-sm text-slate-300 hover:bg-slate-900"
            title="Ctrl+Shift+Space from anywhere"
          >
            + Quick capture
          </button>
          <button
            onClick={() => setScreen("settings")}
            className={`w-full rounded-lg px-3 py-2 text-left text-sm hover:bg-slate-900 ${screen === "settings" ? "text-slate-100" : "text-slate-500"}`}
          >
            Settings
          </button>
          <button onClick={toggleAgenda} className="w-full rounded-lg px-3 py-2 text-left text-sm text-slate-500 hover:bg-slate-900">
            {agendaOpen ? "Hide agenda" : "Show agenda"}
          </button>
          <button onClick={lock} className="w-full rounded-lg px-3 py-2 text-left text-sm text-slate-500 hover:bg-slate-900">
            Lock
          </button>
          {version.data && <div className="px-3 pt-2 text-xs text-slate-700">v{version.data}</div>}
        </div>
      </nav>

      <main className="min-w-0 flex-1 overflow-y-auto p-8">
        {screen === "today" ? (
          <TodayView />
        ) : screen === "calendar" ? (
          <CalendarView />
        ) : screen === "tasks" ? (
          <TasksView />
        ) : screen === "rules" ? (
          <RulesView />
        ) : (
          <SettingsView />
        )}
      </main>

      {/* The calendar already shows the day, so the agenda panel steps aside there. */}
      {agendaOpen && screen !== "calendar" && <AgendaPanel />}
      <QuickCapture />
    </div>
  );
}
