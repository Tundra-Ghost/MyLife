// Right panel: today's events with countdown bars.
import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api, type TodayEvent } from "./api";
import { countdownFill, formatDay, formatTime, untilText } from "./time";
import { ShortList } from "./ui";

/** Re-renders every 30 seconds so countdowns stay current. */
function useNow() {
  const [now, setNow] = useState(Date.now());
  useEffect(() => {
    const t = setInterval(() => setNow(Date.now()), 30000);
    return () => clearInterval(t);
  }, []);
  return now;
}

export function EventRow({ ev, now }: { ev: TodayEvent; now: number }) {
  const start = Date.parse(ev.starts_at);
  const end = ev.ends_at ? Date.parse(ev.ends_at) : null;
  const fill = countdownFill(now, start, end);
  const label = ev.all_day ? "All day" : now < start ? untilText(start - now) : end && now < end ? "Now" : "Done";
  return (
    <div className="rounded-lg bg-slate-900 p-3">
      <div className="flex justify-between gap-2 text-sm">
        <span className="truncate">{ev.title}</span>
        <span className="shrink-0 text-slate-400">{ev.all_day ? "" : formatTime(ev.starts_at)}</span>
      </div>
      {!ev.all_day && (
        <div className="mt-2 h-1.5 overflow-hidden rounded bg-slate-800">
          <div className="h-full bg-brand" style={{ width: `${Math.round(fill * 100)}%` }} />
        </div>
      )}
      <div className="mt-1 text-xs text-slate-500">{label}</div>
    </div>
  );
}

export function AgendaPanel() {
  const today = useQuery({ queryKey: ["today"], queryFn: api.today });
  const now = useNow();
  return (
    <aside className="w-72 shrink-0 space-y-3 overflow-y-auto border-l border-slate-800 p-4">
      <h2 className="text-sm font-semibold text-slate-300">{today.data ? formatDay(today.data.date) : "Today"}</h2>
      <ShortList
        items={today.data?.events ?? []}
        extra={today.data?.events_more ?? 0}
        empty="Nothing on the calendar today."
        render={(ev) => <EventRow key={ev.id} ev={ev} now={now} />}
      />
    </aside>
  );
}
