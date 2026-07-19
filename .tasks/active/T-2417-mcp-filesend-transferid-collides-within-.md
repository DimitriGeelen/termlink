---
id: T-2417
name: "MCP file_send transfer_id collides within a session (PID-only mint)"
description: >
  MCP file_send transfer_id collides within a session (PID-only mint)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-19T12:44:12Z
last_update: 2026-07-19T12:44:12Z
date_finished: null
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── BVP scoring fields (T-1918, arc-006). See docs/reports/T-1915-bvp-inception.md for semantics. ──
# bvp_scores:                     # confirmed per-driver scores 0-5, set by `fw bvp confirm` (T-1924).
#                                 # Sovereignty boundary — only set after human or agent confirmation.
#                                 # Shape: {D1: <int 0-5>, D2: <int 0-5>, D3: <int 0-5>, D4: <int 0-5>, [<free-driver-id>: <int>]...}
# bvp_scores_proposed:            # estimator-proposed scores (T-1922 worker). Persists when ≥2 delta
#                                 # from bvp_scores: on any driver (M3 v2-delta). Shape: list of timestamped entries.
# cost_estimate:                  # F8 composite: 0.6×blast_radius + 0.3×tier + 0.1×effort.
#                                 # Q2 fallback: T-shirt S/M/L/XL mapped to 2/4/6/8 when blast_radius is not yet computable.
---

# T-2417: MCP file_send transfer_id collides within a session (PID-only mint)

## Context

Surfaced live on the fleet: the AEF (`0e7ee6ca`, project 999) ↔ workflow-designer
(`6a646ce8`, project 832) integration thread lost a full delivery round-trip to a
`file_send` failure. workflow-designer sent `aef-workflow-designer-0.3.0.html`
(826643 bytes, 17 chunks) under `transfer_id=xfer-mcp-3273116`; AEF's
`file_receive` failed reassembly with **"got 17/1 chunks"**. AEF root-caused it as
a **transfer-id collision**: two back-to-back sends in the same MCP process drew
the same id, and the receiver — which scopes reassembly ONLY by `transfer_id` —
blended both chunk streams. A re-fire from a fresh process (new PID → new id)
landed clean.

Root cause confirmed in code: `crates/termlink-mcp/src/tools.rs:13484` mints
`transfer_id = format!("xfer-mcp-{}", std::process::id())` — **PID-only**, so every
`termlink_file_send` within one MCP server process is byte-identical. The CLI path
(`util.rs::generate_request_id`) already mints PID+timestamp and does not collide.
This is a comms-rail reliability defect (part of the doorbell/transport substrate),
not an AEF or workflow-designer bug. Both agents asked for an upstream termlink flag.

## Acceptance Criteria

### Agent
- [x] MCP `file_send` transfer_id is unique per send within one process — the mint (`new_transfer_id()`) includes PID + ms-timestamp + a per-process atomic nonce (mirrors the CLI's timestamped scheme), so two back-to-back sends in the same process cannot draw the same id.
- [x] A regression test asserts that two mints in the same process produce distinct transfer_ids (guards against a future revert to a per-process-constant id) — `tools::tests::transfer_id_unique_per_send` (checks 2 distinct + a 1000-mint batch all-unique). PASS.
- [x] `cargo build -p termlink-mcp` compiles clean — Finished dev profile, exit 0.

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.

     ── Prefix routing (T-1811, T-1878): default to [REVIEWER] if Expected is grep-able ──
     If your Expected clause is grep-able / file-exists / structural (a deterministic
     shell check), prefer [REVIEWER] — that AC should be an Agent AC with the reviewer
     command in `## Verification` instead of a Human AC here. Only keep [REVIEW] if
     verification genuinely needs human taste (tone, feel, layout rhythm).
     See CLAUDE.md §AC Classification Guidance for the conversion rule.

     [REVIEW] example (genuine human judgment):
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error

     [REVIEWER] example (static-scan-verifiable — convert to Agent AC + Verification):
       - [ ] [REVIEWER] Block message names both bypass mechanisms
         **Steps:**
         1. Run `bin/fw reviewer T-XXX`
         **Expected:** Verdict: PASS; no findings on `block-message-completeness`
         **If not:** Inspect hook block-message string and add missing mechanism
       Conversion: this AC should be moved to ### Agent and
       `bin/fw reviewer T-XXX 2>&1 | grep -q "Overall:.*PASS"` added to ## Verification.
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
#
# Pipefail/SIGPIPE hint (L-387): P-011 runs each command under `set -eo pipefail`.
# `cmd | grep -q PATTERN` exits 141 (SIGPIPE) when grep matches and closes stdin
# while the upstream is still writing — verification then "fails" even though
# the pattern was present. Safe pattern: capture first, grep the capture:
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"
# Or:
#     cmd > /tmp/.out 2>&1 && grep -q "PATTERN" /tmp/.out
# Origin: L-387, captured 4× (T-1716, T-1838, T-1862, T-1863) before this hint.
#
# Single pipe only — no intermediate tail/awk/sed stages between capture and grep
# (T-2090): `echo "$out" | tail -3 | grep -q PAT` re-introduces the SIGPIPE risk
# the capture step closed off — the middle stage is what `grep -q` slams its
# stdin on. `echo "$out"` is small and immediate; grep scans the whole captured
# string anyway, so the tail-3 was cosmetic. Drop it: `echo "$out" | grep -q PAT`.
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

# T-2417 verification: mint is no longer PID-only, and the regression test passes.
grep -q 'xfer-mcp-{}-{}-{}' crates/termlink-mcp/src/tools.rs
cargo test -p termlink-mcp --lib transfer_id_unique_per_send 2>&1 | grep -q '1 passed'

## RCA

**Symptom:** MCP `termlink_file_send` of a 17-chunk file failed reassembly on the
receiver with `"incomplete transfer — got 17/1 chunks for transfer xfer-mcp-3273116"`.
A live AEF↔workflow-designer integration delivery was lost and had to be re-fired.

**Root cause:** `crates/termlink-mcp/src/tools.rs:13484` minted the transfer id as
`format!("xfer-mcp-{}", std::process::id())` — derived from the PID alone. Every
`file_send` in one MCP server process therefore produces the identical id. Because
`file_receive` (`tools.rs:13843-13876`) scopes reassembly ONLY by `transfer_id`
(collecting every stream chunk matching the id, keyed by chunk index in a BTreeMap),
two sends sharing one id merge: `total_chunks` comes from the last `file.init` (1)
while chunks accumulate from both sends (17) → the "17/1 chunks" mismatch.

**Why structurally allowed:** the id was scoped to the wrong lifetime — per-process
instead of per-send. The CLI transfer path already mints PID+timestamp
(`util.rs::generate_request_id`) and does not collide, but the MCP path diverged and
no test pinned per-send uniqueness. The "ok:true = hub-accepted, NOT delivered"
transport contract meant the sender saw success while reassembly silently failed on
the far side — a Directive-2 (no silent failure) blind spot in the transfer layer.

**Prevention:** (fix) collision-proof mint = PID + ms-timestamp + per-process atomic
nonce, so even two sends in the same millisecond in one process are unique.
(regression guard) a unit test asserting two mints in-process are distinct — pins the
per-send lifetime so a future edit can't silently reintroduce a per-process-constant
id. Registered as a learning (transfer_id lifetime = per-send, never per-process).

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

### 2026-07-19T12:44:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2417-mcp-filesend-transferid-collides-within-.md
- **Context:** Initial task creation
