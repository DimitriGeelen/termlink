---
id: T-1986
name: "Federation gap .107↔.122 — dm topic posts don't relay since T-1166 cut"
description: >
  Both-direction hub relay gap. dm:9219671e28054458:d1993c2c3ec44c94 has 30 posts on .122 (21 d1993c2c + 8 9219671e) but only 22 posts on .107 (18 d1993c2c + 2 9219671e). 8 of .107's d1993c2c posts to that topic were made via --hub 192.168.10.122:9100 and landed on .122 but did NOT come back via federation to local .107 hub. Conversely, 6 of .122's 9219671e posts (incl. offsets 22, 23) never appeared on .107's local view. Both directions appear broken since the T-1166 legacy-primitive cut (~2026-05-12). Self-documenting evidence in offset 25 of the dm topic, by cohort-agent. T-1985 investigation found, scoped, and dispatched (presence listener restored as immediate user-visible fix). Scope here: (1) inspect hub relay path in crates/termlink-hub for channel.post replication logic; (2) repro by posting a test envelope to .107 and verifying it federates to .122; (3) bisect against T-1166 cut commits; (4) propose fix or document expected behavior if federation was intentionally severed in T-1166. Predecessor: T-1166 (legacy primitive cut). Related: G-060 (channel topic per-hub semantics).

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [fleet, federation, hub-relay, t-1166-followup]
components: []
related_tasks: []
created: 2026-06-04T08:33:05Z
last_update: 2026-06-04T08:36:04Z
date_finished: 2026-06-04T08:53:17Z
---

# T-1986: Federation gap .107↔.122 — dm topic posts don't relay since T-1166 cut

## Context

**Premise disproven** — same shape as T-1665 close. The "federation gap" framing assumed that DM topics SHOULD federate across hubs and that T-1166 broke that. Per PL-176 + `docs/operations/channel-topic-semantics.md` (G-060 mitigation), TermLink has NO inter-hub channel-topic federation primitive. Channel topics are hub-local BY DESIGN, not by regression. The 22-vs-30 post count delta observed in T-1985 between .107 and .122 views of the same DM topic is the documented architecture — the cohort-agent's "federation broken since T-1166 cut" self-description in offset 25 was also a misdiagnosis (it predated PL-176's filing on 2026-05-21). T-1166 didn't break anything here; it just made the absence-of-federation more visible after the legacy primitive `event.broadcast` (which had naive fan-out) was retired.

Closing as premise-disproven. The user-visible problem (cross-host messages not reaching their target) is real, but the structural answer is operator UX + discipline (use `--hub <addr>` or `scripts/chat-arc-broadcast.sh`), not a federation primitive. T-1985 shipped the immediate fix (presence listener on .122 so DMs CAN be sent reliably via `--hub`).

## Acceptance Criteria

### Agent
- [x] Verify PL-176 + channel-topic-semantics.md document the per-hub design (confirmed in T-1985 close investigation)
- [x] Note T-1986 was filed before re-checking the related-knowledge surface (PL-176 was already listed in `fw work-on T-1986` related-knowledge output)
- [x] No code change required; the perceived gap is operator-visibility, addressed by existing tooling (`--hub`, `chat-arc-broadcast.sh`, `/agent-handoff`)

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
grep -q "NO inter-hub channel-topic federation primitive" /opt/termlink/.context/project/learnings.yaml

## RCA

<!-- REQUIRED for bug-class tasks (workflow_type=build with bug-tag, OR title matches
     fix/bug/rca/broken/crash/error/regression/fail/hotfix).
     Non-bug-class tasks may leave this section empty or remove it.

     For bug-class, fill in:
       **Symptom:** what was observed (the user-facing manifestation).
       **Root cause:** the specific structural/logical gap — not "the code was wrong".
       **Why structurally allowed:** what in the framework/code/tooling let this go undetected.
       **Prevention:** what catches the next instance (test/lint/gate/doc/learning) — distinct from the fix itself.

     The completion gate (T-1550, G-019) blocks --status work-completed when
     bug-class AND this section is empty/template-only. Use --skip-rca to bypass (logged).
-->

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

### 2026-06-04T08:33:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1986-federation-gap-107122--dm-topic-posts-do.md
- **Context:** Initial task creation

### 2026-06-04T08:36:04Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
