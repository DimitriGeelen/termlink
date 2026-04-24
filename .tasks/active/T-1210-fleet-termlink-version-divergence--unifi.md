---
id: T-1210
name: "Fleet termlink version divergence — unified install or federated lineages"
description: >
  Inception: Fleet termlink version divergence — unified install or federated lineages

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-24T09:59:44Z
last_update: 2026-04-24T10:00:00Z
date_finished: null
---

# T-1210: Fleet termlink version divergence — unified install or federated lineages

## Problem Statement

Discovered 2026-04-24 while scoping T-1165/T-1168 install bump: the `termlink` binary across the fleet is **not from a single source lineage**. Observed versions:

| Host | Installed `termlink --version` | Has `channel` subcommand |
|------|-------------------------------|--------------------------|
| /opt/termlink (this host) | 0.9.206 | No (source is 0.9.385, not installed) |
| ring20-management (.122) | 0.9.844 | **No** |
| ring20-dashboard (.121) | (no active session — not probed) | unknown |

Version numbers are non-monotonic with respect to the T-1160 channel API: a *numerically higher* 0.9.844 install on .122 lacks `channel` that our 0.9.385 source has. This is either (a) a different branch/fork's build, (b) a different `build.rs` version-stamping scheme, or (c) a detached install from before the channel merge. **Whatever the reason, "just cargo install --path crates/termlink-cli on every peer" will silently break what's already working there** — we'd overwrite features the peer was relying on.

This blocks the T-1165 bridge runtime (needs `termlink channel post`) and T-1168 publisher/subscriber (same dep) across the fleet. Also affects any future cross-project feature that depends on CLI verb availability.

## Assumptions

- A1: At least one peer's `termlink` binary was built from a code base this repo doesn't track (forked or rewritten). Confirm by comparing `termlink --version --verbose` / `strings /usr/local/bin/termlink | grep commit` on each peer.
- A2: `build.rs` version derivation differs per-checkout — local git state drives the stamp, so tag history on the peer's checkout produces a different version from ours.
- A3: A single unified install (all peers from the same source tree) is operationally cheaper than maintaining capability-handshake shims across divergent CLIs.

## Exploration Plan

- **S1 (1h, diagnosis):** On each peer, capture `termlink --version`, `termlink --help` subcommand list, `which termlink`, mtime of the binary, and (if possible) the source path it was built from. Classify each install: (i) same-lineage-older, (ii) same-lineage-newer, (iii) forked, (iv) stranger-lineage.
- **S2 (2h, unified-install feasibility):** Attempt `cargo install --path crates/termlink-cli` from this repo's source onto one peer (pick the lowest-risk one). Verify (a) build succeeds, (b) resulting version has all expected verbs including `channel`, (c) existing peer functionality still works (fleet doctor from that peer still passes against the hub).
- **S3 (2h, federated alternative):** Design a CLI-capability probe: `termlink --capabilities` JSON output listing supported verbs. Callers (bridge scripts, subscriber daemons) gate on capability before invoking. Decide if this is a viable fallback when S2 isn't safe for a given peer.

## Technical Constraints

- Each peer is a live working system — destructive install is Tier 0.
- No central fleet-state registry today; `termlink fleet doctor` only reports hub reachability + auth, not client CLI version.
- Rust toolchain presence on peers is unknown — assume absent unless proven; binary artifact distribution (from release pipeline) is the portable path.
- Framework directive D4 (Portability): whatever we build must not lock peers to this repo's git URL.

## Scope Fence

**IN:** diagnose every reachable peer, decide converge-vs-federate, pick a deployment mechanism (source install vs release-binary download) for the chosen direction, and pilot on one peer.

**OUT:** actually bumping the whole fleet (separate build task after GO); retrofitting version-stamping in this or peer repos; rebuilding the release pipeline.

## Acceptance Criteria

### Agent
- [ ] Problem statement validated
- [ ] Assumptions tested
- [ ] Recommendation written with rationale

### Human
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO (converge) if:**
- S1 shows all peers are same-lineage or forked-but-reconcilable.
- S2 pilot succeeds: cargo install from this repo onto one peer produces a working `termlink channel *` without breaking fleet doctor on that peer.
- Framework-directives D2 (reliability) and D4 (portability) are satisfied by the chosen deployment mechanism (binary download preferred over cargo install).

**GO (federate) if:**
- S1 shows peers are strangers (different genealogy, intentional divergence).
- S3 capability-probe design works end-to-end and costs <1 day to implement in termlink-cli.

**NO-GO if:**
- Convergence would destroy features peers rely on AND federation design is non-trivial (>2 weeks). Revisit later with smaller scope (per-feature capability flags instead of a full probe).

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** GO — preliminary (direction depends on S1 diagnosis)

**Rationale:** The divergence is a real blocker, not a cosmetic one. Any future feature that depends on a CLI verb (T-1165 bridge, T-1168 publisher, any cross-project tool) will silently no-op on peers where that verb is missing, with no single source of truth for "who has what". This is G-011 territory for version drift. The right move is to diagnose first (S1 — 1h, cheap, non-destructive), then choose between convergence and federation based on what S1 shows. Preliminary GO authorizes S1 only; the S2/S3 branch decision comes back for re-review. Skipping this inception means every future cross-fleet feature has to re-discover the divergence independently, wasting sessions.

**Evidence:**
- .122 observed 2026-04-24: `termlink --version` = 0.9.844 (numerically ahead of our 0.9.385) but `termlink channel` is **not recognized**. Version numbers alone don't indicate capability.
- This host observed: installed 0.9.206, source 0.9.385 — we ourselves have an install/source gap.
- Memory entry `reference_ring20_infrastructure` notes 4 renumbers in 5 days for ring20-management — fleet is volatile enough that manual per-peer install coordination is fragile.
- Framework CLAUDE.md §CI/Release Flow already describes a GitHub Actions binary pipeline (onedev → github → release) — if convergence wins, binary download is likely the portable install mechanism rather than cargo install.
- T-1155 bus rollout (T-1162/T-1163/T-1164/T-1165/T-1168) leans heavily on CLI-verb availability being uniform — this inception unblocks that whole wave.

**Human direction (2026-04-24):** captured as follow-up inception after the .122 probe surfaced the divergence. Scope: diagnose + decide direction, not ship the converged fleet.

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

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-24T10:00:00Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
