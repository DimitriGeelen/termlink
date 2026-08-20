#!/usr/bin/env bash
# T-2805 — fixtures for scripts/check-episodic-parse.sh
#
# Every class is built from a REAL failure shape observed in this repo's
# .context/episodic/, not from an invented one, so a change to the classifier
# that stops recognising the actual corpus reddens here.
#
# The classification assertions matter more than the counts. A checker that
# lumped all 29 findings together would still pass a naive "did it fire?" test
# while being useless to an operator, because the repair for a legacy-format
# file (mechanical migration, content intact) and for a corrupt one (damaged,
# blocked on the vendored generator) are not the same job.
#
# Run: bash tests/episodic-parse-check-fixtures.sh

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="$REPO_ROOT/scripts/check-episodic-parse.sh"

PASS=0
FAIL=0

ok()   { PASS=$((PASS + 1)); printf '  ok   %s\n' "$1"; }
bad()  { FAIL=$((FAIL + 1)); printf '  FAIL %s\n     %s\n' "$1" "${2:-}"; }

assert_eq() {
    # assert_eq <label> <expected> <actual>
    if [ "$2" = "$3" ]; then ok "$1"; else bad "$1" "expected '$2', got '$3'"; fi
}

assert_contains() {
    # assert_contains <label> <needle> <haystack-file>
    if grep -q -- "$2" "$3"; then ok "$1"; else bad "$1" "missing '$2'"; fi
}

assert_absent() {
    if grep -q -- "$2" "$3"; then bad "$1" "unexpectedly found '$2'"; else ok "$1"; fi
}

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "== check-episodic-parse fixtures =="

# ---------------------------------------------------------------------------
# A clean store exits 0 and says so.
# ---------------------------------------------------------------------------
CLEAN="$TMP/clean"
mkdir -p "$CLEAN"
printf 'task_id: T-1\nsummary: fine\ntags: [a]\n' > "$CLEAN/T-1.yaml"
printf 'task_id: T-2\nsummary: also fine\ntags: []\n' > "$CLEAN/T-2.yaml"

bash "$CHECK" --dir "$CLEAN" > "$TMP/clean.out" 2>&1
assert_eq "clean store exits 0" "0" "$?"
assert_contains "clean store reports the scan count" "2 episodic(s) scanned, all readable" "$TMP/clean.out"

# ---------------------------------------------------------------------------
# CORRUPT-ESCAPE — the reported generator bug. A mined git subject containing a
# regex class reaches a double-quoted scalar with the backslash intact.
# ---------------------------------------------------------------------------
ESC="$TMP/escape"
mkdir -p "$ESC"
cat > "$ESC/T-100.yaml" <<'YAML'
task_id: T-100
git_timeline:
  - time: "2026-05-01 10:00:00 +0200"
    action: "T-100: strip /[^a-z0-9_\-]/g from the slug"
YAML

bash "$CHECK" --dir "$ESC" > "$TMP/esc.out" 2>&1
assert_eq "corrupt-escape fires" "1" "$?"
assert_contains "corrupt-escape is classified as such" "CORRUPT-ESCAPE" "$TMP/esc.out"
assert_contains "corrupt-escape names the offending character" 'illegal YAML escape \\-' "$TMP/esc.out"
assert_contains "corrupt-escape warns regeneration is premature" "AFTER the vendored" "$TMP/esc.out"
# The load-bearing negative: real damage must never be excused as legacy format.
assert_absent "corrupt-escape is not mistaken for legacy" "LEGACY" "$TMP/esc.out"

# ---------------------------------------------------------------------------
# LEGACY-MULTIDOC — frontmatter plus a body. Content is wholly intact; only the
# single-document assumption is wrong.
# ---------------------------------------------------------------------------
MULTI="$TMP/multidoc"
mkdir -p "$MULTI"
cat > "$MULTI/T-121.yaml" <<'YAML'
---
task_id: T-121
name: "PTY mode detection"
workflow_type: build
---

summary: >
  Added tcgetattr-based terminal mode detection.
YAML

bash "$CHECK" --dir "$MULTI" > "$TMP/multi.out" 2>&1
assert_eq "legacy-multidoc fires" "1" "$?"
assert_contains "legacy-multidoc is classified as such" "LEGACY-MULTIDOC" "$TMP/multi.out"
assert_contains "legacy-multidoc reports the document count" "2 documents" "$TMP/multi.out"
assert_contains "legacy-multidoc says content is safe" "No content is at risk" "$TMP/multi.out"

# ---------------------------------------------------------------------------
# LEGACY-MARKDOWN — an older generator: one yaml line then a markdown document.
# Classified by SHAPE, because the parse error depends on whichever markdown
# character is hit first and would otherwise split one format across classes.
# ---------------------------------------------------------------------------
MD="$TMP/markdown"
mkdir -p "$MD"
cat > "$MD/T-894.yaml" <<'YAML'
summary: "Converted termlink_spawn to structured JSON output."

# T-894: Standardize MCP termlink_spawn output

**Completed:** 2026-04-05
**Type:** refactor
YAML

bash "$CHECK" --dir "$MD" > "$TMP/md.out" 2>&1
assert_eq "legacy-markdown fires" "1" "$?"
assert_contains "legacy-markdown is classified as such" "LEGACY-MARKDOWN" "$TMP/md.out"

# Same format, but the body opens with a heading rather than bold — the shape
# test must catch both, since keying on the first offending character would not.
cat > "$MD/T-895.yaml" <<'YAML'
summary: "Another older-format episodic."

