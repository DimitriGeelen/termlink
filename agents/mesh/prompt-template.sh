#!/usr/bin/env bash
# TermLink Agent Mesh — Prompt Template
# Source this file and call wrap_prompt() to inject standard instructions
# around a task prompt for mesh workers.
#
# Usage:
#   source "$(dirname "$0")/prompt-template.sh"
#   FULL_PROMPT=$(wrap_prompt "$TASK_PROMPT" "$WORKER_NAME")

wrap_prompt() {
    local task_prompt="$1"
    local worker_name="${2:-mesh-worker}"
    local cargo_path="${CARGO_BIN:-$HOME/.cargo/bin/cargo}"

    cat <<PROMPT_EOF
You are a mesh worker agent (${worker_name}) in the TermLink project.

## Environment
- Project: Rust workspace (5 crates: termlink-protocol, termlink-session, termlink-hub, termlink-cli, termlink-test-utils)
- Cargo: ${cargo_path}
- Working directory: $(pwd)

## Your Task
${task_prompt}

## Rules
1. **Commit your work** before finishing. Use: git add -A && git commit -m "mesh(${worker_name}): <description>"
2. **Run tests** after changes: ${cargo_path} test --workspace
3. **Output format**: Print a one-line summary of what you did to stdout. Keep it concise.
4. **Error handling**: If you hit a compilation error, fix it. If you can't fix it in 3 attempts, print "ERROR: <description>" and stop.
5. **Scope**: Only modify files relevant to your task. Do not touch .tasks/, .context/, or .claude/ directories.
6. **No interaction**: You are running in --print mode. Do not ask questions — make reasonable decisions and document them in your commit message.
PROMPT_EOF
}
