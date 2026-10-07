// One task line: a big checkbox, the title, and calm metadata.
import { api, type Task } from "../../core/api";
import { formatDay } from "../../core/time";
import { EnergyTag, useRefresh } from "../../core/ui";

export function TaskRow({ task, onOpen }: { task: Task; onOpen?: (t: Task) => void }) {
  const refresh = useRefresh();
  const done = task.status === "done";

  async function toggle() {
    await api.setDone(task.id, !done);
    refresh();
  }

  return (
    <div className="flex items-center gap-3 rounded-lg bg-slate-900 px-3 py-2">
      <button
        onClick={toggle}
        aria-label={done ? "Mark not done" : "Mark done"}
        className={`h-6 w-6 shrink-0 rounded-full border-2 ${done ? "border-brand bg-brand" : "border-slate-600 hover:border-brand"}`}
      />
      <button onClick={() => onOpen?.(task)} className="min-w-0 flex-1 text-left">
        <div className={`truncate ${done ? "text-slate-500 line-through" : ""}`}>{task.title}</div>
        <div className="flex flex-wrap items-center gap-2 text-xs text-slate-500">
          {task.data.due_on && <span>{formatDay(task.data.due_on)}</span>}
          {task.est_minutes && <span>{task.est_minutes} min</span>}
          {task.open_steps > 0 && <span>{task.open_steps} steps left</span>}
          <EnergyTag energy={task.energy} />
        </div>
      </button>
    </div>
  );
}
