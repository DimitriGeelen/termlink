#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
#
# check-verification-misfile.sh (T-2831)
#
# Detects task files carrying SHELL COMMANDS in a section other than
# `## Verification` — the P-011 vacuous-pass class.
#
# WHY THIS EXISTS
#   P-011 extracts commands from the `## Verification` section ONLY. A block that
#   lands under a neighbouring heading is silent in BOTH directions: the task file
#   renders identically, and P-011 reports nothing when it finds nothing — "no
#   commands to run" and "all commands passed" are the same output. T-2830
#   completed reporting "Acceptance criteria: 8/8 checked" having executed ZERO
#   verification commands; its six commands sat under `## Evolution`.
#
#   That is the defect class this repo's guard layer exists to catch, committed in
#   the guard layer itself: a check that asserts a property ADJACENT to the one it
#   claims. The tell — if the check would still pass while the subsystem is
#   entirely broken, it is the wrong check. P-011 would have passed on T-2830 even
#   if the merge had been garbage.
#
# SCOPE — read this before trusting a green.
#   This detects commands in the WRONG SECTION. It does NOT verify that a task's
#   Verification block is adequate, that its commands actually test the ACs, or
#   that a task has any verification at all. A task with an EMPTY `## Verification`
#   passes this check and still gates on nothing. Those are different questions.
#
# ANCHOR (precision over recall, per the sibling static checks)
#   A line fires only when ALL hold:
#     1. it is outside HTML comments and fenced code blocks,
#     2. it starts at column 0 (no leading whitespace),
#     3. it is not a markdown marker line (list / table / quote / heading / link),
#     4. its leading token is command-shaped — a member of the vocabulary MEASURED
#        from this repo's real Verification blocks (2559 task files, 141 distinct
#        leading tokens), or `out=$(`, `VAR=`, `!`, or a termlink/fw binary path,
#     5. the line is a complete statement rather than an English clause — it carries
#        a shell operator or flag (`-x`, `--x`, `&&`, `|`, `$(`, `;`, `>>`, `2>`) OR a
#        path-like argument (`bash scripts/foo.sh`, `python3 tools/x.py`), and
#     6. it opens a command BLOCK: the preceding line is blank, a `#` comment, a
#        heading, or another command line — never mid-paragraph.
#
#   Rules 5 and 6 are what make it usable. Rule 4 alone yields 49 candidates across
#   the corpus, and every one is a wrapped PROSE line that happens to begin with
#   `timeout`, `test`, `git` or `fw`. Adding 5 and 6 takes that to zero without
#   losing the real defect. A permanently-false-positive check is one nobody reads.
#
# Exit: 0 = no misfiled commands, 1 = misfile found, 2 = tooling error (fail-closed).
set -uo pipefail

TASKS_DIR="${VERIFICATION_MISFILE_TASKS_DIR:-.tasks}"
ALLOWLIST_DEFAULT=".context/checks/verification-misfile-allowlist"
ALLOWLIST="${VERIFICATION_MISFILE_ALLOWLIST:-$ALLOWLIST_DEFAULT}"
FORMAT=human
QUIET=0

while [ $# -gt 0 ]; do
    case "$1" in
        --json)          FORMAT=json ;;
        --quiet)         QUIET=1 ;;
        --tasks-dir)     TASKS_DIR="${2:-}"; shift ;;
        --allowlist)     ALLOWLIST="${2:-}"; shift ;;
        --no-heartbeat)  : ;;
        -h|--help)
            sed -n '2,47p' "$0" | sed 's/^# \{0,1\}//'
            exit 0 ;;
        *) echo "check-verification-misfile: unknown argument: $1" >&2; exit 2 ;;
    esac
    shift
done

command -v python3 >/dev/null 2>&1 || {
    echo "check-verification-misfile: python3 not found — cannot scan (fail-closed)" >&2
    exit 2
}
[ -d "$TASKS_DIR" ] || {
    echo "check-verification-misfile: tasks dir not found: $TASKS_DIR (fail-closed)" >&2
    exit 2
}

VERIFICATION_MISFILE_TASKS_DIR="$TASKS_DIR" \
VERIFICATION_MISFILE_ALLOWLIST="$ALLOWLIST" \
VERIFICATION_MISFILE_FORMAT="$FORMAT" \
VERIFICATION_MISFILE_QUIET="$QUIET" \
python3 - <<'PYEOF'
import os, re, sys, json

tasks_dir = os.environ['VERIFICATION_MISFILE_TASKS_DIR']
allowlist_path = os.environ['VERIFICATION_MISFILE_ALLOWLIST']
fmt = os.environ['VERIFICATION_MISFILE_FORMAT']
quiet = os.environ['VERIFICATION_MISFILE_QUIET'] == '1'

SCOPE = ("detects shell commands filed under a heading other than '## Verification'; "
         "does NOT audit whether a task's verification is adequate or present at all")

