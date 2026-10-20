#!/usr/bin/env bash
# reviewer-verdict-ledger-fixtures.sh — T-3373: the operator ruling of 2026-09-28 against
# upstream's verdict ledger (AEF T-3579/T-3581: lib/verdict_ledger.py, HMAC-signed dispatch
# provenance, the apply_reviewer_verdicts path in update-task.sh).
#
# Ruling: an EXTERNAL reviewer's PASS closes a low-risk task's reviewer-judged Human AC.
# Everything else stays open. Replaces the retired tests/reviewer-sovereignty-fixtures.sh
# (T-3198), which extracted external_reviewer_clears_sovereignty from update-task.sh.
#
# Weighted to the REFUSE cases. One ALLOW case proves the refusals are not vacuous: the same
# builder, with every knob at its honest setting, DOES tick. Every case runs in a scratch git
# project under mktemp -d (own .tasks/, .context/, dispatch key). Nothing touches the real
# .context/ or .tasks/. The vendored code is only read, never edited; the mutant runs on a copy.
#
# What `apply` is: update-task.sh apply_reviewer_verdicts runs exactly
#   PROJECT_ROOT=<root> python3 $FRAMEWORK_ROOT/lib/verdict_ledger.py apply <task>
# on every close attempt, so these cases drive that command. The signed chain is built with the
# module's OWN signing helpers (the same-user honesty the module documents: whoever reads the
# dispatch key can sign a coherent chain); the verdict row itself is written by the real
# record().
#
# guard-layer: fixture suites (tests/*fixtures*.sh) are members by naming convention.
set -u

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
FW="${VERDICT_FIXTURE_FW:-$REPO/.agentic-framework}"
LEDGER="$FW/lib/verdict_ledger.py"
[ -f "$LEDGER" ] || { echo "SKIP: $LEDGER not present" >&2; exit 2; }
command -v git >/dev/null || { echo "SKIP: git missing" >&2; exit 2; }
python3 -c "import yaml" 2>/dev/null || { echo "SKIP: PyYAML missing" >&2; exit 2; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); echo "ok   $*"; }
bad() { FAIL=$((FAIL+1)); echo "FAIL $*"; }

# ── scratch-project builder (python, so it can use the ledger's own signing) ──────────────
cat > "$TMP/build.py" <<'PYEOF'
import hashlib, json, os, subprocess, sys, time
from pathlib import Path

fw, root, knobs = Path(sys.argv[1]), Path(sys.argv[2]), set(sys.argv[3:])
sys.path.insert(0, str(fw))
os.environ["PROJECT_ROOT"] = str(root)
from lib import verdict_ledger as vl                      # the vendored (or mutant) copy
from lib.delegation import human_criteria

TASK, OTHER = "T-9001", "T-9002"
DID = "d-fixture-0001"
WORKER = vl.worker_identity(DID)

def git(*a, who="producer", env=None):
    e = {**os.environ, "GIT_AUTHOR_NAME": who, "GIT_COMMITTER_NAME": who,
         "GIT_AUTHOR_EMAIL": f"{who}@x.local", "GIT_COMMITTER_EMAIL": f"{who}@x.local"}
    e.update(env or {})
    r = subprocess.run(["git", *a], cwd=root, env=e, capture_output=True, text=True)
    if r.returncode:
        raise SystemExit(f"git {a}: {r.stderr}")
    return r.stdout.strip()

AC = {
 "judged":  ("- [ ] [REVIEW] The summary reads clearly and the tone is friendly\n"
             "  **Steps:**\n  1. Read the summary\n  **Expected:** it is clear\n  **If not:** reword\n"),
 "tier0":   ("- [ ] Approve the blocked command with fw tier0 approve\n"
             "  **Steps:**\n  1. run it\n  **Expected:** approved\n  **If not:** no\n"),
 "determ":  ("- [ ] [REVIEWER] The output file exists\n"
             "  **Steps:**\n  1. Run `test -f out.txt`\n  **Expected:** exit 0\n  **If not:** create it\n"),
}
kind = "tier0" if "ac-tier0" in knobs else "determ" if "ac-deterministic" in knobs else "judged"
tags = "[security]" if "high-risk" in knobs else "[]"
desc = "tidy the notes"
task_md = (f"---\nid: {TASK}\nname: \"Tidy the notes\"\ndescription: {desc}\nstatus: started-work\n"
           f"workflow_type: build\nowner: human\nhorizon: now\ntags: {tags}\ncomponents: []\n---\n\n"
           f"# {TASK}\n\n## Acceptance Criteria\n\n### Agent\n- [x] notes tidied\n\n### Human\n"
           + AC[kind] + "\n## Verification\n\n## Updates\n")
