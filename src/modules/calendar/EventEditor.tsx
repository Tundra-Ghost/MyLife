// Create or edit one event. Editing a repeating event changes every occurrence.
import { useState } from "react";
import { api, errorText, type CalEvent } from "../../core/api";
import { Button, useEscape, useRefresh } from "../../core/ui";
import { fromInputs, REPEATS, toDateInput, toTimeInput } from "./dates";

const field = "w-full rounded-lg bg-slate-900 px-3 py-2 text-sm outline-none ring-1 ring-slate-700 focus:ring-brand";

export type EditorTarget = { kind: "new"; start: Date; end: Date; allDay: boolean } | { kind: "edit"; event: CalEvent };

export function EventEditor({ target, onClose }: { target: EditorTarget; onClose: () => void }) {
  const refresh = useRefresh();
  const ev = target.kind === "edit" ? target.event : null;
  const start0 = ev ? new Date(ev.starts_at) : (target as { start: Date }).start;
  const end0 = ev ? (ev.ends_at ? new Date(ev.ends_at) : start0) : (target as { end: Date }).end;

  const [title, setTitle] = useState(ev?.title ?? "");
  const [date, setDate] = useState(toDateInput(start0));
  const [startTime, setStartTime] = useState(toTimeInput(start0));
  const [endTime, setEndTime] = useState(toTimeInput(end0));
  const [allDay, setAllDay] = useState(ev ? ev.all_day : (target as { allDay: boolean }).allDay);
  const [rrule, setRrule] = useState<string | null>(ev?.rrule ?? null);
  const [error, setError] = useState("");
  useEscape(onClose);

  // A rule we don't have a label for (from Google, later) still shows.
  const repeats = REPEATS.some((r) => r.rrule === rrule) ? REPEATS : [...REPEATS, { label: "Custom", rrule }];

  async function save() {
    setError("");
    const start = allDay ? fromInputs(date, "00:00") : fromInputs(date, startTime);
    let end = allDay ? fromInputs(date, "23:59") : fromInputs(date, endTime);
    // An end time earlier than the start means it ends the next day.
    if (end < start) end = new Date(end.getTime() + 24 * 3600 * 1000);
    const input = {
      title,
      starts_at: start.toISOString(),
      ends_at: end.toISOString(),
      all_day: allDay,
      rrule,
      item_id: ev?.item_id ?? null,
    };
    try {
      if (ev) await api.updateEvent(ev.id, input);
      else await api.createEvent(input);
      refresh();
      onClose();
    } catch (e) {
      setError(errorText(e));
    }
  }

  async function remove() {
    if (!ev) return;
    const what = ev.rrule ? "every time this repeats" : "this event";
    if (!window.confirm(`Delete ${what}? You can't undo this from the app yet.`)) return;
    await api.archiveEvent(ev.id);
    refresh();
    onClose();
  }

  return (
    <div className="fixed inset-0 z-10 flex items-start justify-center bg-black/50 pt-[12vh]" onClick={onClose}>
      <div className="w-full max-w-md space-y-3 rounded-xl bg-slate-950 p-6 ring-1 ring-slate-800" onClick={(e) => e.stopPropagation()}>
        <input
          autoFocus
          className={`${field} text-lg`}
          placeholder="Title"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && save()}
        />
        <div className="grid grid-cols-3 gap-2">
          <input className={`${field} col-span-3`} type="date" value={date} onChange={(e) => setDate(e.target.value)} />
          {!allDay && (
            <>
              <input className={field} type="time" value={startTime} onChange={(e) => setStartTime(e.target.value)} />
              <span className="self-center text-center text-sm text-slate-500">to</span>
              <input className={field} type="time" value={endTime} onChange={(e) => setEndTime(e.target.value)} />
            </>
          )}
        </div>
        <label className="flex items-center gap-2 text-sm text-slate-300">
          <input type="checkbox" checked={allDay} onChange={(e) => setAllDay(e.target.checked)} />
          All day
        </label>
        <select className={field} value={rrule ?? ""} onChange={(e) => setRrule(e.target.value || null)}>
          {repeats.map((r) => (
            <option key={r.label} value={r.rrule ?? ""}>
              {r.label}
            </option>
          ))}
        </select>
        {ev?.rrule && <p className="text-xs text-slate-500">Changes apply to every time this repeats.</p>}
        {error && <p className="text-sm text-accent">{error}</p>}
        <div className="flex gap-2">
          <Button kind="primary" onClick={save}>
            Save
          </Button>
          <Button onClick={onClose}>Cancel</Button>
          <div className="flex-1" />
          {ev && <Button onClick={remove}>Delete</Button>}
        </div>
      </div>
    </div>
  );
}
