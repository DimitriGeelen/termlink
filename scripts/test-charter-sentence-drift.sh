#!/usr/bin/env bash
# test-charter-sentence-drift.sh (T-2484) -- host-independent unit tests for the
# charter canonical-sentence sync canary. Stages a fixture repo (three files with
# the per-surface decorations the real files use) and points the canary at it via
# CHARTER_SENTENCE_REPO_ROOT (PL-213 test hook), so no reliance on live repo state.
#
# Prints one line per case + a final "PASS"/"FAIL" summary (P-011 greps "PASS").
set -uo pipefail

SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPT="$SELF_DIR/check-charter-sentence-drift.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

fails=0
check() { # <label> <expected-exit> <actual-exit>
    if [ "$2" -eq "$3" ]; then echo "  ok   $1 (exit $3)"; else echo "  FAIL $1 (expected $2, got $3)"; fails=$((fails+1)); fi
}

# The canonical sentence (single source for the fixtures). The point of the canary
# is that these three renderings must normalize to the SAME thing.
SENT="TermLink is a hub-mediated, durable append-log message bus with terminal endpoints — the coordination substrate that lets a fleet of AI agents (and humans) discover each other, exchange durable messages, claim work, and control terminal sessions across one or many machines."

# Build a fixture repo at $1 with a given README sentence ($2) and ARCHITECTURE
# sentence ($3); CHARTER always uses the canonical $SENT (it is authoritative).
build_repo() { # <root> <readme-sentence> <arch-sentence>
    local root="$1" readme_s="$2" arch_s="$3"
    mkdir -p "$root/docs"
    # CHARTER: bold + line-wrapped (mirror the real file's rendering).
    {
        echo "## Canonical purpose"
        echo
        echo "**TermLink is a hub-mediated, durable append-log message bus with terminal"
        echo "endpoints — the coordination substrate that lets a fleet of AI agents (and humans)"
        echo "discover each other, exchange durable messages, claim work, and control terminal"
        echo "sessions across one or many machines.**"
    } > "$root/docs/CHARTER.md"
    # README: plain single line + a SECOND sentence that also ends in "machines."
    # (the greedy-match trap the extractor must survive).
    {
        echo "![TermLink](header.svg)"
        echo
        echo "$readme_s"
        echo
        echo "TermLink lets a fleet of agents discover each other and work across one or many machines."
    } > "$root/README.md"
    # ARCHITECTURE: blockquote-prefixed.
    {
        echo "# TermLink Architecture"
        echo
        echo "> $arch_s"
    } > "$root/docs/ARCHITECTURE.md"
}

run() { CHARTER_SENTENCE_REPO_ROOT="$1" bash "$SCRIPT" --quiet --no-heartbeat >/dev/null 2>&1; echo $?; }

echo "test-charter-sentence-drift:"

# 1. in-sync — all three agree (README+ARCH use the canonical sentence). Expect 0.
R="$TMP/insync"; build_repo "$R" "$SENT" "$SENT"
check "in-sync (three agree, greedy-trap present) -> 0" 0 "$(run "$R")"

# 2. README drift — one word changed. Expect 1.
R="$TMP/readme"; build_repo "$R" "${SENT/control terminal sessions/control terminal WINDOWS}" "$SENT"
check "README sentence drifts -> 1" 1 "$(run "$R")"

# 3. ARCHITECTURE drift — one word changed. Expect 1.
R="$TMP/arch"; build_repo "$R" "$SENT" "${SENT/coordination substrate/coordination FABRIC}"
check "ARCHITECTURE sentence drifts -> 1" 1 "$(run "$R")"

# 4. missing file — README absent. Expect tooling(2), NOT a false in-sync.
R="$TMP/missing"; build_repo "$R" "$SENT" "$SENT"; rm -f "$R/README.md"
check "missing README -> tooling(2)" 2 "$(run "$R")"

# 5. anchor absent — README has no canonical sentence at all. Expect tooling(2).
R="$TMP/noanchor"; build_repo "$R" "Some unrelated intro text with no anchor phrase." "$SENT"
check "README missing the anchor -> tooling(2)" 2 "$(run "$R")"

# 6. drift diagnostic names the drifted surface (stderr table).
R="$TMP/readme2"; build_repo "$R" "${SENT/control terminal sessions/control terminal WINDOWS}" "$SENT"
err="$(CHARTER_SENTENCE_REPO_ROOT="$R" bash "$SCRIPT" --no-heartbeat 2>&1 || true)"
if printf '%s' "$err" | grep -q "readme"; then
    echo "  ok   drift table names the drifted surface"
else
    echo "  FAIL drift table missing surface name"; fails=$((fails+1))
fi

# 7. --help documents the concept.
if bash "$SCRIPT" --help 2>&1 | grep -q "canonical"; then
    echo "  ok   --help documents the canonical sentence"
else
    echo "  FAIL --help missing 'canonical'"; fails=$((fails+1))
fi

echo
if [ "$fails" -eq 0 ]; then echo "test-charter-sentence-drift: PASS"; exit 0
else echo "test-charter-sentence-drift: FAIL ($fails failing)"; exit 1; fi
