#!/usr/bin/env bash
# T-2698 — fixtures for the .context/working gitignore scope.
#
# Pins the property the change exists for: a static-check allowlist under
# .context/working/ must be trackable by a plain `git add`, so the governance
# decisions it records are recoverable off the machine that made them. And pins
# the trap: under the OLD blanket rule a `!` negation is a SILENT no-op, which is
# the fix the next person will reach for first.
#
# Host-independent (PL-213): everything happens in a scratch git repo. No real
# .context/working, no real allowlist, no network.

set -uo pipefail

PASS=0
FAIL=0
ok()  { PASS=$((PASS + 1)); printf '  \033[0;32mPASS\033[0m  %s\n' "$1"; }
bad() { FAIL=$((FAIL + 1)); printf '  \033[0;31mFAIL\033[0m  %s\n' "$1"; [ $# -gt 1 ] && printf '        %s\n' "$2"; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Build a scratch repo with a given .gitignore body and a populated working dir.
mk_repo() {
    local d="$1" ignore_body="$2"
    rm -rf "$d"
    mkdir -p "$d/.context/working" "$d/scripts"
    ( cd "$d" && git init -q && git config user.email t@t && git config user.name t )
    printf '%s\n' "$ignore_body" > "$d/.gitignore"
    # Load-bearing: a governance record.
    printf 'crates/x.rs::Vec::with_capacity(n)  # bounded upstream\n' \
        > "$d/.context/working/.alloc-sink-allowlist"
    # Scratch state the rule exists to keep out.
    printf 'session: junk\n' > "$d/.context/working/session.yaml"
    dd if=/dev/zero of="$d/.context/working/fw-vec-index.db" bs=1024 count=4 2>/dev/null
    printf 'code\n' > "$d/scripts/thing.sh"
}

is_ignored() {  # $1 repo, $2 path -> 0 if git ignores it
    ( cd "$1" && git check-ignore -q "$2" )
}

OLD_RULE='.context/working/'
NEW_RULE='.context/working/*
!.context/working/.*-allowlist'

echo "T-2698 .context/working gitignore-scope fixtures"
echo ""

# ---------------------------------------------------------------------------
# 1. Reproduce the defect: under the old blanket rule the allowlist is ignored.
# ---------------------------------------------------------------------------
D="$TMP/old"; mk_repo "$D" "$OLD_RULE"
if is_ignored "$D" ".context/working/.alloc-sink-allowlist"; then
    ok "OLD rule ignores the allowlist (the defect reproduces)"
else
    bad "OLD rule ignores the allowlist" "it was not ignored — fixture no longer reproduces"
fi

# ---------------------------------------------------------------------------
# 2. THE TRAP. Adding a `!` negation under the blanket DIRECTORY rule does
#    nothing at all — git cannot re-include a path whose parent is excluded.
#    It fails silently, which is why it is the wrong fix and worth pinning.
# ---------------------------------------------------------------------------
D="$TMP/naive"; mk_repo "$D" "$OLD_RULE
!.context/working/.*-allowlist"
if is_ignored "$D" ".context/working/.alloc-sink-allowlist"; then
    ok "a ! negation under the blanket rule is a SILENT no-op (still ignored)"
else
    bad "a ! negation under the blanket rule is a no-op" "it worked — the trap is gone, revisit the fix"
fi

# ---------------------------------------------------------------------------
# 3. THE LOAD-BEARING ONE. Under the new rule the allowlist is trackable, and a
#    plain `git add -A` — what anyone actually runs — picks it up.
# ---------------------------------------------------------------------------
D="$TMP/new"; mk_repo "$D" "$NEW_RULE"
if ! is_ignored "$D" ".context/working/.alloc-sink-allowlist"; then
    ok "NEW rule leaves the allowlist trackable"
else
    bad "NEW rule leaves the allowlist trackable" "still ignored"
fi
( cd "$D" && git add -A >/dev/null 2>&1 )
if ( cd "$D" && git diff --cached --name-only | grep -qx '.context/working/.alloc-sink-allowlist' ); then
    ok "plain 'git add -A' stages the allowlist"
else
    bad "plain 'git add -A' stages the allowlist" "$( cd "$D" && git diff --cached --name-only | tr '\n' ' ')"
fi

# ---------------------------------------------------------------------------
# 4. The rule still does its job: scratch state stays out.
# ---------------------------------------------------------------------------
for f in session.yaml fw-vec-index.db; do
    if is_ignored "$D" ".context/working/$f"; then
        ok "NEW rule still ignores $f"
    else
        bad "NEW rule still ignores $f" "it became trackable — scratch state would get committed"
    fi
done
if ( cd "$D" && git diff --cached --name-only | grep -q 'fw-vec-index.db' ); then
    bad "'git add -A' does not stage the vector index" "it staged it"
else
    ok "'git add -A' does not stage the vector index"
fi

# ---------------------------------------------------------------------------
# 5. Re-inclusion is BY PATTERN: a check that adopts the convention tomorrow is
#    trackable with no further gitignore edit. Enumerating today's filenames
#    would reproduce the defect one file later.
# ---------------------------------------------------------------------------
printf 'x\n' > "$D/.context/working/.future-check-allowlist"
if ! is_ignored "$D" ".context/working/.future-check-allowlist"; then
    ok "a NEW check's allowlist is trackable without editing .gitignore again"
else
    bad "a future allowlist is trackable by pattern" "still ignored"
fi

# ---------------------------------------------------------------------------
# 6. Nothing outside .context/working is affected.
# ---------------------------------------------------------------------------
if ! is_ignored "$D" "scripts/thing.sh"; then
    ok "paths outside .context/working are untouched"
else
    bad "paths outside .context/working are untouched" "scripts/thing.sh became ignored"
fi

# ---------------------------------------------------------------------------
# 7. The real repo behaves the same way — the fixture is not testing a rule that
#    differs from the one actually shipped.
# ---------------------------------------------------------------------------
if grep -q '^\.context/working/\*$' "$REPO_ROOT/.gitignore" \
   && grep -q '^!\.context/working/\.\*-allowlist$' "$REPO_ROOT/.gitignore"; then
    ok "the shipped .gitignore carries the contents-plus-re-include form"
else
    bad "the shipped .gitignore carries the fixed form" "the rule under test is not the shipped rule"
fi

echo ""
echo "----------------------------------------"
printf 'T-2698 fixtures: %d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" = "0" ] || exit 1
exit 0
