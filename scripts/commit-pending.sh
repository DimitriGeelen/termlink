#!/usr/bin/env bash
# commit-pending.sh — T-3269: the "pending commit" manifest (SQ-22 option C).
#
# OPERATOR RULING (SQ-22 = C, 2026-09-30): nothing commits unattended. A job that runs
# without a session (the release canary's WARN-ledger refresh, the WARN filer T-3267, …)
# writes its files and RECORDS them here; the next SESSION commits them as a standing
# first step — /resume Step 3, the handover pre-compact commit, the procAsFit round
# prompt. An unattended commit would race a live session's shared index (T-3231), and a
# session commit that takes "whatever is dirty" sweeps other people's work (T-3090).
# So the commit is by NAME: exactly the recorded paths, nothing else.
#
# Why a manifest and not `git status`: on this host ~20 routine working files are
# always dirty, so "N uncommitted files" at /resume hides a single filed task among
# noise. The manifest names what an unattended writer produced and on whose behalf.
#
# Usage:
#   commit-pending.sh add <T-ID> <reason> <path>...   record paths (writers call this)
#   commit-pending.sh list [--quiet]                  print entries, one per line, by path
#   commit-pending.sh commit [--dry-run]              commit them; default subcommand
#
# Manifest: .context/working/pending-commit.list (PENDING_COMMIT_LIST overrides; the
# directory is gitignored — this is host-local state, never committed itself).
# One TSV line per entry:  <path> <TAB> <T-ID> <TAB> <reason> <TAB> <UTC ISO>
#
# commit rules:
#   - one commit per task id, message "<T-ID>: commit unattended write(s) — <reasons>",
#     made with `git commit -- <paths>`: git's --only form, so anything ELSE staged in
#     the index stays staged and uncommitted (the T-3090/T-3231 sweep case, pinned by
#     fixture). New files are `git add`ed first, by the same exact paths.
#   - SCOPE: only paths under .tasks/ or .context/ are ever committed. Anything else
#     (absolute, `..`, source code) is REFUSED loudly and KEPT in the manifest for a
#     human — `add` refuses it up front too.
#   - a path whose file is gone is REPORTED and DROPPED (never commits a deletion);
#     a path identical to HEAD is reported as already committed and dropped.
#   - after each commit, HEAD is checked to carry each path's worktree content; only
#     verified entries are removed. A failed commit (a hook refusing) keeps its entries.
#   - idempotent: a second run finds nothing pending and exits 0.
#
# Focus: commits are attributed to the task that WROTE the files (the manifest's T-ID),
# not the session's focus. The T-1730 focus-drift gate is a PreToolUse hook reading the
# literal Bash command, so it does not see this script's inner `git commit`; no bypass
# is used or needed, and git's own commit-msg / pre-commit hooks still run on every
# commit. What stands in for the drift check is the attribution rule itself: the only
# ids this script ever commits under are ones a writer recorded next to its own files.
#
# Exit: 0 nothing pending or all handled · 1 something refused or failed (kept, named)
#       · 2 tooling (not a git repo, bad usage, unwritable manifest).
set -uo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || { echo "commit-pending: not a git repository" >&2; exit 2; }
cd "$ROOT" || exit 2
LIST="${PENDING_COMMIT_LIST:-.context/working/pending-commit.list}"
LOCK="$LIST.lock"

