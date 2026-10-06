-- vault_0001_init.sql
-- Schema for vault.db (the separate password vault file). Copied from docs/SPEC.md.
-- The vault is built in a later phase. This file is here so the schema lives with the others.

CREATE TABLE vault_meta (key TEXT PRIMARY KEY, value BLOB NOT NULL);  -- salt, kdf params, key check
CREATE TABLE vault_entries (id TEXT PRIMARY KEY, account_item_id TEXT NOT NULL,
  nonce BLOB NOT NULL, ciphertext BLOB NOT NULL, changed_at TEXT NOT NULL);