tdir = root / ".tasks" / "active"; tdir.mkdir(parents=True)
tf = tdir / f"{TASK}-tidy.md"; tf.write_text(task_md)
(root / "src.txt").write_text("v1\n")
git("init", "-q", "-b", "main"); git("config", "user.name", "producer"); git("config", "user.email", "p@x.local")
prod = WORKER if "reviewer-is-producer" in knobs else "producer"
git("add", "-A"); git("commit", "-q", "-m", f"{TASK}: tidy the notes", who=prod)
REV = git("rev-parse", "HEAD")

# a signed review dispatch (the dispatcher's act), registered BEFORE the worker runs
wdir = root.parent / "wdir"; wdir.mkdir(exist_ok=True)
drow = {"dispatch_id": DID, "task": OTHER if "dispatch-other-task" in knobs else TASK,
        "task_type": "review", "issuer_session": "issuer-sess", "issuer_identity": "issuer",
        "ts": vl._now(), "revision": REV, "wdir": str(wdir.resolve()), "worker_kind": "claude",
        "vendor": "anthropic", "start_by": int(time.time()) + 300, "complete_by": int(time.time()) + 4500}
key = vl._dispatch_key(root, create=True)
drow["sig"] = "0" * 64 if "badsig" in knobs else vl._sign(key, drow)
if "unsigned" in knobs:
    drow.pop("sig")
vl._append(vl.DISPATCHES, drow, root)

# the verdict row, through the REAL record()
(root / ".context/reviews/evidence").mkdir(parents=True, exist_ok=True)
(root / ".context/reviews/evidence/e.md").write_text("checked: the summary is clear\n")
crit = human_criteria(tf.read_text())[0]
outcome = "red" if "red" in knobs else "green"
try:
    vl.record(TASK, crit.index, outcome, reviewer=WORKER, rung="rung-1-same-vendor-independent",
              guidance="not clear enough" if outcome == "red" else "",
              evidence=[".context/reviews/evidence/e.md"], digest=vl.criterion_digest(crit),
              dispatch_id=DID)
except vl.VerdictRefused as e:
    print(f"RECORD-REFUSED: {e}")
    if "forge-row" not in knobs:
        raise SystemExit(0)

if "forge-row" in knobs:     # a row record() would refuse, appended by hand: apply must still refuse
    from lib import judge_verdict
    jv = judge_verdict.verdict("green", "", judged=f"{TASK}#AC{crit.index}", judge=WORKER,
                               evidence=[".context/reviews/evidence/e.md"])
    row = {"id": "V-20260101-forged001", "ts": jv["ts"], "task": TASK, "ac": crit.index,
           "ac_digest": vl.criterion_digest(crit), "ac_text": "x", "outcome": "green",
           "verdict": jv["state"], "guidance": "", "reviewer": WORKER, "dispatch_id": DID,
           "rung": "rung-1-same-vendor-independent", "evidence": [".context/reviews/evidence/e.md"],
           "evidence_sha256": {".context/reviews/evidence/e.md": vl._hash_path(root / ".context/reviews/evidence/e.md")},
           "judgement": jv, "worker": WORKER, "revision": REV}
    vl._append(vl.VERDICTS, row, root)

if "other-task-row" in knobs:    # the only verdict on file names another task
    rows = [json.loads(l) for l in (root / vl.VERDICTS).read_text().splitlines()]
    rows[0]["task"] = OTHER
    (root / vl.VERDICTS).write_text("".join(json.dumps(r, separators=(",", ":"), sort_keys=True) + "\n" for r in rows))

# the runtime's signed start + completion (what run.sh writes after the worker exits)
if "no-completion" not in knobs:
    now = int(time.time())
    st = {"kind": "start", "source": "runtime", "dispatch_id": DID, "task": TASK, "wdir": str(wdir.resolve()),
          "pid": 1, "epoch": now, "secret_sha256": hashlib.sha256(b"s").hexdigest(), "ts": vl._now()}
    st["sig"] = vl._sign_row(key, st); vl._append(vl.COMPLETIONS, st, root)
    verdicts = [{"verdict_id": r["id"], "ac": r["ac"], "ac_digest": r["ac_digest"], "outcome": r["outcome"],
                 "evidence_sha256": r.get("evidence_sha256") or {}, "verdict_sha256": vl.verdict_hash(r)}
                for r in vl._read(vl.VERDICTS, root) if r.get("dispatch_id") == DID]
    (wdir / "result.md").write_text("review done\n")
    comp = {"kind": "completion", "source": "runtime", "dispatch_id": DID, "task": TASK, "session": DID,
            "worker": WORKER, "wdir": str(wdir.resolve()), "exit_code": 0, "worker_kind": "claude",
            "worker_session": "", "result_sha256": vl._result_sha(wdir), "revision": REV,
            "verdicts": verdicts, "consults": {}, "inputs": {}, "epoch": now, "ts": vl._now()}
    comp["sig"] = vl._sign_row(key, comp); vl._append(vl.COMPLETIONS, comp, root)

