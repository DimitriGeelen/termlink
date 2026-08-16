#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
# check-verification-pipefail.sh (T-2775, G-019 prevention for the PL-080 / L-613 / PL-161 class)
#
# WHY: `update-task.sh` runs every `## Verification` line as `if ( eval "$cmd" ); then`
# under `set -euo pipefail` (update-task.sh:14, :1066). pipefail survives into the
# condition, so a pipeline's exit status is decided by the WORST stage — including a
# producer that was SIGPIPEd because the consumer exited early:
#
#     cargo test 2>&1 | grep -q "test result: ok"
#
# `grep -q` exits 0 the instant it matches and closes the pipe; the producer takes
# SIGPIPE and exits 141; pipefail propagates the 141. **The gate FAILS precisely when
# the check SUCCEEDS** — and the earlier the match, the more reliably it fails.
#
# This is not theoretical and it is not new here. PL-080 recorded it on 2026-04-25
# with the explicit instruction "Avoid bare '| grep -q' in verification commands."
# Measured on 2026-08-16, ~4 months later: 1490 such lines across 802 tasks, 262 of
# them in 61 ACTIVE tasks — 58% of all active verification commands. A learning that
# precise, failing that comprehensively, is the argument for a structural check rather
# than more documentation (the T-2746 argument, one layer up).
#
# The compounding harm is worse than the false failure itself. A gate that fails
# spuriously teaches its operator to reach for `--force`, which bypasses the ENTIRE
# verification block — so a flaky guard does not merely annoy, it trains people out of
# the control. That is this repo's recurring theme (T-2680, T-2683) applied to P-011.
#
# WHICH IDIOMS ARE ACTUALLY SAFE — measured, not assumed (T-2775):
#
#   seq 1 3000000 | grep -q '^1$'                        rc=141  UNSAFE
#   seq 1 3000000 | head -1                              rc=141  UNSAFE
#   test -n "$(seq 1 3000000 | grep -m1 '^1$')"          rc=0    safe   (PL-080)
#   out=$(echo hello); printf '%s' "$out" | grep -q x    rc=0    safe   (small)
#   out=$(seq 1 3000000); printf '%s' "$out" | grep -q x rc=141  UNSAFE (large!)
#   out=$(seq 1 3000000); grep -q '^1$' <<< "$out"       rc=0    safe   (any size)
#
# Note the fifth line. The remediation published by AEF (L-613) and 050-email-archive
# (PL-161) — capture, then `printf '%s' "$out" | parser` — is safe only while the
# captured output fits the pipe buffer. It is a real improvement over the raw pipeline
# but it is SIZE-DEPENDENT, and a fix that works until output grows is how this class
# recurs. This check therefore steers to the two idioms that hold at any size: the
# pipeline inside `$(...)` (PL-080) or a herestring, which spawns no producer at all.
#
# WHAT: scans the `## Verification` block of each task file and flags any line whose
# exit status is decided by a top-level pipeline feeding an early-exiting or parsing
# consumer. Pipelines wholly inside `$(...)` are cleared — the substitution's status is
# discarded and the OUTER command decides. Comment and blank lines are skipped.
#
# Confirmed-safe lines are acknowledged in .context/checks/verification-pipefail-allowlist
# (git-tracked per T-2681), one `<task-file>::<sha1-of-normalized-command>` per line, so a
# reworded command re-fires rather than riding on a stale acknowledgement.
#
# SCOPE LIMIT, stated on every output path: this check reads the SHAPE of a command. It
# does not run it, and it cannot tell whether a given producer would actually be SIGPIPEd
# at today's output size. A clean result means no line carries a shape whose exit status
# is decided by the pipeline — NOT that every verification block is semantically correct.
#
# EXIT: 0 = clean, 1 = unacknowledged risky line(s), 2 = tooling error (fail-closed).
# FLAGS: --json  --quiet  --tasks-dir <p> (repeatable)  --allowlist <p>  --include-completed
# TEST SEAM: --tasks-dir + --allowlist point at fixtures (PL-213).

set -uo pipefail

WANT_JSON=0
QUIET=0
INCLUDE_COMPLETED=0
ALLOWLIST=""
declare -a TASK_DIRS=()

