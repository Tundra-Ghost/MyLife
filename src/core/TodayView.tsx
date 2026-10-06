// Today: top 3, due today, and a calm Catch-up count.
// "Just one thing" hides everything except the next step.
import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api, type Task } from "./api";
import { useUi } from "./store";
import { formatDay } from "./time";
import { Button, Section, ShortList } from "./ui";
import { TaskDetail } from "../modules/tasks/TaskDetail";
import { TaskRow } from "../modules/tasks/TaskRow";

export function TodayView() {
  const today = useQuery({ queryKey: ["today"], queryFn: api.today });
  const { justOne, setJustOne, setScreen, setCaptureOpen } = useUi();
  const [open, setOpen] = useState<Task | null>(null);

  if (!today.data) return null;
  const t = today.data;
  const next = t.top[0];

  const detail = open && <TaskDetail key={open.id} task={open} onClose={() => setOpen(null)} />;

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

      <Section title="Top 3">
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
          render={(task) => <TaskRow key={task.id} task={task} onOpen={setOpen} />}
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
    </div>
  );
}
