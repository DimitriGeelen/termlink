#!/usr/bin/env bash
# Fixtures for scripts/check-budget-ladder-drift.sh (T-3029).
#
# Weighted toward the FIRING cases and the fail-closed contract. A drift check is trivially
# green when everything is acknowledged, and a green check that cannot go red is not a check
# (PL-328: a guard's green is not evidence until it has been fed the violation it names).

set -uo pipefail
CHECK="${CHECK:-scripts/check-budget-ladder-drift.sh}"
PASS=0; FAIL=0
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT
NOTRUN_LOG="$TMP/.notrun"; : > "$NOTRUN_LOG"

ok()   { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad()  { FAIL=$((FAIL+1)); printf '  FAIL %s\n' "$1"; }
is()   { [ "$2" = "$3" ] && ok "$1" || bad "$1 (got '$2', want '$3')"; }
has()  { case "$2" in *"$3"*) ok "$1";; *) bad "$1 (missing '$3')";; esac; }
hasnt(){ case "$2" in *"$3"*) bad "$1 (unexpected '$3')";; *) ok "$1";; esac; }
# A mis-spelled assertion is a NOT-RUN assertion. On this file's first run two `hasnt"..."`
# calls lost their space, bash read them as unknown commands, and the suite still printed
# "0 failed" — the T-2831 vacuous pass, reproduced inside the guard layer. An ERR trap is the
# wrong instrument here (this suite deliberately runs commands exiting 1 and 2); a
# command-not-found handler fires on exactly that failure and on nothing else.
# NOTE: bash runs command_not_found_handle in a SEPARATE EXECUTION ENVIRONMENT, so a
# FAIL=$((FAIL+1)) inside it is discarded — measured: the handler fires, the counter stays 0,
# and the suite still reports success. The record has to leave the subshell through a file.
# $1 is the unknown command name; later positionals are its args, which here include
# multi-line captured output — logging "$*" inflated the count from 1 to 3. Log $1 only.
command_not_found_handle() { printf '%s\n' "${1:-?}" >> "${NOTRUN_LOG:-/dev/null}"; printf '  FAIL not-run assertion: %s\n' "${1:-?}"; return 127; }

# --- fixture builders -------------------------------------------------------------------
mkgate() { # $1=path $2=window $3=warn% $4=urgent% $5=crit%
    cat > "$1" <<EOF
#!/usr/bin/env bash
CONTEXT_WINDOW=\$(fw_config_int "CONTEXT_WINDOW" $2)
TOKEN_WARN=\$((CONTEXT_WINDOW * $3 / 100))
TOKEN_URGENT=\$((CONTEXT_WINDOW * $4 / 100))
TOKEN_CRITICAL=\$((CONTEXT_WINDOW * $5 / 100))
EOF
}
mkdoc() { # $1=path $2=warnK $3=urgentK $4=critK $5=band1 $6=band2 $7=band3 $8=critSentenceK $9=critSentencePct
    cat > "$1" <<EOF
# Doc
- Below $5% (${2}K tokens): proceed normally
- $5-$6% (${2}K-${3}K): propose only small, bounded tasks; commit first
- Above $6% (${3}K+): propose only wrap-up actions
- Above $7% (${4}K+): handover immediately, no new work
- Escalation ladder: **${2}K** ok→warn (note), **${3}K** warn→urgent (warning), **${4}K** urgent→critical (**BLOCK**)

**Structural enforcement:** blocks Write/Edit/Bash tool calls when context reaches critical level (>=${8}K tokens, ~${9}%).
EOF
}

echo "budget-ladder-drift fixtures"

# --- 1. the REAL tree: clean, because the three known drifts are acknowledged -------------
out=$(bash "$CHECK" 2>&1); rc=$?
is   "real tree exits 0 (known drift acknowledged)" "$rc" "0"
has  "real tree names its acknowledged entries"     "$out" "ACKNOWLEDGED CLAUDE.md::escalation-ladder-absolutes"
has  "real tree prints the live gate numbers"       "$out" "window=300000"
has  "real tree carries a scope disclaimer"         "$out" "Scope:"

# --- 2. the REAL tree with an EMPTY ledger: all three must fire ---------------------------
: > "$TMP/empty-allow"
out=$(bash "$CHECK" --allowlist "$TMP/empty-allow" 2>&1); rc=$?
is   "real tree + empty ledger fires"               "$rc" "1"
has  "  fires on absolutes"   "$out" "escalation-ladder-absolutes"
has  "  fires on bands"       "$out" "work-proposal-bands"
has  "  fires on self-contradiction" "$out" "structural-enforcement-critical"
has  "  names the internal contradiction" "$out" "contradicts this same file's own escalation ladder"

# --- 3. prose that AGREES with the gate is clean with no ledger at all --------------------
mkgate "$TMP/gate.sh" 300000 75 85 95
cp "$TMP/gate.sh" "$TMP/ckpt.sh"
mkdoc  "$TMP/ok.md" 225 255 285 75 85 95 285 95
out=$(bash "$CHECK" --claude-md "$TMP/ok.md" --gate "$TMP/gate.sh" --checkpoint "$TMP/ckpt.sh" --allowlist "$TMP/empty-allow" 2>&1); rc=$?
is   "matching prose is clean"                      "$rc" "0"
hasnt "matching prose reports no drift"              "$out" "DRIFT"

