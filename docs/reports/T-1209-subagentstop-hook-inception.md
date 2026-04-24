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

## Revised goal (per human direction 2026-04-24)

**Goal restated: do not lose information.** Hard-blocking actually LOSES information (orchestrator never sees the sub-agent output if hook exits non-zero). The right pattern is **Mode B — advisory + auto-migrate to fw bus**. Information is preserved on disk; orchestrator gets a pointer; nothing is dropped.

**What's needed for "no information loss":**

1. **Capture before truncation.** SubagentStop receives `agent_transcript_path` natively — full transcript is on disk before the orchestrator ingests the response. Hook reads from disk, not from a streaming buffer. ✓
2. **Persistent storage path.** `fw bus` already does this — typed YAML envelopes, blobs ≥2KB auto-moved to `.context/bus/blobs/`. Already designed for sub-agent results (per CLAUDE.md). ✓
3. **Pointer in the orchestrator's view.** Hook rewrites the agent's return such that the orchestrator sees `R-NNN: 1-line summary @ /path/to/blob` instead of 25KB of raw content. Question: **can SubagentStop modify the message Claude Code passes to the orchestrator, or is the message already locked in by hook-fire time?** This is the key unknown — must spike (S1' below).
4. **Discoverability.** The orchestrator needs to know the full content exists and how to read it. Standard `fw bus read T-XXX R-NNN` pattern. Already implemented.
5. **Threshold tuning.** Don't migrate everything — only over-threshold returns. Otherwise legitimate small returns get extra indirection. S2 (size survey) decides the threshold.

**Revised spike plan:**

- **S1' — message-mutation semantics (1h, gates everything):** can SubagentStop rewrite the orchestrator-visible response, or is it post-hoc only? Test: emit a stderr message + post-process the transcript file; observe what the orchestrator sees. **If mutation is impossible**, the bus-migration becomes "agent gets nudged in stderr to read the bus next turn" — still preserves information, just adds one round-trip.
- **S2 — size-distribution survey (2h, unchanged):** histogram of `last_assistant_message` bytes across 1 week. Pick threshold T from data.
- **S3 — bus-migration handler (2h):** when over-threshold detected, post the full transcript-derived summary to `fw bus`, write the agent-visible nudge. Test with one synthetic large-return dispatch.

## Dialogue Log

### 2026-04-24 — inception kickoff + direction received
- **Agent:** Drafted plan with S1 (exit-code semantics test) as the gate. Asked whether Mode B (advisory + auto-migrate) is acceptable.
- **Human:** We want not to lose information. What's needed for that?
- **Agent:** Mode B is the answer — hard-blocking would actually lose information. Re-stated goal as "no information loss" and enumerated the 5 prerequisites (capture, storage, pointer, discoverability, threshold). 4 of 5 are already in place via `fw bus`. The one unknown is whether SubagentStop can mutate the orchestrator-visible message — that's S1'. Pending user direction on the threshold-tuning approach (1-week passive observation before migration kicks in, OR migrate-from-day-1 with a conservative initial threshold like 8KB).
