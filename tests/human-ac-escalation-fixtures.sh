#!/usr/bin/env bash
# tests/human-ac-escalation-fixtures.sh — T-3186
#
# guard-layer: source
#
# Fixtures for the escalation bar. The check is only worth having if a NEW
# declared-mechanical criterion fires while the 29 known ones stay quiet — a ledger-driven
# check is trivially green when everything is acknowledged, and a green that cannot go red
# is not a check (T-2812's fixture lesson).
#
# Case 4 is the one that would have shipped broken. The task template's own `### Human`
# section carries worked `[RUBBER-STAMP]` and `[REVIEWER]` EXAMPLES inside HTML comments,
# so a detector that does not strip comment regions flags every task in the corpus. That
# exact trap defeated five separate detectors in the session that produced this file.
#
# Exit 0 = all pass, 1 = failure, 2 = tooling.

set -uo pipefail

PROJECT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="$PROJECT/scripts/check-human-ac-escalation.sh"

PASS=0; FAIL=0
ok()   { printf '  \033[0;32mok\033[0m    %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '  \033[0;31mFAIL\033[0m  %s\n' "$1"; FAIL=$((FAIL+1)); }

[ -f "$CHECK" ] || { echo "TOOLING: $CHECK not found" >&2; exit 2; }
command -v python3 >/dev/null 2>&1 || { echo "TOOLING: python3 missing" >&2; exit 2; }

SCRATCH="$(mktemp -d)"; trap 'rm -rf "$SCRATCH"' EXIT
T="$SCRATCH/tasks"; mkdir -p "$T"
LEDGER="$SCRATCH/ledger"; : > "$LEDGER"

run() { bash "$CHECK" --tasks-dir "$T" --allowlist "$LEDGER" "$@" 2>&1; }
rc_of() { bash "$CHECK" --tasks-dir "$T" --allowlist "$LEDGER" >/dev/null 2>&1; echo $?; }

mk() { # <id> <human-block-body>
    cat > "$T/$1-fx.md" <<MD
---
id: $1
status: started-work
---

## Acceptance Criteria

### Agent
- [x] done

### Human
$2

## Verification
MD
}

echo "=== T-3186: the escalation bar ==="

# ================================================================= Case 0
echo
echo "Case 0 — harness control: the check runs and can discriminate"
mk T-9001 '- [ ] [REVIEW] you agree the semantics should outrank the geometry'
if [ "$(rc_of)" != "0" ]; then
    echo "  HARNESS BROKEN: a [REVIEW]-only corpus did not come back clean." >&2
    echo "  Got: $(run | head -3)" >&2
    exit 2
fi
ok "a corpus with only [REVIEW] ACs is clean"
mk T-9002 '- [ ] [RUBBER-STAMP] step 4 prints a clean verdict, exit 0'
if [ "$(rc_of)" != "1" ]; then
    echo "  HARNESS BROKEN: adding a [RUBBER-STAMP] AC did not make it fire." >&2
    exit 2
fi
ok "adding a [RUBBER-STAMP] AC makes it fire — the check can go red"

# ================================================================= Case 1
echo
echo "Case 1 ★ a [RUBBER-STAMP] AC in ### Human fires"
out="$(run)"
grep -qF 'T-9002' <<< "$out" && ok "T-9002 named in the findings" \
    || fail "T-9002 not reported"
grep -qF 'author declared it mechanical' <<< "$out" \
    && ok "the reason cites the author's own declaration" \
    || fail "reason not reported"

# ================================================================= Case 2
echo
echo "Case 2 ★ a [REVIEWER] AC sitting in ### Human fires (the T-2198 shape)"
rm -f "$T"/*.md
mk T-9003 '- [ ] [REVIEWER] Verdict: PASS; no findings on block-message-completeness'
out="$(run)"
grep -qF 'belongs in ### Agent' <<< "$out" \
    && ok "flagged as misplaced, with where it belongs" \
    || fail "misplaced [REVIEWER] not flagged"

# ================================================================= Case 3
echo
echo "Case 3 ★ a [REVIEW] AC is NEVER flagged — genuine judgement stays human"
rm -f "$T"/*.md
mk T-9004 '- [ ] [REVIEW] a recorded choice — repair or delete'
[ "$(rc_of)" = "0" ] && ok "[REVIEW] does not fire" \
    || fail "[REVIEW] fired — the bar would push judgement onto the agent"

# ================================================================= Case 4  ★★
echo
echo "Case 4 ★★ the template's own EXAMPLES, inside HTML comments, must not fire"
echo "          (this is the trap that defeated five detectors in one session)"
rm -f "$T"/*.md
cat > "$T/T-9005-fx.md" <<'MD'
---
id: T-9005
status: started-work
---

## Acceptance Criteria

### Agent
- [x] done

### Human
<!-- Criteria requiring human verification. Not blocking.
     [RUBBER-STAMP] example (mechanical, no judgment needed):
       - [ ] [RUBBER-STAMP] Publish the release
     [REVIEWER] example (static-scan-verifiable):
       - [ ] [REVIEWER] Block message names both bypass mechanisms
-->
- [ ] [REVIEW] you agree the framing is right

## Verification
MD
[ "$(rc_of)" = "0" ] && ok "comment-region examples are stripped before matching" \
    || fail "fired on the template's own examples — comment stripping is broken"

# ================================================================= Case 5
echo
echo "Case 5 — an acknowledged task does not fire, but IS counted and reported"
rm -f "$T"/*.md
mk T-9006 '- [ ] [RUBBER-STAMP] deterministic command, deferred by priority'
printf 'T-9006  # deferred by priority; delete when revisited\n' > "$LEDGER"
[ "$(rc_of)" = "0" ] && ok "acknowledged entry does not fire" \
    || fail "acknowledged entry still fired"
out="$(run)"
grep -qE '1 acknowledged' <<< "$out" \
    && ok "the clean path REPORTS the acknowledgement count" \
    || fail "clean path hid the acknowledgement — green conflates none with acknowledged"

# ================================================================= Case 6
echo
echo "Case 6 — an ABSENT ledger acknowledges nothing rather than excusing everything"
rm -f "$LEDGER"
[ "$(rc_of)" = "1" ] && ok "with no ledger, the same AC fires again" \
    || fail "an absent ledger silenced the finding"
: > "$LEDGER"

# ================================================================= Case 7
echo
echo "Case 7 — fail-closed: a corpus of zero task files is TOOLING, never clean"
rm -f "$T"/*.md
[ "$(rc_of)" = "2" ] && ok "empty corpus exits 2, not 0 (T-3105)" \
    || fail "empty corpus did not exit 2 — a vacuous clean"
mk T-9007 '- [ ] [REVIEW] fine'
bash "$CHECK" --tasks-dir "$SCRATCH/nope" --allowlist "$LEDGER" >/dev/null 2>&1
[ "$?" = "2" ] && ok "missing tasks dir exits 2" || fail "missing tasks dir did not exit 2"

# ================================================================= Case 8
echo
echo "Case 8 — the real corpus is clean, and says so with its scope"
rout="$(bash "$CHECK" 2>&1)"; rrc=$?
[ "$rrc" = "0" ] && ok "real tree clean (rc 0)" || fail "real tree fires: $(head -2 <<< "$rout")"
grep -qF 'Scope: declared prefixes only' <<< "$rout" \
    && ok "the clean path states its scope, so a green is not over-read (T-2680)" \
    || fail "clean path does not disclose scope"
grep -qE '29 acknowledged' <<< "$rout" \
    && ok "all 29 known escalations are acknowledged and counted" \
    || fail "acknowledgement count changed unexpectedly: $(head -1 <<< "$rout")"

echo
echo "=== SUMMARY ==="
echo "Pass: $PASS"
echo "Fail: $FAIL"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
