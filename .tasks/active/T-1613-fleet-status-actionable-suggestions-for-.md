---
id: T-1613
name: "fleet status: actionable suggestions for misconfigured profiles (Secret-file-not-found, /tmp residue)"
description: >
  fleet status: actionable suggestions for misconfigured profiles (Secret-file-not-found, /tmp residue)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-06T09:09:26Z
last_update: 2026-05-06T09:09:26Z
date_finished: null
---

# T-1613: fleet status: actionable suggestions for misconfigured profiles (Secret-file-not-found, /tmp residue)

## Context

`termlink fleet status` shows DOWN hubs with raw error text but the ACTIONS NEEDED block doesn't always surface a specific remediation. Real example today (2026-05-06):

```
DOWN  testhub  1.2.3.4:9100  Secret file not found: /tmp/tmp.Q4Vqbq4g6u/.termlink/secrets/test.hex: ...
ACTIONS NEEDED:
  1. testhub: Secret file not found: /tmp/tmp.Q4Vqbq4g6u/.termlink/secrets/test.hex: ...
```

The action is just the raw error. The operator has to know that `/tmp/tmp.*` paths come from ephemeral cargo TempDir test fixtures and that stale test profiles can be cleanly removed via `termlink remote profile remove <name>`. Neither is obvious.

This task adds two failure-mode classifications in `cmd_fleet_status`'s action-string assembly:

1. **Stale-test-residue:** `Secret file not found` + path matches `/tmp/tmp.*` (cargo TempDir convention) → suggest `termlink remote profile remove <name>`.
2. **Secret-file-genuinely-missing:** `Secret file not found` but path is non-tempfile → suggest profile inspection + reauth with `--bootstrap-from`.

Pure additive change in `crates/termlink-cli/src/commands/remote.rs::cmd_fleet_status` action-string assembly. Touches no protocol, no hub-side code, no MCP surface.

## Acceptance Criteria

### Agent
- [x] `cmd_fleet_status` action-string classifies "Secret file not found" failures into stale-test-residue vs genuinely-missing
- [x] Stale-test-residue (path matches `/tmp/tmp.*` prefix) suggests `termlink remote profile remove <name>` plus a brief reason
- [x] Genuinely-missing path (non-tempfile) suggests profile inspection + reauth incantation
- [x] No regression for AUTH-FAIL, Connection-refused/Cannot-connect, or Timeout classifications (existing branches untouched)
- [x] `cargo build --release` clean — 5m04s, 1 pre-existing unused-assignment warning unrelated
- [x] `target/release/termlink fleet status` for testhub showed: "Stale test-fixture profile (secret_file under /tmp/tmp.* — cargo TempDir residue). Remove with: termlink remote profile remove testhub" — dogfooded, command worked, testhub gone, fleet now clean (5 up / 0 down)

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
grep -aqF "Stale test-fixture profile" target/release/termlink
grep -aqF "cargo TempDir residue" target/release/termlink
grep -qF "Stale test-fixture profile" crates/termlink-cli/src/commands/remote.rs

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

## Recommendation

**Recommendation:** GO
**Rationale:** Operator-facing CLI ergonomics improvement, pure additive. Real pain observed today: testhub fleet-status entry has zero actionable hint. Compresses diagnose-and-fix into one-glance suggestion. ~30 LOC, no protocol/hub/MCP surface change.
**Evidence:** see Updates after build + dogfood.

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

### 2026-05-06T09:09:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1613-fleet-status-actionable-suggestions-for-.md
- **Context:** Initial task creation
