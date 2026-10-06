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
