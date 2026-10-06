# MyLife Design Document

Oct 5, 2026 · @Tanner

## Instructions for the AI builder

This document is the build spec for MyLife. The owner is Tanner. You build. Tanner tests and approves each phase.

### Build rules

1. Build in phase order (see Roadmap). Do not start a phase until its acceptance criteria pass and Tanner approves.
2. Inside a phase, finish one module at a time. Each module must run, have tests, and be shown to Tanner before the next starts.
3. Build only what this document describes. Write new ideas in `docs/IDEAS.md` instead of building them.
4. If something is unclear, or two sections conflict, stop and ask Tanner. Never guess on money math, security, or deleting data.
5. Where this document gives a default for an open decision, use the default and note it in `docs/DECISIONS.md`.
6. Never delete or overwrite user data without a confirm dialog and a backup taken first.
7. The Security, Password vault, and Technical specification sections are mandatory. Do not swap in other crypto or storage approaches.
8. Keep `docs/CHANGELOG.md` and `docs/DECISIONS.md` up to date.
9. Write clear, commented code. Tanner has an Android background and must be able to maintain it.
10. All text the user sees is short and plain. No em dashes.
11. Target Windows 11 (x64). Keep code cross-platform where it costs nothing, but test only on Windows in v1.

### Glossary

| Term | Meaning |
| --- | --- |
| Item | Anything tracked: a task, chore, appliance, card, tenant, account. |
| Event | Anything with a date or time. |
| Rule | An automation: trigger, conditions, actions. |
| Reminder | A scheduled alert created by a rule. |
| Ladder level | How hard a reminder pushes: 1 quiet, 2 nudge, 3 persistent, 4 escalate. |
| Module | A feature area with its own screens, tables, and starter rules. |
| Gate | The test a phase must pass before the next phase starts. |
| Vault | The separate encrypted password file. |
| Safe-to-spend | Balance minus bills due before next payday minus savings goals. |
| Manual account | A card or account tracked by hand, with no bank link. |
| Agent | The background tray process that runs rules and reminders. |

## Overview

MyLife is a local-first desktop app that runs your whole life from one place. It holds your calendar, tasks, goals, money, home, vehicles, health, rental rooms, documents, and people. A rules engine watches all of it and does the remembering for you.

The core idea is simple. **You should never have to remember to remember.** If something recurs, expires, wears out, or costs money, MyLife tracks it and tells you at the right time.

### ADHD-first design principles

1. **One screen to start the day.** A "Today" view shows only what matters now. Everything else stays hidden until needed.
2. **Capture in under 3 seconds.** A global hotkey opens a quick-add box from anywhere. Sort it later.
3. **Externalize time.** Show time as visual bars and countdowns, not just numbers.
4. **Small next steps.** Every big item breaks into a first step that takes under 5 minutes.
5. **Forgiving, not punishing.** Missed tasks reschedule gently. No red walls of overdue items.
6. **Reward progress.** Streaks, XP, and visible wins give quick dopamine.
7. **Automate the boring parts.** If the app can do it or predict it, it should.

### Goals

- Replace 5 to 10 separate apps with one.
- Cut missed bills, renewals, and maintenance to near zero.
- Turn long-term goals into daily actions.
- Keep all data private and on your own machine by default.

### Non-goals (v1)

- No team or family sharing in v1.
- No full banking app. MyLife reads and tracks money. It does not move money.
- No cloud account required.

## Core architecture

MyLife is a Tauri desktop app with a React front end and an encrypted SQLite database. It runs in the system tray so reminders fire even when the main window is closed. Every feature is a module that plugs into one shared core.

### Tech stack

| Layer | Choice | Why |
| --- | --- | --- |
| Desktop shell | Tauri 2 (Rust) | Small, fast, low memory. Runs on Windows, Mac, Linux. |
| UI | React + TypeScript + Tailwind | Fast to build. Huge component ecosystem. |
| Database | SQLite with SQLCipher | Local, encrypted, one file to back up. |
| Background service | Rust tray process | Runs the rules engine and fires notifications 24/7. |
| Search | SQLite FTS5 | Instant search across every module. |
| AI assist | Ollama, local (optional) | Free model on your PC for task breakdown and weekly review. On by default. No paid API. |
| OCR | Tesseract (local) | Reads receipts, manuals, and documents offline. |
| Sync (later) | Encrypted file sync to your own cloud folder | Optional backup and a future Android companion. |

### The shared core

Every module uses the same five building blocks. This is what makes automation work across the whole app.

- **Items.** Anything you track: a task, a bill, a furnace, a car, a tenant.
- **Events.** Anything with a date: due dates, appointments, expirations.
- **Rules.** "When X happens, do Y." The automation engine.
- **Links.** Any item can link to any other. The furnace links to its manual, its receipt, and its filter task.
- **Timeline.** One log of everything that happened, searchable forever.

### Module system

Each module is a folder with its own data tables, screens, and starter rules. New modules can be added without touching the core. This lets us ship a small v1 and grow it safely.

## Modules

MyLife ships with 18 modules. Each one comes with starter rules so it is useful on day one. You can turn off any module you do not need.