in_scope() {  # repo-relative, no traversal, under .tasks/ or .context/
    local p="$1"
    case "$p" in /*|*'/../'*|../*|*/..|'..'|*$'\t'*|*$'\n'*|'') return 1 ;; esac
    case "$p" in .tasks/*|.context/*) return 0 ;; esac
    return 1
}

with_lock() {  # run "$@" holding the manifest lock where flock exists (Linux); else plain
    mkdir -p "$(dirname "$LIST")" || return 2
    if command -v flock >/dev/null 2>&1; then
        ( flock -w 30 9 || { echo "commit-pending: manifest lock busy >30s" >&2; exit 2; }; "$@" ) 9>"$LOCK"
    else
        "$@"
    fi
}

cmd_add() {
    local tid="${1:-}" reason="${2:-}"; shift 2 2>/dev/null || true
    [[ "$tid" =~ ^T-[0-9]+$ ]] || { echo "commit-pending add: task id must be T-NNNN, got '$tid'" >&2; return 2; }
    [ -n "$reason" ] || { echo "commit-pending add: reason required" >&2; return 2; }
    [ $# -gt 0 ] || { echo "commit-pending add: at least one path required" >&2; return 2; }
    reason="${reason//$'\t'/ }"; reason="${reason//$'\n'/ }"
    local p bad=0
    for p in "$@"; do
        p="${p#./}"
        in_scope "$p" || { echo "commit-pending add: REFUSED '$p' — only .tasks/ and .context/ paths may be recorded" >&2; bad=1; }
    done
    [ "$bad" -eq 0 ] || return 2      # all-or-nothing: a bad path writes nothing
    _append() {
        local ts; ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
        touch "$LIST" || return 2
        for p in "$@"; do
            p="${p#./}"
            # dedupe on (path, task): re-recording the same write is a no-op
            awk -F'\t' -v p="$p" -v t="$tid" '$1==p && $2==t {f=1} END {exit !f}' "$LIST" && continue
            printf '%s\t%s\t%s\t%s\n' "$p" "$tid" "$reason" "$ts" >> "$LIST" || return 2
        done
    }
    with_lock _append "$@"
}

cmd_list() {
    local quiet=0; [ "${1:-}" = "--quiet" ] && quiet=1
    if [ ! -s "$LIST" ]; then
        [ "$quiet" -eq 1 ] || echo "commit-pending: nothing pending ($LIST)"
        return 0
    fi
    awk -F'\t' 'NF>=2 {printf "  %s  [%s] %s (recorded %s)\n", $1, $2, $3, $4}' "$LIST"
}

cmd_commit() {
    local dry=0; [ "${1:-}" = "--dry-run" ] && dry=1
    [ -s "$LIST" ] || { echo "commit-pending: nothing pending"; return 0; }
    local snap; snap="$(mktemp)"; cp "$LIST" "$snap"
    local done_lines; done_lines="$(mktemp)"
    local rc=0 tid
    # group by task id, in first-seen order
    for tid in $(awk -F'\t' 'NF>=2 && !s[$2]++ {print $2}' "$snap"); do
        local -a paths=() lines=()
        local reasons=""
        while IFS= read -r line; do
            local p t r
            p="$(cut -f1 <<<"$line")"; t="$(cut -f2 <<<"$line")"; r="$(cut -f3 <<<"$line")"
            [ "$t" = "$tid" ] || continue
            if ! [[ "$t" =~ ^T-[0-9]+$ ]] || ! in_scope "$p"; then
                echo "commit-pending: REFUSED '$p' [$t] — outside .tasks/ and .context/; kept for a human" >&2
                rc=1; continue
            fi
            if [ ! -e "$p" ]; then
                echo "commit-pending: DROPPED '$p' [$t] — file is gone (nothing to commit)"
                printf '%s\n' "$line" >> "$done_lines"; continue
            fi
            if git ls-files --error-unmatch -- "$p" >/dev/null 2>&1 && git diff --quiet HEAD -- "$p" 2>/dev/null; then
                echo "commit-pending: ALREADY '$p' [$t] — identical to HEAD; dropped"
                printf '%s\n' "$line" >> "$done_lines"; continue
            fi
            paths+=("$p"); lines+=("$line")
            case "$reasons" in *"$r"*) ;; *) reasons="${reasons:+$reasons; }$r" ;; esac
        done < "$snap"
        [ "${#paths[@]}" -gt 0 ] || continue
        if [ "$dry" -eq 1 ]; then
            printf 'commit-pending: WOULD commit [%s] %s\n' "$tid" "${paths[*]}"
            continue
        fi
        local n="${#paths[@]}" s="s"; [ "$n" -eq 1 ] && s=""
        local msg="$tid: commit unattended write$s — $reasons

Recorded by an unattended writer in the pending-commit manifest and committed
by the next session (SQ-22 option C, scripts/commit-pending.sh, T-3269).
Paths (exactly these, nothing else from the index):
$(printf '  %s\n' "${paths[@]}")"
        if ! git add -- "${paths[@]}" 2>/tmp/.commit-pending.err \
           || ! git commit -q -m "$msg" -- "${paths[@]}" >>/tmp/.commit-pending.err 2>&1; then
            echo "commit-pending: FAILED to commit [$tid] ${paths[*]} — kept:" >&2
            tail -5 /tmp/.commit-pending.err >&2
            rc=1; continue
        fi
        local i ok=1
        for i in "${!paths[@]}"; do
            if git cat-file -e "HEAD:${paths[$i]}" 2>/dev/null && git diff --quiet HEAD -- "${paths[$i]}"; then
                printf '%s\n' "${lines[$i]}" >> "$done_lines"
            else
                echo "commit-pending: VERIFY FAILED — HEAD does not carry '${paths[$i]}'; kept" >&2
                ok=0; rc=1
            fi
        done
        [ "$ok" -eq 1 ] && echo "commit-pending: committed [$tid] $(git rev-parse --short HEAD) ${paths[*]}"
    done
    # remove exactly the handled lines; anything appended meanwhile survives
    _prune() {
        [ -s "$done_lines" ] || return 0
        local tmp; tmp="$(mktemp)"
        awk 'NR==FNR {d[$0]=1; next} !($0 in d)' "$done_lines" "$LIST" > "$tmp" && cat "$tmp" > "$LIST"
        rm -f "$tmp"
    }
    [ "$dry" -eq 1 ] || with_lock _prune
    rm -f "$snap" "$done_lines"
    return "$rc"
}

sub="${1:-commit}"; [ $# -gt 0 ] && shift
case "$sub" in
    add)    cmd_add "$@" ;;
    list)   cmd_list "$@" ;;
    commit) cmd_commit "$@" ;;
    --dry-run) cmd_commit --dry-run ;;
    -h|--help) sed -n '2,48p' "$0" | sed 's/^# \{0,1\}//' ;;
    *) echo "commit-pending: unknown subcommand '$sub' (add|list|commit)" >&2; exit 2 ;;
esac
