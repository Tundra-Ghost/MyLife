-- 0002_events_archive.sql
-- Soft delete for events (spec hard rule: deletes set archived_at).
ALTER TABLE events ADD COLUMN archived_at TEXT;
CREATE INDEX events_starts_at ON events (starts_at);
