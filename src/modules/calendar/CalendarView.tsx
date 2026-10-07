// Calendar screen: day, week, and month views, plus a task list you can
// drag onto the calendar to time-block.
import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api, errorText, type Occurrence } from "../../core/api";
import { Button, ShortList, useRefresh } from "../../core/ui";
import { addDays, atMinutes, step, viewDays, type CalView } from "./dates";
import { EventEditor, type EditorTarget } from "./EventEditor";
import { MonthGrid } from "./MonthGrid";
import { TASK_MIME, TimeGrid } from "./TimeGrid";

const VIEWS: { id: CalView; label: string; key: string }[] = [
  { id: "day", label: "Day", key: "d" },
  { id: "week", label: "Week", key: "w" },
  { id: "month", label: "Month", key: "m" },
];

function useMinuteClock() {
  const [now, setNow] = useState(new Date());
  useEffect(() => {
    const t = setInterval(() => setNow(new Date()), 60000);
    return () => clearInterval(t);
  }, []);
  return now;
}

function title(view: CalView, days: Date[], anchor: Date) {
  const fmt = (o: Intl.DateTimeFormatOptions, d: Date) => new Intl.DateTimeFormat("en-US", o).format(d);
  if (view === "day") return fmt({ weekday: "long", month: "long", day: "numeric" }, days[0]);
  if (view === "month") return fmt({ month: "long", year: "numeric" }, anchor);
  return `${fmt({ month: "short", day: "numeric" }, days[0])} to ${fmt({ month: "short", day: "numeric" }, days[6])}`;
}

export function CalendarView() {
  const [view, setView] = useState<CalView>("week");
  const [anchor, setAnchor] = useState(new Date());
  const [editor, setEditor] = useState<EditorTarget | null>(null);
  const [error, setError] = useState("");
  // Click-to-place: pick a task, then click a time. Same result as dragging.
  const [placing, setPlacing] = useState<{ id: string; title: string } | null>(null);
  const now = useMinuteClock();
  const refresh = useRefresh();

  const days = viewDays(view, anchor);
  const from = days[0];
  const to = addDays(days[days.length - 1], 1);
  const events = useQuery({
    queryKey: ["events", from.toISOString(), to.toISOString()],
    queryFn: () => api.events(from, to),
  });
  const tasks = useQuery({ queryKey: ["tasks", "active"], queryFn: () => api.tasks("active") });

  // Keyboard: D, W, M switch views. T jumps to today. Arrows move.
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") setPlacing(null);
      if (editor || e.ctrlKey || e.altKey || e.metaKey) return;
      if ((e.target as HTMLElement).closest("input, textarea, select")) return;
      const v = VIEWS.find((x) => x.key === e.key.toLowerCase());
      if (v) setView(v.id);
      if (e.key.toLowerCase() === "t") setAnchor(new Date());
      if (e.key === "ArrowLeft") setAnchor((a) => step(view, a, -1));
      if (e.key === "ArrowRight") setAnchor((a) => step(view, a, 1));
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [view, editor]);

  async function open(o: Occurrence) {
    try {
      setEditor({ kind: "edit", event: await api.event(o.event_id) });
    } catch (e) {
      setError(errorText(e));
    }
  }

  async function dropTask(taskId: string, start: Date) {
    setError("");
    try {
      await api.blockTask(taskId, start);
      setPlacing(null);
      refresh();
    } catch (e) {
      setError(errorText(e));
    }
  }

  const occ = events.data ?? [];

  return (
    <div className="flex h-full gap-6">
      <div className="flex min-w-0 flex-1 flex-col gap-4">
        <div className="flex flex-wrap items-center gap-2">
          <Button onClick={() => setAnchor(new Date())} title="T">
            Today
          </Button>
          <Button onClick={() => setAnchor(step(view, anchor, -1))} title="Left arrow">
            ‹
          </Button>
          <Button onClick={() => setAnchor(step(view, anchor, 1))} title="Right arrow">
            ›
          </Button>
          <h1 className="ml-2 text-xl font-semibold">{title(view, days, anchor)}</h1>
          <div className="ml-auto flex gap-1">
            {VIEWS.map((v) => (
              <button
                key={v.id}
                title={v.key.toUpperCase()}
                onClick={() => setView(v.id)}
                className={`rounded-full px-4 py-1.5 text-sm ${view === v.id ? "bg-slate-700" : "bg-slate-900 text-slate-400"}`}
              >
                {v.label}
              </button>
            ))}
          </div>
          <Button
            kind="primary"
            onClick={() => {
              const s = new Date(now);
              s.setMinutes(0, 0, 0);
              s.setHours(s.getHours() + 1);
              setEditor({ kind: "new", start: s, end: new Date(s.getTime() + 3600000), allDay: false });
            }}
          >
            New event
          </Button>
        </div>
        {error && <p className="text-sm text-accent">{error}</p>}
        {placing && (
          <p className="text-sm text-slate-300">
            Click a time to place <span className="font-semibold">{placing.title}</span>. Esc to cancel.
          </p>
        )}

        {view === "month" ? (
          <MonthGrid
            days={days}
            month={anchor.getMonth()}
            events={occ}
            now={now}
            onDay={(d) => {
              if (placing) return dropTask(placing.id, atMinutes(d, 9 * 60));
              setAnchor(d);
              setView("day");
            }}
            onOpen={open}
            onDropTask={dropTask}
          />
        ) : (
          <TimeGrid
            days={days}
            events={occ}
            now={now}
            onSlot={(s) =>
              placing
                ? dropTask(placing.id, s)
                : setEditor({ kind: "new", start: s, end: new Date(s.getTime() + 3600000), allDay: false })
            }
            onOpen={open}
            onDropTask={dropTask}
          />
        )}
      </div>

      <aside className="w-56 shrink-0 space-y-3 overflow-y-auto">
        <h2 className="text-sm font-semibold uppercase tracking-wide text-slate-400">Tasks</h2>
        <p className="text-xs text-slate-500">Drag one onto the calendar to block time for it. Or click it, then click a time.</p>
        <ShortList
          items={tasks.data ?? []}
          empty="No open tasks."
          render={(t) => (
            <div
              key={t.id}
              role="button"
              tabIndex={0}
              onClick={() => setPlacing(placing?.id === t.id ? null : { id: t.id, title: t.title })}
              onKeyDown={(e) => e.key === "Enter" && setPlacing({ id: t.id, title: t.title })}
              draggable
              onDragStart={(e) => {
                e.dataTransfer.setData(TASK_MIME, t.id);
                e.dataTransfer.effectAllowed = "copy";
              }}
              className={`cursor-grab rounded-lg px-3 py-2 text-sm active:cursor-grabbing ${
                placing?.id === t.id ? "bg-slate-700 ring-1 ring-brand" : "bg-slate-900"
              }`}
            >
              <div className="truncate">{t.title}</div>
              <div className="text-xs text-slate-500">{t.est_minutes ?? 30} min</div>
            </div>
          )}
        />
      </aside>

      {editor && <EditorWrap target={editor} onClose={() => setEditor(null)} />}
    </div>
  );
}

// Keyed so the form resets when a different event opens.
function EditorWrap({ target, onClose }: { target: EditorTarget; onClose: () => void }) {
  const key = target.kind === "edit" ? target.event.id : target.start.toISOString();
  return <EventEditor key={key} target={target} onClose={onClose} />;
}
