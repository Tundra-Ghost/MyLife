import { describe, expect, it } from "vitest";
import { countdownFill, formatDay, untilText } from "./time";

describe("time helpers", () => {
  it("formats a local day", () => {
    expect(formatDay("2026-10-06")).toBe("Tue, Oct 6");
  });

  it("says how long until", () => {
    expect(untilText(0)).toBe("now");
    expect(untilText(25 * 60000)).toBe("in 25 min");
    expect(untilText(125 * 60000)).toBe("in 2 h 5 min");
    expect(untilText(120 * 60000)).toBe("in 2 h");
  });

  it("fills the countdown bar", () => {
    const h = 3600000;
    expect(countdownFill(0, 3 * h, null)).toBe(0);
    expect(countdownFill(2 * h, 3 * h, null)).toBe(0.5);
    expect(countdownFill(4 * h, 3 * h, 5 * h)).toBe(0.5);
    expect(countdownFill(6 * h, 3 * h, 5 * h)).toBe(1);
  });
});
