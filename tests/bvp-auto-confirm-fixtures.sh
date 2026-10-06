#!/usr/bin/env bash
# tests/bvp-auto-confirm-fixtures.sh — T-3184 (supersedes T-3176's contract)
#
# guard-layer: source
#
# OPERATOR RULING 2026-09-27: setting BVP scores requires no human involvement, and the
# same applies to value drivers and arc drivers. All five §ACD-gated verbs open. The
# human keeps an OVERRIDE, flagged, and the override is STICKY.
#
# THIS SUITE WAS REWRITTEN, NOT EXTENDED. T-3176's version asserted the opposite contract
# — that `confirm` was gated unless a switch was on, and that the other four verbs must
# STILL refuse with that switch on. Those cases are now false by ruling, so they are
# replaced rather than left passing against behaviour nobody wants. What carried over
# unchanged is everything the ruling did not touch: the telemetry ledger, the no-signal
# count, `confirmed_by` never falling back to $USER, and the path-override env var.
#
# THE LOAD-BEARING CASE IS CASE 3, and it is not "an agent can confirm".
#
# It is the operator's own requirement: "after a next BVP assessment run, that doesn't get
# overridden." Removing the gate without sticky provenance would give every human
# correction a shelf life — the estimator re-proposes, the next automated confirm
# promotes, and the correction is gone with no error and no trace that it existed. That
# silent overwrite would be WORSE than the gate it replaces, because the gate at least
# failed loudly. Case 3 drives the full cycle: score, human-override, re-propose,
# agent-confirm, and asserts the human's values survived.
#
# Exit 0 = all pass, 1 = a failure, 2 = tooling (fail-closed).

set -uo pipefail

PROJECT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FW_ROOT="$PROJECT/.agentic-framework"
BVP="$FW_ROOT/lib/bvp.sh"

PASS=0; FAIL=0
ok()   { printf '  \033[0;32mok\033[0m    %s\n' "$1"; PASS=$((PASS+1)); }
fail() { printf '  \033[0;31mFAIL\033[0m  %s\n' "$1"; FAIL=$((FAIL+1)); }

[ -f "$BVP" ] || { echo "TOOLING: $BVP not found" >&2; exit 2; }
command -v python3 >/dev/null 2>&1 || { echo "TOOLING: python3 missing" >&2; exit 2; }
python3 -c 'import yaml' 2>/dev/null || { echo "TOOLING: PyYAML missing" >&2; exit 2; }
command -v git >/dev/null 2>&1 || { echo "TOOLING: git missing" >&2; exit 2; }

SCRATCH="$(mktemp -d)"; trap 'rm -rf "$SCRATCH"' EXIT

mk_project() {
    local d="$1"
    mkdir -p "$d/.tasks/active" "$d/.context" "$d/policy"
    printf 'project_name: fixture-project\nversion: 1.6.29\n' > "$d/.framework.yaml"
    cat > "$d/.tasks/active/T-9001-fixture.md" <<'MD'
---
id: T-9001
name: "Fixture task"
status: started-work
workflow_type: build
owner: agent
bvp_scores_proposed:
  - ts: '2026-09-27T00:00:00Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 4
      D3: 2
    rationale: D1=2 (no-signal); D2=4 (body:structural-gate); D3=2 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-9001
MD
    cp -r "$PROJECT/policy/." "$d/policy/" 2>/dev/null || true
}

# bin/fw sources the library then calls bvp_dispatch (bin/fw:4867). Reproducing those
# two lines. A first draft of the T-3176 suite ran `bash lib/bvp.sh`, which merely
# defines functions and exits 0 in silence, so every verb read as ALLOWED and a mutant
# reported KILLED while nothing had run. Case 0 exists because of that.
run_at() { # <lib> <argv...>
    local lib="$1"; shift
    env PROJECT_ROOT="$PROJ" FRAMEWORK_ROOT="$FW_ROOT" CLAUDECODE=1 \
        FW_BVP_TELEMETRY_PATH="$LEDGER" \
        ${APPROVAL+BVP_HUMAN_APPROVAL="$APPROVAL"} \
        BVP_LIB="$lib" \
        bash -c '. "$BVP_LIB"; bvp_dispatch "$@"' _ "$@" 2>&1
}
run() { run_at "$BVP" "$@"; }