| Module | What it tracks | Built-in automation |
| --- | --- | --- |
| Calendar | Events, appointments, time blocks. Two-way sync with Google and Outlook. | Adds travel buffer time. Blocks prep time before big events. Warns when a day is overbooked. |
| Tasks and projects | To-dos, projects, checklists, waiting-on items. | Breaks big tasks into steps. Moves missed tasks to the next open slot. Nags about "waiting on" items after 3 days. |
| Goals and habits | Long-term goals, milestones, daily habits, streaks. | Turns each goal into weekly targets. Suggests a daily habit for each goal. Weekly progress report. |
| Finance | Accounts, budget, bills, subscriptions, debts, savings goals, net worth. | Reads transactions from bank feeds. Auto-tags spending. Flags new subscriptions and price hikes. Bill reminders 5 days out. |
| Receipts and warranties | Receipts, purchase dates, warranty terms, return windows. | OCR reads the receipt. Return window and warranty end dates become reminders automatically. |
| Home | Rooms, systems, appliances, filters, seasonal jobs, utilities. | Maintenance schedules per appliance. Seasonal checklists by date and weather. Tracks utility usage spikes. |
| Vehicles | Each vehicle, mileage, service history, fuel, registration, insurance, fault codes. | Service due by miles or months, whichever comes first. Tire swap windows. Registration and insurance renewal alerts. |
| Rental rooms | Rooms, tenants, agreements, rent, deposits, expenses, turnover. | Rent due and late notices. Lease end reminders. Turnover checklist. Year-end income and expense report for taxes. |
| Health | Appointments, medications, refills, sleep, exercise, water, energy. | Med reminders. Refill alerts based on pill count. Annual checkups. Links energy to sleep and habits. |
| People | Family, friends, contacts, birthdays, gift ideas, last contact. | Birthday reminders 2 weeks out with gift ideas. "You have not talked to X in 60 days" nudges. |
| Documents vault | IDs, policies, titles, military records, manuals, scans. | Expiration alerts for licenses, passports, and policies. Every document is searchable through OCR. |
| Career and learning | Skills, certifications, language practice, courses, training files. | Recert and test date reminders. Daily language practice streak. Tracks study hours. |
| Meals and groceries | Pantry, shopping list, meal plan, recipes. | Low-stock items go on the list. Meal plan builds the grocery list. Expiration alerts for food. |
| Preparedness | Emergency kit, generator, fuel, water, smoke and CO detectors, outage plan. | Kit item expirations. Monthly generator test. Detector battery and replacement dates. |
| Digital life | Domains, subscriptions, backups, device updates, account security checks. | Domain and license renewals. Monthly backup check. Quarterly security review. Accounts and passwords are covered by the Email and accounts module. |
| Journal and brain dump | Quick notes, ideas, daily log, wins. | Turns notes with dates into events. Pulls wins into the weekly review. |
| Travel | Trips, packing lists, bookings, rewards numbers, points balances, traveler IDs. | Packing list from a template. Checks that IDs are valid for the trip dates. Pauses home reminders while away. |
| Email and accounts | Email inboxes, every online account, logins, 2FA status, recovery info. | Bills and receipts from email flow into Finance. Bulk unsubscribe. Password audit for weak, reused, and leaked passwords. |

### Creative reminder library

These ship as ready-made templates. Each one is one click to turn on.

- **Home:** furnace filter (every 90 days), dryer vent cleaning (yearly), fridge coils (yearly), water heater flush (yearly), dishwasher filter (monthly), range hood filter (quarterly), garbage disposal clean (monthly), gutter check (fall), heat tape test (before first freeze).
- **Alaska seasonal:** studded tire window open and close, block heater check (fall), garden hose and outdoor spigot shutoff (before freeze), roof snow load check (after heavy snow), PFD application window (January 1 to March 31).
- **Vehicle:** oil change, tire rotation, cabin and engine air filters, wiper blades (fall), battery test (before winter), coolant check, registration renewal.
- **Safety:** smoke and CO detector test (monthly), detector replacement (every 10 years), fire extinguisher check (yearly), first aid kit restock.
- **Money:** insurance shopping (yearly), credit report check (every 4 months, rotating bureaus), subscription audit (quarterly), quarterly estimated taxes on rental income.
- **Life admin:** passport renewal (9 months before expiry), license renewal, dentist (every 6 months), eye exam (yearly), backup photos (monthly).
- **Odd but useful:** mattress rotation (every 3 months), toothbrush swap (every 3 months), check under sinks for leaks (quarterly), test sump pump (spring), clean phone and keyboard (monthly).

All seasonal dates are editable. Where a rule depends on law, such as tire stud dates, the app shows the source and asks you to confirm the dates each year.

## Automation and reminders

Every automation is a rule with three parts: a trigger, an optional condition, and one or more actions. Rules are built with a simple form, not code. You can also type them in plain English and the AI turns them into a rule.

### Triggers

- **Time:** a date, a repeat, or an offset ("30 days before expiry").
- **Usage:** miles driven, hours run, pill count, pantry count.
- **Data change:** a new transaction, a task completed, a tenant moved in.
- **Weather:** first forecast freeze, heavy snow, heat wave.
- **Inbox (optional):** a bill or receipt email arrives.
- **Pattern:** something stops happening, like no workouts in 7 days.

### Actions

Notify, create a task, add a calendar block, add to a shopping list, log to the timeline, update a budget, or start a checklist. One trigger can fire many actions.

### Example rules

