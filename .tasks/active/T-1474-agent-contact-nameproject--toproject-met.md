---
id: T-1474
name: "agent contact <name>:<project> + to_project metadata"
description: >
  agent contact <name>:<project> + to_project metadata

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T10:41:18Z
last_update: 2026-05-04T10:41:18Z
date_finished: null
---

# T-1474: agent contact <name>:<project> + to_project metadata

## Context

T-1448 inception build-task (b) — operator-visible payoff for the co-resident-agent disambiguation campaign.

T-1472 closed the **sender** side: `channel post` auto-injects `from_project` metadata so receivers can tell co-resident agents apart. T-1473 closed the **render** side: `channel subscribe` shows `(project)` next to sender_id.

This task closes the **addressing** side. Today `agent contact penelope --message ...` resolves `penelope` → identity_fingerprint and posts to `dm:<a>:<b>`. With FP collisions across co-resident sessions, the post lands in the dm topic that BOTH penelope-002 and penelope-050 subscribe to — operator can't direct the message at one specifically.

Extension:
- Accept `<name>:<project>` syntax (e.g. `agent contact penelope:050-email-archive`)
- Parse the `:project` suffix and stamp `to_project=<project>` in the post metadata, mirroring T-1472's `from_project` on the sender side
- The receiver can filter on `to_project == own from_project` (in-app or via subscribe filters) to know whether the message is for them
- No protocol change. No name resolution change. Pure metadata stamp.

Also: surface the `:project` in the CLI doc-comment so it's discoverable via `--help`.

## Acceptance Criteria

### Agent
- [x] `agent contact <target>` accepts a `<name>:<project>` form for the target argument; the colon-separated form is parsed at the CLI boundary into `(name, Some(project))` (parse_contact_target in agent.rs)
- [x] When `:<project>` is present, the resulting `dm:` post carries `metadata.to_project=<project>` — verified end-to-end at offset=7 of `dm:d1993c2c3ec44c94:d1993c2c3ec44c94` (`to_project=test-T-1474`)
- [x] When the bare `<name>` form is used (no colon), no `to_project` metadata is added (back-compat preserved — `parse_contact_target("penelope")` returns `(name, None)` and the metadata-push is gated on `Some(p)`)
- [x] `--target-fp <hex>` path unchanged; doc-comment in cli.rs documents the `name:project` syntax applies only to the positional target
- [x] Multi-colon target rejected with clear error — verified: `Error: target may contain at most one ':' (form is '<name>[:<project>]'), got "name:foo:bar"`
- [x] Empty project rejected — verified: `Error: project qualifier cannot be empty after ':' in "name:"`
- [x] Empty name (`":proj"`) rejected — verified: `Error: target name cannot be empty in ":proj"`
- [x] Pure helpers unit-tested: 7 new tests in `contact_tests` (bare/colon/empty-input/empty-name/empty-project/multi-colon/special-chars) — all pass
- [x] `cargo build -p termlink` succeeds (debug build, finished in 22.38s)
- [x] `cargo test -p termlink --bin termlink contact_tests` passes (8 passed; 0 failed)
- [x] CLI doc-comment for the `Contact` variant updated — verified via `termlink agent contact --help`

### Human
<!-- All ACs above are agent-verifiable; no human action needed. -->

## Verification

cargo build -p termlink
cargo test -p termlink --bin termlink contact_tests

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

### 2026-05-04T10:41:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1474-agent-contact-nameproject--toproject-met.md
- **Context:** Initial task creation
