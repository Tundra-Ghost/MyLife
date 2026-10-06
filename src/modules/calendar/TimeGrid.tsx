// Day and week views: one column per day, one row per hour.
// Drop a task on a column to time-block it. Click empty space to add an event.
import { useEffect, useRef, type DragEvent, type MouseEvent } from "react";
import type { Occurrence } from "../../core/api";
import { formatTime } from "../../core/time";
import { atMinutes, layoutLanes, minutesOfDay, sameDay, snap15, startOfDay } from "./dates";

export const HOUR_PX = 48;
export const TASK_MIME = "application/x-mylife-task";

function dayLabel(d: Date) {
  return new Intl.DateTimeFormat("en-US", { weekday: "short", day: "numeric" }).format(d);
}

/** Minutes from the top of a day column for a pointer event. */
function minutesAt(e: { clientY: number; currentTarget: Element }): number {
  const top = e.currentTarget.getBoundingClientRect().top;
  return snap15(((e.clientY - top) / HOUR_PX) * 60);
}

export function TimeGrid({
  days,
  events,
  now,
  onSlot,
  onOpen,
  onDropTask,
}: {
  days: Date[];
  events: Occurrence[];
  now: Date;
  onSlot: (start: Date) => void;
  onOpen: (o: Occurrence) => void;
  onDropTask: (taskId: string, start: Date) => void;
}) {
  const scroller = useRef<HTMLDivElement>(null);

  // Start scrolled to 7 AM, not midnight.
  useEffect(() => {
    if (scroller.current) scroller.current.scrollTop = 7 * HOUR_PX;
  }, []);

  const allDay = events.filter((e) => e.all_day);
  const timed = events.filter((e) => !e.all_day);

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg ring-1 ring-slate-800">
      {/* Day headers and all-day events */}
      <div className="flex border-b border-slate-800">
        <div className="w-14 shrink-0" />
        {days.map((d) => {
          const today = sameDay(d, now);
          const items = allDay.filter((e) => overlapsDay(e, d));
          return (
            <div key={d.toISOString()} className="min-w-0 flex-1 border-l border-slate-800 p-1">
              <div className={`truncate whitespace-nowrap text-center text-sm ${today ? "font-semibold text-brand" : "text-slate-400"}`}>{dayLabel(d)}</div>
              {items.map((e) => (
                <button
                  key={e.event_id + e.starts_at}
                  onClick={() => onOpen(e)}
                  className="mt-1 block w-full truncate rounded bg-slate-700 px-1 text-left text-xs"
                >
                  {e.title}
                </button>
              ))}
            </div>
          );
        })}
      </div>

      {/* Hour grid */}
      <div ref={scroller} className="min-h-0 flex-1 overflow-y-auto">
        <div className="relative flex" style={{ height: 24 * HOUR_PX }}>
          <div className="w-14 shrink-0">
            {Array.from({ length: 24 }, (_, h) => (
              <div key={h} className="pr-2 text-right text-xs text-slate-600" style={{ height: HOUR_PX }}>
                {h === 0 ? "" : new Intl.DateTimeFormat("en-US", { hour: "numeric" }).format(new Date(2000, 0, 1, h))}
              </div>
            ))}
          </div>
          {days.map((d) => (
            <DayColumn
              key={d.toISOString()}
              day={d}
              events={timed.filter((e) => overlapsDay(e, d))}
              now={now}
              onSlot={onSlot}
              onOpen={onOpen}
              onDropTask={onDropTask}
            />
          ))}
        </div>
      </div>
    </div>
  );
}

function overlapsDay(o: Occurrence, day: Date): boolean {
  const start = new Date(o.starts_at).getTime();
  const end = o.ends_at ? new Date(o.ends_at).getTime() : start;
  const dayStart = startOfDay(day).getTime();
  const dayEnd = atMinutes(day, 24 * 60).getTime();
  return start < dayEnd && (end > dayStart || start >= dayStart);
}

function DayColumn({
  day,
  events,
  now,
  onSlot,
  onOpen,
  onDropTask,
}: {
  day: Date;
  events: Occurrence[];
  now: Date;
  onSlot: (start: Date) => void;
  onOpen: (o: Occurrence) => void;
  onDropTask: (taskId: string, start: Date) => void;
}) {
  const dayStart = startOfDay(day).getTime();
  // Clip each event to this day, in minutes.
  const spans = events.map((e) => {
    const s = Math.max(0, (new Date(e.starts_at).getTime() - dayStart) / 60000);
    const endMs = e.ends_at ? new Date(e.ends_at).getTime() : new Date(e.starts_at).getTime() + 30 * 60000;
    const en = Math.min(24 * 60, (endMs - dayStart) / 60000);
    return { start: s, end: Math.max(en, s + 15) };
  });
  const lanes = layoutLanes(spans);

  function click(e: MouseEvent<HTMLDivElement>) {
    if (e.target !== e.currentTarget) return;
    onSlot(atMinutes(day, minutesAt(e)));
  }

  function drop(e: DragEvent<HTMLDivElement>) {
    const id = e.dataTransfer.getData(TASK_MIME);
    if (!id) return;
    e.preventDefault();
    onDropTask(id, atMinutes(day, minutesAt(e)));
  }

  return (
    <div
      className="relative min-w-0 flex-1 border-l border-slate-800"
      style={{
        backgroundImage: `repeating-linear-gradient(to bottom, transparent 0, transparent ${HOUR_PX - 1}px, rgb(30 41 59) ${HOUR_PX - 1}px, rgb(30 41 59) ${HOUR_PX}px)`,
      }}
      onClick={click}
      onDragOver={(e) => e.dataTransfer.types.includes(TASK_MIME) && e.preventDefault()}
      onDrop={drop}
    >
      {sameDay(day, now) && (
        <div className="pointer-events-none absolute right-0 left-0 z-[1] h-0.5 bg-accent" style={{ top: (minutesOfDay(now) / 60) * HOUR_PX }} />
      )}
      {events.map((e, i) => {
        const { start, end } = spans[i];
        const { lane, lanes: n } = lanes[i];
        return (
          <button
            key={e.event_id + e.starts_at}
            onClick={() => onOpen(e)}
            className={`absolute overflow-hidden rounded px-1.5 py-0.5 text-left text-xs ring-1 ring-slate-950 ${
              e.item_id ? "bg-brand/80 text-slate-950" : "bg-slate-600"
            }`}
            style={{
              top: (start / 60) * HOUR_PX,
              height: ((end - start) / 60) * HOUR_PX - 1,
              left: `${(lane / n) * 100}%`,
              width: `${100 / n}%`,
            }}
            title={e.title}
          >
            <div className="truncate font-medium">{e.title}</div>
            <div className="truncate opacity-75">{formatTime(e.starts_at)}</div>
          </button>
        );
      })}
    </div>
  );
}
