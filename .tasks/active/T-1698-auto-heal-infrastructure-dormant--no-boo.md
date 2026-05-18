---
id: T-1698
name: "Auto-heal infrastructure dormant — no bootstrap_from declared on any profile (T-1680..T-1689 not wired)"
description: >
  Auto-heal infrastructure dormant — no bootstrap_from declared on any profile (T-1680..T-1689 not wired)

status: started-work
workflow_type: inception
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-18T18:51:09Z
last_update: 2026-05-18T18:52:52Z
date_finished: null
---

# T-1698: Auto-heal infrastructure dormant — no bootstrap_from declared on any profile (T-1680..T-1689 not wired)

## Context

**Finding (2026-05-18 fleet-bootstrap-check sweep on .107):** Every profile in
`~/.termlink/hubs.toml` reports `no-anchor`. `termlink fleet bootstrap-check
--all` verdict: `no-anchor`. The entire auto-heal stack built over T-1666 →
T-1689 (10+ tasks; `--auto-heal`, `--watch --auto-heal`, `--dry-run`, bulk
`fleet reauth --all-drifted`, history/analyze, heal.log audit trail, MCP
parity) is operationally dormant: when a hub rotates, no heal will fire.

**Why it matters.** Auto-heal was built specifically to address PL-021
volatile-runtime_dir + the general rotation-detection-without-response gap.
G-058 (16-day silent OneDev→GitHub mirror failure) was its sibling concern.
The implementation milestone landed but the operational milestone never did
— shipping the gate without wiring the anchors mirrors PL-159 ("config-driven
mechanism shipped without operator declaration step is dormant tooling") and
PL-168 ("canary scripts without trigger are dormant").

**Possible paths (Problem → Recommendation candidates).** Per R2, the
anchor source must NOT depend on the auth being healed:

  1. **ssh: anchors (operator wires SSH keys).** Operator-bound — root SSH
     from .107 to .121/.122/.141 is currently `Permission denied
     (publickey,password)`. Requires SSH-key infrastructure (Ed25519 keys
     to each hub, agent on .107 with key authorization on target hub).
     Mechanical once keys exist.
  2. **file: anchors fed by a periodic warm-cache cron.** A cron job that
     runs `termlink remote exec <hub> '<session>' 'termlink hub
     export-secret'` while the session is healthy and writes the secret to
     `~/.termlink/secrets/<hub>.hex`. The file then serves as
     `bootstrap_from = "file:..."`. Subverts R2 chicken-and-egg via
     **proactive warming during the healthy window** — when a rotation
     breaks the live session, the most-recent cached secret is from
     within the last cron tick.
  3. **New anchor type `remote-exec:<hub>` shipped by termlink itself.**
     Termlink natively supports the warm-cache pattern, removing the
     external cron. Requires source change.
  4. **Documented operational scope-out:** declare auto-heal an
     ops-aware-only feature; the fleet here is small enough to manual-heal.
     Concedes the value of T-1680..T-1689. Honest but wasteful.

## Acceptance Criteria

### Agent
- [x] Sweep `~/.termlink/hubs.toml` profiles, confirm `bootstrap_from` count = 0 and document each profile's reachability (SSH-from-.107 results captured in body) — see `docs/reports/T-1698-auto-heal-dormancy-inception.md` § Reachability matrix
- [x] Evaluate all four paths above against R2 (out-of-band rule) and termlink fleet ops reality (operator-driven .107 + 4 field hubs). Output: shortlist with rationale — Path Analysis section: Path 1 R2-clean (operator-bound); Path 2/3 fail R2 (chicken-and-egg in disguise); Path 4 honest interim
- [x] Produce a Go/No-Go recommendation for ONE path, tagged in `## Recommendation` per inception convention — Path 1 (ssh:) with Path 4 (scope-out) as interim
- [x] Surface the finding to the operator on chat-arc with a copy-pasteable runbook for the chosen path — posted chat-arc:1599, _thread=T-1698

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

# Inception verification: the proposal artifact exists with a Go/No-Go decision and a runbook.
test -f docs/reports/T-1698-auto-heal-dormancy-inception.md
grep -qE "^## (Recommendation|Decision)" docs/reports/T-1698-auto-heal-dormancy-inception.md
grep -qE "^### Reachability matrix" docs/reports/T-1698-auto-heal-dormancy-inception.md

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

### 2026-05-18T18:51:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1698-auto-heal-infrastructure-dormant--no-boo.md
- **Context:** Initial task creation

### 2026-05-18T18:52:52Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