| When | Then |
| --- | --- |
| Forecast shows first hard freeze | Create "Winterize" checklist: hoses, spigots, heat tape, block heater, wiper fluid. |
| Car odometer passes last oil change + 5,000 mi | Create oil change task. Suggest a time slot this week. |
| A charge from a new merchant repeats 2 months in a row | Ask: "Is this a subscription?" If yes, track it and remind before renewal. |
| A subscription price goes up | Alert with old and new price and a "Cancel?" button. |
| Rent not marked paid 1 day after due | Remind you to check. On day 3, draft a polite late notice for review. |
| Tenant lease ends in 45 days | Start turnover: renew or list the room, inspection checklist, deposit return deadline. |
| Receipt scanned with a warranty | Set warranty end and return window reminders. Link the receipt to the item. |
| Medication count drops below 7 days | Create "Request refill" task with the pharmacy phone number. |
| Birthday in 14 days | Remind with saved gift ideas. Create "Order gift" task at 10 days. |
| Same task skipped 3 times | Ask: "Too big, wrong time, or not needed?" Then split, move, or delete it. |

### The reminder ladder

ADHD brains tune out alerts fast. MyLife uses a ladder instead of one ping.

1. **Quiet:** shows in the Today view only.
2. **Nudge:** a desktop notification at a smart time, not 3 AM.
3. **Persistent:** a notification that stays until you act or snooze.
4. **Escalate:** a push to your phone (via a free push service) or an email.

You set how high each type of reminder may climb. Bills and meds can escalate. Fridge coils cannot.

### Snooze that thinks

Snooze options are "later today," "tomorrow," "this weekend," and "when I'm free." The last one finds the next open block in your calendar. Snoozing the same thing 5 times triggers a check-in question.

## ADHD support features

These features sit on top of every module. They are the reason MyLife is more than a to-do app.

- **Quick capture.** Press a hotkey anywhere. Type "oil change next Tuesday" or "Pay Jake $40." The app files it in the right module. Voice capture is included.
- **Today view.** One screen with the 3 most important things, today's calendar, and due items. A "just one thing" button hides everything except the next step.
- **Daily brief.** A short morning summary: weather, schedule, bills due, and one goal step. Ends with a single question: "What is your one must-do today?"
- **Evening shutdown.** A 2-minute routine. Review what got done, move what didn't, and set tomorrow's top 3.
- **Weekly review.** Guided, 15 minutes. Shows wins, slipped items, money in and out, and upcoming week.
- **Time blindness tools.** Visual countdown bars on every event. "Leave in 15 minutes" alerts that include drive time. Estimates vs. actual time tracking that learns how long things really take you.
- **Focus mode.** A timer (Pomodoro or custom) that hides the rest of the app. Optional body-doubling sound or ambient noise.
- **Task breakdown.** Click "Break it down" on any task. The AI suggests small steps. The first step is always under 5 minutes.
- **Energy matching.** Tag tasks as high, medium, or low energy. When you feel drained, the app shows only low-energy tasks.
- **Rewards.** XP for finished tasks, streaks for habits, and a monthly "wins" summary. Rewards can be turned off.
- **Gentle overdue.** Overdue items don't pile up in red. They move to a "Catch-up" list and the app helps you pick 1 to clear.
- **Doom-pile sweeper.** Old, untouched tasks get archived after 30 days with a one-time check. Nothing is deleted.
- **Hyperfocus guard.** Optional alert when you have been in one window or task for too long, so you don't miss meals or meetings.

## Priority modules: money, gym, chores

These three slip most often, so they get the deepest design and ship first after the core.

### Finance

- **Safe-to-spend number.** One number on the Today view: balance minus bills due before next payday minus savings goals. No math needed in the moment.
- **Bills on the calendar.** Every bill shows as an event. Reminder 5 days out. One tap marks it paid.
- **72-hour pause.** Add an impulse buy to a wish list. After 3 days the app asks, "Still want it?"
- **Subscription hunter.** Finds repeat charges, flags price hikes, and reminds before renewals.
- **10-minute money check.** Part of the weekly review. Tag new transactions, check the budget, done.
- **Rental money kept separate.** Room rent and rental expenses get their own tags for tax time.
- **Start simple.** CSV import from your bank first. Automatic bank feeds come later.

### Your cards

All five of your cards are listed as supported by Plaid. Five cards use 5 of the 10 free Trial connections, leaving room for checking and savings.

