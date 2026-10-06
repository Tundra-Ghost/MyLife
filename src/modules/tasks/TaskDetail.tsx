// Edit one task: title, energy, time estimate, due date, notes, and steps.
import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api, errorText, type Energy, type Task } from "../../core/api";
import { Button, useRefresh } from "../../core/ui";
import { TaskRow } from "./TaskRow";

const field = "w-full rounded-lg bg-slate-900 px-3 py-2 text-sm outline-none ring-1 ring-slate-700 focus:ring-brand";

export function TaskDetail({ task, onClose }: { task: Task; onClose: () => void }) {
  const refresh = useRefresh();
  const [title, setTitle] = useState(task.title);
  const [energy, setEnergy] = useState<Energy | "">(task.energy ?? "");
  const [est, setEst] = useState(task.est_minutes?.toString() ?? "");
  const [actual, setActual] = useState(task.actual_minutes?.toString() ?? "");
  const [due, setDue] = useState(task.data.due_on ?? "");
  const [notes, setNotes] = useState(task.data.notes ?? "");
  const [stepText, setStepText] = useState("");
  const [error, setError] = useState("");
  const steps = useQuery({ queryKey: ["steps", task.id], queryFn: () => api.steps(task.id) });

  const toNum = (s: string) => (s.trim() ? Number(s) : null);

  async function save() {
    setError("");
    try {
      await api.updateTask(task.id, {
        title,
        energy: energy || null,
        est_minutes: toNum(est),
        actual_minutes: toNum(actual),
        due_on: due || null,
        notes: notes || null,
        inbox: false, // Editing a task counts as sorting it.
      });
      refresh();
      onClose();
    } catch (e) {
      setError(errorText(e));
    }
  }

  async function addStep() {
    if (!stepText.trim()) return;
    setError("");
    try {
      await api.addSteps(task.id, [stepText]);
      setStepText("");
      refresh();
    } catch (e) {
      setError(errorText(e));
    }
  }

  async function archive() {
    await api.archiveTask(task.id);
    refresh();
    onClose();
  }

  const isStep = task.parent_id !== null;

  return (
    <div className="fixed inset-0 z-10 flex justify-end bg-black/50" onClick={onClose}>
      <div className="h-full w-full max-w-md space-y-4 overflow-y-auto bg-slate-950 p-6" onClick={(e) => e.stopPropagation()}>
        <input className={`${field} text-lg`} value={title} onChange={(e) => setTitle(e.target.value)} />

        <div className="grid grid-cols-2 gap-3">
          <label className="space-y-1 text-xs text-slate-400">
            Energy
            <select className={field} value={energy} onChange={(e) => setEnergy(e.target.value as Energy | "")}>
              <option value="">Not set</option>
              <option value="low">Low</option>
              <option value="medium">Medium</option>
              <option value="high">High</option>
            </select>
          </label>
          <label className="space-y-1 text-xs text-slate-400">
            Due
            <input className={field} type="date" value={due} onChange={(e) => setDue(e.target.value)} />
          </label>
          <label className="space-y-1 text-xs text-slate-400">
            Estimate (min)
            <input className={field} type="number" min={1} value={est} onChange={(e) => setEst(e.target.value)} />
          </label>
          <label className="space-y-1 text-xs text-slate-400">
            Actual (min)
            <input className={field} type="number" min={0} value={actual} onChange={(e) => setActual(e.target.value)} />
          </label>
        </div>

        <textarea className={`${field} h-24`} placeholder="Notes" value={notes} onChange={(e) => setNotes(e.target.value)} />

        {!isStep && (
          <div className="space-y-2">
            <h3 className="text-sm font-semibold text-slate-300">Steps</h3>
            <p className="text-xs text-slate-500">Make the first step tiny. Under 5 minutes.</p>
            {steps.data?.map((s) => <TaskRow key={s.id} task={s} />)}
            <div className="flex gap-2">
              <input
                className={field}
                placeholder="Add a step"
                value={stepText}
                onChange={(e) => setStepText(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && addStep()}
              />
              <Button onClick={addStep}>Add</Button>
            </div>
          </div>
        )}

        {error && <p className="text-sm text-accent">{error}</p>}
        <div className="flex gap-2">
          <Button kind="primary" onClick={save}>
            Save
          </Button>
          <Button onClick={onClose}>Cancel</Button>
          <div className="flex-1" />
          <Button onClick={archive} title="Archived tasks are kept, just hidden.">
            Archive
          </Button>
        </div>
      </div>
    </div>
  );
}