# commit the ledger as the worker (author AND committer), unless the case wants it uncommitted
if "uncommitted" not in knobs:
    committer = "someone-else" if "committed-by-other" in knobs else WORKER
    git("add", "-f", "--", ".context/reviews");
    git("commit", "-q", "-m", "review record", who=committer)

if "stale" in knobs:    # producer work for the task lands AFTER the reviewed revision
    (root / "src.txt").write_text("v2\n"); git("add", "src.txt"); git("commit", "-q", "-m", f"{TASK}: more work")
print("BUILT")
PYEOF

cat > "$TMP/why.py" <<'PYEOF'
import os, sys
fw, root, tid = sys.argv[1:4]
sys.path.insert(0, fw); os.environ["PROJECT_ROOT"] = root
from pathlib import Path
from lib import verdict_ledger as vl
from lib.delegation import human_criteria
ctx = vl._task_ctx(Path(root), tid)
print(vl.satisfying_verdict(ctx, human_criteria(ctx.text)[0])[1])
PYEOF

# new scratch project for a case; echoes its root. $1 = framework dir, rest = knobs.
mk() {
  local fw="$1"; shift
  local d; d="$(mktemp -d "$TMP/case.XXXXXX")"; mkdir "$d/proj"
  python3 "$TMP/build.py" "$fw" "$d/proj" "$@" > "$d/build.out" 2>&1 || { cat "$d/build.out" >&2; echo "$d/proj"; return 1; }
  echo "$d/proj"
}
apply() {  # $1 = framework dir, $2 = project root; prints the JSON, rc via $?
  PROJECT_ROOT="$2" python3 "$1/lib/verdict_ledger.py" apply T-9001 2>&1
}
ticked() { grep -qE '^- \[x\] \[(REVIEW|REVIEWER)\]|^- \[x\] Approve' "$1"/.tasks/active/T-9001-tidy.md; }
owner()  { sed -n 's/^owner: *//p' "$1"/.tasks/active/T-9001-tidy.md | head -1; }

# refuse: build with knobs, apply, require: not ticked, owner still human
# $name's leading "Rn" is looked up in WHY (below): the refusal must be for the RIGHT reason,
# read from the ledger's own satisfying_verdict(), not merely "nothing got ticked".
refuse() {
  local name="$1" fw="$2"; shift 2
  local p out why want; p="$(mk "$fw" "$@")" || { bad "$name (builder failed)"; return; }
  out="$(apply "$fw" "$p")"
  want="${WHY[${name%% *}]:-}"
  why="$(python3 "$TMP/why.py" "$fw" "$p" T-9001 2>&1 | tail -1)"
  if [ -n "$want" ] && ! grep -qiE "$want" <<< "$why"; then bad "$name — refused for the wrong reason: want /$want/, got: $why"
  elif ticked "$p"; then bad "$name — criterion was TICKED: $out"
  elif [ "$(owner "$p")" != human ]; then bad "$name — owner changed to $(owner "$p")"
  elif grep -q '"verdict_id"' <<< "$out" && grep -q '"ticked": \[{' <<< "$out"; then bad "$name — apply reported a tick: $out"
  else ok "$name"; fi
}

echo "== ALLOW (the operator ruling) =="
p="$(mk "$FW")" || { bad "allow: builder failed"; }
out="$(apply "$FW" "$p")"
if ticked "$p" && [ "$(owner "$p")" = agent ] && grep -q '"verdict_id"' <<< "$out" \
   && grep -q "Reviewer verdict:\*\* green V-" "$p/.tasks/active/T-9001-tidy.md" \
   && grep -q verdict-apply "$p/.context/reviews/applied.jsonl"; then
  ok "A1 signed external PASS ticks the reviewer-judged Human AC of a low-risk task; owner human→agent; applied.jsonl row"
else bad "A1 signed external PASS did not close the criterion: $out"; fi
out2="$(apply "$FW" "$p")"
if ticked "$p" && ! grep -q '"ticked": \[{' <<< "$out2"; then ok "A2 apply is idempotent (second run ticks nothing, keeps the tick)"
else bad "A2 second apply changed state: $out2"; fi
# withdrawal: editing the criterion after the review voids the tick (fresh consent)
sed -i 's/it is clear/it is very clear/' "$p/.tasks/active/T-9001-tidy.md"
out3="$(apply "$FW" "$p")"
if ! ticked "$p" && [ "$(owner "$p")" = human ] && grep -q '"withdrawn": \[{' <<< "$out3"; then
  ok "A3 editing the criterion after the verdict WITHDRAWS the tick and returns ownership to human"
