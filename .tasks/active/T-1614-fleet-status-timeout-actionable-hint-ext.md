---
id: T-1614
name: "fleet-status TIMEOUT actionable hint (extend T-1613)"
description: >
  fleet-status TIMEOUT actionable hint (extend T-1613)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-06T10:47:39Z
last_update: 2026-05-06T10:47:39Z
date_finished: null
---

# T-1614: fleet-status TIMEOUT actionable hint (extend T-1613)

## Context

T-1613 added per-class actionable hints for `fleet status` failures, but the
TIMEOUT branch (line 2300 in `crates/termlink-cli/src/commands/remote.rs`) still
emits a generic "check network connectivity to {addr}" — restating the symptom
rather than directing the next operator action. This task classifies the
timeout by the address kind so the hint actually tells the operator which probe
to run first:

- **Loopback** (127.x, localhost) → hub not running locally, `termlink hub start`
- **RFC5737 test ranges** (192.0.2.x, 198.51.100.x, 203.0.113.x) → stale config
  pointing at documentation IPs; remove the profile
- **RFC1918 private** (10/8, 172.16/12, 192.168/16) → route + remote process check;
  `nc -zv` + `ssh systemctl status`
- **Other** (public/routable) → likely firewall/route; `nc -zv` + `ping`

Synthetic dogfood: add a profile pointing at `192.0.2.1` (RFC5737 TEST-NET-1,
guaranteed to time out as it's documented as never routable), run
`fleet status`, observe the new RFC5737-class hint, then remove the profile.

## Acceptance Criteria

### Agent
- [ ] `is_rfc1918()` helper added near `classify_fleet_error` in remote.rs, matches 10/8, 172.16-31/12, 192.168/16
- [ ] TIMEOUT branch (around line 2300 in `cmd_fleet_status`) replaced with 4-way classification (loopback / RFC5737 / RFC1918 / other)
- [ ] Each class produces a distinct, copy-pasteable hint string
- [ ] `cargo test --package termlink-cli is_rfc1918` passes (≥2 unit tests)
- [ ] Synthetic dogfood: add a profile pointing at 192.0.2.1, run `target/release/termlink fleet status`, observe the RFC5737-class hint emitted, capture the output, remove the profile
- [ ] No regression in existing fleet status output for already-DOWN profiles (Connection refused, Secret file not found classes still hit their existing hints)

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

test -x target/release/termlink
grep -aqF "RFC5737 documentation/test range" target/release/termlink
grep -aqF "Localhost timeout" target/release/termlink
grep -aqF "Private-network hub" target/release/termlink
grep -qF "fn is_rfc1918" crates/termlink-cli/src/commands/remote.rs

## Recommendation

**Recommendation:** GO (extends T-1613 ship; same pattern, same operator-fluent value).
**Rationale:** Operator gets actionable next-step instead of restated symptom. Synthetic dogfood (192.0.2.1 RFC5737) gives end-to-end validation without needing real infrastructure breakage.
**Evidence:** T-1613 dogfooded the same shape — Secret-file-not-found stale-test-residue hint surfaced the right command and shipped real value (testhub cleanup).

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

## Updates

### 2026-05-06T10:47:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1614-fleet-status-timeout-actionable-hint-ext.md
- **Context:** Initial task creation
