# Changelog

## Phase 1, step 2: Calendar

- Calendar screen with day, week, and month views. Keys: D, W, M, T for today, arrows to move.
- Create, edit, and delete events. All-day events. Repeats: daily, weekdays, weekly, monthly, yearly.
- Time-block a task by dragging it onto the calendar, or click it and then click a time.
- Overlapping events sit side by side. A line marks the current time.
- Today's agenda panel now shows repeating events too.
- Esc closes the event and task editors.
- Migration 0002: events can be soft-deleted. A backup is taken before it runs.

## 0.1.0 (unreleased): Phase 1, step 1

- Tauri 2 + React + TypeScript + Tailwind app, laid out as the spec's repo layout.
- Encrypted `mylife.db` (SQLCipher). First run creates the app password. Later runs unlock with it. Lock button in the sidebar.
- Numbered migrations with a backup before each one.
- Tasks: create, edit, complete, archive, split into steps, energy tag, time estimate and actual time, due date, notes.
- Catch-up list for missed tasks. "Today" button moves one back with no penalty.
- Today view: top 3, also due today, calendar events with countdown bars, Catch-up and Inbox counts, "Just one thing" mode. Lists cap at 7 with "Show more".
- Quick capture: Ctrl+Shift+Space from any app (Ctrl+N inside). Reads dates like "next Tuesday at 3" offline. Saves to the Inbox.
- System tray icon. Closing the window keeps MyLife running.
- Scheduler logic (not wired to the agent yet): RRULE repeats in Anchorage time, month-end, quiet hours 10 PM to 7 AM, ladder levels 1 to 3, missed reminders fire once.
- Rule JSON format parsing and time/offset trigger timing.
- Windows installer (MSI) built and published to GitHub Releases on every merge to main.
- App version shown in the sidebar.