scores_of() { # <task-file> -> the confirmed scores dict, or empty
    python3 -c "
import re,sys,yaml
raw=open(sys.argv[1]).read(); m=re.match(r'^---\n(.*?)\n---',raw,re.S)
fm=yaml.safe_load(m.group(1)) or {}
print(fm.get('bvp_scores') or '')" "$1" 2>/dev/null
}
source_of() {
    python3 -c "
import re,sys,yaml
raw=open(sys.argv[1]).read(); m=re.match(r'^---\n(.*?)\n---',raw,re.S)
fm=yaml.safe_load(m.group(1)) or {}
print(fm.get('confirmed_via') or '(absent)')" "$1" 2>/dev/null
}
# T-3370: source_of() was adapted to upstream's provenance (T-3487/T-3523): it reads
# confirmed_via, which replaces our old bvp_scores_source field. Same property (who
# set these scores), upstream's mechanism.

echo "=== T-3184: BVP scoring is an agent decision, with a sticky human override ==="

# ================================================================= Case 0
echo
echo "Case 0 — harness control: is the subject actually running?"
PROJ="$SCRATCH/c0"; LEDGER="$SCRATCH/c0.ndjson"; mk_project "$PROJ"; unset APPROVAL
ctl="$(run confirm --help)"
if ! grep -qF 'Usage: fw bvp confirm' <<< "$ctl"; then
    echo "  HARNESS BROKEN: 'confirm --help' produced no usage text." >&2
    echo "  Got: $(head -c 200 <<< "$ctl")" >&2
    exit 2
fi
ok "the subject executes and produces output"
run confirm T-9001 --i-am-human >/dev/null
if ! grep -q '^bvp_scores:' "$PROJ/.tasks/active/T-9001-fixture.md"; then
    echo "  HARNESS BROKEN: a human confirm did not write bvp_scores:." >&2
    exit 2
fi
ok "the subject mutates the task file on a known-good path"

# ================================================================= Case 1
echo
echo "Case 1 — the ruling: an agent may drive all FIVE verbs with no --i-am-human"
PROJ="$SCRATCH/c1"; LEDGER="$SCRATCH/c1.ndjson"; mk_project "$PROJ"; unset APPROVAL
out="$(run confirm T-9001)"; rc=$?
if [ "$rc" -eq 0 ]; then ok "confirm proceeds (rc 0)"; else fail "confirm refused: rc $rc"; fi
if grep -qF 'agents must not invoke' <<< "$out"; then
    fail "confirm still printed the §ACD refusal"
else ok "no §ACD refusal printed"; fi
for spec in "weight|--set|D1=4|--rationale|raising D1: the gate is load-bearing now" \
            "driver|--add|F-NEW|--weight|3|--rationale|new driver for the trial" \
            "driver|--remove|F-NEW|--rationale|trial over, retiring the driver" \
            "auto-promote|--enable|--rationale|operator ruled AutoPromote is auto now"; do
    IFS='|' read -r -a argv <<< "$spec"
    vout="$(run "${argv[@]}")"; vrc=$?
    label="${argv[0]} ${argv[1]}"
    if grep -qF 'agents must not invoke' <<< "$vout"; then
        fail "$label still refuses with the §ACD gate"
    else
        ok "$label is no longer §ACD-refused (rc $vrc)"
    fi
done

# ================================================================= Case 2
echo
echo "Case 2 — reversibility: BVP_HUMAN_APPROVAL re-arms the gate on all five"
PROJ="$SCRATCH/c2"; LEDGER="$SCRATCH/c2.ndjson"; mk_project "$PROJ"; APPROVAL=true
for spec in "confirm|T-9001" \
            "weight|--set|D1=4|--rationale|raising D1 for the trial" \
            "driver|--add|F-NEW|--weight|3|--rationale|new driver" \
            "driver|--remove|F-NEW|--rationale|retiring it" \
            "auto-promote|--enable|--rationale|turning it on"; do
    IFS='|' read -r -a argv <<< "$spec"
    vout="$(run "${argv[@]}")"
    if grep -qF 'agents must not invoke' <<< "$vout"; then
        ok "${argv[0]} ${argv[1]:-} refuses again with approval re-armed"
    else
        fail "${argv[0]} ${argv[1]:-} did NOT refuse with BVP_HUMAN_APPROVAL=true — not reversible"
    fi
