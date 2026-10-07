import { describe, expect, it } from "vitest";
import { guessModule, localDate, parseCapture } from "./parse";

// Monday Oct 5 2026, 7 PM in Anchorage (Oct 6 03:00 UTC).
const REF = new Date("2026-10-06T03:00:00Z");

describe("parseCapture", () => {
  it("pulls out a weekday and keeps the title", () => {
    const c = parseCapture("oil change next Tuesday", REF);
    expect(c.title).toBe("oil change");
    expect(c.dueOn).toBe("2026-10-13");
    expect(c.dueAt).toBeUndefined();
  });

  it("keeps an exact time when one is given", () => {
    const c = parseCapture("dentist tomorrow at 3pm", REF);
    expect(c.title).toBe("dentist");
    expect(c.dueOn).toBe("2026-10-06");
    // 3 PM Anchorage (UTC-8) is 23:00 UTC.
    expect(c.dueAt).toBe("2026-10-06T23:00:00.000Z");
  });

  it("uses the Anchorage date, not UTC", () => {
    // At REF it is already Oct 6 in UTC but still Oct 5 in Anchorage.
    expect(localDate(REF)).toBe("2026-10-05");
    expect(parseCapture("call mom today", REF).dueOn).toBe("2026-10-05");
  });

  it("leaves text without a date alone", () => {
    const c = parseCapture("  buy   furnace filter ", REF);
    expect(c).toEqual({ title: "buy furnace filter", suggestedModule: undefined });
  });

  it("guesses the module from keywords", () => {
    expect(parseCapture("Pay Jake $40", REF).suggestedModule).toBe("finance");
    expect(guessModule("do laundry")).toBe("chores");
    expect(guessModule("gym leg day")).toBe("gym");
    expect(guessModule("call the vet")).toBeUndefined();
  });

  it("keeps the text if it was only a date", () => {
    expect(parseCapture("tomorrow", REF).title).toBe("tomorrow");
  });
});
