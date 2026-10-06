import { describe, expect, it } from "vitest";
import { addDays, fromInputs, layoutLanes, snap15, step, toDateInput, viewDays } from "./dates";

describe("calendar dates", () => {
  it("week view starts on Sunday", () => {
    const days = viewDays("week", new Date(2026, 9, 7)); // Wed Oct 7
    expect(toDateInput(days[0])).toBe("2026-10-04");
    expect(days).toHaveLength(7);
  });

  it("month view is 6 weeks from the Sunday before the 1st", () => {
    const days = viewDays("month", new Date(2026, 9, 20));
    expect(days).toHaveLength(42);
    expect(toDateInput(days[0])).toBe("2026-09-27");
  });

  it("adds days across the DST change", () => {
    // DST ends Nov 1 2026 in Anchorage. 9 AM stays 9 AM.
    const d = addDays(new Date(2026, 9, 31, 9, 0), 1);
    expect(d.getHours()).toBe(9);
    expect(toDateInput(d)).toBe("2026-11-01");
  });

  it("steps months from the 31st without skipping", () => {
    expect(toDateInput(step("month", new Date(2026, 0, 31), 1))).toBe("2026-02-01");
  });

  it("snaps to 15 minutes inside the day", () => {
    expect(snap15(67)).toBe(60);
    expect(snap15(-5)).toBe(0);
    expect(snap15(24 * 60 + 10)).toBe(24 * 60 - 15);
  });

  it("reads form inputs as local time", () => {
    const d = fromInputs("2026-10-06", "15:30");
    expect(d.toISOString()).toBe("2026-10-06T23:30:00.000Z");
  });

  it("puts overlapping events side by side", () => {
    const l = layoutLanes([
      { start: 60, end: 120 },
      { start: 90, end: 150 },
      { start: 200, end: 230 },
    ]);
    expect(l).toEqual([
      { lane: 0, lanes: 2 },
      { lane: 1, lanes: 2 },
      { lane: 0, lanes: 1 },
    ]);
  });
});
