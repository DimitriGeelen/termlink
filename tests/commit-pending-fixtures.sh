#!/usr/bin/env bash
# T-3269 — fixtures for scripts/commit-pending.sh (the SQ-22 option C pending-commit
# manifest) and its wiring into /resume, the handover path and the procAsFit prompt.
#
# Hermetic (PL-213): every case runs in a scratch git repo under mktemp; the real
# repository's index and manifest are never touched. Weighted to the cases the
# operator named — the sweep case (unrelated staged files must stay uncommitted),
# scope refusal, idempotency, a missing file — each with a mutant proving it bites.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$REPO_ROOT/scripts/commit-pending.sh"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); echo "  ok   $1"; }
bad() { FAIL=$((FAIL+1)); echo "  FAIL $1 — $2"; }

newrepo() {  # newrepo <name> → R (scratch repo with one base commit)
    R="$TMP/$1"; mkdir -p "$R/.tasks/active" "$R/.context/checks" "$R/.context/working" "$R/src"
    git -C "$R" init -q -b main
    git -C "$R" config user.email f@x; git -C "$R" config user.name fixture
    echo "base" > "$R/src/a.rs"; echo "l1" > "$R/.context/checks/ledger"; echo "n" > "$R/.context/checks/other"
    git -C "$R" add -A; git -C "$R" commit -q -m "T-1: base"
}
cp_() { (cd "$R" && bash "$S" "$@"); }       # run the script under test inside $R
LIST() { cat "$R/.context/working/pending-commit.list" 2>/dev/null; }

