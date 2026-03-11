#!/usr/bin/env bash
# transcripts.sh — Claude Code transcript retention management
#
# Usage:
#   ./agents/transcripts/transcripts.sh clean [--older-than N] [--dry-run]
#   ./agents/transcripts/transcripts.sh size
#
# Commands:
#   clean        Delete session directories older than N days (default: 30)
#   size         Show current usage breakdown
#
# Flags:
#   --older-than N   Age threshold in days (default: 30)
#   --dry-run        Show what would be deleted, no action taken

set -euo pipefail

# ── Config ────────────────────────────────────────────────────────────────────

PROJECT_ROOT="${PROJECT_ROOT:-$(pwd)}"
CLAUDE_DIR="$HOME/.claude"

# Encode project path the same way Claude Code does
PROJECT_ENCODED="${PROJECT_ROOT//\//-}"

TRANSCRIPTS_DIR="$CLAUDE_DIR/projects/$PROJECT_ENCODED"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# ── Helpers ───────────────────────────────────────────────────────────────────

die() { echo -e "${RED}ERROR:${NC} $*" >&2; exit 1; }
info() { echo -e "${CYAN}$*${NC}"; }
warn() { echo -e "${YELLOW}$*${NC}"; }
ok() { echo -e "${GREEN}$*${NC}"; }

human_size() {
    # Convert bytes to human-readable
    local bytes="$1"
    if [ "$bytes" -ge 1073741824 ]; then
        echo "$(( bytes / 1073741824 )) GB"
    elif [ "$bytes" -ge 1048576 ]; then
        echo "$(( bytes / 1048576 )) MB"
    elif [ "$bytes" -ge 1024 ]; then
        echo "$(( bytes / 1024 )) KB"
    else
        echo "${bytes} B"
    fi
}

get_current_session() {
    # Find the most recently modified JSONL file — that's the current session
    if [ ! -d "$TRANSCRIPTS_DIR" ]; then return; fi
    find "$TRANSCRIPTS_DIR" -maxdepth 1 -name "*.jsonl" -not -name "agent-*" \
        -newer "$TRANSCRIPTS_DIR" 2>/dev/null | head -1 | xargs -I{} basename {} .jsonl 2>/dev/null || true
}

# ── size command ──────────────────────────────────────────────────────────────

cmd_size() {
    if [ ! -d "$TRANSCRIPTS_DIR" ]; then
        die "Transcript directory not found: $TRANSCRIPTS_DIR"
    fi

    echo -e "${BOLD}Claude Code Transcript Storage${NC}"
    echo -e "Directory: $TRANSCRIPTS_DIR"
    echo ""

    local total_bytes
    total_bytes=$(du -sb "$TRANSCRIPTS_DIR" 2>/dev/null | awk '{print $1}' || du -sk "$TRANSCRIPTS_DIR" | awk '{print $1 * 1024}')
    echo -e "Total: ${BOLD}$(human_size "$total_bytes")${NC}"
    echo ""

    # List session dirs by modification time (newest first)
    echo -e "${BOLD}Sessions (newest first):${NC}"
    local session_count=0
    while IFS= read -r session_dir; do
        [ -d "$session_dir" ] || continue
        local session_id
        session_id=$(basename "$session_dir")
        local size_bytes
        size_bytes=$(du -sb "$session_dir" 2>/dev/null | awk '{print $1}' || du -sk "$session_dir" | awk '{print $1 * 1024}')
        local mtime
        mtime=$(stat -f "%Sm" -t "%Y-%m-%d" "$session_dir" 2>/dev/null || stat -c "%y" "$session_dir" 2>/dev/null | cut -d' ' -f1)
        local subagent_count=0
        [ -d "$session_dir/subagents" ] && subagent_count=$(find "$session_dir/subagents" -name "agent-*.jsonl" 2>/dev/null | wc -l | tr -d ' ')

        local label=""
        [ "$subagent_count" -gt 0 ] && label=" [${subagent_count} sub-agents]"

        printf "  %-40s  %8s  %s%s\n" "${session_id:0:40}" "$(human_size "$size_bytes")" "$mtime" "$label"
        session_count=$(( session_count + 1 ))
    done < <(find "$TRANSCRIPTS_DIR" -maxdepth 1 -mindepth 1 -type d ! -name "memory" -print0 | xargs -0 ls -dt 2>/dev/null || find "$TRANSCRIPTS_DIR" -maxdepth 1 -mindepth 1 -type d ! -name "memory" 2>/dev/null)

    echo ""
    echo -e "Total sessions: ${BOLD}$session_count${NC}"

    # Also show loose JSONL files
    local jsonl_count
    jsonl_count=$(find "$TRANSCRIPTS_DIR" -maxdepth 1 -name "*.jsonl" 2>/dev/null | wc -l | tr -d ' ')
    echo -e "Session JSONL files: ${BOLD}$jsonl_count${NC}"
}

# ── clean command ─────────────────────────────────────────────────────────────