## Outcome

Some prose.
YAML

bash "$CHECK" --dir "$MD" --json > "$TMP/md.json" 2>&1
MD_LEGACY=$(python3 -c "import json;print(json.load(open('$TMP/md.json'))['by_class'].get('LEGACY-MARKDOWN',0))")
assert_eq "legacy-markdown detected by shape, not by first bad char" "2" "$MD_LEGACY"

# ---------------------------------------------------------------------------
# NOT-A-MAPPING — parses cleanly, but get_episodic_tags's isinstance(dict)
# guard discards it just as silently as a parse error does.
# ---------------------------------------------------------------------------
NM="$TMP/notmap"
mkdir -p "$NM"
printf -- '- one\n- two\n' > "$NM/T-200.yaml"

bash "$CHECK" --dir "$NM" > "$TMP/nm.out" 2>&1
assert_eq "not-a-mapping fires despite parsing" "1" "$?"
assert_contains "not-a-mapping is classified as such" "NOT-A-MAPPING" "$TMP/nm.out"
assert_contains "not-a-mapping names the type found" "parses as list" "$TMP/nm.out"

# ---------------------------------------------------------------------------
# CORRUPT-OTHER — unreadable for a reason that is neither an illegal escape nor
# a known legacy format. Must not be silently folded into another class.
# ---------------------------------------------------------------------------
OTH="$TMP/other"
mkdir -p "$OTH"
printf 'task_id: T-300\nchallenges: `backtick opens an unquoted scalar\n' > "$OTH/T-300.yaml"

bash "$CHECK" --dir "$OTH" > "$TMP/oth.out" 2>&1
assert_eq "corrupt-other fires" "1" "$?"
assert_contains "corrupt-other is classified as such" "CORRUPT-OTHER" "$TMP/oth.out"

# ---------------------------------------------------------------------------
# Fail-closed. A check that cannot run must never report clean.
# ---------------------------------------------------------------------------
bash "$CHECK" --dir "$TMP/does-not-exist" > "$TMP/absent.out" 2>&1
assert_eq "absent dir exits 2, not 0" "2" "$?"
assert_contains "absent dir says fail-closed" "fail-closed" "$TMP/absent.out"

bash "$CHECK" --bogus-flag > "$TMP/badarg.out" 2>&1
assert_eq "unknown argument exits 2" "2" "$?"

bash "$CHECK" --dir "$TMP/does-not-exist" --json > "$TMP/absent.json" 2>&1
ABSENT_OK=$(python3 -c "import json;print(json.load(open('$TMP/absent.json'))['ok'])" 2>/dev/null || echo PARSE_FAIL)
assert_eq "absent dir json reports ok=false" "False" "$ABSENT_OK"

# ---------------------------------------------------------------------------
# JSON envelope shape.
# ---------------------------------------------------------------------------
MIX="$TMP/mixed"
mkdir -p "$MIX"
cp "$ESC/T-100.yaml" "$MIX/"
cp "$MULTI/T-121.yaml" "$MIX/"
cp "$CLEAN/T-1.yaml" "$MIX/"

bash "$CHECK" --dir "$MIX" --json > "$TMP/mix.json" 2>&1
assert_eq "mixed store exits 1" "1" "$?"

python3 - "$TMP/mix.json" > "$TMP/mix.assert" 2>&1 <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
assert d["ok"] is False,                    "ok should be False"
assert d["scanned"] == 3,                   "scanned should be 3, got %s" % d["scanned"]
assert d["unreadable_count"] == 2,          "unreadable_count should be 2, got %s" % d["unreadable_count"]
assert d["by_class"]["CORRUPT-ESCAPE"] == 1,   "one escape finding"
assert d["by_class"]["LEGACY-MULTIDOC"] == 1,  "one multidoc finding"
assert len(d["findings"]) == 2,             "findings length"
for f in d["findings"]:
    assert set(("file", "class", "detail")) <= set(f), "finding keys"
print("json-shape-ok")
PY
assert_contains "json envelope carries counts, classes and findings" "json-shape-ok" "$TMP/mix.assert"

# ---------------------------------------------------------------------------
# --quiet drops the explanatory footer but keeps the findings.
# ---------------------------------------------------------------------------
bash "$CHECK" --dir "$MIX" --quiet > "$TMP/quiet.out" 2>&1
assert_contains "--quiet still lists findings" "CORRUPT-ESCAPE" "$TMP/quiet.out"
assert_absent "--quiet drops the footer" "get_episodic_tags" "$TMP/quiet.out"

# A clean store under --quiet prints nothing at all (cron-friendly).
bash "$CHECK" --dir "$CLEAN" --quiet > "$TMP/quietclean.out" 2>&1
assert_eq "--quiet clean store exits 0" "0" "$?"
assert_eq "--quiet clean store is silent" "0" "$(wc -c < "$TMP/quietclean.out" | tr -d ' ')"

# ---------------------------------------------------------------------------
# EPISODIC_DIR env hook is equivalent to --dir (PL-213).
# ---------------------------------------------------------------------------
EPISODIC_DIR="$CLEAN" bash "$CHECK" > "$TMP/env.out" 2>&1
assert_eq "EPISODIC_DIR env hook works" "0" "$?"
assert_contains "EPISODIC_DIR scans the fixture tree" "2 episodic(s) scanned" "$TMP/env.out"

echo
echo "  passed: $PASS   failed: $FAIL"
[ "$FAIL" -eq 0 ] || exit 1