cases() {  # cases <script> <label-prefix> — returns number of failures via globals
    S="$1"
    # --- A1 add + list by name -------------------------------------------------
    newrepo a1; echo "l2" >> "$R/.context/checks/ledger"
    cp_ add T-3260 "ledger refreshed" .context/checks/ledger >/dev/null 2>&1; arc=$?
    out="$(cp_ list 2>&1)"
    [ "$arc" = 0 ] && grep -q "\.context/checks/ledger  \[T-3260\] ledger refreshed" <<< "$out" \
        && ok "$2 A1 add records; list names the entry by path" || bad "$2 A1" "rc=$arc out=$out"
    # --- A2 scope refusal at add (all-or-nothing) -------------------------------
    newrepo a2
    cp_ add T-5 "x" .tasks/active/T-5.md src/a.rs >/dev/null 2>"$TMP/e"; r1=$?
    cp_ add T-5 "x" /etc/passwd >/dev/null 2>&1; r2=$?
    cp_ add T-5 "x" .tasks/../src/a.rs >/dev/null 2>&1; r3=$?
    cp_ add bogus "x" .tasks/active/T-5.md >/dev/null 2>&1; r4=$?
    [ "$r1" = 2 ] && [ "$r2" = 2 ] && [ "$r3" = 2 ] && [ "$r4" = 2 ] && [ -z "$(LIST)" ] && grep -q "REFUSED 'src/a.rs'" "$TMP/e" \
        && ok "$2 A2 add refuses src/, absolute, traversal, bad id — writes nothing" || bad "$2 A2" "rc=$r1/$r2/$r3/$r4 list=$(LIST)"
    # --- C1 THE SWEEP CASE: only listed paths commit; unrelated staged stays staged
    newrepo c1
    echo "l2" >> "$R/.context/checks/ledger"
    printf -- '---\nid: T-9001\n---\n' > "$R/.tasks/active/T-9001-x.md"
    echo "someone else's edit" >> "$R/src/a.rs"; echo "other session" >> "$R/.context/checks/other"
    git -C "$R" add src/a.rs .context/checks/other           # a concurrent session's staged work
    cp_ add T-3260 "ledger refreshed" .context/checks/ledger >/dev/null
    cp_ add T-9001 "WARN escalation filed" .tasks/active/T-9001-x.md >/dev/null
    out="$(cp_ commit 2>&1)"; rc=$?
    committed="$(git -C "$R" log --format= --name-only main~2..main | sort | tr '\n' ' ')"
    staged="$(git -C "$R" diff --cached --name-only | sort | tr '\n' ' ')"
    subj="$(git -C "$R" log --format=%s main~2..main | sort | tr '\n' '|')"
    [ "$rc" = 0 ] && [ "$committed" = ".context/checks/ledger .tasks/active/T-9001-x.md " ] \
        && [ "$staged" = ".context/checks/other src/a.rs " ] \
        && grep -q "^T-3260: commit unattended write — ledger refreshed|T-9001: commit unattended write — WARN escalation filed|$" <<< "$subj" \
        && [ -z "$(LIST)" ] \
        && ok "$2 C1 sweep case: commits exactly the listed paths, one commit per task id; unrelated staged files stay staged" \
        || bad "$2 C1" "rc=$rc committed=[$committed] staged=[$staged] subj=[$subj] list=$(LIST) out=$out"
    # --- C2 idempotent ----------------------------------------------------------
    head1="$(git -C "$R" rev-parse HEAD)"; out="$(cp_ commit 2>&1)"; rc=$?
    [ "$rc" = 0 ] && grep -q "nothing pending" <<< "$out" && [ "$head1" = "$(git -C "$R" rev-parse HEAD)" ] \
        && ok "$2 C2 second run: nothing pending, HEAD unchanged" || bad "$2 C2" "rc=$rc out=$out"
    # --- C3 missing file reported and dropped -----------------------------------
    newrepo c3
    printf 'x\n' > "$R/.tasks/active/T-9002-gone.md"
    cp_ add T-9002 "filed" .tasks/active/T-9002-gone.md >/dev/null; rm -f "$R/.tasks/active/T-9002-gone.md"
    head1="$(git -C "$R" rev-parse HEAD)"; out="$(cp_ commit 2>&1)"; rc=$?
    [ "$rc" = 0 ] && grep -q "DROPPED '.tasks/active/T-9002-gone.md'" <<< "$out" && [ -z "$(LIST)" ] \
        && [ "$head1" = "$(git -C "$R" rev-parse HEAD)" ] \
        && ok "$2 C3 missing file: reported DROPPED, entry removed, nothing committed" || bad "$2 C3" "rc=$rc out=$out list=$(LIST)"
    # --- C4 out-of-scope entry smuggled into the manifest: refused, kept, loud ---
    newrepo c4; echo "evil" >> "$R/src/a.rs"
    printf 'src/a.rs\tT-7\tsmuggled\t2026-09-30T00:00:00Z\n' > "$R/.context/working/pending-commit.list"
    head1="$(git -C "$R" rev-parse HEAD)"; out="$(cp_ commit 2>&1)"; rc=$?
    [ "$rc" = 1 ] && grep -q "REFUSED 'src/a.rs'" <<< "$out" && grep -q "^src/a.rs" <<< "$(LIST)" \
        && [ "$head1" = "$(git -C "$R" rev-parse HEAD)" ] \
        && ok "$2 C4 outside .tasks/.context: refused loudly, kept, rc 1, nothing committed" || bad "$2 C4" "rc=$rc out=$out"
    # --- C5 dry-run -------------------------------------------------------------
    newrepo c5; echo "l2" >> "$R/.context/checks/ledger"; cp_ add T-3260 "r" .context/checks/ledger >/dev/null
    head1="$(git -C "$R" rev-parse HEAD)"; out="$(cp_ commit --dry-run 2>&1)"; rc=$?
    [ "$rc" = 0 ] && grep -q "WOULD commit \[T-3260\] .context/checks/ledger" <<< "$out" && [ -n "$(LIST)" ] \
        && [ "$head1" = "$(git -C "$R" rev-parse HEAD)" ] \
        && ok "$2 C5 --dry-run: names what it would commit, commits nothing, keeps entries" || bad "$2 C5" "rc=$rc out=$out"
    # --- C6 a hook refusing the commit keeps the entries ------------------------
    newrepo c6; echo "l2" >> "$R/.context/checks/ledger"; cp_ add T-3260 "r" .context/checks/ledger >/dev/null
    printf '#!/bin/sh\necho "hook says no" >&2\nexit 1\n' > "$R/.git/hooks/pre-commit"; chmod +x "$R/.git/hooks/pre-commit"
    out="$(cp_ commit 2>&1)"; rc=$?
    [ "$rc" = 1 ] && grep -q "FAILED to commit \[T-3260\]" <<< "$out" && [ -n "$(LIST)" ] \
        && ok "$2 C6 a refusing hook: FAILED, rc 1, entry kept (no bypass attempted)" || bad "$2 C6" "rc=$rc out=$out"
    # --- C7 dedupe + already-at-HEAD --------------------------------------------
    newrepo c7; echo "l2" >> "$R/.context/checks/ledger"
    cp_ add T-3260 "r" .context/checks/ledger >/dev/null; cp_ add T-3260 "r" .context/checks/ledger >/dev/null
    n="$(LIST | wc -l)"; git -C "$R" commit -qam "T-1: someone committed it already"
    out="$(cp_ commit 2>&1)"; rc=$?
    [ "$n" = 1 ] && [ "$rc" = 0 ] && grep -q "ALREADY '.context/checks/ledger'" <<< "$out" && [ -z "$(LIST)" ] \
        && ok "$2 C7 re-recording is a no-op; an entry already at HEAD is dropped" || bad "$2 C7" "n=$n rc=$rc out=$out"
}

