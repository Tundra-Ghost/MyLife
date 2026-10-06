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

export interface CalEvent {
  id: string;
  item_id: string | null;
  title: string;
  starts_at: string;
  ends_at: string | null;
  all_day: boolean;
  rrule: string | null;
  source: string;
}

export interface Occurrence {
  event_id: string;
  item_id: string | null;
  title: string;
  starts_at: string;
  ends_at: string | null;
  all_day: boolean;
  repeats: boolean;
}

export interface EventInput {
  title: string;
  starts_at: string;
  ends_at: string | null;
  all_day: boolean;
  rrule: string | null;
  item_id: string | null;
}

export interface Reminder {
  id: string;
  rule_id: string | null;
  item_id: string | null;
  title: string;
  fire_at: string;
  ladder_level: number;
  max_ladder: number;
  urgent: boolean;
  snooze_count: number;
  state: string;
  last_notified_at: string | null;
}

export type SnoozeOption = "later_today" | "tomorrow" | "this_weekend" | "when_free";

export type Trigger =
  | { type: "time"; at?: string | null; rrule?: string | null }
  | { type: "offset"; event?: string | null; date_field?: string | null; days_before: number }
  | { type: string; [k: string]: unknown };

export interface Rule {
  name: string;
  trigger: Trigger;
  conditions: unknown[];
  actions: ({ type: string } & Record<string, unknown>)[];
  max_ladder: number;
}

export interface SavedRule {
  id: string;
  module: string;
  enabled: boolean;
  rule: Rule;
  last_fired_at: string | null;
  created_at: string;
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
  events: (from: Date, to: Date) =>
    invoke<Occurrence[]>("events_range", { from: from.toISOString(), to: to.toISOString() }),
  event: (id: string) => invoke<CalEvent>("event_get", { id }),
  createEvent: (event: EventInput) => invoke<CalEvent>("event_create", { event }),
  updateEvent: (id: string, event: EventInput) => invoke<CalEvent>("event_update", { id, event }),
  archiveEvent: (id: string) => invoke<void>("event_archive", { id }),
  reminders: () => invoke<Reminder[]>("reminders_active"),
  reminderDone: (id: string) => invoke<void>("reminder_done", { id }),
  snooze: (id: string, option: SnoozeOption) => invoke<Reminder>("reminder_snooze", { id, option }),
  rules: () => invoke<SavedRule[]>("rules_list"),
  createRule: (rule: Rule) => invoke<SavedRule>("rule_create", { rule }),
  setRuleEnabled: (id: string, enabled: boolean) => invoke<SavedRule>("rule_set_enabled", { id, enabled }),
  deleteRule: (id: string) => invoke<void>("rule_delete", { id }),
  blockTask: (id: string, startsAt: Date) => invoke<CalEvent>("task_block", { id, startsAt: startsAt.toISOString() }),
};

/** Commands reject with a plain string. Turn anything into readable text. */
export function errorText(e: unknown): string {
  return typeof e === "string" ? e : e instanceof Error ? e.message : "Something went wrong.";
}
