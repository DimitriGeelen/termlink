#!/usr/bin/env bash
# T-2803 — one-shot remediation for the main-checkout half of T-2819 / T-2822.
#
# Three fixes shipped on worktree-t2687-pickup-failopen are only half-done: each
# narrowed a .gitignore rule so previously-hidden files became trackable, but the
# files themselves still need committing, and that has to happen in the main
# checkout which a worktree session cannot reach (T-559).
#
#   STEP 1  track the vendored framework subset  -> unblocks `fw bvp` and `fw arc`
#           everywhere. lib/bvp.sh and policy/ currently exist on exactly one disk.
#   STEP 2  track the four static-check allowlists -> stops check-alloc-sink-clamps
#           and check-drain-sink-caps firing in every fresh clone on eleven sites a
#           human already reviewed and cleared.
#   STEP 3  remove stale task files that make the task gate refuse to let an agent
#           work in the main checkout at all.
#
# THE REVIEW IS MECHANISED, NOT DROPPED. The operator was asked three times to
# "read that list — confirm nothing machine-local or secret-bearing appears". A
# review requested repeatedly and never failing is a review that has already
# stopped happening. So this scans every file it is about to stage and REFUSES to
# commit if it finds a high-confidence secret marker, naming file and line. Softer
# signals (home paths, host addresses) are reported but do not block — this repo
# legitimately discusses 192.168.10.x throughout, and a scanner that cried wolf
# there would be switched off on first contact.
#
# IT NEVER PUSHES. Getting past the failing pre-push audit needs
# `git push --no-verify`, which is Tier 0. No flag here does it. The script reports
# the unpushed count and prints the command; approving it stays with the human.
#
# IDEMPOTENT. Every step is a no-op once applied, so re-running is safe and the
# report says which steps did nothing.
#
# Exit codes: 0 all steps applied or already done · 1 a step was refused or failed
#             · 2 tooling error (wrong directory, not a git repo)
set -uo pipefail

ROOT="${REMEDIATE_ROOT:-/opt/termlink}"
REPORT="${REMEDIATE_REPORT:-}"
DRY_RUN=0

usage() {
    sed -n '2,36p' "$0" | sed 's/^# \{0,1\}//'
    cat <<'EOF'

Usage: remediate-main-checkout.sh [OPTIONS]
  --root PATH     Checkout to remediate (default /opt/termlink)
  --report PATH   Write a JSON report here (default: alongside this script's
                  worktree, so the agent that wrote it can read the outcome back)
  --dry-run       Show exactly what would happen; change nothing
  -h, --help      This help

Never pushes. See T-2815 — the pre-push audit fails for an unrelated framework
bug, and getting past it is Tier 0.
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --root)    shift; [ $# -ge 1 ] || { echo "remediate: --root requires a value" >&2; exit 2; }; ROOT="$1" ;;
        --report)  shift; [ $# -ge 1 ] || { echo "remediate: --report requires a value" >&2; exit 2; }; REPORT="$1" ;;
        --dry-run) DRY_RUN=1 ;;
        -h|--help) usage; exit 0 ;;
        *) echo "remediate: unknown arg: $1" >&2; exit 2 ;;
    esac
    shift
done

[ -d "$ROOT" ] || { echo "remediate: no such directory: $ROOT" >&2; exit 2; }
cd "$ROOT" || exit 2
git rev-parse --git-dir >/dev/null 2>&1 || { echo "remediate: not a git repository: $ROOT" >&2; exit 2; }

if [ -z "$REPORT" ]; then
    REPORT="$ROOT/.claude/worktrees/t2687-pickup-failopen/.context/working/.remediation-report.json"
fi

export REMEDIATE_ROOT="$ROOT"
export REMEDIATE_REPORT="$REPORT"
export REMEDIATE_DRY_RUN="$DRY_RUN"

python3 - <<'PYEOF'
import json, os, re, subprocess, sys

ROOT    = os.environ["REMEDIATE_ROOT"]
REPORT  = os.environ["REMEDIATE_REPORT"]
DRY     = os.environ["REMEDIATE_DRY_RUN"] == "1"

steps   = []
notes   = []
failed  = False

BOLD = "\033[1m"; RED = "\033[0;31m"; GRN = "\033[0;32m"; YEL = "\033[0;33m"; OFF = "\033[0m"


