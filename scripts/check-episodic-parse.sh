#!/usr/bin/env bash
# guard-layer: source
# T-2805 — episodic-store readability check.
#
# Episodic memory is one of the framework's three memory types, and it is the
# only one with no reader that would notice it had gone bad.
#
#   * The GENERATOR (agents/context/lib/episodic.sh, mine_git_timeline) writes
#     git subjects into a DOUBLE-quoted YAML scalar escaping only `"`. A commit
#     message containing a regex class or an alternation emits \- / \| / \* /
#     \. / \[ — none of which are legal YAML escapes — and the file is
#     unparseable from that line on. Reported upstream by
#     832-Workflow-designer on framework:pickup offset 18; reproduced here
#     independently with four further escape characters.
#
#   * The AUDIT runs an `episodic` section HOURLY on cron and checks only that
#     the file EXISTS. It never opens it. A corrupt episodic passes as
#     "All completed tasks have episodic summaries".
#
#   * The one real READER, web/shared.py::get_episodic_tags, ends its parse in
#     `except yaml.YAMLError: continue` — no log line, no counter, no degraded
#     mode. The task simply has no tags and the page renders fine.
#
# So corruption is invisible at write time, at audit time, and at read time.
# Measured on this repo the first time anyone looked: 29 of 2259 files, the
# oldest dead since March. This check is the reader that was missing.
#
# WHAT IT ASSERTS is deliberately the property the real consumer needs, not a
# weaker one that is easier to pass: every episodic must `yaml.safe_load` to a
# mapping. That is exactly what get_episodic_tags does, so a file this check
# calls unreadable IS being silently dropped by Watchtower today.
#
# It CLASSIFIES rather than merely counting, because the repair differs so
# sharply per class that a flat failure list would not tell an operator what to
# do. All classes fire — a legacy-format file is just as invisible to the
# reader as a corrupt one — but each names its own remediation:
#
#   CORRUPT-ESCAPE   the generator bug above. Content is mangled mid-scalar;
#                    regenerate (`fw context generate-episodic T-XXX`) once the
#                    vendored generator is fixed, or the same bytes come back.
#   CORRUPT-OTHER    unreadable for some other reason (an unquoted scalar
#                    opening with a backtick, a decisions block whose
#                    indentation broke). Content may be recoverable by hand.
#   LEGACY-MULTIDOC  `---` frontmatter plus a body — parses under safe_load_ALL,
#                    not safe_load. Content is fully intact; this is a format
#                    migration, not a data loss.
#   LEGACY-MARKDOWN  a `summary:` line followed by a markdown document. An older
#                    generation of the generator. Content intact, same as above.
#
# Exit codes: 0 = every episodic is readable, 1 = one or more are not,
# 2 = tooling error (no python3 / no PyYAML / unreadable dir). FAIL-CLOSED: a
# check that cannot run reports 2, never 0 — reporting "clean" because the
# reader itself failed to load is the very failure this exists to end.
#
# Usage:  bash scripts/check-episodic-parse.sh [--json] [--quiet] [--dir PATH]
#
# Test hook: EPISODIC_DIR=<dir> points the scan at a fixture tree (PL-213).
# Fixtures: bash tests/episodic-parse-check-fixtures.sh

set -uo pipefail

JSON=0
QUIET=0
DIR="${EPISODIC_DIR:-}"

while [ $# -gt 0 ]; do
    case "$1" in
        --json)  JSON=1 ;;
        --quiet) QUIET=1 ;;
        --dir)   shift; DIR="${1:-}" ;;
        -h|--help)
            sed -n '3,58p' "$0" | sed 's/^# \{0,1\}//'
            exit 0 ;;
        *)
            echo "unknown argument: $1" >&2
            echo "usage: $0 [--json] [--quiet] [--dir PATH]" >&2
            exit 2 ;;
    esac
    shift
done

if ! command -v python3 >/dev/null 2>&1; then
    echo "check-episodic-parse: python3 not found — cannot verify (fail-closed)" >&2
    exit 2
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
[ -n "$DIR" ] || DIR="$REPO_ROOT/.context/episodic"

EPISODIC_DIR="$DIR" CHECK_JSON="$JSON" CHECK_QUIET="$QUIET" python3 - <<'PY'
import glob, json, os, re, sys

try:
    import yaml
except ImportError:
    sys.stderr.write("check-episodic-parse: PyYAML not installed — cannot verify (fail-closed)\n")
    sys.exit(2)

DIR   = os.environ["EPISODIC_DIR"]
JSON  = os.environ.get("CHECK_JSON") == "1"
QUIET = os.environ.get("CHECK_QUIET") == "1"

if not os.path.isdir(DIR):
    # An absent store is NOT healthy-by-default. It may legitimately not exist
    # yet, but it may equally be a wrong --dir, and silently reporting "0
    # unreadable" for a directory that was never read is precisely the false
    # assurance this check exists to remove.
    if JSON:
        print(json.dumps({"ok": False, "error": "episodic dir not found", "dir": DIR}))
    else:
        sys.stderr.write("check-episodic-parse: %s does not exist — cannot verify (fail-closed)\n" % DIR)
    sys.exit(2)

