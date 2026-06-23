#!/usr/bin/env bash
# T-2255 — Bootstrap a git worktree with the complete vendored framework.
#
# WHY: In /opt/termlink the vendored `.agentic-framework/` is only PARTIALLY
# tracked — ~1565 files are committed, but a few hundred (managed by `fw upgrade`,
# gitignored) are NOT, including `lib/arc_membership.sh`. A fresh `git worktree
# add` / harness `EnterWorktree` checkout therefore materializes a PARTIAL
# framework, and every `fw` op in the worktree breaks:
#   - `fw task update --status work-completed` dies in evolution_log.sh (sources
#     the absent lib/arc_membership.sh) — work can't be finalized in the worktree.
#   - the reviewer static-scan errors on missing framework files.
#   - merging/cleaning the worktree branch is Tier-0.
#
# FIX: copy the MISSING (untracked/gitignored) framework files from the MAIN
# checkout into the worktree's existing dir — filling the gap WITHOUT touching the
# tracked files. The copied files are themselves gitignored, so `git status` stays
# clean (no spurious deletions/additions) and finalize/reviewer/merge all work.
# Run this once right after creating a worktree.
#
#   ⚠ We deliberately do NOT symlink/replace `.agentic-framework` wholesale: most
#   of it is tracked, so replacing it would show 1565 files as deleted and poison
#   the worktree's git state — the opposite of the goal.
#
# This is a LOCAL mitigation. The structural fix (worktree-aware `fw` resolution /
# honoring an absolute framework_path) belongs upstream — see the follow-up task.
#
# Usage:
#   scripts/worktree-bootstrap.sh             # bootstrap the CURRENT worktree
#   scripts/worktree-bootstrap.sh --self-test # stand up a throwaway worktree, prove it, tear down
#   scripts/worktree-bootstrap.sh -h|--help
#
# Exit codes: 0 = bootstrapped / no-op-in-main / self-test pass; 2 = error (loud).

set -eu

VENDOR_DIR=".agentic-framework"

log()  { printf '%s\n' "$*"; }
err()  { printf 'worktree-bootstrap: %s\n' "$*" >&2; }
die()  { err "$*"; exit 2; }

# Resolve the MAIN checkout root from CWD's repo. The shared common git dir lives
# at <main>/.git; its parent is the main checkout. Works from main or a worktree.
main_checkout_root() {
    local common_dir
    common_dir="$(git rev-parse --git-common-dir 2>/dev/null)" || return 1
    common_dir="$(cd "$(dirname "$common_dir")" 2>/dev/null && pwd)/$(basename "$common_dir")"
    [ -n "$common_dir" ] || return 1
    dirname "$common_dir"
}

is_linked_worktree() {
    local gd cgd
    gd="$(git rev-parse --absolute-git-dir 2>/dev/null)" || return 1
    cgd="$(git rev-parse --git-common-dir 2>/dev/null)" || return 1
    cgd="$(cd "$(dirname "$cgd")" 2>/dev/null && pwd)/$(basename "$cgd")"
    [ "$gd" != "$cgd" ]
}

# Fill the worktree (rooted at CWD's toplevel) with the framework files that the
# main checkout has but the worktree lacks. Tracked files are left untouched.
do_bootstrap() {
    local wt_root main_root src dst copied=0 missing_lib=""
    wt_root="$(git rev-parse --show-toplevel 2>/dev/null)" || die "not inside a git repository"
    main_root="$(main_checkout_root)" || die "cannot resolve the main checkout (git --git-common-dir failed)"

    if [ "$(cd "$wt_root" && pwd)" = "$(cd "$main_root" && pwd)" ]; then
        log "In the main checkout ($main_root) — nothing to bootstrap (it already has the complete $VENDOR_DIR)."
        return 0
    fi
    if ! is_linked_worktree; then
        log "Not a linked worktree — nothing to bootstrap."
        return 0
    fi

    src="$main_root/$VENDOR_DIR"
    dst="$wt_root/$VENDOR_DIR"
    [ -d "$src" ] || die "main checkout has no framework at $src — run \`fw upgrade\` in $main_root first"
    [ -e "$src/bin/fw" ] || die "$src looks incomplete (no bin/fw) — run \`fw upgrade\` in $main_root first"

    # Copy every file present in main's framework but ABSENT in the worktree.
    # That set is exactly the untracked/gitignored files (tracked ones are already
    # checked out into the worktree). cp -p preserves mode/exec bits.
    local rel reldir
    while IFS= read -r -d '' rel; do
        rel="${rel#./}"
        if [ ! -e "$dst/$rel" ]; then
            reldir="$(dirname "$rel")"
            [ "$reldir" = "." ] || mkdir -p "$dst/$reldir"
            cp -p "$src/$rel" "$dst/$rel"
            copied=$((copied + 1))
        fi
    done < <(cd "$src" && find . -type f -print0)

    # Verify the finalize-critical file is now present (the canary the bug is about).
    if [ ! -e "$dst/lib/arc_membership.sh" ]; then
        missing_lib="lib/arc_membership.sh still missing after bootstrap"
    fi
    [ -z "$missing_lib" ] || die "$missing_lib — main checkout may itself be incomplete; run \`fw upgrade\` in $main_root"

    log "Bootstrapped worktree $wt_root: filled $copied missing framework file(s) from $src."
    log "  fw now resolves the complete framework here (finalize/reviewer/merge unblocked); git status stays clean (copied files are gitignored)."
}

