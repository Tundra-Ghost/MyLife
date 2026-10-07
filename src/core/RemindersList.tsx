// Active reminders on the Today view, with Done and smart snooze.
import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api, errorText, type Reminder, type SnoozeOption } from "./api";
import { formatTime } from "./time";
import { useUi } from "./store";
import { Button, Section, ShortList, useRefresh } from "./ui";

const SNOOZES: { id: SnoozeOption; label: string }[] = [
  { id: "later_today", label: "Later today" },
  { id: "tomorrow", label: "Tomorrow" },
  { id: "this_weekend", label: "This weekend" },
  { id: "when_free", label: "When I'm free" },
];

/** Matches CHECK_IN_AFTER_SNOOZES in src-tauri/src/scheduler/reminders.rs. */
const CHECK_IN_AFTER = 5;

function ReminderRow({ r }: { r: Reminder }) {
  const refresh = useRefresh();
  const setScreen = useUi((s) => s.setScreen);
  const [menu, setMenu] = useState(false);
  const [checkIn, setCheckIn] = useState(false);
  const [error, setError] = useState("");

  async function done() {
    await api.reminderDone(r.id);
    refresh();
  }

  async function snooze(option: SnoozeOption) {
    setMenu(false);
    try {
      const after = await api.snooze(r.id, option);
      if (after.snooze_count >= CHECK_IN_AFTER) setCheckIn(true);
      else refresh();
    } catch (e) {
      setError(errorText(e));
    }
  }

  return (
    <div className="space-y-2 rounded-lg bg-slate-900 px-3 py-2">
      <div className="flex items-center gap-2">
        <span className={`h-2 w-2 shrink-0 rounded-full ${r.ladder_level >= 2 ? "bg-accent" : "bg-slate-600"}`} />
        <div className="min-w-0 flex-1">
          <div className="truncate">{r.title}</div>
          <div className="text-xs text-slate-500">{formatTime(r.fire_at)}</div>
        </div>
        <Button onClick={done}>Done</Button>
        <Button onClick={() => setMenu(!menu)}>Snooze</Button>
      </div>
      {menu && (
        <div className="flex flex-wrap gap-2">
          {SNOOZES.map((s) => (
            <Button key={s.id} onClick={() => snooze(s.id)}>
              {s.label}
            </Button>
          ))}
        </div>
      )}
      {checkIn && (
        <div className="space-y-2 text-sm">
          <p className="text-slate-300">You've snoozed this {CHECK_IN_AFTER} times. What's going on?</p>
          <div className="flex flex-wrap gap-2">
            <Button onClick={() => setScreen("tasks")}>Too big. I'll split it up.</Button>
            <Button onClick={refresh}>Wrong time. Keep it.</Button>
            <Button onClick={done}>Not needed. Drop it.</Button>
          </div>
        </div>
      )}
      {error && <p className="text-sm text-accent">{error}</p>}
    </div>
  );
}

export function RemindersList() {
  const reminders = useQuery({ queryKey: ["reminders"], queryFn: api.reminders });
  const items = reminders.data ?? [];
  if (items.length === 0) return null;
  return (
    <Section title="Reminders">
      <ShortList items={items} empty="" render={(r) => <ReminderRow key={r.id} r={r} />} />
    </Section>
  );
}