while [ $# -gt 0 ]; do
    case "$1" in
        --json) WANT_JSON=1 ;;
        --quiet) QUIET=1 ;;
        --no-heartbeat) : ;;   # accepted for guard-layer symmetry; this is not a canary
        --include-completed) INCLUDE_COMPLETED=1 ;;
        --tasks-dir) shift; [ $# -gt 0 ] || { echo "check-verification-pipefail: --tasks-dir needs a value" >&2; exit 2; }; TASK_DIRS+=("$1") ;;
        --allowlist) shift; [ $# -gt 0 ] || { echo "check-verification-pipefail: --allowlist needs a value" >&2; exit 2; }; ALLOWLIST="$1" ;;
        -h|--help) sed -n '2,60p' "$0"; exit 0 ;;
        *) echo "check-verification-pipefail: unknown argument '$1'" >&2; exit 2 ;;
    esac
    shift
done

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)" || exit 2

if [ ${#TASK_DIRS[@]} -eq 0 ]; then
    TASK_DIRS=("$REPO_ROOT/.tasks/active")
    [ "$INCLUDE_COMPLETED" -eq 1 ] && TASK_DIRS+=("$REPO_ROOT/.tasks/completed")
fi

if [ -z "$ALLOWLIST" ]; then
    # Tracked-first, legacy fallback (T-2681).
    if [ -f "$REPO_ROOT/.context/checks/verification-pipefail-allowlist" ]; then
        ALLOWLIST="$REPO_ROOT/.context/checks/verification-pipefail-allowlist"
    else
        ALLOWLIST="$REPO_ROOT/.context/working/.verification-pipefail-allowlist"
    fi
fi

for d in "${TASK_DIRS[@]}"; do
    if [ ! -d "$d" ]; then
        echo "check-verification-pipefail: tasks dir not found: $d" >&2
        exit 2
    fi
done

command -v python3 >/dev/null 2>&1 || {
    echo "check-verification-pipefail: python3 not found (required)" >&2; exit 2; }

ALLOWLIST="$ALLOWLIST" WANT_JSON="$WANT_JSON" QUIET="$QUIET" \
python3 - "${TASK_DIRS[@]}" <<'PYEOF'
import hashlib
import json
import os
import re
import sys

allowlist_path = os.environ.get("ALLOWLIST", "")
want_json = os.environ.get("WANT_JSON") == "1"
quiet = os.environ.get("QUIET") == "1"
task_dirs = sys.argv[1:]

# Consumers that either exit early (SIGPIPEing the producer) or whose own exit
# status masks the producer's. Anchored to a word boundary so `grepdiff` or a
# path ending in `head` is not matched.
EARLY_EXIT = r"(?:grep\s+(?:-\w*\s+)*-\w*q|head|python3?|jq)"
PIPE_TO_CONSUMER = re.compile(r"\|\s*(?:sudo\s+|command\s+)?" + EARLY_EXIT + r"\b")

# `printf`/`echo` producers are BOUNDED but not unconditionally safe — measured at
# rc=141 for a large captured value. Reported with their own reason so the operator
# is steered to a herestring rather than to a size-dependent fix.
BOUNDED_PRODUCER = re.compile(r"^\s*(?:printf|echo)\b")


def strip_substitutions(cmd):
    """Remove `$( ... )` spans (nesting-aware), backtick spans, and quoted spans.

    A pipeline inside a command substitution cannot decide the line's exit status:
    the substitution's value is what the OUTER command consumes, and the outer
    command's status is what `if ( eval ... )` tests. This is exactly PL-080's
    recommended idiom (`test -n "$(cmd | grep PAT)"`), measured safe at any size,
    so it must not be flagged.

    QUOTED SPANS (T-2777). A `|` inside a quoted argument is literal text, not a
    shell pipeline, so it cannot decide anything either. Without this, an absence
    assertion whose PATTERN happens to contain a pipe —

        ! grep -Fq 'bin/fw reviewer T-XXX 2>&1 | grep -q' .tasks/templates/default.md

    — read as a pipeline and fired, even though the command contains no pipeline at
    all. Found when T-2777's own Verification block tripped this check. Quote
    tracking is deliberately simple: no backslash-escape handling, since a verification
    line needing an escaped quote inside a quoted pipe pattern is not a shape seen in
    this tree.

    `sh -c "cmd | grep -q PAT"` would also be hidden by this, because the pipeline is
    inside quotes — but there the inner shell RUNS it and `sh -c` exits with its
    status, so it is genuinely unsafe. That case is handled before stripping, by
    `unwrap_shell_c()`; see its docstring for why it is not an acceptable blind spot.
    """
    out = []
    i, n = 0, len(cmd)
    depth = 0
    while i < n:
        if cmd.startswith("$(", i):
            depth += 1
            i += 2
            continue
        if depth and cmd[i] == ")":
            depth -= 1
            i += 1
            continue
        if depth == 0 and cmd[i] == "`":
            j = cmd.find("`", i + 1)
            i = n if j == -1 else j + 1
            continue
        if depth == 0 and cmd[i] in ("'", '"'):
            q = cmd[i]
            j = cmd.find(q, i + 1)
            if j == -1:
                i = n
            else:
                # Keep a placeholder so adjacent tokens don't fuse into one word.
                out.append(" ")
                i = j + 1
            continue
        if depth == 0:
            out.append(cmd[i])
        i += 1
    return "".join(out)


def verification_lines(path):
    """Yield (lineno, command) for each executable line of the ## Verification block."""
    rows, inblock = [], False
    with open(path, encoding="utf-8", errors="replace") as fh:
        for lineno, raw in enumerate(fh, 1):
            line = raw.rstrip("\n")
            if line.startswith("## "):
                inblock = line.strip() == "## Verification"
                continue
            if not inblock:
                continue
            s = line.strip()
            if not s or s.startswith("#") or s.startswith("<!--"):
                continue
            rows.append((lineno, s))
    return rows


def normalize(cmd):
    return " ".join(cmd.split())


def signature(relpath, cmd):
    h = hashlib.sha1(normalize(cmd).encode("utf-8")).hexdigest()[:12]
    return f"{os.path.basename(relpath)}::{h}"


SHELL_WRAPPER = re.compile(
    r"""\b(?:ba|z|k|da)?sh\s+-c\s+(['"])(?P<script>.*)\1\s*$"""
)


def unwrap_shell_c(cmd):
    """If the line is a `sh -c '<script>'` wrapper, return <script> ONLY when the
    inner script re-enables pipefail; otherwise return a neutralised form.

    MEASURED (T-2777), under the gate's own `if ( eval "$cmd" )` with
    `set -euo pipefail`:

        seq 1 3000000 | grep -q '^1$'                              141  UNSAFE
        bash -c "seq 1 3000000 | grep -q '^1$'"                      0  safe
        sh   -c "seq 1 3000000 | grep -q '^1$'"                      0  safe
        bash -c "set -o pipefail; seq ... | grep -q '^1$'"         141  UNSAFE
        export SHELLOPTS; bash -c "seq ... | grep -q '^1$'"        141  UNSAFE

    `pipefail` is a shell option, not an environment variable, so `sh -c` starts a
    FRESH shell without it and the pipeline's status is just the consumer's. The
    wrapper is therefore an accidental mitigation, not a hiding place.

    Two rounds of reasoning got this backwards before the measurement settled it:
    first that quote-stripping would hide a real risk, then that `sh -c` propagates
    the pipeline status. Both were plausible and both were wrong. It matters
    concretely — three lines in this tree (T-1673 ×2, T-1885) are this shape and
    were ledgered as risky by T-2775; they are not risky, and their acknowledgements
    were removed rather than carried as debt that does not exist.

    The two genuinely unsafe variants are still caught: an inner `pipefail` is
    detected here, and an exported SHELLOPTS is out of scope for a static read of
    the command (it is a property of the environment, not the line) — noted in the
    check's scope disclaimer rather than silently ignored.
    """
    m = SHELL_WRAPPER.search(cmd.strip())
    if not m:
        return cmd
    script = m.group("script")
    if "pipefail" in script:
        return script
    return ""  # isolated from the outer pipefail: nothing here can decide the status


def classify(cmd):
    """Return a reason string if this line's status is pipeline-decided, else None."""
    bare = strip_substitutions(unwrap_shell_c(cmd))
    m = PIPE_TO_CONSUMER.search(bare)
    if not m:
        return None
    # The producer is the LAST command in the sequence before the pipe: in
    # `out=$(cmd); printf '%s' "$out" | grep -q PAT` the thing feeding the pipe is
    # `printf`, not the whole line. Splitting on `;`/`&&`/`||` and taking the tail is
    # what makes the prescribed capture-then-parse idiom classify as bounded rather
    # than being lumped in with an arbitrary producer.
    head = bare.split("|", 1)[0]
    producer = re.split(r"(?:;|&&|\|\|)", head)[-1]
    if BOUNDED_PRODUCER.match(producer):
        return ("bounded-producer-pipeline",
                "producer is printf/echo — bounded, but measured rc=141 once the "
                "captured value exceeds the pipe buffer; prefer a herestring")
    return ("pipeline-sigpipe",
            "an early-exiting consumer can SIGPIPE the producer; under pipefail the "
            "gate then fails exactly when the check succeeds")


acknowledged_sigs = {}
if allowlist_path and os.path.isfile(allowlist_path):
    with open(allowlist_path, encoding="utf-8", errors="replace") as fh:
        for raw in fh:
            s = raw.strip()
            if not s or s.startswith("#"):
                continue
            sig = s.split("#", 1)[0].strip()
            reason = s.split("#", 1)[1].strip() if "#" in s else ""
            if sig:
                acknowledged_sigs[sig] = reason

firing, acknowledged = [], []
scanned_tasks = scanned_lines = 0

for d in task_dirs:
    for name in sorted(os.listdir(d)):
        if not name.endswith(".md"):
            continue
        path = os.path.join(d, name)
        rows = verification_lines(path)
        if not rows:
            continue
        scanned_tasks += 1
        for lineno, cmd in rows:
            scanned_lines += 1
            verdict = classify(cmd)
            if not verdict:
                continue
            kind, why = verdict
            sig = signature(path, cmd)
            entry = {
                "task": name,
                "line": lineno,
                "signature": sig,
                "kind": kind,
                "why": why,
                "command": cmd[:200],
            }
            if sig in acknowledged_sigs:
                entry["acknowledged_reason"] = acknowledged_sigs[sig]
                acknowledged.append(entry)
            else:
                firing.append(entry)

SCOPE = ("detects command SHAPES whose exit status is decided by a pipeline; it does "
         "not execute them and cannot confirm a given producer is SIGPIPEd at today's "
         "output size, and it reads the line only — an exported SHELLOPTS would re-arm "
         "pipefail inside an `sh -c` wrapper this check reads as isolated (T-2777)")

census = {
    "tasks_with_verification": scanned_tasks,
    "verification_lines": scanned_lines,
    "risky_total": len(firing) + len(acknowledged),
    "acknowledged": len(acknowledged),
    "firing": len(firing),
}

if want_json:
    print(json.dumps({
        "ok": not firing,
        "firing": firing,
        "acknowledged": acknowledged,
        "census": census,
        "scope": SCOPE,
    }, sort_keys=True))
    sys.exit(1 if firing else 0)

if not firing:
    if not quiet:
        print(f"check-verification-pipefail: clean — 0 unacknowledged risky line(s) "
              f"({census['verification_lines']} verification command(s) across "
              f"{census['tasks_with_verification']} task(s); "
              f"{census['acknowledged']} acknowledged).")
        print(f"  Scope: this check {SCOPE}.")
    sys.exit(0)

print("=== check-verification-pipefail: FIRING ===")
print(f"{len(firing)} verification line(s) whose pass/fail is decided by the pipeline, "
      f"not by the check:")
for e in firing[:40]:
    print(f"  ↳ {e['task']}:{e['line']}  [{e['kind']}]")
    print(f"      {e['command']}")
if len(firing) > 40:
    print(f"  ... and {len(firing) - 40} more")
print()
print("  Under `set -euo pipefail` (update-task.sh:14) the gate runs each line as")
print("  `if ( eval \"$cmd\" )`, so a consumer that exits early SIGPIPEs the producer and")
print("  the line reports 141 — failing precisely when the check succeeded (PL-080).")
print("  Safe at any output size (measured, T-2775):")
print("      test -n \"$(cmd | grep -m1 PAT)\"        # pipeline inside $( ), outer cmd decides")
print("      out=$(cmd 2>&1 || true); grep -q PAT <<< \"$out\"   # herestring, no producer")
print("  NOT unconditionally safe: `printf '%s' \"$out\" | grep -q PAT` — measured rc=141")
print("  once $out exceeds the pipe buffer.")
print(f"  Or acknowledge a confirmed-safe line in {allowlist_path} with a cited reason.")
print(f"  Scope: this check {SCOPE}.")
print("---")
sys.exit(1)
PYEOF
rc=$?
exit $rc