done
# Fail-safe: only a recognised truthy value re-arms. A typo must not silently gate.
for bad in 0 false no off garbage ""; do
    PROJ="$SCRATCH/c2b"; LEDGER="$SCRATCH/c2b.ndjson"; rm -rf "$PROJ"; mk_project "$PROJ"
    APPROVAL="$bad"
    vout="$(run confirm T-9001)"
    if grep -qF 'agents must not invoke' <<< "$vout"; then
        fail "BVP_HUMAN_APPROVAL='$bad' re-armed the gate — only truthy values may"
    else ok "BVP_HUMAN_APPROVAL='$bad' leaves the gate open"; fi
done

# ================================================================= Case 3  ★★
echo
echo "Case 3 ★★ LOAD-BEARING — a human override SURVIVES a later assessment run"
echo "          (score -> human override -> re-propose -> agent confirm -> still there)"
PROJ="$SCRATCH/c3"; LEDGER="$SCRATCH/c3.ndjson"; mk_project "$PROJ"; unset APPROVAL
TASK="$PROJ/.tasks/active/T-9001-fixture.md"

# 1. the human overrides D2 from the proposed 4 to 1
run confirm T-9001 --override D2=1 --i-am-human >/dev/null
human_scores="$(scores_of "$TASK")"
if grep -qF "'D2': 1" <<< "$human_scores"; then ok "human override recorded (D2=1)"; else
    fail "human override not recorded — got: $human_scores"; fi
if [ "$(source_of "$TASK")" = "human" ]; then
    ok "provenance stamped confirmed_via: human (upstream T-3523 mechanism)"
else fail "provenance NOT stamped — got: $(source_of "$TASK")"; fi

# 2. the estimator re-proposes (simulating the next assessment run) with D2 back at 4.
#
# Done by LOADING and REPLACING the key, not by appending YAML text. A first draft
# appended a second `bvp_scores_proposed:` block to frontmatter that already carried
# `bvp_scores_proposed: []` from the confirm that cleared it. That duplicate key made
# the file a parse hazard — and the agent confirm below then refused for THAT reason
# while this suite scored it as the sticky override working. A refusal for the wrong
# reason reads identically to the right one, which is the same false-green shape the
# rest of this suite exists to prevent.
if ! python3 - "$TASK" <<'PY'
import sys,re,yaml
p=sys.argv[1]; raw=open(p).read()
m=re.match(r'^---\n(.*?)\n---',raw,re.S)
fm=yaml.safe_load(m.group(1)) or {}
fm['bvp_scores_proposed']=[{
    'ts':'2026-09-27T12:00:00Z',
    'estimator':'bvp-estimator-v1-heuristic',
    'scores':{'D1':2,'D2':4,'D3':2},
    'rationale':'D1=2 (no-signal); D2=4 (body:structural-gate); D3=2 (no-signal)',
    'rubric_sha':'e4a00f38e801'}]
new=yaml.safe_dump(fm,sort_keys=False,default_flow_style=False).rstrip()
open(p,'w').write('---\n'+new+'\n---'+raw[m.end():])
PY
then
    fail "MEASUREMENT INVALID — could not stage the re-proposal"
fi
# Prove the staging worked AND left exactly one key, before relying on what follows.
dupes="$(grep -c '^bvp_scores_proposed:' "$TASK")"
if [ "$dupes" = "1" ] && grep -q 'D2: 4' "$TASK"; then
    ok "next assessment run re-proposed D2=4 (single key, file still parses)"
else
    fail "MEASUREMENT INVALID — staging left $dupes bvp_scores_proposed key(s)"
fi