VOCAB = set('''grep cargo test bash python3 curl git cd termlink shellcheck diff
systemctl echo set fw jq sed awk cat ls rm mkdir cp mv chmod timeout sqlite3 node
npm make'''.split())
SHELLISH = re.compile(r'(\s-{1,2}[a-zA-Z][\w-]*|&&|\|\||\$\(|\||;|>>|2>)')
# A bare `bash scripts/foo.sh` / `python3 tools/x.py` carries no flag or operator,
# so SHELLISH alone misses it — and that is a COMMON verification line. Accept a
# path-like argument as the completeness signal instead. Rule 6 (block context)
# still keeps prose that merely mentions such a path from firing.
PATHARG = re.compile(r'^\S+\s+\S*(/\S*|\.(sh|py|rs|ya?ml|toml|json|md))(\s|$)')
MARKER = re.compile(r'^(#|-|\*|\||>|<|\d+\.|\[|\*\*|!\[)')
BINPATH = re.compile(r'^(\./)?(target/(release|debug)/termlink|\.?/?\.agentic-framework/bin/fw)$')
ASSIGN = re.compile(r'^[A-Za-z_][A-Za-z0-9_]*=')


def is_cmd_token(tok):
    if tok in VOCAB or tok == '!':
        return True
    if tok.startswith('out=$('):
        return True
    if BINPATH.match(tok):
        return True
    return bool(ASSIGN.match(tok))


# Allowlist entries are drift-stable "<relpath>::<Section>" signatures, where
# <Section> is the heading text WITHOUT its leading '##'. Dropping the hashes is
# deliberate: the house convention strips a trailing '# reason' comment, and a
# signature containing '##' would be truncated to nothing by that same rule —
# silently acknowledging everything, or (as here) nothing. Caught by fixtures.
acknowledged = set()
if os.path.isfile(allowlist_path):
    try:
        for raw in open(allowlist_path, errors='replace'):
            entry = re.split(r'(?:^|\s)#', raw, maxsplit=1)[0].strip()
            if entry:
                acknowledged.add(entry)
    except OSError as exc:
        print("check-verification-misfile: cannot read allowlist %s: %s (fail-closed)"
              % (allowlist_path, exc), file=sys.stderr)
        sys.exit(2)


def scan(path):
    """Yield (lineno, section, text) for command lines outside ## Verification."""
    try:
        lines = open(path, errors='replace').read().split('\n')
    except OSError as exc:
        raise RuntimeError('%s: %s' % (path, exc))
    section = '(frontmatter)'
    in_comment = in_fence = False
    fm = 0
    prev = None
    for i, line in enumerate(lines, 1):
        s = line.strip()
        if s == '---' and fm < 2:
            fm += 1
            prev = None
            continue
        if fm < 2:
            continue
        if s.startswith('```') or s.startswith('~~~'):
            in_fence = not in_fence
            prev = 'FENCE'
            continue
        if in_fence:
            continue
        if s.startswith('## '):
            section = s
            prev = 'HEAD'
            continue
        if '<!--' in line and '-->' not in line:
            in_comment = True
            continue
        if in_comment:
            if '-->' in line:
                in_comment = False
                prev = 'COMMENT'
            continue
        if '<!--' in line and '-->' in line:
            prev = 'COMMENT'
            continue
        if not s:
            prev = 'BLANK'
            continue
        if line[:1] in (' ', '\t'):
            prev = 'PROSE'
            continue
        if s.startswith('#'):
            prev = 'HASH'
            continue
        if MARKER.match(s):
            prev = 'PROSE'
            continue
        cand = (is_cmd_token(s.split()[0])
                and bool(SHELLISH.search(line) or PATHARG.match(s)))
        opens_block = prev in (None, 'BLANK', 'HASH', 'HEAD', 'COMMENT', 'CMD', 'FENCE')
        if cand and opens_block and section != '## Verification':
            yield (i, section, s)
        prev = 'CMD' if cand else 'PROSE'


files = []
for root, _dirs, names in os.walk(tasks_dir):
    if os.path.basename(root) == 'templates':
        continue
    for n in sorted(names):
        if n.endswith('.md'):
            files.append(os.path.join(root, n))

if not files:
    print("check-verification-misfile: no task files under %s (fail-closed)" % tasks_dir,
          file=sys.stderr)
    sys.exit(2)

firing, acked = [], []
try:
    for p in sorted(files):
        for lineno, section, text in scan(p):
            rec = {'file': p, 'line': lineno, 'section': section, 'text': text}
            sig = '%s::%s' % (p, section.lstrip('#').strip())
            (acked if sig in acknowledged else firing).append(rec)
except RuntimeError as exc:
    print("check-verification-misfile: %s (fail-closed)" % exc, file=sys.stderr)
    sys.exit(2)

if fmt == 'json':
    print(json.dumps({
        'ok': not firing,
        'checked': len(files),
        'firing': firing,
        'firing_count': len(firing),
        'acknowledged': acked,
        'acknowledged_count': len(acked),
        'scope': SCOPE,
    }, indent=2))
elif firing:
    nfiles = len({f['file'] for f in firing})
    print("check-verification-misfile: FIRING — %d shell command(s) filed outside "
          "'## Verification' in %d task file(s)" % (len(firing), nfiles))
    print("  P-011 runs the '## Verification' section ONLY. These commands never execute,")
    print("  and the completion gate passes VACUOUSLY — indistinguishable from a real pass.")
    for f in firing:
        print("  %s:%d  under %s" % (f['file'], f['line'], f['section']))
        print("      %s" % f['text'])
    print("  Fix: move the block under the task's '## Verification' heading, then re-run")
    print("  the commands so the claim is backed by an execution rather than by silence.")
    print("  Scope: %s" % SCOPE)
elif not quiet:
    print("check-verification-misfile: healthy — %d task file(s) scanned, 0 misfiled "
          "command block(s), %d acknowledged" % (len(files), len(acked)))
    print("  Scope: %s" % SCOPE)

sys.exit(1 if firing else 0)
PYEOF
