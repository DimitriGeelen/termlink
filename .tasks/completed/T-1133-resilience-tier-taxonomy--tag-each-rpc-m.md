---
id: T-1133
name: "Resilience-tier taxonomy — tag each RPC method Tier-A (opaque) or Tier-B (typed) in protocol doc comments (from T-1071 GO)"
description: >
  From T-1071 inception GO. Tag every RPC method as Tier-A (opaque payload, drift-tolerant — event.broadcast, event.emit, kv.set strings) or Tier-B (typed struct, drift-fragile — command.inject, command.exec, session.update). Document in crates/termlink-protocol/src/control.rs as doc comments on each method constant. fleet doctor can then flag fleets where Tier-B methods would fail across the observed version diversity (extends T-1132). This is the 'codify the event.broadcast resilience property' deliverable — promotes a happy accident into a documented design tier.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [termlink, protocol, taxonomy, documentation, T-1071]
components: [crates/termlink-protocol/src/control.rs]
related_tasks: []
created: 2026-04-18T23:00:36Z
last_update: 2026-04-23T19:13:57Z
date_finished: 2026-04-19T14:00:56Z
---

# T-1133: Resilience-tier taxonomy — tag each RPC method Tier-A (opaque) or Tier-B (typed) in protocol doc comments (from T-1071 GO)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `control.rs` has a module-level doc block defining Tier-A (opaque/drift-tolerant) and Tier-B (typed/drift-fragile)
- [x] Every method constant in the `method` module carries a `/// Tier-A` or `/// Tier-B` doc comment (31 tagged)
- [x] Tier-A set includes at minimum: `event.emit`, `event.broadcast`, `event.emit_to`, `event.state_change`, `kv.set`, `kv.get`
- [x] Tier-B set includes at minimum: `command.execute`, `command.inject`, `session.update`, `query.status`
- [x] `cargo check -p termlink-protocol` succeeds (doc comments don't break compilation)

### Human
- [x] [RUBBER-STAMP] Sanity-check the Tier-A/Tier-B split against your reading of the methods — ticked by user direction 2026-04-23. Evidence: User direction 2026-04-23 — taxonomy split accepted.
  **Steps:** `rg "Tier-[AB]" crates/termlink-protocol/src/control.rs`
  **Expected:** every method in the `method` module is labeled; labels match your intuition
  **If not:** note which method looks misclassified and why


**Agent evidence (auto-batch 2026-04-19, G-008 remediation, code-grep, tier-taxonomy-doc):** Code: `crates/termlink-protocol/src/control.rs` lines 7-25 carry the taxonomy doc comment: "Every method is tagged Tier-A or Tier-B to make protocol-skew...", explains Tier-A (opaque/drift-tolerant) vs Tier-B (typed/drift-fragile), and references T-1132 + T-1071. Tags are checked in and authoritative. RUBBER-STAMPable.

## Verification

grep -q "Tier-A" /opt/termlink/crates/termlink-protocol/src/control.rs
grep -q "Tier-B" /opt/termlink/crates/termlink-protocol/src/control.rs
bash -c 'cd /opt/termlink && cargo check -p termlink-protocol 2>&1 | tail -5 | grep -qE "Finished|Compiling"'

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

### 2026-04-18T23:00:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1133-resilience-tier-taxonomy--tag-each-rpc-m.md
- **Context:** Initial task creation

### 2026-04-19T13:59:14Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

### 2026-04-19T14:00:56Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
