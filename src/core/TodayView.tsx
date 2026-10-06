// Today: top 3, due today, and a calm Catch-up count.
// "Just one thing" hides everything except the next step.
import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api, type Task } from "./api";
import { useUi } from "./store";
import { formatDay } from "./time";
import { Button, Section, ShortList } from "./ui";
import { RemindersList } from "./RemindersList";
import { DailyBrief, EveningShutdown } from "./ReviewFlows";
import { TaskDetail } from "../modules/tasks/TaskDetail";
import { TaskRow } from "../modules/tasks/TaskRow";

export function TodayView() {
  const today = useQuery({ queryKey: ["today"], queryFn: api.today });
  const { justOne, setJustOne, setScreen, setCaptureOpen } = useUi();
  const [open, setOpen] = useState<Task | null>(null);
  const [flow, setFlow] = useState<"brief" | "shutdown" | null>(null);

  if (!today.data) return null;
  const t = today.data;
  const next = t.top[0];

  const detail = open && <TaskDetail key={open.id} task={open} onClose={() => setOpen(null)} />;
  const evening = new Date().getHours() >= 17;
  const flows = (
    <>
      {flow === "brief" && <DailyBrief today={t} onClose={() => setFlow(null)} />}
      {flow === "shutdown" && <EveningShutdown onClose={() => setFlow(null)} />}
    </>
  );

  if (justOne) {
    return (
      <div className="mx-auto flex h-full max-w-xl flex-col items-center justify-center gap-6 text-center">
        <p className="text-sm uppercase tracking-wide text-slate-500">Just one thing</p>
        {next ? (
          <div className="w-full">
            <TaskRow task={next} onOpen={setOpen} />
          </div>
        ) : (
          <p className="text-xl">Nothing waiting. Nice.</p>
        )}
        <Button onClick={() => setJustOne(false)}>Show everything</Button>
        {detail}
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-2xl space-y-8">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm text-slate-500">{formatDay(t.date)}</p>
          <h1 className="text-2xl font-semibold">Today</h1>
        </div>
        <Button kind="primary" onClick={() => setJustOne(true)}>
          Just one thing
        </Button>
      </div>

      {!t.brief_done && !evening && (
        <button onClick={() => setFlow("brief")} className="w-full rounded-xl bg-slate-900 p-4 text-left ring-1 ring-brand/40 hover:bg-slate-800">
          <div className="font-semibold">Start your day</div>
          <div className="text-sm text-slate-400">A 1-minute look at today, then pick your one must-do.</div>
        </button>
      )}
      {evening && !t.shutdown_done && (
        <button onClick={() => setFlow("shutdown")} className="w-full rounded-xl bg-slate-900 p-4 text-left ring-1 ring-brand/40 hover:bg-slate-800">
          <div className="font-semibold">Shut down for the day</div>
          <div className="text-sm text-slate-400">2 minutes: see your wins, move what's left, pick tomorrow's top 3.</div>
        </button>
      )}

      <RemindersList />

      <Section
        title="Top 3"
        right={
          <div className="flex gap-3 text-xs text-slate-500">
            <button className="hover:text-slate-300" onClick={() => setFlow("brief")}>
              Daily brief
            </button>
            <button className="hover:text-slate-300" onClick={() => setFlow("shutdown")}>
              Evening shutdown
            </button>
          </div>
        }
      >
        <ShortList
          items={t.top}
          empty={
            <span>
              Nothing yet.{" "}
              <button className="text-brand" onClick={() => setCaptureOpen(true)}>
                Capture something
              </button>
            </span>
          }
          render={(task) => (
            <div key={task.id} className="relative">
              {task.id === t.must_do_id && (
                <span className="absolute -top-2 right-3 z-[1] rounded bg-brand px-1.5 text-[10px] font-semibold text-slate-950">MUST-DO</span>
              )}
              <TaskRow task={task} onOpen={setOpen} />
            </div>
          )}
        />
      </Section>

      {(t.due_today.length > 0 || t.due_today_more > 0) && (
        <Section title="Also due today">
          <ShortList
            items={t.due_today}
            extra={t.due_today_more}
            empty=""
            render={(task) => <TaskRow key={task.id} task={task} onOpen={setOpen} />}
          />
        </Section>
      )}

      {(t.catch_up_count > 0 || t.inbox_count > 0) && (
        <div className="flex flex-wrap gap-2">
          {t.catch_up_count > 0 && (
            <Button kind="action" onClick={() => setScreen("tasks")}>
              {t.catch_up_count} in Catch-up. Pick one.
            </Button>
          )}
          {t.inbox_count > 0 && <Button onClick={() => setScreen("tasks")}>{t.inbox_count} in Inbox to sort</Button>}
        </div>
      )}
      {detail}
      {flows}
    </div>
  );
}
