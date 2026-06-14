#!/usr/bin/env bash
# T-2227 — regression tests for substrate-preflight.sh Check 4 (binary
# freshness / T-2181 + T-2226 feature-aware staleness).
#
# Black-box: shims `termlink --version` via PATH and asserts ONLY the Check-4
# `binary` verdict line. Overall exit code is intentionally NOT asserted —
# Checks 1/2/3/5 depend on the live environment (a shimmed termlink can't probe
# the hub, so Check 5 WARNs), which would make exit-code assertions flaky.
#
# The crates/-aware cases compute their version delta dynamically from
# `git rev-list <last-crates-commit>..HEAD`, so the suite is robust to repo
# state rather than hard-coded to a version number.
#
# Covers:
#   T1 version >= VERSION                          → PASS ">="
#   T2 cross-minor old (0.1.0)                      → WARN "older than"
#   T3 same-minor spanning a real crates/ change    → WARN "older than"
#   T4 same-minor with NO crates/ change in range   → PASS "version drift only"
#   T5 unparseable version                          → WARN "no parseable version"
set -u

SCRIPT="${SCRIPT:-scripts/substrate-preflight.sh}"
PASS=0; FAIL=0; SKIP=0
pass() { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }
skip() { echo "  SKIP: $*"; SKIP=$((SKIP + 1)); }

[ -r VERSION ] || { echo "no VERSION in cwd — run from project root"; exit 2; }
repo_version="$(head -n1 VERSION | tr -d '[:space:]')"
major="$(printf '%s' "$repo_version" | cut -d. -f1)"
minor="$(printf '%s' "$repo_version" | cut -d. -f2)"
patch="$(printf '%s' "$repo_version" | cut -d. -f3)"
patch="${patch//[^0-9]/}"

# Commits since crates/ last changed (drives the dynamic PASS/WARN deltas).
last_crates="$(git log -1 --format=%H -- crates/ 2>/dev/null || true)"
if [ -n "$last_crates" ]; then
    cnt="$(git rev-list --count "${last_crates}..HEAD" 2>/dev/null || echo 0)"
else
    cnt=0
fi

# run_binary_line <version|EMPTY> → echoes the Check-4 verdict line
run_binary_line() {
    local ver="$1" d line
    d="$(mktemp -d)"
    if [ "$ver" = "EMPTY" ]; then
        printf '#!/bin/sh\necho ""\n' > "$d/termlink"
    else
        printf '#!/bin/sh\necho "termlink %s"\n' "$ver" > "$d/termlink"
    fi
    chmod +x "$d/termlink"
    line="$(PATH="$d:$PATH" bash "$SCRIPT" --no-heartbeat 2>&1 | grep -E '\] binary ' | head -n1)"
    rm -rf "$d"
    printf '%s' "$line"
}

# -------- T1: version >= VERSION → PASS ">=" --------
echo "T1: version >= VERSION → PASS"
l="$(run_binary_line "${major}.${minor}.$((patch + 1))")"
if printf '%s' "$l" | grep -q '\[PASS\]' && printf '%s' "$l" | grep -q '>='; then
    pass "T1: $l"
else
    fail "T1: $l"
fi

# -------- T2: cross-minor old → WARN "older than" --------
echo "T2: cross-minor old (0.1.0) → WARN"
l="$(run_binary_line "0.1.0")"
if printf '%s' "$l" | grep -q '\[WARN\]' && printf '%s' "$l" | grep -q 'older than'; then
    pass "T2: $l"
else
    fail "T2: $l"
fi

# -------- T3: same-minor spanning a real crates/ change → WARN --------
echo "T3: same-minor spanning a crates/ change → WARN"
warn_delta=$((cnt + 1))
warn_patch=$((patch - warn_delta))
if [ "$warn_patch" -le 0 ] || ! git rev-parse --verify -q "HEAD~${warn_delta}" >/dev/null 2>&1; then
    skip "T3: cannot construct a same-minor version spanning a crates/ change (cnt=$cnt)"
else
    l="$(run_binary_line "${major}.${minor}.${warn_patch}")"
    if printf '%s' "$l" | grep -q '\[WARN\]' && printf '%s' "$l" | grep -q 'older than'; then
        pass "T3: delta=$warn_delta $l"
    else
        fail "T3: delta=$warn_delta $l"
    fi
fi

# -------- T4: same-minor, NO crates/ change in range → PASS "version drift only" --------
echo "T4: same-minor, no crates/ change in range → PASS (version drift only)"
if [ "$cnt" -lt 1 ] || [ "$patch" -lt 2 ]; then
    skip "T4: crates/ changed at HEAD (cnt=$cnt) — cannot construct an unchanged range"
else
    # delta=1: HEAD~1..HEAD excludes the crates commit (which is cnt>=1 back).
    l="$(run_binary_line "${major}.${minor}.$((patch - 1))")"
    if printf '%s' "$l" | grep -q '\[PASS\]' && printf '%s' "$l" | grep -q 'version drift only'; then
        pass "T4: $l"
    else
        fail "T4: $l"
    fi
fi

# -------- T5: unparseable version → WARN "no parseable version" --------
echo "T5: unparseable version → WARN"
l="$(run_binary_line "EMPTY")"
if printf '%s' "$l" | grep -q '\[WARN\]' && printf '%s' "$l" | grep -q 'no parseable version'; then
    pass "T5: $l"
else
    fail "T5: $l"
fi

echo ""
echo "Results: $PASS pass / $FAIL fail / $SKIP skip"
[ "$FAIL" -eq 0 ]