# --- 4. MUTANT: move the window, prose now stale -----------------------------------------
mkgate "$TMP/gate2.sh" 200000 75 85 95
cp "$TMP/gate2.sh" "$TMP/ckpt2.sh"
out=$(bash "$CHECK" --claude-md "$TMP/ok.md" --gate "$TMP/gate2.sh" --checkpoint "$TMP/ckpt2.sh" --allowlist "$TMP/empty-allow" 2>&1); rc=$?
is   "window change re-fires the check"             "$rc" "1"
has  "  reports the new window"                     "$out" "window=200000"

# --- 5. MUTANT: move only the percentages ------------------------------------------------
mkgate "$TMP/gate3.sh" 300000 60 80 90
cp "$TMP/gate3.sh" "$TMP/ckpt3.sh"
out=$(bash "$CHECK" --claude-md "$TMP/ok.md" --gate "$TMP/gate3.sh" --checkpoint "$TMP/ckpt3.sh" --allowlist "$TMP/empty-allow" 2>&1); rc=$?
is   "percentage change re-fires the check"         "$rc" "1"

# --- 6. code-vs-code: the two enforcing scripts disagreeing is its own finding ------------
out=$(bash "$CHECK" --claude-md "$TMP/ok.md" --gate "$TMP/gate.sh" --checkpoint "$TMP/ckpt2.sh" --allowlist "$TMP/empty-allow" 2>&1); rc=$?
is   "gate/checkpoint divergence fires"             "$rc" "1"
has  "  names the code-vs-code signature"           "$out" "code::gate-vs-checkpoint"

# --- 7. the ledger acknowledges ONE signature without hiding the others -------------------
echo 'ok.md::work-proposal-bands  # acknowledged for test' > "$TMP/one-allow"
out=$(bash "$CHECK" --claude-md "$TMP/ok.md" --gate "$TMP/gate2.sh" --checkpoint "$TMP/ckpt2.sh" --allowlist "$TMP/one-allow" 2>&1); rc=$?
is   "partial ledger still fires on the rest"       "$rc" "1"
firing_block="${out%%Live gate*}"
hasnt "  acknowledged signature absent from firing set" "$firing_block" "ok.md::work-proposal-bands"
has   "  unacknowledged signature still in firing set"  "$firing_block" "ok.md::escalation-ladder-absolutes"

# --- 8. FAIL-CLOSED: a checker that cannot look must never report clean -------------------
out=$(bash "$CHECK" --claude-md "$TMP/nope.md" --gate "$TMP/gate.sh" --checkpoint "$TMP/ckpt.sh" 2>&1); rc=$?
is   "missing doc exits 2, not 0"                   "$rc" "2"
out=$(bash "$CHECK" --claude-md "$TMP/ok.md" --gate "$TMP/nope.sh" --checkpoint "$TMP/ckpt.sh" 2>&1); rc=$?
is   "missing gate exits 2, not 0"                  "$rc" "2"

echo 'CONTEXT_WINDOW=$(fw_config_int "OTHER" 300000)' > "$TMP/unparseable.sh"
out=$(bash "$CHECK" --claude-md "$TMP/ok.md" --gate "$TMP/unparseable.sh" --checkpoint "$TMP/ckpt.sh" 2>&1); rc=$?
is   "unparseable gate exits 2, not 0"              "$rc" "2"

# --- 9. BLIND-ANCHOR: prose restructured away entirely must be tooling, not clean ---------
echo '# a document with no ladder in it at all' > "$TMP/noladder.md"
out=$(bash "$CHECK" --claude-md "$TMP/noladder.md" --gate "$TMP/gate.sh" --checkpoint "$TMP/ckpt.sh" --allowlist "$TMP/empty-allow" 2>&1); rc=$?
is   "prose with no ladder exits 2 (anchors stale)" "$rc" "2"
has  "  says the check went blind"                  "$out" "blind"

# --- 10. JSON envelope ------------------------------------------------------------------
out=$(bash "$CHECK" --json --allowlist "$TMP/empty-allow" 2>&1)
python3 -c "
import json,sys
d=json.loads(sys.stdin.read())
assert d['ok'] is False, 'ok should be false when firing'
assert d['drift_total']==3, d['drift_total']
assert d['live']['window']==300000, d['live']
assert 'scope' in d
print('json-ok')
" <<< "$out" >/dev/null 2>&1 && ok "json envelope shape" || bad "json envelope shape"

notrun=$(wc -l < "$NOTRUN_LOG" | tr -d ' ')
[ "$notrun" -gt 0 ] && FAIL=$((FAIL + notrun))
printf '\n%d passed, %d failed' "$PASS" "$FAIL"
[ "$notrun" -gt 0 ] && printf ' (%d assertion(s) never ran)' "$notrun"
printf '\n'
[ "$FAIL" -eq 0 ]