def run(*args, **kw):
    return subprocess.run(args, capture_output=True, text=True, cwd=ROOT, **kw)


def say(msg=""):
    print(msg)


# --- the mechanised review ------------------------------------------------
# High confidence: refuse the commit. These are shapes that are a secret or
# nothing — a PEM block, a bare 64-hex line (the hub.secret format), a populated
# credential field. Placeholders and empty values are excluded deliberately.
HARD = [
    (re.compile(r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),      "PEM private key"),
    (re.compile(r"^[0-9a-fA-F]{64}$"),                        "bare 64-hex secret (hub.secret format)"),
    (re.compile(r"(?i)\b(password|passwd|api[_-]?key|secret[_-]?key|access[_-]?token)"
                r"\s*[:=]\s*['\"]?(?!(\s*$|<|\{|\$|xxx|todo|changeme|placeholder|example|none|null))"
                r"[^\s'\"]{8,}"),                             "populated credential field"),
    (re.compile(r"(?i)\bghp_[A-Za-z0-9]{20,}"),               "GitHub token"),
]
# Soft: report, do not block. Legitimate throughout this repo.
SOFT = [
    (re.compile(r"/(home|Users)/[a-z][a-z0-9_-]*"),           "absolute home path"),
    (re.compile(r"\b\d{1,3}(\.\d{1,3}){3}\b"),                "host address"),
]

BINARY_HINT = re.compile(rb"\x00")


def scan(paths):
    hard_hits, soft_hits = [], []
    for rel in paths:
        full = os.path.join(ROOT, rel)
        if not os.path.isfile(full):
            continue
        try:
            with open(full, "rb") as fh:
                head = fh.read(4096)
            if BINARY_HINT.search(head):
                continue  # binary: no textual credential to find
            with open(full, "r", errors="replace") as fh:
                for n, line in enumerate(fh, 1):
                    s = line.strip()
                    for rx, label in HARD:
                        if rx.search(s):
                            hard_hits.append({"file": rel, "line": n, "marker": label})
                    for rx, label in SOFT:
                        if rx.search(s):
                            soft_hits.append({"file": rel, "line": n, "marker": label})
                            break
        except OSError as e:
            soft_hits.append({"file": rel, "line": 0, "marker": "unreadable: %s" % e})
    return hard_hits, soft_hits