| Card | Connection | Note |
| --- | --- | --- |
| Chase Sapphire | Plaid | Usually very reliable. |
| Wells Fargo credit card | Plaid | Usually reliable. |
| Capital One Mastercard | Plaid | Supported, but users report frequent reconnects. Manual fallback ready. ([report](https://community.tillerhq.com/t/best-credit-cards-to-use-with-tiller/142?page=2)) |
| Military Star | Plaid | Supported. ([tracker](https://openbankingtracker.com/plaid/military-star-card)) |
| Credit One Visa | Plaid | Supported. ([tracker](https://openbankingtracker.com/plaid/credit-one-bank)) |

Support lists come from a third-party tracker, so we test each connection during setup.

### Manual accounts

Any card, loan, or account can be added by hand and tracked without a bank link.

- Enter the balance, APR, minimum payment, due date, and credit limit.
- Update the balance in 10 seconds from a monthly reminder on statement day.
- Log payments by hand or import a CSV.
- Switch an account between manual and Plaid at any time. History is kept.
- Mix freely. The budget and debt tracker treat linked and manual accounts the same.

### Debt-free tracker

One screen answers one question: **"When will I be debt free?"** The mortgage is left out by default. Any debt can be included or excluded with a switch.

- **Debt-free date.** A month-by-month projection using each card's balance, APR, and your monthly payment budget.
- **Pick a strategy.** Avalanche pays the highest APR first and saves the most interest. Snowball pays the smallest balance first and gives faster wins. You can also set a custom order. The app shows the date and total interest for each.
- **What-if slider.** "Add $100 a month" instantly shows the new date and the interest saved.
- **Windfall planner.** Plan a PFD check, tax refund, or bonus. "Put $800 of the PFD on Credit One" shows the new date.
- **Payment plan.** Each month the app tells you exactly how much to pay on each card. Due dates go on the calendar with reminders.
- **Setback alerts.** New charges on a card you are paying off show how far the date moved, like "+2 weeks."
- **Promo APR watch.** Alerts 60 and 30 days before a 0% or promo rate ends.
- **Utilization check.** Shows each card's usage of its limit, and flags anything over 30%.
- **Wins.** A progress bar for total debt paid, and a celebration each time a card hits $0.

APR and minimum payment may need a one-time manual entry from your statement. Plaid's card details product may not be in the free Trial.

### Gym

No routine yet? MyLife builds one with you.

- **Routine builder.** A 5-question setup: goal, days per week, time per session, equipment, and experience. It suggests a beginner plan, such as 3-day full body.
- **Phase 0: just show up.** For the first 2 weeks, the only goal is getting to the gym on your days. Any workout counts.
- **Auto progression.** When you hit all your reps, the app raises the weight next time. No planning needed.
- **Exercise library.** Each exercise has a short how-to and the muscles it works, pulled from a free exercise database.
- **4-week check-in.** The app asks what felt good and what felt bad, then adjusts the plan.

* **Gym profiles.** You use two gyms. Each gets an equipment profile. Planet Fitness usually has dumbbells, machines, and Smith machines, but no free barbell racks. Your local gym has full equipment. Pick the gym when you start a workout, and the app swaps in exercises that fit. For example, Smith machine squats replace barbell squats at Planet Fitness.

- **Gym blocks on the calendar.** Set your days. The app blocks the time and sends a "leave in 15 minutes" alert.
- **Bag check the night before.** A short checklist fires the evening before a gym day.
- **Fast workout log.** Pick a template. Last session's weights and reps show next to each exercise. Log a set in one tap.
- **Minimum version.** Every workout has a 10-minute fallback. Doing the short one still keeps the streak.
- **Missed sessions.** After 2 misses in a row, the app asks why and offers to reschedule. No guilt messages.
- **Progress you can see.** Charts for attendance and lift numbers. Phone health app import comes in Phase 4.

### Daily chores

- **Chore list by frequency.** Daily, weekly, and monthly chores, grouped by room or zone.
- **10-minute reset.** Each day the app picks chores that fit 10 minutes and starts a timer.
- **Chore roulette.** Stuck? Hit one button and it picks a chore for you.
- **Zone of the week.** One area gets a deeper clean each week, on rotation.
- **Laundry and dishwasher timers.** Start the washer, tap one button, and get a reminder to move it to the dryer.
- **Trash night.** Reminder the night before pickup.
- **No pile-up.** Missed daily chores reset the next day instead of stacking up.
- **Shared chore chart (optional).** A printable chart for common areas the renters share.

## Travel and rewards

One place for every trip and every rewards program. Most rewards programs have no public API, so balances are entered by hand or read from statement emails.

- **Rewards wallet.** Airline, hotel, rental car, and credit card programs. Member numbers copy in one click.
- **Points tracker.** Balances, plus alerts before points expire.
- **Traveler IDs.** Known Traveler Number, Global Entry, passport, and REAL ID, each with expiry alerts.
- **Card perks.** Annual fees and use-it-or-lose-it credits get reminders before they reset.
- **Trip builder.** Confirmation emails become a trip itinerary automatically (through the Email module).
- **Check-in alerts.** A reminder 24 hours before each flight.
- **Packing templates.** Including an Alaska winter list.
- **Away mode.** While you travel, chores and gym pause, and home reminders wait until you get back.

## Email and accounts

This module manages your inboxes and keeps a full inventory of your online accounts.

### Email

- **Connect Gmail or Outlook.** Uses the official free APIs, or IMAP with an app password.
- **Three-pile triage.** Each email goes to Act, Read later, or Archive. The goal is a calm inbox, not zero.
- **Bulk unsubscribe.** Lists every newsletter and sender by volume. One click unsubscribes using the standard unsubscribe link.
- **Smart pickup.** Bills, receipts, shipping updates, and travel bookings get pulled into the right module.
- **Follow-up reminders.** "No reply in 3 days" alerts on emails you mark as waiting.

Note: for a personal Gmail app that is not published, Google may expire the login every 7 days. IMAP with an app password avoids that. We will confirm the current rules at build time.

### Accounts and passwords

You can import your Google and Firefox passwords. Both browsers export a CSV file. That file holds every password in plain text, so the import must be handled with care.

**Decision: a built-in local vault.** Passwords live in their own encrypted file on your PC, locked separately from the rest of the app.

**Import flow**

1. You export the CSV from Google Password Manager or Firefox.
2. MyLife imports it into the vault and builds your account list.
3. It runs a health check: weak, reused, and leaked passwords. The leak check never sends your password.
4. MyLife securely deletes the CSV and reminds you to check Downloads and the Recycle Bin for copies.

**How the vault is locked**

- **Separate file, separate password.** The vault is its own file with its own master password, different from the app password. Opening MyLife does not open the vault.
- **Proven encryption only.** We use only the audited libraries listed in Technical specification. The key comes from your master password through Argon2id, which makes guessing very slow. No homemade crypto.
- **Windows Hello unlock (optional).** After you unlock once with the master password, you can use your PIN or fingerprint for quick unlocks. The master password is still needed after a restart.
- **Auto-lock.** The vault locks after 5 minutes idle, when the PC sleeps, and when you lock Windows.
- **Re-check to reveal.** Passwords stay masked. Showing or copying one asks for Windows Hello if the vault has been open a while.
- **Clipboard clears.** Copied passwords are wiped from the clipboard after 30 seconds.
- **Walled off.** Passwords never go into search, logs, the timeline, backups in plain form, or any AI call.

**Recovery**

If you lose the master password, the passwords are gone. That is the price of real encryption. At setup, MyLife creates a one-time recovery key for you to print and store somewhere safe, like a fire safe.

**Extras**

- Password generator with adjustable length.
- "Last changed" date on every login, with a yearly nudge for important accounts.
- 2FA codes stay in your phone's authenticator app. Keeping them in the same vault as passwords would turn two factors into one.

### Account inventory features

- Which email each account uses, and recovery phone and email.
- 2FA on or off for each account, with a nudge to turn it on for important ones.
- "Change password" tasks for any leaked or reused password, sorted by importance (email and bank first).
- Dead account cleanup: flags accounts you have not used in years so you can close them.

## Data model, security and privacy

All data lives in one encrypted file on your computer. Nothing leaves the machine unless you turn on a feature that needs it.

The full schema is in Technical specification below. Most module data fits in `items` with JSON fields. Money, debt, and gym data get their own tables because they need exact math and fast queries.

### Security

- The database is encrypted with SQLCipher (AES-256). The key comes from your master password.
- Optional Windows Hello unlock.
- Auto-lock after inactivity.
- Sensitive fields, such as ID numbers, get a second layer of encryption and stay masked until clicked.
- Bank logins are never stored. Bank feeds use read-only tokens from the provider. Passwords follow the vault design in Email and accounts.
- Encrypted automatic backups to a folder you choose, plus a one-click export to plain files.

### Privacy

- No telemetry by default.
- AI features are opt-in, per module. Health and finance are off by default.
- Before any AI call, the app shows what will be sent. Account and ID numbers are stripped first.

## Technical specification

This section is binding. Follow it exactly unless Tanner approves a change.

### Repo layout

```text
mylife/
  src-tauri/            Rust backend
    src/
      db/               connection, migrations, queries
      rules/            trigger checks, action runner
      scheduler/        reminder timing, ladder escalation
      vault/            password vault
      integrations/     plaid, google, nws, ntfy, hibp, nhtsa, wger
      modules/          backend logic per module
      agent.rs          tray agent entry point
  src/                  React + TypeScript UI
    core/               layout, Today view, quick capture, search
    modules/<name>/     screens, components, starter-rules.json
  migrations/           numbered SQL files: 0001_init.sql, 0002_...
  tests/
  docs/                 SPEC.md (this doc), DECISIONS.md, CHANGELOG.md, IDEAS.md
```

### Approved libraries

| Need | Use |
| --- | --- |
| App shell | Tauri 2 with plugins: notification, autostart, global-shortcut, dialog, fs |
| Database | `rusqlite` with the `bundled-sqlcipher` feature |
| Vault crypto | RustCrypto `argon2` (Argon2id) and `chacha20poly1305` (XChaCha20-Poly1305). No other crypto. |
| Secrets (API tokens) | `keyring` crate, backed by Windows Credential Manager |
| Secure memory | `zeroize` for keys and passwords in memory |
| HTTP | `reqwest` with rustls |
| UI | React 18, TypeScript (strict), Tailwind, TanStack Query, Zustand |
| Charts | Recharts |
| Dates | Store UTC, show America/Anchorage. Repeats use RFC 5545 RRULE (`rrule` crate). |
| OCR | Tesseract (Phase 3) |
| Tests | `cargo test` for Rust, Vitest for UI, Playwright for key flows |
| Quick capture parsing | chrono-node for dates and times ("next Tuesday at 3"), plus simple keyword rules to pick the module. Free, offline, no AI. |
| Voice capture | whisper.cpp through whisper-rs, small model, runs locally. Free. |
| Local AI (optional) | Ollama with a small open model. If the PC can't run it, AI buttons stay hidden. No paid AI APIs. |

### Hard rules

- Money is stored as integer cents. Never use floats for money.
- APR is stored as integer basis points (19.99% = 1999).
- Every table has `created_at` and `updated_at` (UTC ISO 8601). IDs are UUID v7 strings.
- Take an automatic backup before every migration.
- Plaid, Google, and other tokens live only in Windows Credential Manager. Never in the database, logs, or files.
- Logs never contain passwords, tokens, account numbers, or transaction details.
- Deletes are soft (`archived_at`) except where the user confirms a hard delete.

### Database schema (main file: mylife.db, encrypted)

```sql
CREATE TABLE items (
  id TEXT PRIMARY KEY,
  module TEXT NOT NULL,          -- 'tasks','chores','home','vehicle','gym',...
  type TEXT NOT NULL,            -- 'task','chore','appliance','vehicle',...
  title TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'active',  -- active, done, archived
  energy TEXT,                   -- low, medium, high
  est_minutes INTEGER,
  actual_minutes INTEGER,
  data TEXT NOT NULL DEFAULT '{}',        -- JSON custom fields
  created_at TEXT NOT NULL, updated_at TEXT NOT NULL, archived_at TEXT
);
CREATE TABLE events (
  id TEXT PRIMARY KEY, item_id TEXT REFERENCES items(id),
  title TEXT NOT NULL, starts_at TEXT NOT NULL, ends_at TEXT,
  all_day INTEGER NOT NULL DEFAULT 0, rrule TEXT,
  source TEXT NOT NULL DEFAULT 'local',  -- local, google
  external_id TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL
);
CREATE TABLE rules (
  id TEXT PRIMARY KEY, module TEXT NOT NULL, name TEXT NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 1,
  trigger_json TEXT NOT NULL, conditions_json TEXT NOT NULL DEFAULT '[]',
  actions_json TEXT NOT NULL, max_ladder INTEGER NOT NULL DEFAULT 2,
  last_fired_at TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL
);
CREATE TABLE reminders (
  id TEXT PRIMARY KEY, rule_id TEXT REFERENCES rules(id), item_id TEXT REFERENCES items(id),
  fire_at TEXT NOT NULL, ladder_level INTEGER NOT NULL DEFAULT 1,
  snooze_count INTEGER NOT NULL DEFAULT 0,
  state TEXT NOT NULL DEFAULT 'pending',  -- pending, fired, snoozed, done, dismissed
  created_at TEXT NOT NULL, updated_at TEXT NOT NULL
);
CREATE TABLE links (from_id TEXT NOT NULL, to_id TEXT NOT NULL, relation TEXT NOT NULL,
  PRIMARY KEY (from_id, to_id, relation));
CREATE TABLE meters (id TEXT PRIMARY KEY, item_id TEXT NOT NULL REFERENCES items(id),
  value REAL NOT NULL, unit TEXT NOT NULL, read_at TEXT NOT NULL);
CREATE TABLE timeline (id TEXT PRIMARY KEY, item_id TEXT, action TEXT NOT NULL,
  details TEXT, at TEXT NOT NULL);
CREATE TABLE files (id TEXT PRIMARY KEY, item_id TEXT REFERENCES items(id),
  path TEXT NOT NULL, sha256 TEXT NOT NULL, ocr_text TEXT, created_at TEXT NOT NULL);

-- Finance
CREATE TABLE accounts (
  id TEXT PRIMARY KEY, name TEXT NOT NULL,
  kind TEXT NOT NULL,            -- checking, savings, credit_card, loan, mortgage
  source TEXT NOT NULL,          -- plaid, manual, csv
  institution TEXT, last4 TEXT, credit_limit_cents INTEGER,
  include_in_debt_free INTEGER NOT NULL DEFAULT 1,  -- mortgage defaults to 0
  plaid_account_id TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, archived_at TEXT
);
CREATE TABLE debts (
  account_id TEXT PRIMARY KEY REFERENCES accounts(id),
  balance_cents INTEGER NOT NULL, apr_bp INTEGER NOT NULL,
  min_payment_cents INTEGER NOT NULL, due_day INTEGER NOT NULL,
  promo_apr_bp INTEGER, promo_ends_on TEXT,
  balance_as_of TEXT NOT NULL, updated_at TEXT NOT NULL
);
CREATE TABLE transactions (
  id TEXT PRIMARY KEY, account_id TEXT NOT NULL REFERENCES accounts(id),
  posted_on TEXT NOT NULL, amount_cents INTEGER NOT NULL,  -- negative = money out
  merchant TEXT, category TEXT, is_rental INTEGER NOT NULL DEFAULT 0,
  item_id TEXT, external_id TEXT UNIQUE, created_at TEXT NOT NULL
);
CREATE TABLE bills (id TEXT PRIMARY KEY, name TEXT NOT NULL, amount_cents INTEGER,
  account_id TEXT, rrule TEXT NOT NULL, autopay INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL, updated_at TEXT NOT NULL);

-- Gym
CREATE TABLE gym_profiles (id TEXT PRIMARY KEY, name TEXT NOT NULL, equipment TEXT NOT NULL); -- JSON list
CREATE TABLE workouts (id TEXT PRIMARY KEY, gym_profile_id TEXT, template_id TEXT,
  started_at TEXT NOT NULL, ended_at TEXT, is_minimum INTEGER NOT NULL DEFAULT 0);
CREATE TABLE workout_sets (id TEXT PRIMARY KEY, workout_id TEXT NOT NULL REFERENCES workouts(id),
  exercise_id TEXT NOT NULL, set_no INTEGER NOT NULL, weight_lb REAL, reps INTEGER);

CREATE VIRTUAL TABLE search_index USING fts5(item_id, module, title, body);  -- never index vault data
```

The vault is a separate file, `vault.db`, not encrypted by SQLCipher but by the vault crypto above. Each password is encrypted on its own with a fresh nonce.

```sql
CREATE TABLE vault_meta (key TEXT PRIMARY KEY, value BLOB NOT NULL);  -- salt, kdf params, key check
CREATE TABLE vault_entries (id TEXT PRIMARY KEY, account_item_id TEXT NOT NULL,
  nonce BLOB NOT NULL, ciphertext BLOB NOT NULL, changed_at TEXT NOT NULL);
```

### Rule format

Rules are JSON. Starter rules ship in each module's `starter-rules.json`.

```json
{
  "name": "Oil change by mileage",
  "trigger": { "type": "meter", "item": "<vehicle item id>", "unit": "mi", "every": 5000 },
  "conditions": [],
  "actions": [
    { "type": "create_task", "title": "Oil change", "energy": "medium", "est_minutes": 60 },
    { "type": "suggest_slot", "within_days": 7 }
  ],
  "max_ladder": 3
}
```

| Trigger type | Fields |
| --- | --- |
| `time` | `at` or `rrule` |
| `offset` | `event` or `date_field`, `days_before` |
| `meter` | `item`, `unit`, `every` or `at_value` |
| `data_change` | `table`, `match` (field filters) |
| `weather` | `condition` (freeze, snow\_in, heat), `threshold` |
| `inbox` | `kind` (bill, receipt, shipping, booking) |
| `pattern` | `item_type`, `absent_days` (nothing happened for N days) |

Action types: `notify`, `create_task`, `add_event`, `add_to_list`, `start_checklist`, `log`, `suggest_slot`, `ask` (shows a question with buttons).

### Agent and reminders

- The agent starts at Windows login (autostart plugin) and lives in the system tray.
- It checks rules and reminders every 60 seconds.
- After sleep or shutdown, each missed reminder fires once on wake. Never fire a backlog of repeats.
- Quiet hours default to 10 PM to 7 AM. Only reminders marked urgent (bills due today) may break through.
- Ladder timing: level 2 fires at `fire_at`. If not acted on in 2 hours, level 3. Level 3 repeats every 4 hours. Level 4 (phone push) only if the rule allows it.

### Debt payoff projection

Run a monthly simulation. Unit test it against a spreadsheet.

1. Each month, add interest to each debt: `balance * apr / 12` (use the promo APR until `promo_ends_on`).
2. Pay the minimum on every debt.
3. Put the rest of the monthly debt budget on the target debt. Avalanche targets the highest APR. Snowball targets the smallest balance. Custom uses the user's order.
4. When a debt hits $0, its minimum rolls into the budget for the next target.
5. Stop when all included debts are $0 (that month is the debt-free date) or after 600 months.
6. If the budget is less than the sum of minimums, or a debt's interest exceeds its payment, show a warning instead of a date.

Rounding: round interest to the nearest cent each month. Results may differ from the card issuer by a few dollars because issuers compute interest daily. Say so in the UI.

### Safe-to-spend

`safe_to_spend = checking balance - bills due before next payday - savings goal contributions due before next payday - a buffer (default $100)`. Payday is set by the user as an RRULE. Never show a negative number as a dollar amount. Show "Tight. Hold spending." instead.

## Free and cheap APIs

Most of MyLife's automation can run on free services. Total cost for one person should be $0 to about $15 a year, with no AI costs.

### Bank and expense feeds

Yes, you can sign in through Plaid Link to pull your expenses. Plaid now has a free Trial plan for new developer accounts in the US and Canada. It uses real bank data and allows up to 10 bank connections. That covers most people. ([Plaid](https://support.plaid.com/hc/en-us/articles/16110502116887))

| Service | Use | Cost |
| --- | --- | --- |
| Plaid (Link) | Bank and card transactions, balances | Free Trial plan, up to 10 connections |
| SimpleFIN Bridge | Backup bank feed, simple setup, updates daily | $15 per year ([SimpleFIN](https://bridge.simplefin.org)) |
| Teller | Alternative bank feed | Free developer tier ([Teller](https://docs.rutter.com/platforms/banking/teller)) |
| CSV import | Works with any bank, no account needed | Free |

Plan: start with Plaid's free Trial. Keep CSV import as the fallback for any bank Plaid can't reach.

### Everything else

| Service | Use in MyLife | Cost |
| --- | --- | --- |
| Google Calendar and Gmail APIs | Calendar sync, email triage, bill pickup | Free |
| Microsoft Graph | Outlook calendar and mail | Free |
| National Weather Service API | Freeze and snow triggers for Alaska | Free, no key |
| Open-Meteo | Backup weather forecasts | Free for personal use |
| NHTSA vPIC and Recalls | Decode your VIN, check for recalls automatically | Free |
| Have I Been Pwned (Pwned Passwords) | Leaked password check without sending your password | Free |
| wger | Exercise library for the gym module | Free, open source |
| Open Food Facts | Scan barcodes for pantry items | Free |
| USDA FoodData Central | Nutrition data for meals | Free with a key |
| Nager.Date | Holidays for scheduling | Free |
| ntfy | Push reminders to your phone | Free |
| Pushover | Push reminders to your phone, more reliable | About $5 one time |
| Bitwarden CLI | Password vault link | Free |
| Tesseract | Read receipts and documents on your PC | Free, offline |
| Ollama (local AI) | Optional local AI for task breakdown and weekly review | Free, runs on your PC. Needs about 8 GB of free RAM. |

The bank feed prices above were checked this week. The rest come from general knowledge, so we confirm each one at build time.

## UI layout and key screens

The main window has three zones: a slim left sidebar for modules, a center work area, and a right panel for today's agenda. The right panel can collapse. Dark mode is the default with a light option.

### Key screens

1. **Today.** Top 3 tasks, timeline of the day with countdown bars, due bills, active reminders, and one goal step.
2. **Calendar.** Day, week, and month views. Tasks can be dragged onto the calendar to time-block them.
3. **Life dashboard.** One tile per module with a health score. Example: "Home: 2 items due soon." Click a tile to open the module.
4. **Module view.** A list or card grid of items with filters. Each item opens a detail page with its schedule, files, links, and history.
5. **Rules.** A list of all automations, grouped by module, with an on/off switch and "last fired" time.
6. **Money.** Budget bars, upcoming bills, subscription list, and net worth trend chart.
7. **Review.** The daily brief, evening shutdown, and weekly review flows.
8. **Search.** One box that searches every module, document, and receipt.

### Design rules

- No more than 7 items visible in any list by default. A "show more" link reveals the rest.
- One accent color for "needs action." Everything else stays calm.
- Big click targets and keyboard shortcuts for everything.
- Every screen answers one question: "What should I do next?"

## Roadmap

We build in four phases. Each phase ends with a gate, so the app is useful before it grows. Phase 1 is small on purpose. A tool you actually open every day beats a huge one you abandon.

&#91;embedded content: build plan · 4 phases, 3 gates\]

We do not start a phase until the gate before it passes. Phase 2 ships chores first, then gym, then money. Chores and gym are quick wins. Money takes the most setup.

### Phase 1 acceptance criteria

All must pass before Phase 2 starts.

- [ ] Installs on Windows 11 from an MSI. Warm start opens in under 2 seconds.
- [ ] First run creates an app password. `mylife.db` cannot be opened without it (test with a plain SQLite tool).
- [ ] Global hotkey Ctrl+Shift+Space opens quick capture from any app. Typing and pressing Enter saves to the inbox.
- [ ] Today view shows top 3 tasks, today's events with countdown bars, and due items. No more than 7 items per list by default.
- [ ] Tasks: create, edit, complete, split into steps, set energy and time estimate. Missed tasks move to Catch-up, not a red overdue list.
- [ ] Calendar: day, week, and month views. Drag a task onto the calendar to time-block it. Calendar sync per Decisions.
- [ ] Rules: time and offset triggers fire within 60 seconds of target. Unit tests cover RRULE repeats and month-end dates.
- [ ] Agent runs from login with the main window closed. Missed reminders fire once on wake.
- [ ] Ladder levels 1 to 3 and quiet hours work as specified. Smart snooze options work, including "when I'm free."
- [ ] Daily encrypted backup to a chosen folder. A restore from backup is tested and works.
- [ ] Daily brief and evening shutdown flows work.
- [ ] All Rust and UI tests pass.

### Phase 2 acceptance criteria

- [ ] Chores: daily, weekly, and monthly lists. The 10-minute reset picks chores that fit and starts a timer. Missed daily chores reset, not stack.
- [ ] Gym: routine builder creates a plan. Both gym profiles exist. Logging a set takes one tap. Last session's numbers show. Minimum workouts keep the streak.
- [ ] Plaid Trial connects all 5 cards. Each card can switch to manual. CSV import works for at least one bank's export.
- [ ] Debt tracker: projection matches a hand-built spreadsheet within $5 and the same payoff month, for both avalanche and snowball, using test data.
- [ ] Safe-to-spend matches a hand calculation on test data.
- [ ] Subscriptions are detected from 2 or more repeat charges. Price hikes alert.
- [ ] Bill reminders fire 5 days before due.

## Decisions and defaults

The builder uses these as given. Items marked "Default" are not yet confirmed by Tanner. Use the default, and change it only if Tanner says so.

| Topic | Decision |
| --- | --- |
| Builder | An AI builds. Tanner tests and approves each phase. |
| Platform | Windows 11 only for v1. Mac is out of scope. |
| Top priorities | Chores, gym, and finance. They lead Phase 2. |
| Bank feeds | Plaid free Trial. CSV import and manual accounts as fallbacks. |
| Cards | Wells Fargo credit card, Chase Sapphire, Military Star, Capital One Mastercard, Credit One Visa. |
| Debt-free date | Excludes the mortgage by default. Any debt can be toggled in or out. |
| Gyms | Planet Fitness and a local full-equipment gym. Tanner has no routine yet. |
| Passwords | Built-in local vault with its own master password. Import from Google and Firefox CSV. |
| 2FA codes | Not stored in the vault. They stay in a phone authenticator app. |
| Calendar sync | Google Calendar, two-way. |
| Phone alerts | Default: ntfy push, built in Phase 4. |
| AI features | No paid AI, ever. Quick capture uses free rule-based parsing. Free local AI through Ollama is on by default, since Tanner's PC has the RAM. It never reads health, finance, or vault data. Every feature must work without AI. |
| Rewards (XP, streaks) | Default: on, with a setting to turn them off. |
| Theme | Dark by default, light option. |

## Build risks

- **Scope.** 18 modules is a lot. Stick to the phase plan. A small app used daily beats a big unfinished one.
- **Money math.** Bugs here cost real money and trust. Use integer cents and test against spreadsheets.
- **Vault.** A security bug here is the worst possible failure. Use only the approved libraries and keep the code small and well tested.
- **Bank connections drop.** Plaid links break sometimes, especially Capital One. Show a clear "Reconnect" prompt and never silently show stale balances. Always show the "as of" date.
- **Data loss.** One encrypted file holds everything. Backups must be on by default and restore must be tested.
