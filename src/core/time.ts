// Time helpers. Stored times are UTC. The user sees Anchorage time.
import { USER_TZ } from "./capture/parse";

export function formatTime(iso: string): string {
  return new Intl.DateTimeFormat("en-US", { timeZone: USER_TZ, hour: "numeric", minute: "2-digit" }).format(
    new Date(iso),
  );
}

/** "Tue, Oct 6" from a YYYY-MM-DD local date. */
export function formatDay(ymd: string): string {
  const [y, m, d] = ymd.split("-").map(Number);
  // Noon UTC keeps the date stable in any time zone.
  return new Intl.DateTimeFormat("en-US", { timeZone: "UTC", weekday: "short", month: "short", day: "numeric" }).format(
    new Date(Date.UTC(y, m - 1, d, 12)),
  );
}

/** "in 25 min", "in 2 h 5 min", "now". */
export function untilText(ms: number): string {
  if (ms <= 0) return "now";
  const mins = Math.ceil(ms / 60000);
  if (mins < 60) return `in ${mins} min`;
  const h = Math.floor(mins / 60);
  const m = mins % 60;
  return m ? `in ${h} h ${m} min` : `in ${h} h`;
}

/**
 * Countdown bar fill, 0 to 1. Before an event the bar fills over the
 * 2 hours leading up to it. During the event it shows how much is done.
 */
export function countdownFill(now: number, start: number, end: number | null): number {
  const lead = 2 * 60 * 60 * 1000;
  if (now < start) return Math.max(0, 1 - (start - now) / lead);
  if (end && now < end) return (now - start) / (end - start);
  return 1;
}
