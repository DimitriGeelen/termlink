#!/usr/bin/env bash
# Apply the recommended resolution in the SCRATCH worktree only, so the result
# can be built and tested before the operator commits to it. Never touches main
# or the real branch.
#
#   take-main  : main is newer or authoritative
#   take-ours  : ours is richer and main's is a stub / older
#   union      : both sides added distinct content; neither may be dropped
set -uo pipefail
T=/root/.claude/jobs/d638a35c/tmp/merge-trial
cd "$T" || exit 2

take_main() { for p in "$@"; do git checkout --theirs -- "$p" 2>/dev/null && git add -- "$p"; done; }
take_ours() { for p in "$@"; do git checkout --ours   -- "$p" 2>/dev/null && git add -- "$p"; done; }

# --- take MAIN: vendored framework (main tracks NEWER copies than the ones we
#     recovered off the main checkout's disk — ours are older, not additive) ---
take_main \
  .agentic-framework/agents/context/revisit-due-scan.sh \
  .agentic-framework/lib/branch-hygiene.sh \
  .agentic-framework/lib/hook_paths.py \
  .agentic-framework/lib/ollama_thin_loop.py \
  .agentic-framework/lib/verification-port.sh \
  .agentic-framework/lib/version-relation.sh \
  .agentic-framework/lib/worktree.sh \
  .agentic-framework/web/blueprints/bvp.py \
  .agentic-framework/web/templates/bvp.html

# --- take MAIN: per-session scratch state (counters, timestamps, transient) ---
take_main \
  .context/working/.budget-status .context/working/.compact-log \
  .context/working/.gate-bypass-log.yaml .context/working/.hook-counter \
  .context/working/.loop-detect.json .context/working/.pre-compact.last-run \
  .context/working/.session-metrics.yaml .context/working/.session-start-ts \
  .context/working/.tool-counter .context/working/focus.yaml \
  .context/working/session.yaml \
  .context/handovers/LATEST.md .termlink-task VERSION

# --- take MAIN: the cron-drift checker. Main's T-2682 added UNINSTALLED_JOBS as
#     its own firing class and demoted comment-churn DRIFT to --strict. That is a
#     more precise version of what our T-2821 was reaching for. FLAGGED for the
#     operator — this discards our allowlist machinery. ---
take_main scripts/check-cron-install-drift.sh

# --- take OURS: episodics carry the hand-written summaries (T-2804); main's are
#     the generator's blank-summary output ---
take_ours \
  .context/episodic/T-2025.yaml .context/episodic/T-2229.yaml \
  .context/episodic/T-2303.yaml .context/episodic/T-2677.yaml

# --- take OURS: fabric cards. Main's are auto-generated stubs
#     ("TODO: describe what this component does", type: script, tags: []) ---
take_ours \
  .fabric/components/crates-termlink-hub-src-retention_sweeper.yaml \
  .fabric/components/crates-termlink-session-src-fleet_presence.yaml \
  .fabric/components/crates-termlink-session-src-identity_dir.yaml \
  .fabric/components/crates-termlink-session-src-ws_consumer.yaml \
  .tasks/active/T-1452-revisit-due-scansh-cron--handover-banner.md

echo "=== remaining unresolved (expect: CLAUDE.md + 3 project registers) ==="
git diff --name-only --diff-filter=U
