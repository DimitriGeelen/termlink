//! Crate-wide serialization primitive for tests that mutate process-global
//! state — specifically `std::env::current_dir`, `std::env::set_var("HOME")`,
//! and anything else that is not thread-safe to read while another test writes.
//!
//! Cargo runs tests in parallel across threads by default. Tests that swap
//! CWD or HOME without a shared lock race with each other, producing flaky
//! "save cwd failed" or HOME-mismatch panics.
//!
//! Usage:
//! ```ignore
//! let _guard = crate::test_env_lock::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
//! // ... mutate CWD / HOME / other global env ...
//! ```

#![cfg(test)]

use std::sync::Mutex;

pub(crate) static ENV_LOCK: Mutex<()> = Mutex::new(());
