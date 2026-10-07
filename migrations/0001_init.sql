-- 0001_init.sql
-- Initial schema for mylife.db, copied from docs/SPEC.md (Technical specification).
-- Do not edit a migration after it ships. Add a new numbered file instead.

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
