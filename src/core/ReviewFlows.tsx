// Daily brief (morning) and evening shutdown. Both are short, guided flows.
import { useState, type ReactNode } from "react";
import { useQuery } from "@tanstack/react-query";
import { api, errorText, type Task, type Today } from "./api";
import { localDate } from "./capture/parse";
import { formatDay, formatTime } from "./time";
import { Button, useEscape, useRefresh } from "./ui";

function Modal({ children, onClose }: { children: ReactNode; onClose: () => void }) {
  useEscape(onClose);
  return (
    <div className="fixed inset-0 z-20 flex items-start justify-center overflow-y-auto bg-black/60 py-[8vh]" onClick={onClose}>
      <div className="w-full max-w-lg space-y-5 rounded-xl bg-slate-950 p-6 ring-1 ring-slate-800" onClick={(e) => e.stopPropagation()}>
        {children}
      </div>
    </div>
  );
}

function greeting(now: Date) {
  const h = now.getHours();
  return h < 12 ? "Good morning" : h < 17 ? "Good afternoon" : "Good evening";
}

export function DailyBrief({ today, onClose }: { today: Today; onClose: () => void }) {
  const refresh = useRefresh();
  const tasks = useQuery({ queryKey: ["tasks", "active"], queryFn: () => api.tasks("active") });
  const reminders = useQuery({ queryKey: ["reminders"], queryFn: api.reminders });
  const [text, setText] = useState("");
  const [error, setError] = useState("");

  async function pick(id: string) {
    try {
      await api.setMustDo(id);
      refresh();
      onClose();
    } catch (e) {
      setError(errorText(e));
    }
  }

  async function addAndPick() {
    if (!text.trim()) return;
    try {
      const t = await api.createTask({ title: text, data: { due_on: today.date } });
      await pick(t.id);
    } catch (e) {
      setError(errorText(e));
    }
  }

  async function skip() {
    await api.skipBrief();
    refresh();
    onClose();
  }

  const timed = today.events.filter((e) => !e.all_day);
  return (
    <Modal onClose={onClose}>
      <div>
        <p className="text-sm text-slate-500">{formatDay(today.date)}</p>
        <h2 className="text-2xl font-semibold">{greeting(new Date())}</h2>
      </div>

      <div className="space-y-1 text-sm">
        <p className="font-semibold text-slate-300">Today</p>
        {today.events.length === 0 && <p className="text-slate-500">Nothing on the calendar.</p>}
        {today.events.filter((e) => e.all_day).map((e) => (
          <p key={e.id}>{e.title} (all day)</p>
        ))}
        {timed.map((e) => (
          <p key={e.id + e.starts_at}>
            <span className="text-slate-500">{formatTime(e.starts_at)}</span> {e.title}
          </p>
        ))}
        <p className="pt-1 text-slate-400">
          {today.due_today.length + today.top.filter((t) => t.data.due_on === today.date).length} due today
          {reminders.data && reminders.data.length > 0 && ` · ${reminders.data.length} reminders waiting`}
          {today.catch_up_count > 0 && ` · ${today.catch_up_count} in Catch-up`}
        </p>
      </div>

      <div className="space-y-2">
        <p className="text-lg font-semibold">What is your one must-do today?</p>
        <div className="space-y-1">
          {(tasks.data ?? []).slice(0, 7).map((t) => (
            <button key={t.id} onClick={() => pick(t.id)} className="block w-full truncate rounded-lg bg-slate-900 px-3 py-2 text-left text-sm hover:bg-slate-800">
              {t.title}
            </button>
          ))}
        </div>
        <div className="flex gap-2">
          <input
            className="flex-1 rounded-lg bg-slate-900 px-3 py-2 text-sm outline-none ring-1 ring-slate-700 focus:ring-brand"
            placeholder="Or type something new"
            value={text}
            onChange={(e) => setText(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && addAndPick()}
          />
          <Button kind="primary" onClick={addAndPick} disabled={!text.trim()}>
            That's it
          </Button>
        </div>
        {error && <p className="text-sm text-accent">{error}</p>}
      </div>
      <button onClick={skip} className="text-sm text-slate-500 hover:text-slate-300">
        Skip for today
      </button>
    </Modal>
  );
}

export function EveningShutdown({ onClose }: { onClose: () => void }) {
  const refresh = useRefresh();
  const data = useQuery({ queryKey: ["shutdown"], queryFn: api.shutdown });
  const [step, setStep] = useState(0);
  const [moved, setMoved] = useState<Record<string, string>>({});
  const [picks, setPicks] = useState<string[]>([]);
  const [error, setError] = useState("");

  if (!data.data) return null;
  const { done_today, left_over, candidates } = data.data;
  const tomorrow = localDate(new Date(Date.now() + 24 * 3600 * 1000));

  async function move(t: Task, choice: "tomorrow" | "leave") {
    if (choice === "tomorrow") await api.reschedule(t.id, tomorrow);
    setMoved({ ...moved, [t.id]: choice === "tomorrow" ? "Moved to tomorrow" : "Left as is" });
  }

  function toggle(id: string) {
    setPicks(picks.includes(id) ? picks.filter((p) => p !== id) : picks.length < 3 ? [...picks, id] : picks);
  }

  async function finish() {
    try {
      await api.setTop3(picks);
      refresh();
      onClose();
    } catch (e) {
      setError(errorText(e));
    }
  }

  return (
    <Modal onClose={onClose}>
      <p className="text-sm text-slate-500">Evening shutdown · step {step + 1} of 3</p>

      {step === 0 && (
        <div className="space-y-3">
          <h2 className="text-2xl font-semibold">{done_today.length > 0 ? `You finished ${done_today.length} today` : "Rest counts too"}</h2>
          {done_today.slice(0, 7).map((t) => (
            <p key={t.id} className="text-sm text-slate-300">
              ✓ {t.title}
            </p>
          ))}
          <Button kind="primary" onClick={() => setStep(1)}>
            Next
          </Button>
        </div>
      )}

      {step === 1 && (
        <div className="space-y-3">
          <h2 className="text-2xl font-semibold">What didn't get done</h2>
          {left_over.length === 0 && <p className="text-sm text-slate-400">Nothing left over. Nice.</p>}
          {left_over.slice(0, 7).map((t) => (
            <div key={t.id} className="flex items-center gap-2 rounded-lg bg-slate-900 px-3 py-2 text-sm">
              <span className="min-w-0 flex-1 truncate">{t.title}</span>
              {moved[t.id] ? (
                <span className="text-xs text-slate-500">{moved[t.id]}</span>
              ) : (
                <>
                  <Button onClick={() => move(t, "tomorrow")}>Tomorrow</Button>
                  <Button onClick={() => move(t, "leave")}>Leave it</Button>
                </>
              )}
            </div>
          ))}
          <Button kind="primary" onClick={() => setStep(2)}>
            Next
          </Button>
        </div>
      )}

      {step === 2 && (
        <div className="space-y-3">
          <h2 className="text-2xl font-semibold">Pick tomorrow's top 3</h2>
          <div className="space-y-1">
            {candidates.slice(0, 12).map((t) => (
              <label key={t.id} className="flex items-center gap-2 rounded-lg bg-slate-900 px-3 py-2 text-sm">
                <input type="checkbox" checked={picks.includes(t.id)} onChange={() => toggle(t.id)} />
                <span className="truncate">{t.title}</span>
              </label>
            ))}
          </div>
          {error && <p className="text-sm text-accent">{error}</p>}
          <Button kind="primary" onClick={finish}>
            Done for today
          </Button>
        </div>
      )}
    </Modal>
  );
}