echo "== commit-pending fixtures (real script) =="
cases "$SCRIPT" "real"

echo "== mutants (each must turn at least one named case red) =="
mut() {  # mut <label> <expected-failing-case> <sed-expr>
    local m="$TMP/mut-$1.sh"; sed "$3" "$SCRIPT" > "$m"
    if cmp -s "$m" "$SCRIPT"; then bad "M $1" "sed did not change the script (mutant is vacuous)"; return; fi
    local before=$FAIL p0=$PASS
    out="$(cases "$m" "mut-$1" 2>&1)"
    if grep -q "FAIL mut-$1 $2" <<< "$out"; then PASS=$((p0+1)); FAIL=$before; echo "  ok   M $1 → $2 goes red (load-bearing)"
    else PASS=$p0; FAIL=$((before+1)); echo "  FAIL M $1 — $2 stayed green"; fi
}
mut sweep   C1 's/git commit -q -m "\$msg" -- "\${paths\[@\]}"/git commit -q -m "$msg"/'
mut scope   A2 's/^    case "\$p" in \.tasks\/\*|\.context\/\*) return 0 ;; esac/    return 0/'
mut prune   C2 's/^    \[ "\$dry" -eq 1 \] || with_lock _prune/    :/'
mut missing C3 's/if \[ ! -e "\$p" \]; then/if false; then/'

echo "== wiring =="
grep -q 'commit-pending.sh list' "$REPO_ROOT/.claude/commands/resume.md" \
    && grep -q 'commit-pending.sh commit' "$REPO_ROOT/.claude/commands/resume.md" \
    && ok "W1 /resume Step 1 lists pending entries by name; Step 3 runs the helper" || bad "W1" "resume.md not wired"
grep -q 'commit-pending.sh' "$REPO_ROOT/scripts/run-procasfit-round.sh" \
    && ok "W2 procAsFit round prompt carries the helper as a first step" || bad "W2" "run-procasfit-round.sh not wired"
awk '/commit-pending/ {p=NR} /commit -m "\$COMMIT_TASK: Session handover/ {c=NR} END {exit !(p && c && p < c)}' \
    "$REPO_ROOT/.agentic-framework/agents/handover/handover.sh" \
    && ok "W3 handover.sh runs the helper BEFORE its own pathspec commit" || bad "W3" "handover.sh not wired (or wired after its commit)"
grep -q 'record_pending "\$tid"' "$REPO_ROOT/scripts/warn-escalation-file.sh" \
    && grep -q 'commit-pending.sh}" add T-3260' "$REPO_ROOT/scripts/check-release-publication-freshness.sh" \
    && ok "W4 the WARN filer and the canary ledger refresh record into the manifest" || bad "W4" "writers not wired"

echo
echo "commit-pending fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
