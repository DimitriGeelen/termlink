# T-1209: SubagentStop hook design — inception

**Parent task:** T-175 (captured with placeholder ACs).
**Status:** Exploration plan drafted; awaiting user review before spikes.

## Problem Statement

Claude Code's `SubagentStop` hook fires when a Task-tool sub-agent finishes, with `agent_transcript_path` and `last_assistant_message` natively in the payload (no parsing required). The framework currently has `check-dispatch.sh` on **PostToolUse** — advisory, not blocking. The Sub-Agent Dispatch Protocol in CLAUDE.md says "content generators MUST write to disk, NOT return full content" and "investigators return structured summaries under 2K tokens". No structural enforcement today — agents routinely return large raw data blobs that waste orchestrator context.

This inception closes **G-015** (sub-agent results bypass governance), acknowledged in `docs/claude-code-settings.md §Rec #8` as "Explore in future".

## Framework-existing position to override

Rec #8 says "Verdict: Explore in future. Could solve G-015". User has now directed inception-and-build.

## Constraints

- **SubagentStop fires after the Task call completes** — the sub-agent's message has already reached the orchestrator by then. Hook cannot prevent a large return from being consumed; it can only warn/log for next time.
- Wait — re-read: if the hook exits non-zero, does Claude Code surface the error to the orchestrator BEFORE ingesting the sub-agent output? **Unknown; must test in S1.** If yes, this is a hard blocking gate. If no, it's advisory-plus-loud-warning.
- **Must allow legitimate large returns** — investigators producing a 5K-token structured report are fine. The target is the "returned 25KB of raw file content" footgun.
- **Threshold design matters.** Too tight blocks legitimate work; too loose lets the footgun through.

## Assumptions (to register)

- A1: The `last_assistant_message` size in the payload is a reliable proxy for the bytes returned to the orchestrator (within ~5%).
- A2: 80% of sub-agent dispatches fit comfortably under 2K tokens when instructed per the Dispatch Protocol; the long tail (>10K) is overwhelmingly the footgun case, not legitimate.
- A3: Blocking non-zero exit on SubagentStop causes the orchestrator to see an error message INSTEAD of the sub-agent output. **If A3 is false, this entire task reduces to advisory-plus-logging.**

## Exploration Plan

**Spike S1 — exit-code semantics (1h, highest priority, gates everything else):** Wire a SubagentStop handler that unconditionally exits non-zero with a known message. Dispatch a trivial Task agent. Observe what the orchestrator receives — the sub-agent output, the hook error, or both? This single test determines whether enforcement is blocking or advisory.

**Spike S2 — size-distribution survey (2h):** Passive SubagentStop that logs `last_assistant_message` length for every dispatch over 1 week. Build a histogram. Pick threshold T such that 95% of legitimate dispatches are under T and 95% of footgun cases are over T.

**Spike S3 — bus-posted result migration (2h):** When over-threshold output is detected, SubagentStop auto-posts it to `fw bus` (result ledger) and rewrites the sub-agent's return to point at the bus entry. Net effect: orchestrator gets `R-NNN @ path+summary` instead of raw content. Requires the hook to have Write access to bus storage. Test with one synthetic large-return dispatch.

## Decision Framework (GO / NO-GO criteria)

**GO (blocking mode) if:** S1 confirms non-zero exit replaces sub-agent output AND S2 finds a clean threshold (>10× footgun/legitimate ratio).
**GO (advisory+migration mode) if:** S1 shows exit code doesn't block ingestion BUT S3 works (auto-migrate to bus). Less satisfying but still closes G-015.
**NO-GO if:** S1 fails AND S3 is infeasible — fall back to keeping check-dispatch.sh as advisory and invest in agent prompt quality instead.

## Implementation destination

Framework-side `agents/context/subagent-stop.sh`, wired through `fw hook subagent-stop`. Retire `check-dispatch.sh` when SubagentStop mode is GO.

## Dialogue Log

### 2026-04-24 — inception kickoff
- **Agent:** Drafted exploration plan with S1 (exit-code semantics) as the gate for all other work — if non-zero exit doesn't block ingestion, design shape changes fundamentally. Noted that this inception also closes G-015 structurally. Pending user direction: is "advisory + auto-bus-migration" (GO mode B) acceptable, or is this inception only valid if S1 proves hard-blocking is possible?
- **Human:** (awaiting input)
