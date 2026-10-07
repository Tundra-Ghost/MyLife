-- 0003_reminders.sql
-- Extra reminder fields the agent needs, and a small key/value settings table.
ALTER TABLE reminders ADD COLUMN title TEXT NOT NULL DEFAULT '';
ALTER TABLE reminders ADD COLUMN max_ladder INTEGER NOT NULL DEFAULT 2;
-- Urgent reminders (bills due today) may break through quiet hours.
ALTER TABLE reminders ADD COLUMN urgent INTEGER NOT NULL DEFAULT 0;
ALTER TABLE reminders ADD COLUMN last_notified_at TEXT;
CREATE INDEX reminders_state_fire ON reminders (state, fire_at);

CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