# 3. the agent tries to confirm — this is the moment the override must hold
aout="$(run confirm T-9001)"; arc=$?
# T-3370: adapted to upstream's mechanism (bvp_sticky.py, "skip and report"): the agent
# confirm is SKIPPED and REPORTED with rc 0 rather than refused with rc != 0. The property
# is unchanged — the agent run does not overwrite — and it is asserted on disk below too.
if grep -qF 'SKIPPED T-9001 bvp_scores: operator-adjusted, not overwritten' <<< "$aout"; then
    ok "agent confirm SKIPPED over human-set scores and reported it (rc $arc)"
else
    fail "agent confirm was ALLOWED over human-set scores — the override is not sticky"; fi
if grep -qF 'skipped as operator-adjusted' <<< "$aout" && grep -qF '[provenance]' <<< "$aout"; then
    ok "report names the sticky route and carries the skip summary"
else fail "skip report did not name the sticky route (provenance) / summary"; fi

# 4. the actual property: the human's value is still on disk
after="$(scores_of "$TASK")"
if grep -qF "'D2': 1" <<< "$after"; then
    ok "★ the human's D2=1 SURVIVED the assessment run"
else
    fail "★ the human's override was overwritten — got: $after"
fi

# ================================================================= Case 4
echo
echo "Case 4 — sovereignty includes changing your mind: a human may overwrite their own"
hout="$(run confirm T-9001 --override D2=5 --i-am-human)"; hrc=$?
if [ "$hrc" -eq 0 ]; then ok "human confirm over a human override proceeds"; else
    fail "human was refused over their own override: rc $hrc"; fi
if grep -qF "'D2': 5" <<< "$(scores_of "$TASK")"; then
    ok "the new human value landed (D2=5)"
else fail "human re-override did not land — got: $(scores_of "$TASK")"; fi

# ================================================================= Case 5
echo
echo "Case 5 — the agent path is recorded as such: confirmed_via: agent (upstream provenance)"
PROJ="$SCRATCH/c5"; LEDGER="$SCRATCH/c5.ndjson"; mk_project "$PROJ"; unset APPROVAL
run confirm T-9001 >/dev/null
# T-3370: ADAPTED. The T-3184 version asserted bvp_scores_source ABSENT and confirmed_by
# 'agent:auto' on the agent path. Upstream records the same agent-vs-human distinction in
# confirmed_via (T-3487), which also drives its sticky rule, so the assertion now checks
# that field. Upstream keeps $USER in confirmed_by by design; confirmed_via carries it.
src="$(source_of "$PROJ/.tasks/active/T-9001-fixture.md")"
if [ "$src" = "agent" ]; then
    ok "agent confirm records confirmed_via: agent"
else fail "confirmed_via did not record the agent path — got '$src'"; fi
task5="$PROJ/.tasks/active/T-9001-fixture.md"
if grep -qF 'bvp_scores_source' "$task5"; then
    fail "legacy bvp_scores_source written — a second provenance mechanism beside upstream's"
else ok "no legacy bvp_scores_source field (one provenance mechanism, upstream's)"; fi

# ================================================================= Case 6
echo
echo "Case 6 — rationale: mandatory on a CHANGE, exempt on a FIRST set, no 30-char floor"
PROJ="$SCRATCH/c6"; LEDGER="$SCRATCH/c6.ndjson"; mk_project "$PROJ"; unset APPROVAL
# A short but real rationale must be accepted — the floor is gone.
o="$(run weight --set D1=4 --rationale 'gate is load-bearing')"
if grep -qE 'must be ≥[0-9]+ characters' <<< "$o"; then
    fail "the 30-character minimum is still enforced"
else ok "a 21-char rationale is accepted (floor removed)"; fi
# An empty rationale is a missing one wearing a flag.
o="$(run weight --set D1=4 --rationale '   ')"
if grep -qF 'is empty' <<< "$o"; then ok "whitespace-only rationale still refused"; else
    fail "whitespace-only rationale was accepted"; fi
# driver --add is a first set, so it needs none.
o="$(run driver --add F-NEW --weight 3)"
if grep -qF -- '--rationale is required' <<< "$o"; then
    fail "driver --add demanded a rationale on a first set"