else bad "A3 tick survived a criterion edit: $out3"; fi

declare -A WHY=(
  [R1]='invalid latest|no-provenance|signature|not in the review-dispatch registry'
  [R2]='invalid signature'
  [R3]='issued for'
  [R4]='no verdict for the current criterion'
  [R5]="latest verdict is 'red'"
  [R6]='not-reviewer-judged'
  [R7]='under-strength'
  [R8]='no-completion'
  [R9]='uncommitted'
  [R10]='introduced-by-other'
  [R11]='reviewer-is-producer'
  [R12]='stale-review'
  [R13]='no verdict for the current criterion|not-reviewer-judged'
)
echo "== REFUSE =="
refuse "R1 unsigned dispatch row (no sig)"                         "$FW" unsigned forge-row
refuse "R2 wrongly signed dispatch row (forged sig)"               "$FW" badsig forge-row
refuse "R3 dispatch issued for ANOTHER task"                       "$FW" dispatch-other-task forge-row
refuse "R4 the only verdict row names ANOTHER task"                "$FW" other-task-row
refuse "R5 FAIL (red) verdict"                                     "$FW" red
refuse "R6 operator-only AC (tier0) — forged green row"            "$FW" ac-tier0 forge-row
refuse "R7 task outside the low-risk class (security) — rung 5 needed, no run" "$FW" high-risk forge-row
refuse "R8 no runtime completion (a registration is not a completion)" "$FW" no-completion
refuse "R9 verdict row never committed"                            "$FW" uncommitted
refuse "R10 row committed by someone other than its worker"        "$FW" committed-by-other
refuse "R11 reviewer is also a producer of the task"               "$FW" reviewer-is-producer forge-row
refuse "R12 task work landed after the reviewed revision"          "$FW" stale
refuse "R13 deterministic [REVIEWER] AC is REVIEWER-CLOSEABLE, not ledger-tickable" "$FW" ac-deterministic forge-row

# record() itself must refuse an operator-only criterion before anything is written
p="$(mk "$FW" ac-tier0)";
if grep -q "RECORD-REFUSED" "$p/../build.out" && [ ! -s "$p/.context/reviews/verdicts.jsonl" ]; then
  ok "R14 record() refuses a verdict on an operator-only AC and appends nothing"
else bad "R14 record() accepted/wrote for an operator-only AC: $(cat "$p/../build.out")"; fi

echo "== MUTANTS (copy of the vendored code with ONE refuse check weakened) =="
mutant() {  # $1 name, $2 python expr replacing text via sed script, $3 case knobs..., turns the named refuse case red
  local name="$1" sedexp="$2"; shift 2
  local m="$TMP/mut-$name"; mkdir -p "$m"; cp -r "$FW/lib" "$m/lib"
  [ -d "$FW/policy" ] && cp -r "$FW/policy" "$m/policy"
  sed -i "$sedexp" "$m/lib/verdict_ledger.py"
  if cmp -s "$FW/lib/verdict_ledger.py" "$m/lib/verdict_ledger.py"; then bad "mutant $name did not change the source"; return; fi
  local p out; p="$(mk "$m" "$@")" || { bad "mutant $name: builder failed"; return; }
  out="$(apply "$m" "$p")"
  if ticked "$p"; then ok "mutant $name: refuse case turns RED (criterion ticked) — the case bites"
  else bad "mutant $name: case stayed green, the refuse case does not bite: $out"; fi
}
# M1: dispatch signature check always passes -> R2 (forged sig) must go red
mutant sigcheck 's/        good = hmac.compare_digest(str(rec.get("sig", "")), _sign(key, rec))/        good = True/' badsig forge-row
# M2: producer separation off in the green path -> R11 must go red
# (two independent checks guard this: is_producer() by name, _dispatch_is_producer() by dispatch id — weaken both)
mutant producer '/^def is_producer/,/^    mine = /{s/^    mine = .*/    return ""/}; /^def _dispatch_is_producer/,/^    d = /{s/^    d = .*/    return ""/}' reviewer-is-producer forge-row
# M3: risk class ignored (no strength check) -> R7 must go red
mutant strength 's/^    return _strength_fault(ctx, row, crit) or _run_fault/    return _run_fault/; s/        return (_strength_fault(ctx, row, crit) or _run_fault(ctx, row, crit, recording=recording)/        return (_run_fault(ctx, row, crit, recording=recording)/' high-risk forge-row

echo
echo "reviewer-verdict-ledger-fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
