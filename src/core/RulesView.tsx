// Reminders and rules: every automation, with an on/off switch and when it last fired.
// The form makes simple time reminders ("Take meds every day at 8 AM").
import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api, errorText, type SavedRule } from "./api";
import { localDate } from "./capture/parse";
import { formatDay, formatTime } from "./time";
import { Button, Section, ShortList, useRefresh } from "./ui";
import { fromInputs, REPEATS, toDateInput } from "../modules/calendar/dates";

const field = "rounded-lg bg-slate-900 px-3 py-2 text-sm outline-none ring-1 ring-slate-700 focus:ring-brand";

/** How hard a reminder may push (the ladder). Level 4 arrives with phone push. */
const LADDER: { level: number; label: string }[] = [
  { level: 1, label: "Quiet: Today view only" },
  { level: 2, label: "Nudge: one notification" },
  { level: 3, label: "Persistent: keeps reminding" },
];

function describe(r: SavedRule): string {
  const t = r.rule.trigger;
  if (t.type === "offset") {
    const n = Number(t.days_before);
    return n === 0 ? "On the due date, 9 AM" : `${n} day${n === 1 ? "" : "s"} before the due date, 9 AM`;
  }
  if (t.type === "time") {
    const repeat = REPEATS.find((x) => x.rrule === t.rrule)?.label ?? (t.rrule ? "Repeats" : "Once");
    const at = typeof t.at === "string" ? t.at : null;
    return at ? `${repeat === "Does not repeat" ? "Once" : repeat}, ${formatTime(at)}` : repeat;
  }
  return t.type;
}

function RuleRow({ r }: { r: SavedRule }) {
  const refresh = useRefresh();
  async function toggle() {
    await api.setRuleEnabled(r.id, !r.enabled);
    refresh();
  }
  async function remove() {
    if (!window.confirm(`Delete "${r.rule.name}"?`)) return;
    await api.deleteRule(r.id);
    refresh();
  }
  return (
    <div className="flex items-center gap-3 rounded-lg bg-slate-900 px-3 py-2">
      <button
        onClick={toggle}
        role="switch"
        aria-checked={r.enabled}
        className={`h-6 w-11 shrink-0 rounded-full p-0.5 ${r.enabled ? "bg-brand" : "bg-slate-700"}`}
      >
        <span className={`block h-5 w-5 rounded-full bg-slate-100 transition ${r.enabled ? "translate-x-5" : ""}`} />
      </button>
      <div className="min-w-0 flex-1">
        <div className={`truncate ${r.enabled ? "" : "text-slate-500"}`}>{r.rule.name}</div>
        <div className="text-xs text-slate-500">
          {describe(r)}
          {r.last_fired_at && ` · Last fired ${formatDay(localDate(new Date(r.last_fired_at)))} ${formatTime(r.last_fired_at)}`}
        </div>
      </div>
      <Button onClick={remove}>Delete</Button>
    </div>
  );
}

export function RulesView() {
  const rules = useQuery({ queryKey: ["rules"], queryFn: api.rules });
  const refresh = useRefresh();
  const [name, setName] = useState("");
  const [date, setDate] = useState(toDateInput(new Date()));
  const [time, setTime] = useState("09:00");
  const [rrule, setRrule] = useState<string | null>("FREQ=DAILY");
  const [ladder, setLadder] = useState(2);
  const [error, setError] = useState("");

  async function add() {
    setError("");
    try {
      await api.createRule({
        name,
        trigger: { type: "time", at: fromInputs(date, time).toISOString(), rrule },
        conditions: [],
        actions: [{ type: "notify" }],
        max_ladder: ladder,
      });
      setName("");
      refresh();
    } catch (e) {
      setError(errorText(e));
    }
  }

  return (
    <div className="mx-auto max-w-2xl space-y-8">
      <h1 className="text-2xl font-semibold">Reminders</h1>

      <Section title="New reminder">
        <div className="space-y-2">
          <input className={`${field} w-full`} placeholder="What to remember" value={name} onChange={(e) => setName(e.target.value)} />
          <div className="flex flex-wrap gap-2">
            <input className={field} type="date" value={date} onChange={(e) => setDate(e.target.value)} />
            <input className={field} type="time" value={time} onChange={(e) => setTime(e.target.value)} />
            <select className={field} value={rrule ?? ""} onChange={(e) => setRrule(e.target.value || null)}>
              {REPEATS.map((r) => (
                <option key={r.label} value={r.rrule ?? ""}>
                  {r.label}
                </option>
              ))}
            </select>
            <select className={field} value={ladder} onChange={(e) => setLadder(Number(e.target.value))}>
              {LADDER.map((l) => (
                <option key={l.level} value={l.level}>
                  {l.label}
                </option>
              ))}
            </select>
          </div>
          {error && <p className="text-sm text-accent">{error}</p>}
          <Button kind="primary" onClick={add} disabled={!name.trim()}>
            Add reminder
          </Button>
        </div>
      </Section>

      <Section title="All rules">
        <ShortList items={rules.data ?? []} empty="No rules yet." render={(r) => <RuleRow key={r.id} r={r} />} />
      </Section>

      <p className="text-xs text-slate-500">Quiet hours are 10 PM to 7 AM. Reminders wait until morning unless they're urgent.</p>
    </div>
  );
}
