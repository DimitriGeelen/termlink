//! Tests that `termlink sidecar ...` re-executes its finalizer through the same subcommand.
//! PH-2 row P1 (T-3365): `termlink sidecar ...` re-executes its finalizer
//! through the same subcommand. Its own test binary, because the prefix is set
//! once per process.
use termlink_sidecar::{cli, hook};

#[test]
fn finalizer_runs_through_termlink_sidecar_when_started_as_termlink() {
    // Before any verb ran, the prefix is empty: the `termlink-sidecar` binary form.
    assert_eq!(hook::finalizer_args("n1"), ["finalize", "--nonce", "n1"]);
    // An unknown verb only prints usage, but it records the prefix.
    assert_eq!(cli::run(&["sidecar"], &["no-such-verb".to_string()]), 2);
    assert_eq!(hook::finalizer_args("n1"), ["sidecar", "finalize", "--nonce", "n1"]);
}
