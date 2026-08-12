---
id: T-2669
name: "Sweep raw rpc_call MCP/CLI handler sites onto bounded variant (T-2641 hang class)"
description: >
  24 MCP + 12 CLI handler functions call unbounded client::rpc_call/rpc_call_addr on a fast RPC with no tokio::time::timeout in scope; a half-open hub blocks the handler forever (T-2641 hazard). Migrate the should-be-bounded subset to a bounded variant; exclude intentional long-pollers.

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
created: 2026-08-12T22:22:30Z
last_update: 2026-08-12T22:22:30Z
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

# T-2669: Sweep raw rpc_call MCP/CLI handler sites onto bounded variant (T-2641 hang class)

## Context

Filed during the T-2468 critical-review mandate (round 16), from a verified static
sweep — NOT autonomously built because the remediation is wire-adjacent, multi-file,
and needs a per-site design judgment (see Decisions).

`termlink_session::client::rpc_call` / `rpc_call_addr` (client.rs:273/283) connect
**unbounded** and then call the **unbounded** `Client::call`. Per the T-2641
doc-comment (client.rs:291) this means: *"if the hub accepts the connection but never
writes a response line, the caller blocks forever."* T-2641 added the bounded
`rpc_call_addr_with_timeout` (client.rs:306) precisely to fix that hang class — but
the caller sweep was never done. There is **no** socket-path bounded convenience
(`rpc_call_with_timeout(&Path, …)`), so MCP handlers that hold a `reg.socket_path()`
cannot migrate without either that helper or an explicit `tokio::time::timeout` wrap.

**Verified surface (36 functions with a raw `rpc_call`/`rpc_call_addr` and zero
`tokio::time::timeout` token in the enclosing function):**

**Bucket A — should-be-bounded (fast RPCs; a hang here is a pure defect):**
- MCP `crates/termlink-mcp/src/tools.rs`: `termlink_ping` (11658), `termlink_status`
  (11815), `termlink_exec` (11845), `termlink_output` (11885), `termlink_inject`
  (11908), `termlink_signal` (11934), `termlink_emit` (11963), `termlink_emit_to`
  (11992), `termlink_resize` (13115), `termlink_request` (13143), `termlink_tag`
  (13257), `termlink_dispatch` (13473), `termlink_hub_governor_status` (14048),
  `termlink_interact` (12634), `termlink_doctor` (12775), `termlink_agent_ask`
  (14661), `termlink_kv_set` (12312), `termlink_kv_get` (12340), `termlink_kv_list`
  (12368), `termlink_kv_del` (12398)
- MCP `crates/termlink-mcp/src/server.rs`: `read_session_detail` (185)
- CLI: `mirror_grid_composer.rs::cmd_mirror_tag` (167), `pty.rs::cmd_attach` (396) /
  `cmd_stream` (534) / `cmd_mirror` (602), `agent.rs::cmd_agent_negotiate` (361),
  `channel.rs::ensure_topic` (2755) / `resolve_latest_offset` (3105) /
  `fetch_topic_names` (12324)

**Bucket B — INTENTIONAL long-poll (must NOT get a naive client timeout — the bound
belongs server-side; excluded from the sweep, allowlisted in any future check):**
- `termlink_event_poll` (12026), `termlink_event_subscribe` (12064 — doc: "Blocks
  until events arrive or timeout"), `termlink_wait` (13309), `termlink_kv_watch`
  (12425), CLI `events.rs::cmd_watch_hub` (830) / `cmd_wait` (961)

**Bucket C — false positive (test):**
- `channel.rs::create_error_already_exists_rejects_genuine_failures` (19699) — a unit
  test, not a handler. Excluded.

## Concrete failure scenario

An MCP agent calls `termlink_ping` (or any Bucket-A handler) against a session whose
hub process has half-opened the socket — TCP/Unix accept succeeded but the peer never
writes the JSON-RPC response line (wedged record-walk, T-2354 blocking-pool
starvation, or a crashed-mid-write hub). `client::rpc_call(reg.socket_path(),
"termlink.ping", …).await` never returns. The MCP tool call hangs indefinitely with
no error surfaced to the agent — Directive #2 violation (silent, unobservable
failure), and the exact hazard T-2641 named and T-2659/2650/2651 fixed one site at a
time.

## Acceptance Criteria

