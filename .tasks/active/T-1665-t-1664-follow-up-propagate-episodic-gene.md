---
id: T-1665
name: "T-1664 follow-up: propagate episodic-generator escape-order fix to upstream /opt/999-AEF"
description: >
  T-1664 fixed the consumer copy. Dispatch from this host's container fails with cd /opt/999-AEF (path inaccessible). Operator must propagate from a host that has /opt/999-AEF mounted, or run termlink dispatch from outside the boundary. Without this fix landing upstream, consumer fw upgrade will silently re-introduce the bug. Three sites (lines 288/304/361 in agents/context/lib/episodic.sh) need 'sed s/\\\\/\\\\\\\\/g' prepended before the existing quote-escape.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-17T18:54:38Z
last_update: 2026-05-26T16:02:58Z
date_finished: null
---

# T-1665: T-1664 follow-up: propagate episodic-generator escape-order fix to upstream /opt/999-AEF

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
**Revised 2026-05-26 after upstream-state check — original premise was wrong.**
The task as filed assumed upstream lacks the fix and needs the same `sed 's/\\/\\\\/g'` prepend that T-1664 applied to the consumer. **Upstream check disproved this.**
Upstream `/opt/999-AEF/agents/context/lib/episodic.sh` does NOT use the
double-quoted-with-backslash-escape approach at all — at the three sites it
uses **single-quoted YAML scalars** (`sed "s/'/''/g"`) which are naturally
safe against the backslash-in-quoted-string bug T-1664 was patching. The
consumer copy has structurally diverged from upstream (different quoting
strategy, not just a missing fix). Real ACs depend on the divergence
decision (human-owner judgment, see Updates 2026-05-26):

- [ ] **Owner decides direction** between (A) re-align consumer with
      upstream's single-quote approach (sweep the whole `generate_episodic`
      function — not just the 3 documented sites — because emit/escape
      pairs come together), or (B) re-apply the T-1664 backslash-prepend
      fix to consumer ONLY and leave upstream's single-quote approach alone
      (accepts permanent divergence), or (C) port the T-1664 backslash
      escape to upstream too (defensive belt-and-braces — single quote
      already covers it, but extra hardening).
- [ ] Once direction chosen, the implementing change passes `bash -n` and
      a smoke run of episodic generation against a backslash-bearing input
      yields YAML that `python3 -c 'import yaml; yaml.safe_load(open("..."))'`
      accepts without raising.
- [ ] If direction (A) or (C): commit landed on `/opt/999-AEF` OneDev
      `origin/master` referencing T-1665. If direction (A) or (B): commit
      landed on `/opt/termlink` referencing T-1665. Push to OneDev.

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

### 2026-05-17T18:54:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1665-t-1664-follow-up-propagate-episodic-gene.md
- **Context:** Initial task creation

### 2026-05-26 — premise check: upstream does not need this fix [agent]

While picking T-1665 up as a small in-initiative unit, did the propagation
preflight (read upstream + consumer at lines 288/304/361). Found a
structural divergence the original task description didn't anticipate:

- **Consumer** `/opt/termlink/.agentic-framework/agents/context/lib/episodic.sh`
  at lines 288, 304, 322, 325, 328, 331, 361: emits YAML using
  **double-quoted** scalars (`echo "  - \"$text\""`) and escapes with
  `sed 's/"/\\"/g'`. This is the form T-1664 patched (prepend a backslash-
  escape `sed 's/\\/\\\\/g'` so a literal `\` in input doesn't break the
  double-quoted scalar parse).
- **Upstream** `/opt/999-Agentic-Engineering-Framework/agents/context/lib/episodic.sh`
  at the corresponding sites: uses **single-quoted** scalars and escapes
  with `sed "s/'/''/g"` (YAML single-quote doubling). Single-quoted YAML
  scalars are literal — no backslash escaping needed, no backslash bug.

**Implication:** T-1665 was filed on the premise that upstream needs the
same `sed 's/\\/\\\\/g'` prepend the consumer got. That premise is
disproven by today's read. The actual situation is divergence: consumer
uses a different YAML quoting strategy than upstream (probably stale —
upstream rewrote to single-quoted for cleaner escape semantics; consumer
never picked up the rewrite, and T-1638 force-downgrade left it on the
old version).

**Not patched this session.** The right next step is owner judgment among
direction (A) re-align consumer with upstream, (B) keep consumer on
double-quote and re-apply T-1664's prepend, (C) port T-1664 to upstream
defensively. ACs revised to gate on that choice. No upstream patch yet;
no consumer patch yet.

### 2026-05-26T16:02:58Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
