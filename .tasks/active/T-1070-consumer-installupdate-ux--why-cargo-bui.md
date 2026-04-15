---
id: T-1070
name: "Consumer install/update UX — why cargo build when we already ship binaries"
description: >
  Inception: Consumer install/update UX — why cargo build when we already ship binaries

status: captured
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-15T21:15:31Z
last_update: 2026-04-15T21:15:31Z
date_finished: null
---

# T-1070: Consumer install/update UX — why cargo build when we already ship binaries

## Problem Statement

**Consumer updates repeatedly fail at `cargo build` / `cargo install --git` even though the release pipeline already ships pre-built binaries for 5 targets (macOS aarch64/x86_64, Linux gnu/musl x86_64, Linux aarch64) to GitHub Releases.**

**Observed pain (cross-session):**
- This-session parallel agent: "Binary install blocked — no cargo on this host."
- PL-021, T-1027, T-1037 work: operator has to ferry binaries manually (scp / send-file) because there's no "download the latest" one-liner for LXC containers without a toolchain.
- T-1016 (termlink deploy) addresses *hub-to-hub* binary distribution, but assumes a working termlink already on the target. Chicken-and-egg for fresh hosts.

**For whom:**
- Operators spinning up a new container / VM / laptop and needing `termlink` quickly.
- Consumer-project agents whose host has no rust toolchain (minimal LXC, Alpine, air-gapped).
- Anyone installing from README today — README currently points to `brew install termlink` (macOS-centric) or `cargo install --git` (requires toolchain).

**Why now:**
- T-1019 shipped musl static builds — the artifact quality is there.
- `.github/workflows/release.yml` already runs on every `v*` tag with a 5-target matrix.
- Gap is between "we build it" and "consumer runs it" — likely small, tooling-shaped.

## Assumptions

- **A1:** Release binaries are actually being published on each `v*` tag (not silently failing).
- **A2:** The 5-target matrix covers the consumer platforms we actually care about (ring20 LXCs, dev boxes, operator laptops).
- **A3:** There is no `install.sh` curl-pipe bootstrap today. README only offers `brew install` or `cargo install --git` (requires toolchain).
- **A4:** Homebrew formula auto-updates on new tags — verify the tap is green.
- **A5:** The cross-session failure mode is discoverability + bootstrap, not missing artifacts. Consumers *could* download the right binary but don't know how.

## Exploration Plan

1. **[15 min]** Verify last 5 tags each produced a full artifact matrix on GitHub Releases.
2. **[15 min]** Inventory the consumer platforms actually hitting this (this dev box + ring20 LXCs + where the parallel session ran). Map each to the correct release artifact.
3. **[15 min]** Audit existing install helpers: `scripts/deploy-remote.sh`, `install-check.yml`, README, any `termlink self-update` CLI surface.
4. **[30 min]** Sketch the one-liner install story. Candidates (pick ≤2 to recommend):
   a. `curl -fsSL termlink.sh/install | sh` — detects target triple, downloads + verifies, installs to `/usr/local/bin`.
   b. `termlink self-update` subcommand (for hosts already running termlink).
   c. Homebrew tap refresh cadence.
   d. OCI image `ghcr.io/…/termlink:latest` for container-native consumers.
5. **[15 min]** Write recommendation with concrete next-step task scope.

Total time-box: **90 minutes**. No code until GO.

## Technical Constraints

- Must work without cargo / rustc on the target host.
- Must handle LXC containers with minimal glibc (prefer musl static).
- Must verify checksum / signature (no blind curl | sh without integrity check).
- Must be idempotent (re-running doesn't break an existing install).
- Must not require changes to OneDev → GitHub mirror pipeline (that's a separate concern, see G-007).
- Target triple auto-detection has to handle: Linux x86_64 gnu, Linux x86_64 musl, Linux aarch64, macOS x86_64, macOS aarch64.

## Scope Fence

**IN scope:**
- Define the canonical "fresh install" flow for a consumer (no-cargo host).
- Define the canonical "update to latest" flow for a consumer with an old termlink.
- Identify 1–2 minimal interventions (install.sh, self-update, tap fix) that close the gap.
- Recommend which intervention(s) to build, with concrete follow-up task scope.

**OUT of scope:**
- Changes to `.github/workflows/release.yml` itself (already ships binaries — that's fine).
- Windows support (not a consumer platform yet).
- Building binaries locally (that's T-1019 territory).
- Fleet / hub-to-hub deployment (that's T-1016).
- OneDev → GitHub mirror reliability (that's G-007).
- Framework vendored copy distribution (that's `fw upgrade` / T-984 territory).

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

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- Root cause identified with bounded fix path
- Fix is scoped, testable, and reversible

**NO-GO if:**
- Problem requires fundamental redesign or unbounded scope
- Fix cost exceeds benefit given current evidence

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

<!-- REQUIRED before fw inception decide. Write your recommendation here (T-974).
     Watchtower reads this section — if it's empty, the human sees nothing.
     Format:
     **Recommendation:** GO / NO-GO / DEFER
     **Rationale:** Why (cite evidence from exploration)
     **Evidence:**
     - Finding 1
     - Finding 2
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

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->
