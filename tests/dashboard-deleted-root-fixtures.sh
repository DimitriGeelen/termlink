#!/usr/bin/env bash
# guard-layer: source
#
# dashboard-deleted-root-fixtures.sh (T-2848)
#
# Hermetic fixture suite for scripts/check-dashboard-deleted-root.sh. Every case
# feeds a canned listener list + canned /api/_identity payloads through the test
# seams, so nothing here needs a live Watchtower, a live port, or /proc.
#
# Group C is the load-bearing one: it reproduces the REAL 2026-08-28 shape -- a
# dashboard answering 200 whose project_root is a deleted worktree -- and
# requires a fire. If C ever goes green, the check has stopped recognising the
# exact condition it was built for.
set -uo pipefail

CHECK="${CHECK:-scripts/check-dashboard-deleted-root.sh}"
[ -f "$CHECK" ] || { echo "fixtures: check script not found at $CHECK" >&2; exit 2; }

PASS=0 FAIL=0
ok()  { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad() { FAIL=$((FAIL+1)); printf '  FAIL %s\n' "$1"; }
assert_rc() { if [ "$1" -eq "$2" ]; then ok "$3 (rc=$2)"; else bad "$3 (expected rc=$1, got $2)"; fi; }
assert_contains()     { if printf '%s' "$1" | grep -qF "$2"; then ok "$3"; else bad "$3 — missing: $2"; fi; }
assert_not_contains() { if printf '%s' "$1" | grep -qF "$2"; then bad "$3 — unexpectedly present: $2"; else ok "$3"; fi; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

IDDIR="$TMP/identity"; mkdir -p "$IDDIR"
LIVE="$TMP/live-tree";   mkdir -p "$LIVE"
DEAD="$TMP/deleted-tree" # deliberately NOT created

run() { DASHBOARD_TEST_PORTS="$1" DASHBOARD_TEST_IDENTITY_DIR="$IDDIR" bash "$CHECK" "${@:2}" 2>&1; }

# ── A. a dashboard whose root exists is clean ────────────────────────────────
printf '{"service":"watchtower","project_root":"%s"}\n' "$LIVE" > "$IDDIR/3000.json"
out="$(run "3000")"; rc=$?
assert_rc 0 $rc "A1 live root is clean"
assert_contains "$out" "all roots live" "A2 clean path says so affirmatively"
assert_contains "$out" "SCOPE:" "A3 clean path carries the scope disclaimer"

# ── B. a port that is not a dashboard is ignored, not misreported ────────────
printf '{"service":"something-else","project_root":"%s"}\n' "$DEAD" > "$IDDIR/4000.json"
out="$(run "3000 4000")"; rc=$?
assert_rc 0 $rc "B1 non-watchtower service on a dead path does not fire"
assert_not_contains "$out" "4000" "B2 foreign service is not reported at all"

# ── C. LOAD-BEARING: the real 2026-08-28 shape — 200 OK, deleted project root ─
# The page answered, identified as watchtower, and named a root that no longer
# existed. It reported "0 across every category" while 75 items were waiting.
printf '{"service":"watchtower","project_root":"%s"}\n' "$DEAD" > "$IDDIR/3003.json"
out="$(run "3003")"; rc=$?
assert_rc 1 $rc "C1 deleted project_root FIRES (LOAD-BEARING)"
assert_contains "$out" "root NOT on disk" "C2 names the condition unambiguously"
assert_contains "$out" "3003" "C3 names the offending port"
assert_contains "$out" "counts are meaningless" "C4 warns the page's numbers cannot be trusted"

# ── C2. LOAD-BEARING: the ACTUAL :3003 shape — root ALIVE, inode DELETED ─────
# The first version of this check tested only "declared root missing from disk"
# and reported CLEAN over the very dashboard that motivated it: the worktree was
# alive, and .agentic-framework had been unlinked underneath the running server.
# Detector 1 is structurally blind to this. If this case goes green, the check
# has regressed to the wrong cause.
PROCDIR="$TMP/proc"; mkdir -p "$PROCDIR"
printf '{"service":"watchtower","project_root":"%s","pid":424242}\n' "$LIVE" > "$IDDIR/3009.json"
printf '%s/.agentic-framework (deleted)\n' "$LIVE" > "$PROCDIR/424242.cwd"
out="$(DASHBOARD_TEST_PROC_DIR="$PROCDIR" run "3009")"; rc=$?
assert_rc 1 $rc "C5 live root + DELETED inode FIRES (LOAD-BEARING, detector 2)"
assert_contains "$out" "DELETED inode" "C6 attributes it to the inode, not the root"
assert_contains "$out" "(deleted)" "C7 shows the actual cwd evidence"

# A live root with a live inode must stay clean — otherwise detector 2 fires on
# everything and the check becomes the flaky guard T-2709/T-2787 warn about.
printf '%s/.agentic-framework\n' "$LIVE" > "$PROCDIR/424242.cwd"
out="$(DASHBOARD_TEST_PROC_DIR="$PROCDIR" run "3009")"; rc=$?
assert_rc 0 $rc "C8 live root + live inode is clean (detector 2 discriminates)"

# ── D. mixed fleet: the healthy one must not mask the dead one ───────────────
out="$(run "3000 3003")"; rc=$?
assert_rc 1 $rc "D1 one dead root among healthy ones still fires"
assert_contains "$out" "3003" "D2 the dead dashboard is named"
assert_not_contains "$out" ":3000  project_root" "D3 the healthy one is not listed as firing"

# ── E. JSON envelope ────────────────────────────────────────────────────────
out="$(run "3003" --json)"; rc=$?
assert_rc 1 $rc "E1 JSON firing exits 1"
assert_contains "$out" '"ok":false' "E2 ok=false when firing"
assert_contains "$out" '"project_root"' "E3 firing entry carries the root"
out="$(run "3000" --json)"; rc=$?
assert_rc 0 $rc "E4 JSON clean exits 0"
assert_contains "$out" '"ok":true' "E5 ok=true when clean"
assert_contains "$out" '"scope"' "E6 clean JSON still carries scope (T-2680)"

# ── F. fail closed: a dashboard that will not name its root is NOT clean ─────
# An unverifiable dashboard is the condition this check exists to refuse, so it
# must never pass silently.
printf '{"service":"watchtower"}\n' > "$IDDIR/3005.json"
out="$(run "3005")"; rc=$?
assert_rc 1 $rc "F1 dashboard with no project_root fails closed"
assert_contains "$out" "names no project_root" "F2 says why it could not be verified"

# ── G. tooling errors are exit 2, never a clean bill ────────────────────────
out="$(bash "$CHECK" --bogus-flag 2>&1)"; rc=$?
assert_rc 2 $rc "G1 unknown flag is a tooling error"

# ── H. a port answering nothing is skipped, not counted as a dashboard ──────
out="$(run "3000 9999")"; rc=$?
assert_rc 0 $rc "H1 silent port does not fire"
assert_contains "$out" "1 dashboard(s) on 2 listener(s)" "H2 counts listeners and dashboards separately"

printf '\ndashboard-deleted-root-fixtures: %d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
