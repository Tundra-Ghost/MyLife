// Calendar date math. Uses the PC's local time zone, which is Anchorage
// on Tanner's machine (see docs/DECISIONS.md). Stored times stay UTC.

export type CalView = "day" | "week" | "month";

export function startOfDay(d: Date): Date {
  return new Date(d.getFullYear(), d.getMonth(), d.getDate());
}

/** Adds calendar days. Safe across daylight saving changes. */
export function addDays(d: Date, n: number): Date {
  return new Date(d.getFullYear(), d.getMonth(), d.getDate() + n, d.getHours(), d.getMinutes());
}

/** Weeks start on Sunday. */
export function startOfWeek(d: Date): Date {
  const s = startOfDay(d);
  return addDays(s, -s.getDay());
}

export function sameDay(a: Date, b: Date): boolean {
  return a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate();
}

/** The days a view shows, from first to last. Month is always 6 full weeks. */
export function viewDays(view: CalView, anchor: Date): Date[] {
  const first =
    view === "day" ? startOfDay(anchor) : view === "week" ? startOfWeek(anchor) : startOfWeek(new Date(anchor.getFullYear(), anchor.getMonth(), 1));
  const count = view === "day" ? 1 : view === "week" ? 7 : 42;
  return Array.from({ length: count }, (_, i) => addDays(first, i));
}

/** Moves the anchor one view forward (+1) or back (-1). */
export function step(view: CalView, anchor: Date, dir: 1 | -1): Date {
  if (view === "day") return addDays(anchor, dir);
  if (view === "week") return addDays(anchor, 7 * dir);
  return new Date(anchor.getFullYear(), anchor.getMonth() + dir, 1);
}

/** Minutes since local midnight. */
export function minutesOfDay(d: Date): number {
  return d.getHours() * 60 + d.getMinutes();
}

/** A local date plus minutes since midnight, as a Date. */
export function atMinutes(day: Date, minutes: number): Date {
  return new Date(day.getFullYear(), day.getMonth(), day.getDate(), 0, minutes);
}

/** Rounds minutes down to a 15-minute slot. */
export function snap15(minutes: number): number {
  return Math.max(0, Math.min(24 * 60 - 15, Math.floor(minutes / 15) * 15));
}

/** YYYY-MM-DD and HH:MM for form inputs, in local time. */
export function toDateInput(d: Date): string {
  const p = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
}
export function toTimeInput(d: Date): string {
  const p = (n: number) => String(n).padStart(2, "0");
  return `${p(d.getHours())}:${p(d.getMinutes())}`;
}
export function fromInputs(date: string, time: string): Date {
  const [y, m, d] = date.split("-").map(Number);
  const [h, min] = (time || "00:00").split(":").map(Number);
  return new Date(y, m - 1, d, h, min);
}

/**
 * Side-by-side lanes for overlapping timed events. Returns, for each item,
 * its lane and how many lanes its overlap group uses.
 */
export function layoutLanes(items: { start: number; end: number }[]): { lane: number; lanes: number }[] {
  const order = items.map((_, i) => i).sort((a, b) => items[a].start - items[b].start || items[b].end - items[a].end);
  const out: { lane: number; lanes: number }[] = items.map(() => ({ lane: 0, lanes: 1 }));
  let group: number[] = [];
  let laneEnds: number[] = [];
  let groupEnd = -Infinity;
  const close = () => {
    for (const i of group) out[i].lanes = laneEnds.length;
    group = [];
    laneEnds = [];
  };
  for (const i of order) {
    const it = items[i];
    if (it.start >= groupEnd) close();
    let lane = laneEnds.findIndex((end) => end <= it.start);
    if (lane === -1) {
      lane = laneEnds.length;
      laneEnds.push(it.end);
    } else laneEnds[lane] = it.end;
    out[i].lane = lane;
    group.push(i);
    groupEnd = Math.max(groupEnd === -Infinity ? it.end : groupEnd, it.end);
  }
  close();
  return out;
}

/** Repeat choices shown in the event form, as RRULE text. */
export const REPEATS: { label: string; rrule: string | null }[] = [
  { label: "Does not repeat", rrule: null },
  { label: "Every day", rrule: "FREQ=DAILY" },
  { label: "Every weekday", rrule: "FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR" },
  { label: "Every week", rrule: "FREQ=WEEKLY" },
  { label: "Every month", rrule: "FREQ=MONTHLY" },
  { label: "Every year", rrule: "FREQ=YEARLY" },
];
