// Tasks screen: Active, Inbox, Catch-up, and Done tabs.
import { useState, type FormEvent } from "react";
import { useQuery } from "@tanstack/react-query";
import { api, errorText, type Energy, type Task, type TaskView } from "../../core/api";
import { Button, ShortList, useRefresh } from "../../core/ui";
import { TaskDetail } from "./TaskDetail";
import { TaskRow } from "./TaskRow";

const TABS: { id: TaskView; label: string }[] = [
  { id: "active", label: "Active" },
  { id: "inbox", label: "Inbox" },
  { id: "catch_up", label: "Catch-up" },
  { id: "done", label: "Done" },
];

export function TasksView() {
  const [view, setView] = useState<TaskView>("active");
  const [lowOnly, setLowOnly] = useState(false);
  const [open, setOpen] = useState<Task | null>(null);
  const [title, setTitle] = useState("");
  const [error, setError] = useState("");
  const refresh = useRefresh();
  const tasks = useQuery({ queryKey: ["tasks", view], queryFn: () => api.tasks(view) });

  // Energy matching: when drained, show only low-energy tasks.
  const list = (tasks.data ?? []).filter((t) => !lowOnly || t.energy === ("low" satisfies Energy));

  async function add(e: FormEvent) {
    e.preventDefault();
    if (!title.trim()) return;
    try {
      await api.createTask({ title });
      setTitle("");
      setError("");
      refresh();
    } catch (err) {
      setError(errorText(err));
    }
  }

  async function clearOne(t: Task) {
    await api.reschedule(t.id);
    refresh();
  }

  return (
    <div className="mx-auto max-w-2xl space-y-6">
      <h1 className="text-2xl font-semibold">Tasks</h1>

      <form onSubmit={add} className="flex gap-2">
        <input
          className="flex-1 rounded-lg bg-slate-900 px-4 py-2 outline-none ring-1 ring-slate-700 focus:ring-brand"
          placeholder="Add a task"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
        />
        <Button kind="primary" type="submit">
          Add
        </Button>
      </form>
      {error && <p className="text-sm text-accent">{error}</p>}

      <div className="flex flex-wrap items-center gap-2">
        {TABS.map((t) => (
          <button
            key={t.id}
            onClick={() => setView(t.id)}
            className={`rounded-full px-4 py-1.5 text-sm ${view === t.id ? "bg-slate-700" : "bg-slate-900 text-slate-400"}`}
          >
            {t.label}
          </button>
        ))}
        <label className="ml-auto flex items-center gap-2 text-sm text-slate-400">
          <input type="checkbox" checked={lowOnly} onChange={(e) => setLowOnly(e.target.checked)} />
          Low energy only
        </label>
      </div>

      {view === "catch_up" && list.length > 0 && (
        <p className="text-sm text-slate-400">These slipped. That's okay. Pick one and move it to today.</p>
      )}

      <ShortList
        items={list}
        empty={view === "catch_up" ? "Nothing to catch up on." : "Nothing here."}
        render={(t) => (
          <div key={t.id} className="flex items-center gap-2">
            <div className="flex-1">
              <TaskRow task={t} onOpen={setOpen} />
            </div>
            {view === "catch_up" && <Button onClick={() => clearOne(t)}>Today</Button>}
          </div>
        )}
      />

      {open && <TaskDetail key={open.id} task={open} onClose={() => setOpen(null)} />}
    </div>
  );
}
