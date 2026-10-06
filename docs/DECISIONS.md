# Decisions

Defaults the builder picked where the spec left a gap (build rule 5).
Tanner can change any of these. Newest at the bottom.

| # | Date | Topic | Decision | Why |
| --- | --- | --- | --- | --- |
| 1 | 2026-10-06 | Phase 1 split | Phase 1 ships in steps. Step 1: scaffold, encrypted database, tasks, Today, quick capture, scheduler logic. Step 2: calendar and Google sync. Step 3: tray agent loop, reminders, snooze. Step 4: backups, restore, daily brief, evening shutdown. | Keeps each PR small enough to review. |
| 2 | 2026-10-06 | SQLCipher build | `rusqlite` uses `bundled-sqlcipher-vendored-openssl`, the same as `bundled-sqlcipher` but it also builds OpenSSL. | No separate OpenSSL install on Windows. |
| 3 | 2026-10-06 | App password | Minimum 8 characters. SQLCipher derives the key from it (PBKDF2-HMAC-SHA512, SQLCipher 4 defaults). | The spec does not give a length. |
| 4 | 2026-10-06 | Top 3 order | Active tasks sorted by due date (soonest first), then undated, then oldest first. Catch-up tasks are left out. | The spec does not say how to pick the top 3. A manual "pin to top 3" can come with the evening shutdown. |
| 5 | 2026-10-06 | Catch-up | A task is in Catch-up when its due date is before today in Anchorage time. "Today" moves it to today with no penalty. | Spec: missed tasks move to Catch-up, not a red list. |
| 6 | 2026-10-06 | Steps | Steps are tasks linked to their parent in `links` with relation `step_of`. Steps can't have steps. | Uses the spec's `links` table. One level keeps it simple. |
| 7 | 2026-10-06 | Quick capture | Saves to the Inbox as a task. Module keywords (chores, gym, finance) are stored as `suggested_module` and used once those modules exist. | Phase 2 modules are not built yet. |
| 8 | 2026-10-06 | Quick capture window | The hotkey shows the main window and opens a capture box in it. | Simpler than a second window. Can split out later. |
| 9 | 2026-10-06 | Date-only fire time | Offset triggers with only a date fire at 9 AM local. | The spec gives no time of day. |
| 10 | 2026-10-06 | Ladder level 4 | Capped at level 3 until phone push ships in Phase 4. | Spec: ntfy is Phase 4. |
| 11 | 2026-10-06 | Tray agent | The agent runs inside the app process, in the tray. Closing the window hides it. | One process is simpler to install and update. Can split later if needed. |
| 12 | 2026-10-06 | In-app shortcuts | Alt+1 Today, Alt+2 Tasks, Ctrl+N quick capture. | Spec: keyboard shortcuts for everything. |
| 13 | 2026-10-06 | Schema | `migrations/0001_init.sql` is the spec schema, copied as-is. | Spec: follow the schema exactly. |
| 14 | 2026-10-06 | Installing and updating | Every merge to `main` builds an MSI and publishes it as a GitHub Release (`.github/workflows/release.yml`). Version is `0.1.<build number>`. A fixed WiX upgrade code makes each MSI upgrade the last one. | Tanner wants to install and update as work lands. Uses only Tauri's built-in bundler. |
| 15 | 2026-10-06 | In-app auto-update | Not built. It needs the Tauri updater plugin, which is not on the approved library list. Waiting on Tanner. | Spec build rule 7: no libraries outside the list without approval. |
| 16 | 2026-10-06 | Calendar time zone | Calendar grids use the PC's time zone, which is Anchorage on Tanner's PC. Stored times stay UTC. | Simplest correct option for a single-user Windows app. |
| 17 | 2026-10-06 | Week start | Weeks start on Sunday. | US default. Easy to change later. |
| 18 | 2026-10-06 | Time-blocking | Drag a task onto the calendar, or click the task and then click a time. Blocks use the task's estimate, or 30 minutes. Dropping on a month day blocks 9 AM. | Click-to-place also works with no mouse drag. |
| 19 | 2026-10-06 | Editing repeating events | Edits and deletes apply to every occurrence. Single-occurrence edits can come later. | Keeps step 2 small. |
| 20 | 2026-10-06 | Event delete | Soft delete through a new `archived_at` column on `events` (migration 0002). | Spec hard rule: deletes are soft. |
| 21 | 2026-10-06 | Agent after restart | MyLife starts at Windows login (installed builds only) and sits in the tray. Reminders run once the app password is entered. Windows Hello unlock (spec, optional) will remove that step later. | The database is encrypted with the app password, so the agent can't read it before unlock. |
| 22 | 2026-10-06 | Snooze times | Later today = 3 hours. Tomorrow = 9 AM. This weekend = Saturday 9 AM (next Saturday if it's already the weekend). When I'm free = next open 30 minutes on the calendar, 9 AM to 9 PM, within 7 days. | The spec names the options but not the times. |
| 23 | 2026-10-06 | Due-date reminders | Starter rule "Task due today" reminds at 9 AM on the due date. Tasks with an exact time remind at that time instead. No reminder is made for a time before the task existed. | Avoids double and stale alerts. |
| 24 | 2026-10-06 | Reminder Done | Done on a reminder closes the reminder only. Finishing a task closes its reminders. | Keeps "I saw it" separate from "I did it". |
| 25 | 2026-10-06 | Rules in Phase 1 | Runs `time` and `offset` (task due date) triggers, and `notify` and `create_task` actions. Other types are saved but run when their modules ship. | Matches the Phase 1 gate. |
| 26 | 2026-10-06 | Snooze check-in | After 5 snoozes the reminder asks: too big, wrong time, or not needed. | Spec. |