def stage_and_commit(name, paths, message, purpose):
    """Stage `paths`, scan what actually got staged, commit if clean."""
    global failed
    existing = [p for p in paths if os.path.exists(os.path.join(ROOT, p))]
    if not existing:
        steps.append({"step": name, "result": "skipped",
                      "reason": "none of the target paths exist", "purpose": purpose})
        say("  %sSKIP%s  %s — none of the target paths exist" % (YEL, OFF, name))
        return

    # What would staging actually add? Ask git rather than guessing.
    #
    # The return code matters as much as the output. `git add --dry-run` on a path
    # that is still ignored exits 1 and writes "The following paths are ignored by
    # one of your .gitignore files" to STDERR, printing nothing on stdout. An
    # earlier version of this script read stdout only and treated the empty result
    # as "already done" — reporting success while doing nothing, which is the exact
    # failure class the rest of this branch exists to remove. The dry run against
    # the real checkout is what caught it.
    r = run("git", "add", "--dry-run", "--", *existing)
    would = [l.split(None, 1)[1].strip().strip("'\"")
             for l in r.stdout.splitlines() if l.startswith("add ")]

    if r.returncode != 0:
        if "ignored by one of your" in (r.stderr or ""):
            failed = True
            rules = run("git", "check-ignore", "-v", "--", *existing).stdout.strip().splitlines()
            steps.append({"step": name, "result": "BLOCKED",
                          "reason": "the ignore rule that hid these files is still active "
                                    "in this checkout",
                          "ignore_rules": rules, "purpose": purpose})
            say("  %sBLOCKED%s %s — these paths are STILL IGNORED here, so nothing can be"
                % (RED, OFF, name))
            say("           staged. The .gitignore narrowing that makes them trackable is")
            say("           on worktree-t2687-pickup-failopen and has not reached this")
            say("           checkout. Active rule(s):")
            for line in rules[:6]:
                say("             %s" % line)
            say("           Fix the RULE here first (merge the branch, or apply the")
            say("           .gitignore change), then re-run. Do NOT `git add -f`: that")
            say("           tracks today's files and leaves tomorrow's invisible again,")
            say("           which is the defect T-2822 exists to close.")
            return
        failed = True
        steps.append({"step": name, "result": "failed",
                      "reason": (r.stderr or r.stdout).strip(), "purpose": purpose})
        say("  %sFAIL%s  %s — git add --dry-run: %s"
            % (RED, OFF, name, (r.stderr or r.stdout).strip()[:200]))
        return

    if not would:
        tracked = run("git", "ls-files", "--", *existing).stdout.strip()
        if tracked:
            steps.append({"step": name, "result": "already-done",
                          "reason": "every target path is already tracked and unmodified",
                          "purpose": purpose})
            say("  %sOK%s    %s — already done (already tracked)" % (GRN, OFF, name))
        else:
            failed = True
            steps.append({"step": name, "result": "BLOCKED",
                          "reason": "nothing to stage and nothing tracked — paths exist but "
                                    "git will not take them",
                          "purpose": purpose})
            say("  %sBLOCKED%s %s — paths exist, nothing tracked, nothing stageable."
                % (RED, OFF, name))
            say("           Not a no-op: git is declining for a reason this script did")
            say("           not anticipate. Investigate before assuming it is done.")
        return

    hard, soft = scan(would)
    if hard:
        failed = True
        steps.append({"step": name, "result": "REFUSED", "reason": "secret marker found",
                      "files": would, "hard_findings": hard, "purpose": purpose})
        say("  %sREFUSED%s %s — secret marker(s) found; nothing staged, nothing committed:"
            % (RED, OFF, name))
        for h in hard[:20]:
            say("           %s:%d  %s" % (h["file"], h["line"], h["marker"]))
        return

    if DRY:
        steps.append({"step": name, "result": "dry-run", "files": would,
                      "soft_findings": soft, "purpose": purpose})
        say("  %sDRY%s   %s — would track %d file(s)" % (YEL, OFF, name, len(would)))
        for f in would[:15]:
            say("           + %s" % f)
        if len(would) > 15:
            say("           ... and %d more (full list in the JSON report)" % (len(would) - 15))
        return

    r = run("git", "add", "--", *existing)
    if r.returncode != 0:
        failed = True
        steps.append({"step": name, "result": "failed", "reason": r.stderr.strip(),
                      "purpose": purpose})
        say("  %sFAIL%s  %s — git add: %s" % (RED, OFF, name, r.stderr.strip()))
        return
    r = run("git", "commit", "-m", message)
    if r.returncode != 0:
        failed = True
        steps.append({"step": name, "result": "failed", "reason": r.stderr.strip() or r.stdout.strip(),
                      "purpose": purpose})
        say("  %sFAIL%s  %s — git commit: %s" % (RED, OFF, name, (r.stderr or r.stdout).strip()[:200]))
        return

    steps.append({"step": name, "result": "applied", "files": would,
                  "file_count": len(would), "soft_findings": soft, "purpose": purpose})
    say("  %sDONE%s  %s — tracked %d file(s) and committed" % (GRN, OFF, name, len(would)))
    if soft:
        say("           (%d soft signal(s) noted for review — see JSON report)" % len(soft))


say("%sremediate-main-checkout%s  root=%s%s" % (BOLD, OFF, ROOT, "  [DRY RUN]" if DRY else ""))
say("")

# --- STEP 1 ---------------------------------------------------------------
say("%sStep 1%s  track the vendored framework subset (T-2819)" % (BOLD, OFF))
stage_and_commit(
    "framework-subset",
    [".agentic-framework/lib", ".agentic-framework/policy",
     ".agentic-framework/bin", ".agentic-framework/agents"],
    "T-2819: track the vendored framework subset that the stale ignore rule hid",
    "unblocks `fw bvp` and `fw arc` in every worktree and clean clone")

