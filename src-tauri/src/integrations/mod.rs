//! Outside services: plaid, google, nws, ntfy, hibp, nhtsa, wger.
//! Each one is added in the phase that needs it. Tokens live only in
//! Windows Credential Manager (`keyring` crate), never in the database.
