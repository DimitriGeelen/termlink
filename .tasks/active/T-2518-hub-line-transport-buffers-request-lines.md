---
id: T-2518
name: "hub line transport buffers request lines unbounded (pre-auth OOM DoS) — no max-line cap on BufReader::lines"
description: >
  handle_line_connection uses uncapped BufReader::lines()/next_line(); a peer streaming bytes with no newline forces unbounded pre-auth String growth -> OOM. Binary codec path is bounded (MAX_PAYLOAD_SIZE 16MiB) and WS inherits tungstenite cap; line transport is the one unbounded inbound path. Fix: capped line reader mirroring MAX_PAYLOAD_SIZE.

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
created: 2026-08-03T21:56:25Z
last_update: 2026-08-03T21:56:38Z
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

# T-2518: hub line transport buffers request lines unbounded (pre-auth OOM DoS) — no max-line cap on BufReader::lines

## Context

`handle_line_connection` (crates/termlink-hub/src/server.rs:1032) reads each
newline-delimited JSON-RPC request via `BufReader::new(reader).lines()` +
`next_line().await` with NO length cap. `next_line()` grows an internal `String`
without limit until a `\n` arrives, and auth runs only *inside*
`process_request_message` (server.rs:1037) — after the whole line is buffered.
The T-2442 first-byte timeout reads a single byte; any non-`'G'` first byte
routes to this line path, after which there is no per-line cap or read timeout.
So a peer that completes TLS and streams bytes without a newline forces
unbounded PRE-AUTH memory growth → allocator abort / OOM-kill of the hub (one
connection suffices; the conn-count governor and RPC rate-limiter never fire
because no complete request is ever parsed). The binary codec path already
bounds inbound at `termlink_protocol::MAX_PAYLOAD_SIZE` (16 MiB, data.rs:102) and
the WS path inherits tungstenite's default message cap — the line transport is
the one unbounded inbound path. Found by T-2468 campaign firing #37 (protocol/
parse-layer lens).

## Acceptance Criteria

### Agent
- [x] `handle_line_connection` no longer uses uncapped `.lines()`/`next_line()`;
      it reads each line through `read_capped_line` that aborts the connection once
      a single line exceeds `MAX_LINE_BYTES` (= `termlink_protocol::MAX_PAYLOAD_SIZE`,
      mirrors the binary codec bound), logging a loud close (no silent drop).
- [x] Existing line-transport behaviour preserved for well-formed clients:
      `\n`-delimited requests still parse and respond (trailing `\r` stripped to
      match `next_line` semantics); EOF ends the loop; the full existing
      server.rs `server::` test suite passes (38 passed; 0 failed).
- [x] A load-bearing unit test (`read_capped_line_rejects_overlong`) feeds a
      no-newline byte stream longer than the cap and asserts TooLong. Proven
      load-bearing by temp-removing the cap check → test failed ("expected TooLong
      ... got Line").
- [x] A companion test (`read_capped_line_reads_normal_lines`) asserts normal
      multi-line reads + CRLF + EOF still work (regression guard on the happy path).
- [x] `cargo build --release -p termlink-hub` compiles and the new
      `read_capped_line` tests pass (2 passed; 0 failed).

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
grep -q "read_capped_line(&mut reader, &mut line_buf, MAX_LINE_BYTES)" crates/termlink-hub/src/server.rs
cargo test -p termlink-hub --lib read_capped_line > /tmp/t2518-verify.txt 2>&1 && grep -q "test result: ok" /tmp/t2518-verify.txt

## RCA

**Symptom:** a single client connection can OOM-kill the hub. After completing
TLS the peer sends any first byte other than `'G'` (routing to the line
transport) and then streams bytes with no `\n`; the hub's memory climbs without
bound until the allocator aborts. The conn-count governor (256) and per-sender
RPC rate-limiter never help — one connection suffices and no complete request is
ever parsed, so neither guard engages.

**Root cause:** `handle_line_connection` read each request line via
`BufReader::new(reader).lines()` + `next_line().await`, which grows an internal
`String` without limit until a newline arrives. Authentication runs only inside
`process_request_message`, i.e. *after* the entire line is buffered — so the
unbounded buffering is entirely pre-auth. The T-2442 first-byte timeout bounds
only the very first read (1 byte); there is no per-line length cap and no
per-line read timeout thereafter.

**Why structurally allowed:** the two *other* inbound transports were already
bounded — the binary codec rejects `payload_length > MAX_PAYLOAD_SIZE`
(protocol/src/data.rs:102) and the WS path inherits tungstenite's default 16 MiB
message cap — but the legacy newline transport (the default path) was extracted
verbatim in T-2305 with its uncapped `.lines()` intact, and no test drove an
unterminated line, so the asymmetry was invisible. Inbound size bounding was
per-transport rather than a shared invariant.

**Prevention:** `read_capped_line` reads through the `AsyncBufRead` fill/consume
interface and aborts once a single line exceeds `MAX_PAYLOAD_SIZE` (the SAME 16
MiB bound the codec enforces), closing the connection loudly. Unit test
`read_capped_line_rejects_overlong` drives an over-cap unterminated stream and
asserts the abort — proven load-bearing (removing the cap check makes it read
the whole line and fail). `read_capped_line_reads_normal_lines` guards the happy
path (multi-line + EOF + CRLF). All three inbound transports now share the same
16 MiB inbound bound.

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

### 2026-08-03T21:56:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2518-hub-line-transport-buffers-request-lines.md
- **Context:** Initial task creation

### 2026-08-03T21:56:38Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
