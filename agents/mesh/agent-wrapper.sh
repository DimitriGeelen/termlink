#!/usr/bin/env bash
# Agent wrapper for TermLink agent mesh PoC
# Runs Claude Code in a clean environment (no nested session detection)
# Usage: agent-wrapper.sh "prompt text"

set -euo pipefail

unset CLAUDECODE

PROMPT="${1:?Usage: agent-wrapper.sh \"prompt text\"}"
WORKDIR="${2:-/tmp}"

cd "$WORKDIR"
exec claude --print --no-session-persistence "$PROMPT"
