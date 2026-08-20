#!/usr/bin/env bash
# T-2812 — fixtures for scripts/check-vendor-divergence.sh
#
# The load-bearing assertions are the FIRING ones. A register-driven check is
# trivially green when everything is registered, and a green check that cannot
# go red is the exact class this session kept finding — a property asserted
# adjacent to the one that matters. So each fixture below builds a scratch repo
# where the answer is known and asserts the check gets it right in BOTH
# directions.
#
# Host-independent (PL-213): builds its own throwaway git repo and register.
#
# Run: bash tests/vendor-divergence-fixtures.sh

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK="$REPO_ROOT/scripts/check-vendor-divergence.sh"

PASS=0
FAIL=0
ok()  { PASS=$((PASS + 1)); printf '  ok   %s\n' "$1"; }
bad() { FAIL=$((FAIL + 1)); printf '  FAIL %s\n     %s\n' "$1" "${2:-}"; }

[ -r "$CHECK" ] || { echo "cannot read $CHECK" >&2; exit 2; }
command -v git >/dev/null 2>&1 || { echo "git not available" >&2; exit 2; }

echo "== vendor-divergence fixtures =="

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

REPO="$TMP/repo"
mkdir -p "$REPO/.agentic-framework/lib"
cd "$REPO" || exit 2
git init -q .
git config user.email fixture@example.invalid
git config user.name fixture

echo "base" > .agentic-framework/lib/thing.sh
git add -A && git commit -qm "T-1000: fw update v1.0 -> v2.0 (bulk re-vendor)"
BASE=$(git rev-parse --short HEAD)

write_register() {  # write_register <divergence-tasks-csv> <not-divergence-csv>
    {
        echo "schema_version: 1"
        echo "last_vendor_event:"
        echo "  commit: $BASE"
        echo "divergences:"
        if [ -n "$1" ]; then
            IFS=, read -ra ds <<< "$1"
            for t in "${ds[@]}"; do
                echo "  - task: $t"
                echo "    status: filed-upstream"
            done
        else
            echo "  []"
        fi
        echo "not_divergence:"
        if [ -n "$2" ]; then
            echo "  - kind: recovery"
            echo "    tasks: [$2]"
        else
            echo "  []"
        fi
    } > "$TMP/register.yaml"
}

run() { VENDOR_DIVERGENCE_REPO="$REPO" VENDOR_DIVERGENCE_REGISTER="$TMP/register.yaml" \
        bash "$CHECK" "$@" 2>&1; }

# --- 1. no local changes since baseline -> clean -----------------------------
write_register "" ""
out=$(run); rc=$?
if [ "$rc" -eq 0 ]; then ok "no changes since baseline exits 0"
else bad "no changes since baseline" "rc=$rc out=$out"; fi

# --- 2. an UNREGISTERED local change FIRES (the load-bearing case) -----------
echo "local fix" >> .agentic-framework/lib/thing.sh
git add -A && git commit -qm "T-2001: fix a real thing in vendored code"
out=$(run); rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$out" | grep -q 'T-2001'; then
    ok "unregistered local change FIRES and names the commit"
else
    bad "unregistered local change must fire" "rc=$rc out=$out"
fi

# --- 3. registering it under divergences clears it --------------------------
write_register "T-2001" ""
out=$(run); rc=$?
if [ "$rc" -eq 0 ]; then ok "registering under divergences clears the firing"
else bad "registered divergence should clear" "rc=$rc out=$out"; fi

# --- 4. registering under not_divergence also clears -------------------------
# Recovery commits are genuinely not at risk; the check must accept that
# classification without demanding an upstream filing.
write_register "" "T-2001"
out=$(run); rc=$?
if [ "$rc" -eq 0 ]; then ok "registering under not_divergence also clears"
else bad "not_divergence should clear" "rc=$rc out=$out"; fi

# --- 5. a vendor event is not itself divergence ------------------------------
echo "more" >> .agentic-framework/lib/thing.sh
git add -A && git commit -qm "T-2002: fw upgrade 2.0 -> 3.0"
write_register "" "T-2001"
out=$(run); rc=$?
if [ "$rc" -eq 0 ]; then ok "a vendor-event commit is excluded, not flagged"
else bad "vendor event should be excluded" "rc=$rc out=$out"; fi

# --- 6. a second unregistered change fires again ------------------------------
# Guards the obvious regression where the check passes once registered and then
# stops noticing anything new.
echo "another" >> .agentic-framework/lib/thing.sh
git add -A && git commit -qm "T-2003: another local fix"
out=$(run); rc=$?
if [ "$rc" -eq 1 ] && printf '%s' "$out" | grep -q 'T-2003'; then
    ok "a NEW unregistered change fires even after earlier ones are registered"
else
    bad "new unregistered change must fire" "rc=$rc out=$out"
fi

# --- 7. fail-closed: missing register ---------------------------------------
out=$(VENDOR_DIVERGENCE_REPO="$REPO" VENDOR_DIVERGENCE_REGISTER="$TMP/nope.yaml" \
      bash "$CHECK" 2>&1); rc=$?
if [ "$rc" -eq 2 ]; then ok "missing register exits 2, never 0"
else bad "missing register must exit 2" "rc=$rc out=$out"; fi

# --- 8. fail-closed: register without a baseline ----------------------------
# The baseline is DECLARED rather than detected precisely because a heuristic
# once picked a commit saying "re-vendor recommended" and silently shortened the
# window. A register that omits it must refuse, not guess.
printf 'schema_version: 1\ndivergences: []\n' > "$TMP/register.yaml"
out=$(run); rc=$?
if [ "$rc" -eq 2 ]; then ok "register without last_vendor_event exits 2 (no guessing)"
else bad "missing baseline must exit 2" "rc=$rc out=$out"; fi

# --- 9. fail-closed: unparseable register -----------------------------------
printf 'schema_version: 1\n  bad: [unclosed\n' > "$TMP/register.yaml"
out=$(run); rc=$?
if [ "$rc" -eq 2 ]; then ok "unparseable register exits 2, never 0"
else bad "unparseable register must exit 2" "rc=$rc out=$out"; fi

# --- 10. --json shape --------------------------------------------------------
write_register "" "T-2001"
jout=$(run --json)
if printf '%s' "$jout" | grep -q '"unregistered_count": 1' \
   && printf '%s' "$jout" | grep -q '"baseline"'; then
    ok "--json carries baseline, counts and the unregistered list"
else
    bad "--json shape" "got: $jout"
fi

echo
echo "  passed: $PASS   failed: $FAIL"
[ "$FAIL" -eq 0 ] || exit 1
