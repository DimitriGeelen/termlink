---
id: T-1663
name: "MCP parity for hub probe + tofu verify primitives"
description: >
  MCP parity for hub probe + tofu verify primitives

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-05-17T18:26:04Z
last_update: 2026-05-17T18:38:08Z
date_finished: 2026-05-17T18:38:08Z
---

# T-1663: MCP parity for hub probe + tofu verify primitives

## Context

T-1661 exposed `fleet verify` (fleet rollup) to MCP but skipped the single-host primitives `hub probe` and `tofu verify`. Agents wanting to check ONE specific hub via MCP currently must invoke `termlink_fleet_verify` (probes every profile, may include unreachable hosts that drag the call out 20+ seconds) or shell out via `termlink_exec` (out-of-band, no JSON parity, no auth model). Closing the parity gap takes ~80 LOC of pattern-replication against the existing T-1661 surface and lets MCP-driven agents (Claude Code, integrations) do per-host rotation diagnosis in the same envelope as every other MCP tool.

## Acceptance Criteria

### Agent
- [x] `termlink_hub_probe` MCP tool ships in `crates/termlink-mcp/src/tools.rs`. Takes `{address: String, ...}` params, opens TLS handshake via `termlink_session::tofu::probe_cert`, returns `{ok, address, fingerprint, error}` JSON. No auth, no profile required, no `KnownHubStore` mutation. Mirrors CLI `hub probe <addr>` semantics. **Verified live 2026-05-17 against 192.168.10.122:9100:** returned `{ok:true, fingerprint:"sha256:22c19fed...d00d46", error:null}` — matches the CLI/fleet-verify ground truth for ring20-management.
- [x] `termlink_tofu_verify` MCP tool ships in same file. Takes `{address: String, ...}` params, probes via TLS, looks up `KnownHubStore.get(address)`, returns `{ok, address, status (match/drift/no-pin/probe-fail), wire, pinned, error, actions[]}` JSON. `actions` populated with heal hints when `status=="drift"`. Mirrors CLI `tofu verify <addr>` semantics. **Verified live 2026-05-17 against 192.168.10.122:9100:** returned `status:"match", wire==pinned, actions:[]` — drift detection path exercises identical branching to T-1661's tested code.
- [x] `cargo build --release` succeeds. **Verified:** `cargo build --release -p termlink-mcp` clean in 1m11s; `cargo build --release -p termlink` clean in 6m26s (one pre-existing warning in `unused_assignments`, not introduced by this change).
- [x] Tool descriptions follow the existing T-1661 phrasing pattern (read-only, no-auth, what-to-use-it-for line). **Verified:** Both descriptions follow the "Pure read-only diagnostic: no authentication, no profile required, no `KnownHubStore` mutation" idiom + Use-for line + companion-to-other-tool line.
- [x] After build, the binary surfaces both new tools (verified via grep of compiled binary). **Verified:** `grep -ao "termlink_hub_probe\|termlink_tofu_verify\|termlink_fleet_verify" target/release/termlink` returns all three names; runtime `tools/list` over stdio JSON-RPC also enumerates both new tools alongside `termlink_fleet_verify`.

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
cargo build --release -p termlink-mcp
grep -q "termlink_hub_probe" target/release/termlink-mcp 2>/dev/null || grep -q "termlink_hub_probe" target/release/termlink
grep -q "termlink_tofu_verify" target/release/termlink-mcp 2>/dev/null || grep -q "termlink_tofu_verify" target/release/termlink

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

### 2026-05-17T18:26:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1663-mcp-parity-for-hub-probe--tofu-verify-pr.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-8f39cf97
- **Timestamp:** 2026-05-17T18:39:18Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#1 (Agent)** — `termlink_hub_probe` MCP tool ships in `crates/termlink-mcp/src/tools.rs`. Takes `{address: String, ...}` params, opens TLS handshake via `termlink_session::tofu::probe_cert`, returns `{ok, address, f
  - **AC-verify-mismatch** (narrow, heuristic) — `path=crates/termlink-mcp/src/tools.rs in: `termlink_hub_probe` MCP tool ships in `crates/termlink-mcp/src/tools.rs`. Takes `{address: String, ...}` params, opens TLS handshake via `termlink_se`

### 2026-05-17T18:38:08Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