### Agent
- [ ] Add a socket-path bounded convenience `rpc_call_with_timeout(&Path, method, params, Duration)` to `client.rs` (delegates to `rpc_call_addr_with_timeout` via `TransportAddr::unix`), mirroring the existing `rpc_call` → `rpc_call_addr` delegation. Unit test proving it errors (does not hang) against a half-open/never-responding socket within ~timeout.
- [ ] Migrate every Bucket-A site to the bounded variant (or an explicit `tokio::time::timeout` wrap where a shared future is already used), with a sensible default timeout constant (see Decisions — the value is a design choice, not a guess).
- [ ] Bucket-B long-poll sites are left unchanged AND documented in-code (one-line comment at each: "intentional long-poll — bound is server-side, not here") so a future sweep/check does not re-flag them.
- [ ] `cargo build -p termlink -p termlink-mcp` clean; existing MCP handler tests still pass.
- [ ] (Regression guard) A source-level static check `scripts/check-unbounded-rpc-call.sh` (sibling of check-alloc-sink-clamps / check-drain-sink-caps / check-silent-exit) that flags a raw `rpc_call`/`rpc_call_addr` in a function with zero `tokio::time::timeout` token, honouring a fn-name allowlist seeded with the Bucket-B long-pollers + Bucket-C test. Tree clean after the Bucket-A migration. Load-bearing proof via temp-revert of one migrated site. **This AC is what makes the fix stick** — file the check ONLY after the tree is clean, else it fires on 36 sites day one.

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

## RCA

**Symptom:** An MCP/CLI handler calling a fast RPC hangs forever (no error surfaced)
when the hub half-opens the socket but never writes a response line.

**Root cause:** `client::rpc_call` / `rpc_call_addr` are unbounded on both connect and
read. The bounded `rpc_call_addr_with_timeout` (T-2641) exists but (a) has no
socket-path sibling, so the many `reg.socket_path()`-based MCP handlers cannot adopt
it without a new helper, and (b) the caller sweep was never performed — T-2659/2650/
2651 each fixed exactly one site, leaving ~24 MCP + ~8 CLI fast-RPC handlers on the
unbounded primitive.

**Why structurally allowed:** No lint/check enforces "an rpc_call must be bounded."
The T-2641 fix hardened the primitive but nothing surfaced the un-migrated callers —
the classic "hardened in one place, siblings not swept" divergence (Class C). The
long-poll sites (event_poll/subscribe/wait/kv_watch) legitimately block, so a naive
"all rpc_call must have a timeout" rule would false-positive them, which is likely why
no blanket check was written.

**Prevention:** `scripts/check-unbounded-rpc-call.sh` (AC #5) — a source-level static
check that flags a raw rpc_call in a timeout-free function, with the intentional
long-pollers explicitly allowlisted. Makes the bounded-caller convention load-bearing
instead of discipline-only, and stops the next fast handler from copy-pasting the
unbounded call.

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

### 2026-08-13 — open design questions (for the implementer, why this was FILED not built)
- **Default timeout value:** what Duration is right for the Bucket-A RPCs? A ping/
  status wants a short bound (~5–10s); `exec`/`interact`/`agent_ask` may legitimately
  run longer (the RPC drives a session command). A single blanket constant may be
  wrong for `exec`. This per-verb judgment is why the sweep is not a mechanical
  find-replace. **Recommendation:** a module default (e.g. `DEFAULT_RPC_TIMEOUT =
  30s`) with per-verb overrides where the RPC is known-slow — decide at build time
  with the handler semantics in view.
- **Bounded-wrap vs helper:** Bucket-A MCP sites hold `reg.socket_path()` (a `&Path`),
  not a `TransportAddr` — so the clean migration needs the new `rpc_call_with_timeout`
  socket helper (AC #1). CLI sites that already assign `let fut = rpc_call(...)` (the
  pty.rs `rpc_future` pattern) can instead take an explicit `tokio::time::timeout`
  wrap. Chose: add the helper (covers the dense MCP cluster uniformly) AND allow the
  wrap form where a shared future already exists.
- **Rejected — blanket "all rpc_call get a timeout":** would break the Bucket-B
  long-pollers (event_poll/subscribe/wait/kv_watch) whose contract is to block until
  data or a server-side timeout. The bound for those belongs server-side; forcing a
  client bound would truncate legitimate long waits.
- **Rejected — autonomous batch migration in this window:** 20+ wire-adjacent MCP
  handler edits + a new primitive + a per-verb timeout decision is a medium build with
  behavior-change risk (a timeout now returns an error where callers/tests previously
  saw a block). Mandate discipline routes this to FILE, not autonomous build.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-08-12T22:22:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2669-sweep-raw-rpccall-mcpcli-handler-sites-o.md
- **Context:** Initial task creation
