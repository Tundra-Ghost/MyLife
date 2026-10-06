//! Password vault (`vault.db`). Built in Phase 3 with the Email and accounts module.
//!
//! Binding design (docs/SPEC.md): separate file and master password,
//! Argon2id key derivation, XChaCha20-Poly1305 per entry with a fresh
//! nonce, `zeroize` for secrets in memory. No other crypto.