cmd_clean() {
    local older_than=30
    local dry_run=false

    while [[ $# -gt 0 ]]; do
        case "$1" in
            --older-than) older_than="$2"; shift 2 ;;
            --dry-run)    dry_run=true; shift ;;
            *) die "Unknown flag: $1" ;;
        esac
    done

    if [ ! -d "$TRANSCRIPTS_DIR" ]; then
        die "Transcript directory not found: $TRANSCRIPTS_DIR"
    fi

    local current_session
    current_session=$(get_current_session)

    if "$dry_run"; then
        warn "DRY RUN — no files will be deleted"
    fi

    echo -e "${BOLD}Cleaning transcripts older than ${older_than} days${NC}"
    echo -e "Directory: $TRANSCRIPTS_DIR"
    [ -n "$current_session" ] && echo -e "Current session (protected): ${CYAN}${current_session}${NC}"
    echo ""

    local deleted_count=0
    local deleted_bytes=0
    local skipped_count=0

    # Find session dirs (UUID directories)
    while IFS= read -r session_dir; do
        [ -d "$session_dir" ] || continue
        local session_id
        session_id=$(basename "$session_dir")

        # Never delete current session
        if [ -n "$current_session" ] && [ "$session_id" = "$current_session" ]; then
            continue
        fi

        # Check age — find returns nothing if dir is newer than threshold
        local is_old=false
        if find "$session_dir" -maxdepth 0 -not -newer "$TRANSCRIPTS_DIR" -mmin "+$(( older_than * 24 * 60 ))" 2>/dev/null | grep -q .; then
            is_old=true
        fi

        # macOS fallback: use stat mtime
        if ! "$is_old"; then
            local mtime_epoch
            mtime_epoch=$(stat -f "%m" "$session_dir" 2>/dev/null || stat -c "%Y" "$session_dir" 2>/dev/null || echo 0)
            local now_epoch
            now_epoch=$(date +%s)
            local age_days=$(( (now_epoch - mtime_epoch) / 86400 ))
            [ "$age_days" -ge "$older_than" ] && is_old=true
        fi

        if ! "$is_old"; then
            skipped_count=$(( skipped_count + 1 ))
            continue
        fi

        local size_bytes
        size_bytes=$(du -sb "$session_dir" 2>/dev/null | awk '{print $1}' || du -sk "$session_dir" | awk '{print $1 * 1024}')

        if "$dry_run"; then
            echo -e "  ${YELLOW}[DRY RUN]${NC} Would delete: $session_id ($(human_size "$size_bytes"))"
        else
            rm -rf "$session_dir"
            ok "  Deleted: $session_id ($(human_size "$size_bytes"))"
        fi

        deleted_count=$(( deleted_count + 1 ))
        deleted_bytes=$(( deleted_bytes + size_bytes ))
    done < <(find "$TRANSCRIPTS_DIR" -maxdepth 1 -mindepth 1 -type d ! -name "memory")

    # Also clean up loose JSONL files for deleted sessions
    # (session-level JSONL files at TRANSCRIPTS_DIR root)
    while IFS= read -r jsonl_file; do
        [ -f "$jsonl_file" ] || continue
        local session_id
        session_id=$(basename "$jsonl_file" .jsonl)

        # Skip if session dir still exists (not old enough)
        [ -d "$TRANSCRIPTS_DIR/$session_id" ] && continue
        # Skip current session
        [ "$session_id" = "$current_session" ] && continue

        local mtime_epoch
        mtime_epoch=$(stat -f "%m" "$jsonl_file" 2>/dev/null || stat -c "%Y" "$jsonl_file" 2>/dev/null || echo 0)
        local now_epoch; now_epoch=$(date +%s)
        local age_days=$(( (now_epoch - mtime_epoch) / 86400 ))
        [ "$age_days" -lt "$older_than" ] && continue

        local size_bytes
        size_bytes=$(du -sb "$jsonl_file" 2>/dev/null | awk '{print $1}' || du -c "$jsonl_file" | tail -1 | awk '{print $1 * 1024}')

        if "$dry_run"; then
            echo -e "  ${YELLOW}[DRY RUN]${NC} Would delete: $(basename "$jsonl_file") ($(human_size "$size_bytes"))"
        else
            rm -f "$jsonl_file"
            ok "  Deleted: $(basename "$jsonl_file") ($(human_size "$size_bytes"))"
        fi

        deleted_count=$(( deleted_count + 1 ))
        deleted_bytes=$(( deleted_bytes + size_bytes ))
    done < <(find "$TRANSCRIPTS_DIR" -maxdepth 1 -name "*.jsonl" -not -name "agent-*")

    echo ""
    if [ "$deleted_count" -eq 0 ]; then
        ok "Nothing to clean — all sessions are within ${older_than} days."
    elif "$dry_run"; then
        warn "DRY RUN complete: would delete $deleted_count item(s), saving $(human_size "$deleted_bytes")"
    else
        ok "Cleaned $deleted_count item(s), freed $(human_size "$deleted_bytes")"
    fi
    echo -e "Sessions kept (recent): ${BOLD}$skipped_count${NC}"
}

# ── Usage ─────────────────────────────────────────────────────────────────────

usage() {
    echo -e "${BOLD}transcripts.sh${NC} — Claude Code transcript retention management"
    echo ""
    echo "Usage:"
    echo "  ./agents/transcripts/transcripts.sh <command> [flags]"
    echo ""
    echo "Commands:"
    echo "  size                         Show current storage usage"
    echo "  clean [--older-than N]       Delete sessions older than N days (default: 30)"
    echo "        [--dry-run]            Preview without deleting"
    echo ""
    echo "Examples:"
    echo "  ./agents/transcripts/transcripts.sh size"
    echo "  ./agents/transcripts/transcripts.sh clean --dry-run"
    echo "  ./agents/transcripts/transcripts.sh clean --older-than 14"
}

# ── Router ────────────────────────────────────────────────────────────────────

COMMAND="${1:-}"
shift || true

case "$COMMAND" in
    size)    cmd_size "$@" ;;
    clean)   cmd_clean "$@" ;;
    help|--help|-h|"") usage ;;
    *) die "Unknown command: $COMMAND. Run with --help for usage." ;;
esac