else ok "driver --add (first set) needs no rationale"; fi
# driver --remove changes an established thing, so it does.
o="$(run driver --remove F-NEW)"
if grep -qF -- '--rationale is required' <<< "$o"; then
    ok "driver --remove still requires a rationale"
else fail "driver --remove accepted no rationale"; fi

# ================================================================= Case 7
echo
echo "Case 7 — carried over from T-3176: the telemetry ledger still works"
PROJ="$SCRATCH/c7"; LEDGER="$SCRATCH/c7.ndjson"; mk_project "$PROJ"; unset APPROVAL
run confirm T-9001 >/dev/null
if [ -s "$LEDGER" ]; then
    row="$(head -1 "$LEDGER")"
    ok "a row was written"
    grep -qF '"no_signal_count": 2' <<< "$row" \
        && ok "amendment (a): no_signal_count survives the rewrite" \
        || fail "no_signal_count missing — got: $(head -c 200 <<< "$row")"
    grep -qF '"proposer_exact": true' <<< "$row" \
        && ok "proposer_exact still computed" || fail "proposer_exact missing"
else fail "no ledger row written"; fi
real="$PROJECT/.context/telemetry/bvp-confirmations.ndjson"
ledger_size() { [ -f "$1" ] && wc -c < "$1" || echo 0; }
before="$(ledger_size "$real")"
PROJ="$SCRATCH/c7b"; LEDGER="$SCRATCH/c7b.ndjson"; mk_project "$PROJ"
run confirm T-9001 >/dev/null
[ "$before" = "$(ledger_size "$real")" ] \
    && ok "FW_BVP_TELEMETRY_PATH kept the real append-only ledger untouched" \
    || fail "a fixture run wrote into the REAL ledger"

# ================================================================= Case 8  ★
echo
echo "Case 8 ★ ground truth: the PRE-RULING code, from git, must still refuse the agent"
PRE="$SCRATCH/pre.sh"; found=""
# Deterministic, not a walk. `git log -S<anchor>` lists commits that changed the anchor's
# occurrence count; the MOST RECENT one REMOVED it, so that commit's parent is the
# pre-ruling tree. The earlier HEAD..HEAD~N walk was fragile twice over: it re-selects HEAD
# as soon as the change is committed, and a comment quoting the anchor defeats the match
# — the trap that fired three times in T-3178 and is now a registered learning.
_rm="$(git -C "$PROJECT" log -S'_AUTO_CONFIRM_KEY' --format=%H -- .agentic-framework/lib/bvp.sh 2>/dev/null | head -1)"
if [ -n "$_rm" ] && git -C "$PROJECT" show "${_rm}^:.agentic-framework/lib/bvp.sh" > "$PRE" 2>/dev/null; then
    found="${_rm:0:9}^"
fi
if [ -z "$found" ]; then
    fail "MUTATION SETUP BROKEN — no ref in HEAD..HEAD~6 carries the pre-ruling gate"
else
    ok "pre-ruling copy recovered from git ($found)"
    PROJ="$SCRATCH/c8"; LEDGER="$SCRATCH/c8.ndjson"; mk_project "$PROJ"; unset APPROVAL
    # Control leg: prove that copy runs at all before trusting its refusal.
    cout="$(run_at "$PRE" confirm --help)"
    if grep -qF 'Usage: fw bvp confirm' <<< "$cout"; then
        ok "control: pre-ruling copy sources and answers"
        pout="$(run_at "$PRE" weight --set D1=4 --rationale 'a rationale long enough to clear the old thirty character floor')"
        if grep -qF 'agents must not invoke' <<< "$pout"; then
            ok "pre-ruling code REFUSES weight --set — the change is load-bearing"
        else
            fail "pre-ruling code did not refuse; Case 1 proves nothing about the change"
        fi
    else
        fail "MUTATION SETUP BROKEN — pre-ruling copy did not run"
    fi
fi

echo
echo "=== SUMMARY ==="
echo "Pass: $PASS"
echo "Fail: $FAIL"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
