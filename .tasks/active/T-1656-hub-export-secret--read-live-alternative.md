---
id: T-1656
name: "hub export-secret — read-live alternative to cat ~/.termlink/secrets/<IP>.hex (G-011 R3 facet 2)"
description: >
  hub export-secret — read-live alternative to cat ~/.termlink/secrets/<IP>.hex (G-011 R3 facet 2)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-17T13:55:46Z
last_update: 2026-05-17T13:55:46Z
date_finished: null
---

# T-1656: hub export-secret — read-live alternative to cat ~/.termlink/secrets/<IP>.hex (G-011 R3 facet 2)

## Context

G-011 R3 facet 2: "When sharing your local hub's secret with a peer (heal-after-rotation handoff), read from the live `<runtime_dir>/hub.secret`, NOT the IP-keyed cache `~/.termlink/secrets/<hub-ip>.hex`. The cache is written once at heal time and is NOT invalidated when the hub regenerates."

Facet 1 (profile config — point `secret_file = ...` at live) is covered by T-1284 `audit_hubs_for_self_hub_cache`. Facet 2 (manual peer handoff — `cat ... | ssh peer`) has no tooling — operators have to remember the live path, which the audit memos out as "use `<runtime_dir>/hub.secret`" but doesn't make trivial.

A `termlink hub export-secret` command always reads the live file, so the operator's habit becomes:
```
termlink hub export-secret | ssh peer 'cat > ~/.termlink/secrets/<this-host>.hex && chmod 600 ~/.termlink/secrets/<this-host>.hex'
```
instead of guessing whether to `cat /var/lib/termlink/hub.secret` or `cat ~/.termlink/secrets/<own-IP>.hex` (the latter is the footgun).

Reuses `termlink_hub::server::hub_secret_path()` which already resolves runtime_dir correctly.

## Acceptance Criteria

### Agent
- [x] `HubAction::ExportSecret` variant added to `cli.rs` enum + dispatched in main.rs
- [x] `cmd_hub_export_secret(out, json)` reads from `hub_secret_path()` (live runtime_dir/hub.secret), prints hex to stdout (live-smoke: exported hex sha256 = live-file sha256)
- [x] `--out <path>` flag: write hex to file with chmod 600 (live-smoke: 64-byte file at mode 0o600, diff vs live = identical)
- [x] `--json` flag: prints `{"path":"<live-path>","hex":"<value>","bytes":<len>}` for scripting (live-smoke: `{"bytes":32,"hex":"b307...","path":"/var/lib/termlink/hub.secret"}`)
- [x] Error path: when live secret file missing → exit non-zero with message "no hub.secret at <path> — is the hub running?" (live-smoke with TERMLINK_RUNTIME_DIR=/tmp/no-hub-here-xyz: exit=1, message exact)
- [x] Unit test: `export_secret_reads_live_not_cache` — stages stale cache (bbbb...) + different live secret (aaaa...) under env-lock; verifies --out captured content == LIVE
- [x] Unit test: `export_secret_missing_live_errors` — no live file under staged TERMLINK_RUNTIME_DIR; verifies cmd returns Err with both "no hub.secret" and "is the hub running?" substrings
- [x] `cargo test -p termlink --bin termlink -- export_secret` passes (2/2)
- [x] `cargo check --workspace` passes
- [ ] Commit with T-1656 prefix

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
cargo check --workspace
cargo test -p termlink --bin termlink -- export_secret
grep -q "ExportSecret" crates/termlink-cli/src/cli.rs

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

### 2026-05-17T13:55:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1656-hub-export-secret--read-live-alternative.md
- **Context:** Initial task creation
