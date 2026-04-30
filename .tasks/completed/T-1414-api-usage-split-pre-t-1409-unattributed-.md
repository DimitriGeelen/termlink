---
id: T-1414
name: "api-usage: split pre-T-1409 unattributed from attributable legacy traffic"
description: >
  api-usage: split pre-T-1409 unattributed from attributable legacy traffic

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-30T06:59:11Z
last_update: 2026-04-30T07:03:03Z
date_finished: 2026-04-30T07:03:03Z
---

# T-1414: api-usage: split pre-T-1409 unattributed from attributable legacy traffic

## Context

The T-1166 bake gate looks worse than reality because the rolling 7d/30d/60d windows
include rpc-audit lines emitted BEFORE T-1409 deployed peer_addr capture
(boundary: ts ≈ 1777499350000 / 2026-04-29 21:49 UTC on .122). Pre-T-1409 lines
have neither `peer_pid` nor `peer_addr` for TCP callers, so they appear as
"unattributable legacy traffic" — masking the fact that 100% of post-T-1409
legacy is from a single IP (192.168.10.143).

Concrete numbers (sampled 2026-04-30T07:00 UTC from .122):
- 7d window: 6.21% legacy (FAIL), of which only 547/3958 attributable
- post-T-1409 only: 7.6% legacy (still FAIL), but 100% attributable to .143

This patches the api-usage agent so the operator sees:
1. Total legacy split into "attributable" vs "pre-T-1409 unattributable"
2. Bake gate decision based on attributable traffic + clear note about unattributable backlog
3. Holdout list is unambiguous (one IP, not "(unknown) 3210")

## Acceptance Criteria

### Agent
- [x] `api-usage.sh --json` output includes `legacy_attributable` and `legacy_unattributable_pre_t1409` integer counts
- [x] `legacy_attributable + legacy_unattributable_pre_t1409 == legacy` (math holds — verified: 3964 = 563 + 3401)
- [x] Human-readable mode shows both numbers under the legacy line, with note about pre-T-1409 backlog
- [x] Existing fields (`legacy`, `legacy_pct`, `legacy_callers_by_ip`, gate logic) unchanged — additive only
- [x] Mirrored upstream to `/opt/999-Agentic-Engineering-Framework/agents/metrics/api-usage.sh`

## Verification

# Run agent --json (its exit reflects the gate; we only care about JSON shape).
# Use ||true so a FAIL gate doesn't fail the verification pipeline.
bash -c '.agentic-framework/agents/metrics/api-usage.sh --last-Nd 7 --json 2>/dev/null || true' | python3 -c "import json,sys;d=json.load(sys.stdin);assert 'legacy_attributable' in d, 'missing legacy_attributable';assert 'legacy_unattributable_pre_t1409' in d, 'missing legacy_unattributable_pre_t1409';t=d['legacy_attributable']+d['legacy_unattributable_pre_t1409'];assert t==d['legacy'], f'split mismatch {t} vs {d[chr(34)+chr(108)+chr(101)+chr(103)+chr(97)+chr(99)+chr(121)+chr(34)]}';print('OK',d['legacy'],'=',d['legacy_attributable'],'+',d['legacy_unattributable_pre_t1409'])"
# Vendored copy matches upstream
diff -q /opt/termlink/.agentic-framework/agents/metrics/api-usage.sh /opt/999-Agentic-Engineering-Framework/agents/metrics/api-usage.sh

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-04-30T06:59:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1414-api-usage-split-pre-t-1409-unattributed-.md
- **Context:** Initial task creation

### 2026-04-30T07:03:03Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
