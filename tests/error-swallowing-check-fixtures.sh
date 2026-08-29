#!/usr/bin/env bash
# error-swallowing-check-fixtures.sh (T-2792)
#
# Fixture suite for scripts/check-error-swallowing-predicate.sh. Hermetic: every case
# builds a scratch tree of .rs files and points the check at it via --root/--allowlist,
# so nothing here depends on the state of the real crates. The last case is the PL-219
# control that the REAL tree scans clean — without it a detector that silently stopped
# matching would pass every synthetic case above.
set -uo pipefail

CHECK="${CHECK:-scripts/check-error-swallowing-predicate.sh}"
[ -f "$CHECK" ] || { echo "fixtures: check script not found at $CHECK" >&2; exit 2; }

PASS=0 FAIL=0
ok()   { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad()  { FAIL=$((FAIL+1)); printf '  FAIL %s\n' "$1"; }

assert_rc() { # <expected-rc> <actual-rc> <label>
    if [ "$1" -eq "$2" ]; then ok "$3 (rc=$2)"; else bad "$3 (expected rc=$1, got $2)"; fi
}
assert_contains() { # <haystack> <needle> <label>
    if printf '%s' "$1" | grep -qF "$2"; then ok "$3"; else bad "$3 — missing: $2"; fi
}
assert_not_contains() { # <haystack> <needle> <label>
    if printf '%s' "$1" | grep -qF "$2"; then bad "$3 — unexpectedly present: $2"; else ok "$3"; fi
}

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
EMPTY_ALLOW="$TMP/empty-allowlist"
: > "$EMPTY_ALLOW"

run_check() { bash "$CHECK" --no-heartbeat --allowlist "$2" --root "$1" 2>&1; }

# ── A. the defect shape fires ────────────────────────────────────────────────
mkdir -p "$TMP/a"
cat > "$TMP/a/lib.rs" <<'RS'
fn list_things(dir: &Path) -> Result<Vec<Thing>, E> {
    if !dir.exists() {
        return Ok(vec![]);
    }
    Ok(read(dir)?)
}
RS
out="$(run_check "$TMP/a" "$EMPTY_ALLOW")"; rc=$?
assert_rc 1 $rc "A1 bare empty-success gate fires"
assert_contains "$out" "list_things" "A2 names the enclosing fn"
assert_contains "$out" "lib.rs:2" "A3 reports the gate line"
assert_contains "$out" "exists" "A4 names the predicate"

# ── B. every empty-success return form ───────────────────────────────────────
mkdir -p "$TMP/b"
cat > "$TMP/b/lib.rs" <<'RS'
fn a(p: &Path) -> Option<T> {
    if !p.exists() {
        return None;
    }
    Some(x)
}
fn b(p: &Path) -> Result<Option<T>, E> {
    if !p.is_dir() {
        return Ok(None);
    }
    Ok(Some(x))
}
fn c(p: &Path) -> Result<Vec<T>, E> {
    if !p.is_file() {
        return Ok(Vec::new());
    }
    Ok(v)
}
RS
out="$(run_check "$TMP/b" "$EMPTY_ALLOW")"; rc=$?
assert_rc 1 $rc "B1 all three return forms fire"
out2="$(bash "$CHECK" --no-heartbeat --allowlist "$EMPTY_ALLOW" --root "$TMP/b" --json 2>&1)"
assert_contains "$out2" '"candidates":3' "B2 all three counted"
assert_contains "$out2" '"predicate":"is_dir"' "B3 is_dir recognised"
assert_contains "$out2" '"predicate":"is_file"' "B4 is_file recognised"

# ── C. NON-empty consequents must NOT fire ───────────────────────────────────
mkdir -p "$TMP/c"
cat > "$TMP/c/lib.rs" <<'RS'
fn create_if_missing(p: &Path) -> Result<(), E> {
    if !p.exists() {
        std::fs::create_dir_all(p)?;
    }
    Ok(())
}
fn loud(p: &Path) -> Result<T, E> {
    if !p.exists() {
        return Err(E::NotFound(p.into()));
    }
    Ok(x)
}
fn bails(p: &Path) -> Result<T> {
    if !p.exists() {
        bail!("no such path: {}", p.display());
    }
    Ok(x)
}
RS
out="$(run_check "$TMP/c" "$EMPTY_ALLOW")"; rc=$?
assert_rc 0 $rc "C1 create-if-missing / Err / bail do not fire"
assert_contains "$out" "clean" "C2 reports clean"

# ── D. acknowledgement clears exactly one site ───────────────────────────────
mkdir -p "$TMP/d"
cat > "$TMP/d/lib.rs" <<'RS'
fn ack_me(p: &Path) -> Result<Vec<T>, E> {
    if !p.exists() {
        return Ok(vec![]);
    }
    Ok(v)
}
fn not_acked(p: &Path) -> Result<Vec<T>, E> {
    if !p.exists() {
        return Ok(vec![]);
    }
    Ok(v)
}
RS
ALLOW_D="$TMP/allow-d"
printf '%s\n' "$TMP/d/lib.rs::ack_me::empty-success-gate  # test" > "$ALLOW_D"
out="$(run_check "$TMP/d" "$ALLOW_D")"; rc=$?
assert_rc 1 $rc "D1 unacknowledged sibling still fires"
assert_contains "$out" "not_acked" "D2 the unacknowledged fn is named"
assert_not_contains "$out" "ack_me" "D3 the acknowledged fn is cleared"

# ── E. comments are not code ─────────────────────────────────────────────────
mkdir -p "$TMP/e"
cat > "$TMP/e/lib.rs" <<'RS'
fn documented(p: &Path) -> Result<Vec<T>, E> {
    // Historically this did:
    //     if !p.exists() {
    //         return Ok(vec![]);
    //     }
    // which swallowed EACCES. Now it does not.
    match std::fs::metadata(p) {
        Err(e) if e.kind() == NotFound => return Ok(vec![]),
        _ => {}
    }
    Ok(v)
}
RS
out="$(run_check "$TMP/e" "$EMPTY_ALLOW")"; rc=$?
assert_rc 0 $rc "E1 a commented-out gate does not fire"

# ── F. a comment BETWEEN gate and return must not hide the site (T-2688 lesson) ──
mkdir -p "$TMP/f"
cat > "$TMP/f/lib.rs" <<'RS'
fn hidden(p: &Path) -> Result<Vec<T>, E> {
    if !p.exists() {
        // nothing here yet, that's fine
        return Ok(vec![]);
    }
    Ok(v)
}
RS
out="$(run_check "$TMP/f" "$EMPTY_ALLOW")"; rc=$?
assert_rc 1 $rc "F1 comment between gate and return does not hide it"
assert_contains "$out" "hidden" "F2 names the fn behind the comment"

# ── G. tooling errors fail closed ────────────────────────────────────────────
out="$(bash "$CHECK" --no-heartbeat --root "$TMP/does-not-exist" 2>&1)"; rc=$?
assert_rc 2 $rc "G1 missing scan root is a tooling error, not clean"
out="$(bash "$CHECK" --no-heartbeat --bogus-flag 2>&1)"; rc=$?
assert_rc 2 $rc "G2 unknown flag is a tooling error"

# ── H. scope disclaimer on BOTH paths (T-2680) ───────────────────────────────
out="$(run_check "$TMP/c" "$EMPTY_ALLOW")"
assert_contains "$out" "SCOPE:" "H1 clean path carries the scope disclaimer"
out="$(run_check "$TMP/a" "$EMPTY_ALLOW")"
assert_contains "$out" "SCOPE:" "H2 firing path carries the scope disclaimer"
out="$(bash "$CHECK" --no-heartbeat --allowlist "$EMPTY_ALLOW" --root "$TMP/c" --json 2>&1)"
assert_contains "$out" '"scope"' "H3 clean JSON carries scope"
assert_contains "$out" '"ok":true' "H4 clean JSON ok=true"

# ── I. LOAD-BEARING: reverting the T-2792 fix re-fires the real site ─────────
# The whole point of the check. Reproduces manager.rs::list_sessions_in as it stood
# before T-2792 and requires a fire; if this ever goes green the detector has stopped
# recognising the very shape it was built for.
mkdir -p "$TMP/i"
cat > "$TMP/i/manager.rs" <<'RS'
pub fn list_sessions_in(
    sessions_dir: &Path,
    include_stale: bool,
) -> Result<Vec<Registration>, SessionError> {
    if !sessions_dir.exists() {
        return Ok(vec![]);
    }

    let mut sessions = Vec::new();
    for entry in std::fs::read_dir(sessions_dir)? {
        sessions.push(entry);
    }
    Ok(sessions)
}
RS
out="$(run_check "$TMP/i" "$EMPTY_ALLOW")"; rc=$?
assert_rc 1 $rc "I1 pre-T-2792 list_sessions_in re-fires (load-bearing)"
assert_contains "$out" "list_sessions_in" "I2 names the reverted fn"

# ── J. PL-219 control: the REAL tree scans clean ─────────────────────────────
out="$(bash "$CHECK" --no-heartbeat 2>&1)"; rc=$?
assert_rc 0 $rc "J1 real tree scans clean with the tracked allowlist"
assert_contains "$out" "0 unacknowledged" "J2 real tree reports zero unacknowledged"

printf '\nerror-swallowing-check-fixtures: %d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