files = sorted(glob.glob(os.path.join(DIR, "*.yaml")))

# The generator's illegal escapes. A double-quoted YAML scalar accepts a fixed
# set (0abtnvfre, space, ", /, \, N, _, L, P, xXX, uXXXX, UXXXXXXXX); anything
# else is an error, and the generator escapes only the double quote, so every
# other backslash in a mined commit subject arrives raw.
ESCAPE_RE = re.compile(r"found unknown escape character")


def classify(raw):
    """Return (class, detail) for content that failed safe_load."""

    # LEGACY-MULTIDOC: frontmatter plus a body. safe_load_all reads it whole, so
    # the content is intact and only the single-document assumption is wrong.
    try:
        docs = list(yaml.safe_load_all(raw))
        if len(docs) > 1 and any(isinstance(d, dict) for d in docs):
            return "LEGACY-MULTIDOC", "%d documents; safe_load_all parses it" % len(docs)
    except yaml.YAMLError:
        pass

    # LEGACY-MARKDOWN: a leading `key: value` line, then a markdown document.
    # Detected by SHAPE rather than by the failure text, because the error a
    # markdown body produces depends on whichever character it happens to hit
    # first (`*` reads as an alias, a backtick as a bad token, `#` as a comment)
    # and keying on that would classify one format four different ways.
    head = raw.lstrip().split("\n", 1)
    if head and re.match(r"^[a-z_]+:\s", head[0]):
        body = head[1] if len(head) > 1 else ""
        if re.search(r"^\s*(#{1,6}\s|\*\*)", body, re.M):
            try:
                if isinstance(yaml.safe_load(head[0]), dict):
                    return "LEGACY-MARKDOWN", "yaml preamble followed by a markdown body"
            except yaml.YAMLError:
                pass

    try:
        yaml.safe_load(raw)
    except yaml.YAMLError as e:
        if ESCAPE_RE.search(str(e)):
            m = re.search(r"found unknown escape character '(.+?)'", str(e))
            ch = m.group(1) if m else "?"
            return "CORRUPT-ESCAPE", "illegal YAML escape \\%s from a mined git subject" % ch
        return "CORRUPT-OTHER", str(e).split("\n")[0].strip()
    return "CORRUPT-OTHER", "unknown"


findings = []
for path in files:
    rel = os.path.relpath(path)
    try:
        raw = open(path, "r", errors="replace").read()
    except OSError as e:
        findings.append({"file": rel, "class": "UNREADABLE", "detail": str(e)})
        continue

    try:
        data = yaml.safe_load(raw)
    except yaml.YAMLError:
        cls, detail = classify(raw)
        findings.append({"file": rel, "class": cls, "detail": detail})
        continue

    if not isinstance(data, dict):
        # Parses, but not into the mapping get_episodic_tags requires. Its
        # `isinstance(edata, dict)` guard drops these just as silently as a
        # parse error does, so they are unreadable in every sense that matters.
        findings.append({"file": rel, "class": "NOT-A-MAPPING",
                         "detail": "parses as %s, not a mapping" % type(data).__name__})

by_class = {}
for f in findings:
    by_class.setdefault(f["class"], []).append(f)

REMEDY = {
    "CORRUPT-ESCAPE":  "regenerate with `fw context generate-episodic T-XXX` AFTER the vendored\n"
                       "     generator emits single-quoted YAML — regenerating first reproduces\n"
                       "     the identical bytes",
    "CORRUPT-OTHER":   "inspect by hand; the content may be recoverable without regenerating",
    "LEGACY-MULTIDOC": "format migration — safe_load_all reads it, so merge the documents or\n"
                       "     teach the reader safe_load_all. No content is at risk",
    "LEGACY-MARKDOWN": "format migration — an older generator's output. No content is at risk",
    "NOT-A-MAPPING":   "the file parses but not into a mapping; check what the generator emitted",
    "UNREADABLE":      "filesystem-level: check permissions",
}

if JSON:
    print(json.dumps({
        "ok": not findings,
        "dir": DIR,
        "scanned": len(files),
        "unreadable_count": len(findings),
        "by_class": {k: len(v) for k, v in sorted(by_class.items())},
        "findings": findings,
    }, indent=2))
    sys.exit(1 if findings else 0)

if not findings:
    if not QUIET:
        print("check-episodic-parse: %d episodic(s) scanned, all readable" % len(files))
    sys.exit(0)

print("check-episodic-parse: %d of %d episodic(s) cannot be read by the code that reads episodics"
      % (len(findings), len(files)))
print()
for cls in sorted(by_class):
    group = by_class[cls]
    print("  %s  (%d)" % (cls, len(group)))
    for f in group:
        print("     %-40s %s" % (f["file"], f["detail"]))
    print("   -> %s" % REMEDY.get(cls, "investigate"))
    print()

if not QUIET:
    print("Each of these is dropped silently today: web/shared.py::get_episodic_tags ends its")
    print("parse in `except yaml.YAMLError: continue`, and the hourly audit's episodic section")
    print("checks only that the file exists.")

sys.exit(1)
PY
