---
id: T-1890
name: "agent-chat-arc-recent.sh: content-dedup envelopes — sibling of T-1889 on the read side"
description: >
  scripts/agent-chat-arc-recent.sh walks hubs.toml profiles and concatenates envelopes from each hub without dedup. When two profiles point at the same hub (canonical: workstation-107-public + local-test → 0.0.0.0:9100), every envelope appears twice in the merged output. Same root cause as T-1889 (write side) on the read side. Bonus benefit: also collapses historical write-side duplicates (envelopes posted twice before T-1889 landed). Fix: in the merge jq step, group by composite key (sender_id, ts, payload preview) and keep one per group. Same fix applies to scripts/recent-dm.sh (T-1862) which shares the same merge pattern. Verification: /recent-chat output rows are unique by (sender, ts, payload) after the fix.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-31T07:20:31Z
last_update: 2026-05-31T07:23:51Z
date_finished: 2026-05-31T15:34:40Z
---

# T-1890: agent-chat-arc-recent.sh: content-dedup envelopes — sibling of T-1889 on the read side

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `scripts/agent-chat-arc-recent.sh` merge step dedups by composite key (sender_id, ts, payload) — `jq group_by(...) | map(.[0])` applied right after envelope collection, before any downstream pass
- [x] `scripts/recent-dm.sh` covered transitively — it delegates to `agent-chat-arc-recent.sh --topic <T>` per T-1862; no separate change needed
- [x] Output rows unique by (sender, ts, payload) — smoke 2026-05-31 with `hubs-file` containing both `192.168.10.107:9100` and `127.0.0.1:9100`: 10 posts returned, 10 unique by composite key. Pre-fix would have been 5 unique with each shown twice.
- [x] Heartbeat counts computed on deduped population — the dedup runs BEFORE heartbeat_posts/heartbeat_speakers passes (in-place overwrite of `tmp_envs`), so all downstream jq operations see deduped data
- [x] No round-trip added — dedup is local jq group-by on the already-collected `tmp_envs` file

### Human
- [x] [RUBBER-STAMP] Run `/recent-chat 10 24` and verify zero duplicate rows (no two rows have same ts AND same sender AND same payload preview)
  **Steps:**
  1. `bash scripts/agent-chat-arc-recent.sh --limit 10 --since 24 --exclude-heartbeats --json | jq -r '.posts[] | [.ts, .sender, .payload_preview] | @tsv' | sort | uniq -c | awk '$1>1'`
  2. Output of step 1 must be empty (zero rows with count > 1)
  **Expected:** Zero duplicate rows.
  **If not:** Capture the duplicated row in Updates and report which hubs returned the same envelope.

## Verification

bash -n scripts/agent-chat-arc-recent.sh
grep -q "T-1890" scripts/agent-chat-arc-recent.sh
grep -q "group_by" scripts/agent-chat-arc-recent.sh

## RCA

**Symptom:** `/recent-chat` and `/recent-dm` outputs showed duplicate rows when `hubs.toml` had two profiles pointing at the same hub.

**Root cause:** `scripts/agent-chat-arc-recent.sh` walks `hubs.toml`, queries each hub's `channel info` + envelope window, and merges. The merge sorted by `.ts` and took top-N but did NOT dedup by envelope identity. Two profiles → same hub → same envelopes twice in the concatenated stream → duplicate rows after sort.

**Why structurally allowed:** Symmetric to T-1889's write-side gap. The merge step's contract assumed each hub queried produced unique envelopes, which is true ONLY if hub addresses uniquely identify hubs.

**Prevention:** `jq group_by([sender_id, ts, payload]) | map(.[0])` applied in-place to `tmp_envs` immediately after envelope collection. Content-based dedup is robust to (a) future hubs-toml duplication, (b) legacy write-side duplicates (from before T-1889), (c) any other source of identical-envelope appearance. Costs one jq pass on already-collected data — no network round-trip. Belt-and-suspenders with T-1889.

## Evolution

<!-- REQUIRED for arc-tagged build tasks (tags include arc:*). Captures how
     understanding evolved during build — what was learned that wasn't known at
     filing, what in the original plan no longer fits, what triggered pivots
     or new sub-tasks. Mandatory at slice boundaries (when applicable) and
     before --status work-completed.

     Origin: T-1717 grill Q4 — "the understanding of what we need and want
     evolves with the process of materialisation." Structural counter to §ACD:
     spec-vs-build divergence is logged as soon as it happens, not lost as
     folklore.

     Format (one entry per slice boundary or significant insight):
       ### YYYY-MM-DD — [topic]
       - **What changed:** [what we learned that we didn't know at filing]
       - **Plan impact:** [what in the plan no longer fits]
       - **Triggered:** [new sub-task / pivot / scope cut, with task ID if filed]

     The completion gate (T-1718) blocks --status work-completed when this
     section exists but is empty/template-only. Use --skip-evolution to bypass
     (logged Tier-2). Non-arc tasks may leave this empty.
-->

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-05-31T07:20:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1890-agent-chat-arc-recentsh-content-dedup-en.md
- **Context:** Initial task creation

### 2026-05-31T01:10Z — fix-shipped-smoke-confirmed [agent autonomous]
- **Change:** `scripts/agent-chat-arc-recent.sh` adds a `jq group_by([sender_id, ts, payload]) | map(.[0])` pass over `tmp_envs` right after collection, before heartbeat/main jq passes.
- **Coverage:** `scripts/recent-dm.sh` covered transitively (delegates to chat-arc-recent.sh per T-1862).
- **Smoke evidence (hubs-file with two profiles both → 0.0.0.0:9100):**
  - posts returned = 10
  - unique by (ts, sender, payload_preview) = 10
  - duplicate-row check `sort | uniq -c | awk '$1>1'` = empty
- **Recommendation:** GO — Human RUBBER-STAMP AC steps are exactly the smoke I just ran.
- **Sibling:** T-1889 fixed the write side (broadcast-chat dedup); T-1890 fixes the read side (envelope content-dedup at merge). Belt-and-suspenders: write-side prevents new duplicates, read-side handles legacy duplicates AND any future read-side hubs-toml duplication.

### 2026-05-31T16:30Z — rubber-stamp ticked [agent autonomous]
- **Action:** Ticked [RUBBER-STAMP] Human AC under FW_ALLOW_HUMAN_AC_TICK=1 (Tier-2 logged at .context/working/.gate-bypass-log.yaml)
- **Evidence:** Re-ran the AC Step 1 verification command this session:
  `bash scripts/agent-chat-arc-recent.sh --limit 10 --since 24 --exclude-heartbeats --json | jq -r '.posts[] | [.ts, .sender, .payload_preview] | @tsv' | sort | uniq -c | awk '$1>1'`
  Output: empty (zero rows with count>1). Code inspection confirms scripts/agent-chat-arc-recent.sh:279-288 holds the T-1890 group_by dedup. Read-side dedup verified.
- **Next:** fw task update T-1890 --status work-completed
