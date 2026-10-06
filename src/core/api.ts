// Typed wrappers around the Rust commands in src-tauri/src/lib.rs.
// Keep these types in sync with the Rust structs.
import { invoke } from "@tauri-apps/api/core";

export type Energy = "low" | "medium" | "high";
export type TaskView = "active" | "inbox" | "catch_up" | "done";

export interface TaskData {
  due_on?: string;
  due_at?: string;
  notes?: string;
  inbox?: boolean;
  suggested_module?: string;
}

export interface Task {
  id: string;
  title: string;
  status: "active" | "done" | "archived";
  energy: Energy | null;
  est_minutes: number | null;
  actual_minutes: number | null;
  data: TaskData;
  parent_id: string | null;
  open_steps: number;
  created_at: string;
  updated_at: string;
}

export interface NewTask {
  title: string;
  energy?: Energy | null;
  est_minutes?: number | null;
  data?: TaskData;
}

/** Leave a field out to keep it. Send null to clear it. */
export interface TaskPatch {
  title?: string;
  energy?: Energy | null;
  est_minutes?: number | null;
  actual_minutes?: number | null;
  due_on?: string | null;
  notes?: string | null;
  inbox?: boolean;
}

export interface TodayEvent {
  id: string;
  title: string;
  starts_at: string;
  ends_at: string | null;
  all_day: boolean;
}

export interface Today {
  date: string;
  top: Task[];
  due_today: Task[];
  due_today_more: number;
  events: TodayEvent[];
  events_more: number;
  catch_up_count: number;
  inbox_count: number;
}

export interface AppStatus {
  created: boolean;
  unlocked: boolean;
}

export const api = {
  appStatus: () => invoke<AppStatus>("app_status"),
  createPassword: (password: string) => invoke<void>("create_password", { password }),
  unlock: (password: string) => invoke<void>("unlock", { password }),
  lock: () => invoke<void>("lock"),
  today: () => invoke<Today>("today"),
  tasks: (view: TaskView) => invoke<Task[]>("tasks_list", { view }),
  createTask: (task: NewTask) => invoke<Task>("task_create", { task }),
  updateTask: (id: string, patch: TaskPatch) => invoke<Task>("task_update", { id, patch }),
  setDone: (id: string, done: boolean) => invoke<Task>("task_set_done", { id, done }),
  archiveTask: (id: string) => invoke<void>("task_archive", { id }),
  addSteps: (id: string, titles: string[]) => invoke<Task[]>("task_add_steps", { id, titles }),
  steps: (id: string) => invoke<Task[]>("task_steps", { id }),
  reschedule: (id: string, date?: string) => invoke<Task>("task_reschedule", { id, date: date ?? null }),
};

/** Commands reject with a plain string. Turn anything into readable text. */
export function errorText(e: unknown): string {
  return typeof e === "string" ? e : e instanceof Error ? e.message : "Something went wrong.";
}
