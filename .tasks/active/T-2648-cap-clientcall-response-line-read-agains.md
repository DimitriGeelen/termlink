---
id: T-2648
name: "cap Client::call response-line read against unbounded next_line OOM from compromised
  remote hub"
description: >
  client.rs:195 Client::call reads a peer JSON-RPC response line via unbounded next_line();
  a compromised TOFU-trusted remote hub can stream GBs without a newline to OOM the
  daemon. Sibling read_capped_line (server.rs:1063, T-2518) hardened only the hub
  request path. Wire-contract/security/async fix filed not built.

status: captured
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
created: 2026-08-12T18:40:26Z
last_update: '2026-08-18T18:58:39Z'
date_finished:
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
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:35Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=3 (body:portability-abstraction); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:39Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2648: cap Client::call response-line read against unbounded next_line OOM from compromised remote hub

## Context

Round-9 divergence-class hunt (hardened-primitive / un-migrated-sibling), verified
in code. **Filed, NOT auto-built** — it is a wire-contract + security + async
change touching a shared field type + 5 constructors, and it needs a
response-size-cap judgment call (see Risk). Autonomous initiative does not extend
to breaking wire-read behavior across the hub-federation + MCP-remote-call paths.

`crates/termlink-session/src/client.rs`:
- Field (line 14): `reader: tokio::io::Lines<BufReader<Box<dyn AsyncRead + Send + Unpin>>>`.
- `Client::call` (line 195): reads the peer's JSON-RPC **response** line via
  `self.reader.next_line().await?` — `tokio::io::Lines::next_line` accumulates
  bytes into a `String` with **no size bound** until it sees `\n`.

This reader is fed by cross-host TCP peers in production: the hub federation
forwarder (`router.rs` `connect_addr_raw` → `c.call(...)`) and the MCP daemon's
`remote_call` / `remote_exec` path. A malicious or compromised — but merely
TOFU-trusted — remote hub can answer a forwarded RPC with a multi-GB line and no
newline; the long-lived hub/MCP daemon grows the `String` unbounded → **OOM-abort
of the whole daemon**. The `tokio::time::timeout` wrapping the forwarder
(router.rs) bounds the *wait*, not the *size* — a fast peer streams gigabytes
inside the timeout window, so the timeout does not mitigate.

**Hardened sibling (proof the convention exists):** `crates/termlink-hub/src/server.rs:1063`
`read_capped_line(reader, buf, max)`, used at `:1122` with
`MAX_LINE_BYTES = MAX_PAYLOAD_SIZE` (16 MiB). The T-2518 note at server.rs:1098
states the uncapped `.lines()`/`next_line()` read "was replaced with
`read_capped_line` to bound a single request line … (pre-auth unbounded-line
DoS)." That migration covered the hub **receive** path only; the structurally
identical `Client::call` **response-read** path was never migrated. Not present
in `.drain-sink-allowlist`.

**Risk / why filed not built:** the fix converts the `reader` field from
`Lines<...>` to a raw `BufReader<...>` (or wraps it) and threads a capped read
through all 5 `Client` constructors (client.rs:51/82/95/160/169). The cap value
is a judgment call: the hub bounds its own *requests* at 16 MiB, but its
*responses* are not obviously bounded — a `channel.subscribe` snapshot or
`channel.state` walk can legitimately produce a large single line. Cap too low →
break legitimate large reads; too high → weaken the DoS bound. Needs a decision on
the response cap (likely ≥ the largest legitimate response, or a separate
`MAX_RESPONSE_BYTES`) plus a test against a realistic large response. Recorded in
`## Decisions` when built.

## Acceptance Criteria

### Agent
- [ ] `Client::call` no longer reads the response line via unbounded `next_line()` — the read is bounded by a cap (mirror `read_capped_line`), returning a typed error (e.g. `ClientError::ResponseTooLarge`) when the peer exceeds it, not an OOM.
- [ ] The chosen response cap is documented in `## Decisions` with the rationale (relationship to `MAX_PAYLOAD_SIZE`, and why it does not break the largest legitimate response such as a `channel.subscribe` snapshot).
- [ ] All 5 `Client` constructors (client.rs:51/82/95/160/169) compile with the new reader shape; the `call_with_timeout` sibling (T-2354) is bounded too (it wraps the same read).
- [ ] A load-bearing unit test feeds an over-cap line (no newline / huge) and asserts the bounded error, not unbounded growth — mirrors `read_capped_line_rejects_overlong` (server.rs:1725).
- [ ] `cargo build` clean across affected crates; existing client tests pass.

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
# (Filled at build time — this is a filed-not-built task.)

## RCA

**Symptom:** A compromised/malicious but TOFU-trusted remote hub can OOM-abort the
local hub or MCP daemon by answering a forwarded RPC with a multi-GB response line
containing no newline.

**Root cause:** `Client::call` (client.rs:195) reads the peer response via
`tokio::io::Lines::next_line()`, which accumulates into a `String` with no size
bound. The `reader` field is a `Lines<...>`, so no cap can be applied at the call
site without changing the field shape.

**Why structurally allowed:** The T-2518 unbounded-line-DoS fix hardened the hub's
**request-receive** path (`read_capped_line`) but did not migrate the structurally
identical **client response-read** path — a hardened-primitive / un-migrated-sibling
divergence. The `.drain-sink-caps` static check (T-2531) scans for `.read_to_end`/
`.read_to_string`/`.collect::<Vec<u8>>` drains but does NOT recognize
`Lines::next_line()` as an unbounded accumulation sink, so this class was invisible
to the existing guard.

**Prevention:** (1) migrate the sibling (this task); (2) consider extending the
T-2531 drain-sink static check to flag `Lines`/`next_line()` accumulation on a
peer-fed reader — otherwise the next `Lines`-based read re-introduces the class
undetected. Log (2) as a follow-up when this is built.

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

### 2026-08-12T18:40:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2648-cap-clientcall-response-line-read-agains.md
- **Context:** Initial task creation
