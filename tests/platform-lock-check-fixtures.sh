#!/usr/bin/env bash
# platform-lock-check-fixtures.sh (T-2693)
#
# Hermetic proof for scripts/check-platform-lock.sh — no live binary, no host state.
# Builds a scratch crate root of synthetic .rs files and drives the check through its
# --root / --allowlist seams.
#
# The load-bearing fixtures are 1 and 5. Fixture 1 is the T-2690 defect shape: a
# /proc read on a path a macOS user reaches, which returned a plausible WRONG ANSWER
# rather than an error. Fixture 5 guards the opposite failure — a check that flagged
# every subprocess would be noise nobody reads, so cross-platform tools must stay
# silent.
#
# Exit 0 = all fixtures pass; exit 1 = a fixture assertion failed.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="$REPO_ROOT/scripts/check-platform-lock.sh"
[ -f "$CHECK" ] || { echo "FAIL: check not found at $CHECK" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "FAIL: jq required" >&2; exit 1; }

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT
SRC="$SCRATCH/src"; ALLOW="$SCRATCH/allowlist"
mkdir -p "$SRC"; : > "$ALLOW"

pass=0; fail=0
ok()  { pass=$((pass+1)); echo "  ok: $1"; }
bad() { fail=$((fail+1)); echo "  FAIL: $1" >&2; }
assert_rc() { [ "$2" -eq "$3" ] && ok "$1 (rc=$3)" || bad "$1 — expected rc=$2 got rc=$3"; }
assert_eq() { [ "$2" = "$3" ] && ok "$1 ($3)" || bad "$1 — expected '$2' got '$3'"; }

run()      { bash "$CHECK" --root "$SRC" --allowlist "$ALLOW" >/dev/null 2>&1; echo $?; }
run_json() { bash "$CHECK" --root "$SRC" --allowlist "$ALLOW" --json 2>/dev/null; }
reset()    { rm -f "$SRC"/*.rs 2>/dev/null; : > "$ALLOW"; }

# --- fixture 1: THE DEFECT SHAPE — a /proc read ---------------------------------
reset
cat > "$SRC/whoami.rs" <<'RS'
fn read_ppid(pid: u32) -> Option<u32> {
    let raw = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    parse(&raw)
}
RS
assert_rc "a /proc read fires" 1 "$(run)"
assert_eq "one firing site" "1" "$(run_json | jq -r '.firing | length')"
assert_eq "primitive is proc-path" "proc-path" "$(run_json | jq -r '.firing[0].primitive')"
assert_eq "enclosing fn is reported" "read_ppid" "$(run_json | jq -r '.firing[0].fn')"

# --- fixture 2: allowlisting suppresses it, and removal re-fires -----------------
echo "$SRC/whoami.rs::read_ppid::proc-path  # guarded by a runtime probe" > "$ALLOW"
assert_rc "allowlisted site is suppressed" 0 "$(run)"
assert_eq "still counted as scanned" "1" "$(run_json | jq -r '.checked')"
: > "$ALLOW"
assert_rc "clearing the allowlist re-fires (load-bearing)" 1 "$(run)"

# --- fixture 3: a comment mentioning /proc is not a dependency -------------------
reset
cat > "$SRC/doc.rs" <<'RS'
/// Walks the ancestor chain by parsing /proc/<pid>/stat on Linux.
/// Returns None on non-Linux hosts where /proc/ does not exist.
fn documented_only() -> u32 { 42 }
RS
assert_rc "comment-only mentions do not fire" 0 "$(run)"
assert_eq "and are not counted as scanned sites" "0" "$(run_json | jq -r '.checked')"

# --- fixture 4: /sys is flagged too ---------------------------------------------
reset
cat > "$SRC/sysfs.rs" <<'RS'
fn read_cpu() -> Option<String> {
    std::fs::read_to_string("/sys/devices/system/cpu/online").ok()
}
RS
assert_rc "a /sys read fires" 1 "$(run)"
assert_eq "primitive is sys-path" "sys-path" "$(run_json | jq -r '.firing[0].primitive')"

# --- fixture 5: cross-platform commands must stay SILENT ------------------------
# A check that flagged every subprocess would be noise nobody reads. pgrep exists on
# BSD/macOS; osascript is macOS-only ON PURPOSE (the Terminal.app spawn backend).
reset
cat > "$SRC/portable.rs" <<'RS'
fn spawn_things() {
    let _ = std::process::Command::new("git").arg("status").output();
    let _ = std::process::Command::new("sh").args(["-c", "echo hi"]).output();
    let _ = std::process::Command::new("tmux").arg("ls").output();
    let _ = std::process::Command::new("pgrep").arg("termlink").output();
    let _ = std::process::Command::new("ssh").arg("host").output();
    let _ = std::process::Command::new("osascript").arg("-e").output();
}
RS
assert_rc "cross-platform + deliberately-macOS commands do not fire" 0 "$(run)"
assert_eq "none are even counted" "0" "$(run_json | jq -r '.checked')"

# --- fixture 6: Linux-only commands DO fire -------------------------------------
reset
cat > "$SRC/linuxonly.rs" <<'RS'
fn probe() {
    let _ = std::process::Command::new("systemctl").arg("status").output();
    let _ = std::process::Command::new("setsid").arg("sh").spawn();
}
RS
assert_rc "Linux-only commands fire" 1 "$(run)"
assert_eq "both are reported" "2" "$(run_json | jq -r '.firing | length')"
assert_eq "primitive names the tool" "cmd:systemctl" \
    "$(run_json | jq -r '.firing[0].primitive')"

# --- fixture 7: a reason-bearing allowlist comment is stripped correctly ---------
reset
cat > "$SRC/fallback.rs" <<'RS'
fn spawn_bg() {
    let _ = std::process::Command::new("setsid").arg("sh").spawn();
}
RS
printf '%s\n' "$SRC/fallback.rs::spawn_bg::cmd:setsid   # falls back to sh -c on macOS" > "$ALLOW"
assert_rc "signature with a trailing reason comment is honoured" 0 "$(run)"

# --- fixture 8: absent scan root is a tooling error, never a clean bill ----------
rc="$(bash "$CHECK" --root "$SCRATCH/nope" --allowlist "$ALLOW" >/dev/null 2>&1; echo $?)"
assert_rc "absent root exits 2 (fail-closed)" 2 "$rc"

# --- fixture 9: a clean tree reports clean --------------------------------------
reset
cat > "$SRC/clean.rs" <<'RS'
fn nothing_platform_locked() -> u32 { 7 }
RS
assert_rc "a clean tree exits 0" 0 "$(run)"
assert_eq "envelope reports ok" "true" "$(run_json | jq -r '.ok')"

echo
echo "platform-lock-check-fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ] || exit 1
exit 0
