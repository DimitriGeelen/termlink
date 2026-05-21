---
id: T-1744
name: "termlink_hub_fingerprint + termlink_hub_export_secret MCP — rotation-protocol primitives (T-1166 MCP-parity)"
description: >
  MCP parity for the rotation-protocol primitive verbs documented in CLAUDE.md: hub_fingerprint (reads local hub.cert.pem, returns 12-char TLS fingerprint for peers to pin) and hub_export_secret (reads <runtime_dir>/hub.secret per R3 'read-live not cache' rule). Both are read-only co-located diagnostics — an MCP agent on the hub host can answer 'what is my hub's fingerprint/secret?' without shelling out to the CLI.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-21T07:40:30Z
last_update: 2026-05-21T07:43:47Z
date_finished: 2026-05-21T07:43:47Z
---

# T-1744: termlink_hub_fingerprint + termlink_hub_export_secret MCP — rotation-protocol primitives (T-1166 MCP-parity)

## Context

Two rotation-protocol primitive verbs documented in CLAUDE.md but only available via CLI shell-out. Both
are read-only co-located diagnostics that read live hub files (NOT IP-keyed caches — R3 rule).

- `hub_fingerprint` (CLI: T-1657, infrastructure.rs:943): reads `<runtime_dir>/hub.cert.pem`, extracts
  first CERTIFICATE block, base64-decodes DER, computes `cert_fingerprint` (sha256:<hex>). Output
  matches KnownHubStore pins so peers can compare-then-pin.
- `hub_export_secret` (CLI: T-1656, infrastructure.rs:866): reads `<runtime_dir>/hub.secret` (the live
  file), returns trimmed 64-hex secret. Authoritative source per R3 — never read from
  `~/.termlink/secrets/<ip>.hex` cache which goes stale across hub restarts.

## Acceptance Criteria

### Agent
- [x] `termlink_hub_fingerprint` tool method: no params, reads `termlink_hub::tls::hub_cert_path()`, extracts and decodes CERT block, returns `{ok, path, fingerprint}` — tools.rs section "Hub fingerprint + export-secret (T-1744)"
- [x] `termlink_hub_export_secret` tool method: no params, reads `termlink_hub::server::hub_secret_path()`, returns `{ok, path, hex, bytes}`. NEVER touches `~/.termlink/secrets/` cache (R3 compliance) — verified: only `hub_secret_path` is read
- [x] Both return `{ok: false, error, path}` on missing file (hub not running) — `path` always surfaced
- [x] Pure helper `parse_first_cert_block_to_fingerprint(pem: &str) -> Result<String, String>` extracted — tools.rs:~1420
- [x] At least 4 unit tests on the helper — 5 added: valid PEM (b"abc"→known sha256), missing BEGIN, missing END, base64 garbage, whitespace-in-body
- [x] `cargo build -p termlink-mcp` clean (only pre-existing cur_run_end warning)
- [x] `cargo test -p termlink-mcp` 367 → 372 passing, 0 regressions
- [x] CLI smoke-baseline (`termlink hub fingerprint` + `hub export-secret`) confirms same hub-state values the MCP tools will report

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
cargo build -p termlink-mcp
cargo test -p termlink-mcp --lib 2>&1 | tail -5 | grep -q "test result: ok"

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

### 2026-05-21T07:40:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1744-termlinkhubfingerprint--termlinkhubexpor.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-71f6e633
- **Timestamp:** 2026-05-21T07:43:47Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-21T07:43:47Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