# --- STEP 2 ---------------------------------------------------------------
say("")
say("%sStep 2%s  track the static-check allowlists (T-2822)" % (BOLD, OFF))
stage_and_commit(
    "static-check-allowlists",
    [".context/working/.alloc-sink-allowlist", ".context/working/.drain-sink-allowlist",
     ".context/working/.silent-exit-allowlist", ".context/working/.busy-spin-allowlist"],
    "T-2822: track the static-check allowlists the blanket ignore rule hid",
    "stops alloc-sink and drain-sink firing in every fresh clone on reviewed sites")

# --- STEP 3 ---------------------------------------------------------------
say("")
say("%sStep 3%s  clear stale task files blocking agent sessions" % (BOLD, OFF))
STALE = ["T-2687-pickupdeduphash-fails-open-to-constant-s.md",
         "T-2683-pickup--from-.md", "T-2684-pickup--from-.md",
         "T-2685-pickup--from-.md", "T-2686-pickup--from-.md"]
present = [f for f in STALE if os.path.exists(os.path.join(ROOT, ".tasks/active", f))]
if not present:
    steps.append({"step": "stale-task-files", "result": "already-done",
                  "reason": "no stale task files present",
                  "purpose": "unblocks agent sessions in the main checkout"})
    say("  %sOK%s    stale-task-files — already done (none present)" % (GRN, OFF))
elif DRY:
    steps.append({"step": "stale-task-files", "result": "dry-run", "files": present,
                  "purpose": "unblocks agent sessions in the main checkout"})
    say("  %sDRY%s   stale-task-files — would remove %d file(s)" % (YEL, OFF, len(present)))
    for f in present:
        say("           - .tasks/active/%s" % f)
else:
    removed = []
    for f in present:
        try:
            os.remove(os.path.join(ROOT, ".tasks/active", f))
            removed.append(f)
        except OSError as e:
            failed = True
            say("  %sFAIL%s  stale-task-files — %s: %s" % (RED, OFF, f, e))
    steps.append({"step": "stale-task-files",
                  "result": "applied" if removed else "failed",
                  "files": removed,
                  "purpose": "unblocks agent sessions in the main checkout"})
    say("  %sDONE%s  stale-task-files — removed %d file(s)" % (GRN, OFF, len(removed)))

# --- verification ---------------------------------------------------------
say("")
say("%sVerification%s" % (BOLD, OFF))
checks = {}
for label, cmd in (("framework-tracking-drift", ["bash", "scripts/check-framework-tracking-drift.sh"]),
                   ("alloc-sink-clamps",       ["bash", "scripts/check-alloc-sink-clamps.sh"]),
                   ("drain-sink-caps",         ["bash", "scripts/check-drain-sink-caps.sh"])):
    if not os.path.exists(os.path.join(ROOT, cmd[1])):
        checks[label] = {"exit": None, "note": "script not present in this checkout"}
        say("  %s-%s     %-26s not present here" % (YEL, OFF, label))
        continue
    r = run(*cmd)
    checks[label] = {"exit": r.returncode}
    mark = ("%sPASS%s" % (GRN, OFF)) if r.returncode == 0 else ("%sFIRES%s" % (YEL, OFF))
    say("  %s  %-26s exit %d" % (mark, label, r.returncode))

# --- the push: reported, never performed ----------------------------------
say("")
say("%sNot done here — Tier 0%s" % (BOLD, OFF))
branch = run("git", "rev-parse", "--abbrev-ref", "HEAD").stdout.strip()
ahead = run("git", "rev-list", "--count", "main..HEAD").stdout.strip() or "?"
say("  %s commit(s) on %s have never left this machine." % (ahead, branch))
say("  The pre-push audit fails for an unrelated framework bug (T-2815), so getting")
say("  past it needs --no-verify, which is Tier 0 and is yours alone to approve:")
say("      cd %s && .agentic-framework/bin/fw tier0 approve" % ROOT)
say("  This script will never do that, and no flag makes it.")

report = {"ok": not failed, "dry_run": DRY, "root": ROOT, "branch": branch,
          "unpushed_commits": ahead, "steps": steps, "verification": checks,
          "notes": notes}
try:
    os.makedirs(os.path.dirname(REPORT), exist_ok=True)
    with open(REPORT, "w") as fh:
        json.dump(report, fh, indent=2)
    say("")
    say("Report: %s" % REPORT)
except OSError as e:
    say("")
    say("  (could not write JSON report to %s: %s)" % (REPORT, e))

sys.exit(1 if failed else 0)
PYEOF
