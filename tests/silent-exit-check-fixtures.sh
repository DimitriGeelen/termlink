#!/usr/bin/env bash
# silent-exit-check-fixtures.sh (T-2666)
#
# Hermetic load-bearing proof for scripts/check-silent-exit.sh — no live binary, no repo
# state. Builds a scratch scan-root with synthetic .rs files exercising each branch of the
# detector and asserts the check's verdict + exit code on each. This is the deterministic
# complement to the real-tree revert proof (reverting the eprintln! at metadata.rs:296 /
# session.rs:617 / remote.rs:1301 re-fires the real check).
#
# Exit 0 = all fixtures pass; exit 1 = a fixture assertion failed.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="$REPO_ROOT/scripts/check-silent-exit.sh"
[ -f "$CHECK" ] || { echo "FAIL: check script not found at $CHECK" >&2; exit 1; }

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT
SRC="$SCRATCH/src"; mkdir -p "$SRC"
ALLOW="$SCRATCH/allowlist"; : > "$ALLOW"

pass=0; fail=0
assert_rc() { # <desc> <expected-rc> <actual-rc>
    if [ "$2" -eq "$3" ]; then pass=$((pass+1)); echo "  ok: $1 (rc=$3)";
    else fail=$((fail+1)); echo "  FAIL: $1 — expected rc=$2 got rc=$3" >&2; fi
}
run() { bash "$CHECK" --no-heartbeat --root "$SRC" --allowlist "$ALLOW" "$@" >/dev/null 2>&1; echo $?; }

# --- fixture 1: the silent json-gated / bare-text-exit pattern (should FIRE) ---
cat > "$SRC/silent.rs" <<'RS'
fn cmd_silent() {
    if found {
        println!("{}", name);
    } else {
        if display.json {
            super::json_error_exit(json!({"ok": false, "error": "No matching sessions"}));
        }
        std::process::exit(1);
    }
}
RS
assert_rc "silent json-gated/bare-text exit fires" 1 "$(run)"

# --- fixture 2: the T-2663 remediation — eprintln! before the exit (should be CLEAN) ---
cat > "$SRC/loud.rs" <<'RS'
fn cmd_loud() {
    if found {
        println!("{}", name);
    } else {
        if display.json {
            super::json_error_exit(json!({"ok": false, "error": "No matching sessions"}));
        }
        eprintln!("No matching sessions.");
        std::process::exit(1);
    }
}
RS
rm -f "$SRC/silent.rs"
assert_rc "loud (eprintln! before exit) is clean" 0 "$(run)"

# --- fixture 3: allowlist suppresses a genuine silent site ---
cat > "$SRC/silent.rs" <<'RS'
fn cmd_ack() {
    if display.json {
        super::json_error_exit(json!({"error": "x"}));
    }
    std::process::exit(1);
}
RS
echo "$SRC/silent.rs::cmd_ack::silent-exit" > "$ALLOW"
assert_rc "allowlisted silent site is suppressed" 0 "$(run)"
: > "$ALLOW"
assert_rc "same site fires once allowlist cleared" 1 "$(run)"

# --- fixture 4: exit-code FORWARDING (variable arg) is out of scope (should be CLEAN) ---
rm -f "$SRC/silent.rs"
cat > "$SRC/forward.rs" <<'RS'
fn cmd_forward() {
    if display.json {
        super::json_error_exit(json!({"error": "x"}));
    }
    std::process::exit(exec_result.exit_code);
}
RS
assert_rc "exit-code forwarding (variable arg) not flagged" 0 "$(run)"
rm -f "$SRC/forward.rs"

# --- fixture 5: a loud exit that prints/flushes right above (should be CLEAN) ---
cat > "$SRC/flush.rs" <<'RS'
fn cmd_check() {
    println!("All checks passed");
    use std::io::Write;
    let _ = std::io::stdout().flush();
    std::process::exit(1);
}
RS
assert_rc "flush-before-exit (loud) is clean" 0 "$(run)"
rm -f "$SRC/flush.rs"

# --- fixture 6: bare exit with NO json_error_exit above is out of the precise class (CLEAN) ---
# (documents the precise scope: this detector targets the json-gated divergence, not every
#  conceivable bare exit — a non-gated bare exit is a different, rarer shape.)
cat > "$SRC/plain.rs" <<'RS'
fn cmd_plain() {
    if something {
        do_work();
    }
    std::process::exit(1);
}
RS
assert_rc "non-json-gated bare exit outside precise class" 0 "$(run)"

# --- fixture 7 (T-2688): a comment between the block and the exit must NOT hide it ---
# THE REGRESSION. Pre-T-2688 the anchor took the nearest NON-BLANK line above the
# exit; a comment is non-blank, so it became the "preceding line", `prevtrim != "}"`,
# and a genuinely silent exit scanned clean. Found while reverting the real T-2663 fix
# to prove this check load-bearing — deleting the eprintln! but keeping its explanatory
# comment produced a silent exit the check could not see.
rm -f "$SRC"/*.rs
cat > "$SRC/commented.rs" <<'RS'
fn cmd_commented() {
    if display.json {
        super::json_error_exit(serde_json::json!({"ok": false}));
    }
    // text mode falls through to a bare exit — no message is emitted
    std::process::exit(1);
}
RS
assert_rc "comment between block and exit no longer hides the site" 1 "$(run)"

# --- fixture 8 (T-2688): a comment does not make a genuinely silent site look loud ---
# The `between` window is comment-stripped, so a comment MENTIONING eprintln! cannot
# clear a site that never calls it.
rm -f "$SRC"/*.rs
cat > "$SRC/mentions.rs" <<'RS'
fn cmd_mentions() {
    if display.json {
        super::json_error_exit(serde_json::json!({"ok": false}));
    }
    // TODO: mirror the JSON branch with an eprintln!("No matching sessions.");
    std::process::exit(1);
}
RS
assert_rc "a comment quoting eprintln! does not clear a silent site" 1 "$(run)"

# --- fixture 9 (T-2688): a loud site with a comment above the print stays clean ------
# Precision must improve in BOTH directions — skipping comments must not start
# flagging sites that genuinely emit output.
rm -f "$SRC"/*.rs
cat > "$SRC/loud_commented.rs" <<'RS'
fn cmd_loud_commented() {
    if display.json {
        super::json_error_exit(serde_json::json!({"ok": false}));
    }
    // T-2663 remediation: name what happened before exiting
    eprintln!("No matching sessions.");
    std::process::exit(1);
}
RS
assert_rc "loud site with a comment above the print stays clean" 0 "$(run)"

# --- fixture 10 (T-2688): a comment must not satisfy the json_error_exit gate --------
# The window grep is comment-stripped, so prose about json_error_exit is not a call.
rm -f "$SRC"/*.rs
cat > "$SRC/gate_prose.rs" <<'RS'
fn cmd_gate_prose() {
    if something {
        do_work();
    }
    // the sibling branch uses json_error_exit for the JSON case
    std::process::exit(1);
}
RS
assert_rc "prose mentioning json_error_exit does not satisfy the gate" 0 "$(run)"

echo
echo "silent-exit-check-fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