# --self-test: stand up a throwaway detached worktree, bootstrap it, and assert
# (1) the finalize-breaker file is now present, (2) git status is NOT polluted by
# the bootstrap, (3) re-running is idempotent. Calls do_bootstrap as an in-process
# function so it tests THIS script even before this file is committed.
run_self_test() {
    command -v git >/dev/null 2>&1 || die "self-test: git not found"
    local main_root tmp wt rc=0 dirty
    main_root="$(main_checkout_root)" || die "self-test: cannot resolve main checkout"
    tmp="$(mktemp -d "${TMPDIR:-/tmp}/wt-bootstrap-selftest.XXXXXX")" || die "self-test: mktemp failed"
    wt="$tmp/wt"
    # shellcheck disable=SC2064
    trap "cd '$main_root' 2>/dev/null; git worktree remove --force '$wt' >/dev/null 2>&1; rm -rf -- '$tmp'; git worktree prune >/dev/null 2>&1 || true" EXIT

    git worktree add --detach "$wt" HEAD >/dev/null 2>&1 || die "self-test: git worktree add failed"

    # Precondition: the fresh worktree is missing the finalize-breaker (the bug).
    if [ -e "$wt/$VENDOR_DIR/lib/arc_membership.sh" ]; then
        log "self-test: NOTE — fresh worktree already had lib/arc_membership.sh (bug may be fixed upstream); continuing."
    fi

    ( cd "$wt" && do_bootstrap ) || { err "self-test: do_bootstrap failed"; rc=1; }

    if [ "$rc" = 0 ]; then
        [ -e "$wt/$VENDOR_DIR/lib/arc_membership.sh" ] \
            || { err "self-test: lib/arc_membership.sh (the finalize-breaker) still missing"; rc=1; }
        [ -e "$wt/$VENDOR_DIR/bin/fw" ] || { err "self-test: bin/fw missing"; rc=1; }
        [ ! -L "$wt/$VENDOR_DIR" ] || { err "self-test: $VENDOR_DIR was replaced by a symlink (would delete tracked files)"; rc=1; }
        # KEY PROPERTY: the bootstrap must not pollute the worktree's git state.
        dirty="$(cd "$wt" && git status --porcelain -- "$VENDOR_DIR" 2>/dev/null)"
        [ -z "$dirty" ] || { err "self-test: bootstrap polluted git status:"; printf '%s\n' "$dirty" >&2; rc=1; }
        # Idempotency.
        ( cd "$wt" && do_bootstrap ) >/dev/null 2>&1 || { err "self-test: second (idempotent) run failed"; rc=1; }
    fi

    if [ "$rc" = 0 ]; then
        log "SELF-TEST PASS — bootstrap fills the missing framework files (arc_membership.sh present), leaves git status clean, idempotent."
        return 0
    fi
    err "SELF-TEST FAIL"
    return 2
}

main() {
    case "${1:-}" in
        -h|--help) sed -n '2,33p' "$0"; exit 0 ;;
        --self-test) run_self_test; exit $? ;;
        "") do_bootstrap ;;
        *) die "unknown arg: $1 (try --help)" ;;
    esac
}

main "$@"
