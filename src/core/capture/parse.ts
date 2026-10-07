// Quick capture parser. Free and offline: chrono-node finds the date,
// and simple keyword rules guess the module. No AI.
import * as chrono from "chrono-node";

export type SuggestedModule = "chores" | "gym" | "finance";

export interface Capture {
  /** What to do, with the date words removed. */
  title: string;
  /** Local due date, YYYY-MM-DD (Anchorage). */
  dueOn?: string;
  /** Exact due time in UTC ISO, only when a time was given. */
  dueAt?: string;
  /** Where this probably belongs. Saved now, used once Phase 2 modules exist. */
  suggestedModule?: SuggestedModule;
}

export const USER_TZ = "America/Anchorage";

// Order matters: first match wins.
const MODULE_KEYWORDS: [SuggestedModule, RegExp][] = [
  ["finance", /(\$\s?\d)|\b(pay|paid|bill|bills|rent|budget|invoice|refund|bank|card|loan|transfer)\b/i],
  ["gym", /\b(gym|workout|work out|lift|lifting|cardio|run|squat|bench|deadlift|planet fitness|treadmill)\b/i],
  [
    "chores",
    /\b(clean|laundry|dishes|dishwasher|vacuum|mop|trash|garbage|recycling|sweep|dust|wipe|tidy|fold|bathroom|kitchen|sheets)\b/i,
  ],
];

/** Formats a Date as YYYY-MM-DD on the Anchorage calendar. */
export function localDate(d: Date): string {
  // en-CA formats as YYYY-MM-DD.
  return new Intl.DateTimeFormat("en-CA", {
    timeZone: USER_TZ,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).format(d);
}

export function guessModule(text: string): SuggestedModule | undefined {
  return MODULE_KEYWORDS.find(([, re]) => re.test(text))?.[0];
}

export function parseCapture(input: string, ref: Date = new Date()): Capture {
  const text = input.trim().replace(/\s+/g, " ");
  const out: Capture = { title: text, suggestedModule: guessModule(text) };

  // forwardDate: "Tuesday" means the next Tuesday, not last week's.
  const [hit] = chrono.parse(text, ref, { forwardDate: true });
  if (!hit) return out;

  const date = hit.start.date();
  out.dueOn = localDate(date);
  if (hit.start.isCertain("hour")) out.dueAt = date.toISOString();

  // Remove the date words, plus a dangling "on", "at", "by", or "due" before them.
  const before = text.slice(0, hit.index).replace(/\b(on|at|by|due|for)\s*$/i, "");
  const after = text.slice(hit.index + hit.text.length);
  const title = `${before} ${after}`.replace(/\s+/g, " ").trim();
  // If the whole thing was a date, keep the original text as the title.
  out.title = title || text;
  return out;
}
