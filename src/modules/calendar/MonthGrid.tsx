// Month view: 6 weeks. Click a day to open it. Drop a task on a day to block 9 AM.
import type { DragEvent } from "react";
import type { Occurrence } from "../../core/api";
import { formatTime } from "../../core/time";
import { atMinutes, sameDay } from "./dates";
import { TASK_MIME } from "./TimeGrid";

const PER_CELL = 3;

export function MonthGrid({
  days,
  month,
  events,
  now,
  onDay,
  onOpen,
  onDropTask,
}: {
  days: Date[];
  month: number;
  events: Occurrence[];
  now: Date;
  onDay: (d: Date) => void;
  onOpen: (o: Occurrence) => void;
  onDropTask: (taskId: string, start: Date) => void;
}) {
  return (
    <div className="grid flex-1 grid-cols-7 grid-rows-[auto_repeat(6,1fr)] overflow-hidden rounded-lg ring-1 ring-slate-800">
      {["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"].map((d) => (
        <div key={d} className="border-b border-slate-800 py-1 text-center text-xs text-slate-500">
          {d}
        </div>
      ))}
      {days.map((d) => {
        const dayEnd = atMinutes(d, 24 * 60).getTime();
        const items = events.filter((e) => {
          const s = new Date(e.starts_at).getTime();
          const en = e.ends_at ? new Date(e.ends_at).getTime() : s;
          return s < dayEnd && (en > d.getTime() || s >= d.getTime());
        });
        const drop = (e: DragEvent) => {
          const id = e.dataTransfer.getData(TASK_MIME);
          if (!id) return;
          e.preventDefault();
          onDropTask(id, atMinutes(d, 9 * 60));
        };
        return (
          <div
            key={d.toISOString()}
            onClick={(e) => e.target === e.currentTarget && onDay(d)}
            onDragOver={(e) => e.dataTransfer.types.includes(TASK_MIME) && e.preventDefault()}
            onDrop={drop}
            className={`min-h-0 overflow-hidden border-b border-l border-slate-800 p-1 ${d.getMonth() === month ? "" : "opacity-40"}`}
          >
            <button
              onClick={() => onDay(d)}
              className={`mb-1 rounded px-1 text-xs ${sameDay(d, now) ? "bg-brand font-semibold text-slate-950" : "text-slate-400"}`}
            >
              {d.getDate()}
            </button>
            {items.slice(0, PER_CELL).map((e) => (
              <button
                key={e.event_id + e.starts_at}
                onClick={() => onOpen(e)}
                className="block w-full truncate rounded px-1 text-left text-xs hover:bg-slate-800"
              >
                {!e.all_day && <span className="text-slate-500">{formatTime(e.starts_at)} </span>}
                {e.title}
              </button>
            ))}
            {items.length > PER_CELL && (
              <button onClick={() => onDay(d)} className="px-1 text-xs text-slate-500">
                +{items.length - PER_CELL} more
              </button>
            )}
          </div>
        );
      })}
    </div>
  );
}
