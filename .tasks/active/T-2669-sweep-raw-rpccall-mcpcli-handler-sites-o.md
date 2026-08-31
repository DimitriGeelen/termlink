---
id: T-2669
name: "Sweep raw rpc_call MCP/CLI handler sites onto bounded variant (T-2641 hang
  class)"
description: >
  24 MCP + 12 CLI handler functions call unbounded client::rpc_call/rpc_call_addr
  on a fast RPC with no tokio::time::timeout in scope; a half-open hub blocks the
  handler forever (T-2641 hazard). Migrate the should-be-bounded subset to a bounded
  variant; exclude intentional long-pollers.

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
created: 2026-08-12T22:22:30Z
last_update: 2026-08-31T15:10:29Z
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
  - ts: '2026-08-29T22:41:47Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 2
      D3: 3
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=2 
      (body:telemetry-or-audit-entry); D3=3 (body:component-discoverability); 
      D4=3 (body:portability-abstraction); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: e4a00f38e801
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
- [x] Add a socket-path bounded convenience `rpc_call_with_timeout(&Path, method, params, Duration)` to `client.rs` (delegates to `rpc_call_addr_with_timeout` via `TransportAddr::unix`), mirroring the existing `rpc_call` → `rpc_call_addr` delegation. Unit test proving it errors (does not hang) against a half-open/never-responding socket within ~timeout.
- [ ] **(PARTIAL — 12 of ~145; blocked on the human per-verb timeout decision for the CLASS 2 verbs, see Evolution)** Migrate every Bucket-A site to the bounded variant (or an explicit `tokio::time::timeout` wrap where a shared future is already used), with a sensible default timeout constant (see Decisions — the value is a design choice, not a guess).
- [x] **(DONE — slice 2. 20 call sites across the 15 genuine CLASS 1 functions now carry an in-code one-liner naming the server-side bound and saying `do NOT migrate`. Verified the added text does not contain the check's `tokio::time::timeout` token, so no site was spuriously cleared: counts held at 182/155.)** Bucket-B long-poll sites are left unchanged AND documented in-code (one-line comment at each: "intentional long-poll — bound is server-side, not here") so a future sweep/check does not re-flag them.
- [x] `cargo build -p termlink -p termlink-mcp` clean; existing MCP handler tests still pass.
- [x] (Regression guard) A source-level static check `scripts/check-unbounded-rpc-call.sh` (sibling of check-alloc-sink-clamps / check-drain-sink-caps / check-silent-exit) that flags a raw `rpc_call`/`rpc_call_addr` in a function with zero `tokio::time::timeout` token, honouring a fn-name allowlist seeded with the Bucket-B long-pollers + Bucket-C test. Tree clean after the Bucket-A migration. Load-bearing proof via temp-revert of one migrated site. **This AC is what makes the fix stick** — file the check ONLY after the tree is clean, else it fires on 36 sites day one.

### Human

- [ ] [REVIEW] Choose the timeout policy for the ~140 CLASS 2 sites so the sweep can continue

  **Steps:**
  1. Read the two-class split at the top of `.context/checks/unbounded-rpc-call-allowlist`
     (CLASS 1 = intentional long-poll, bound belongs server-side; CLASS 2 = not yet
     migrated, still the T-2641 hang class).
  2. Run `bash scripts/check-unbounded-rpc-call.sh --no-heartbeat`
     to see the current state (expect: clean, 182 scanned, 155 acknowledged —
     was 183/156 before slice 2; see Evolution 2026-08-31).
  3. Pick one of the three options in `## Recommendation` above — 1 (per-verb table),
     2 (single generous default + named exceptions, my recommendation), or 3 (leave
     ledgered) — and record it in `## Decisions` below.

  **Expected:** A named option with a one-line reason. If option 2, also name the default
  value (I suggest 300s) and any verbs that must be exempted.

  **If not:** If none of the three fits, say what the fourth is. If you want to defer,
  say so explicitly and set `revisit_at` — a deferral that is recorded is fine; the thing
  to avoid is the 140 sites staying live because nobody said either way.

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

bash scripts/check-unbounded-rpc-call.sh --no-heartbeat > /tmp/.t2669-check.out 2>&1
grep -q "check-unbounded-rpc-call: clean" /tmp/.t2669-check.out
grep -q "182 call site(s) scanned" /tmp/.t2669-check.out
grep -q "pub async fn rpc_call_with_timeout" crates/termlink-session/src/client.rs
grep -q "async fn rpc_call_with_timeout_bounds_a_silent_server" crates/termlink-session/src/client.rs
test "$(grep -c 'rpc_call_with_timeout(' crates/termlink-mcp/src/tools.rs)" -eq 9
cargo build -p termlink -p termlink-mcp 2>&1 | tail -1 > /tmp/.t2669-build.out && grep -q "Finished" /tmp/.t2669-build.out
cargo test -p termlink-session --lib client:: > /tmp/.t2669-test.out 2>&1; grep -q "rpc_call_with_timeout_bounds_a_silent_server ... ok" /tmp/.t2669-test.out
grep -q "0 failed" /tmp/.t2669-test.out
grep -q "RECORD CORRECTION (T-2669" .context/checks/unbounded-rpc-call-allowlist

## Recommendation

**Recommendation:** GO

**Rationale:** **READ THIS FIRST — the GO above is on slice 1 only, which is already
landed. It is NOT a request to close T-2669, and this task must stay OPEN after you
decide.** What is being asked of you is one thing: pick the timeout policy for the
remaining ~140 sites so the sweep can continue. The review page renders the verdict as a
bare "GO" because that is the only token it parses; the scope of that GO is this
paragraph, not the task.

T-2669's own Decisions section rejected an autonomous batch migration for
a good reason: a client timeout returns an ERROR where callers and tests previously saw
a block, so the per-verb timeout value is a design judgment, not a lookup. Slice 1
deliberately took only the 5 sites where that judgment is vacuous — short hub/session
reads (`ping`, `status`, `hub.governor_status`, `channel.list`, `channel.cv_keys`) where
any sane bound is right and 30s is generous. The remaining ~140 are not like that:
`exec`, `interact` and `agent.ask` drive a session command that may legitimately run for
minutes, and a bound chosen carelessly converts a working long operation into a failure.

Three options, and I do not think they are close:

1. **A per-verb timeout table** — correct, and ~140 individual judgments. Weeks.
2. **A single generous default (e.g. 300s) for everything not already CLASS 1**, with a
   short table of named exceptions. Bounds every site against the permanent-hang class
   T-2641 named, while being long enough that no healthy operation is truncated. Cheap.
3. **Leave the 140 ledgered.** Costs nothing today and keeps the hang class live.

I recommend **option 2**. The failure this task exists to prevent is *unbounded* — a
handler that never returns and surfaces nothing, a Directive #2 violation. The distance
between 300s and a perfectly-tuned 45s is a latency question; the distance between 300s
and no bound at all is a liveness question. Fixing the liveness question for 140 sites
now is worth more than fixing the latency question for 140 sites eventually, and option 2
does not preclude tightening individual verbs later.

**Evidence (measured 2026-08-30 on this checkout, not forecast):**

- `scripts/check-unbounded-rpc-call.sh --no-heartbeat` -> **clean, 183 call sites scanned,
  156 acknowledged**, exit 0. Before this slice it FIRED on 5.
- `cargo build -p termlink -p termlink-mcp` -> `Finished` (exit 0).
- `cargo test -p termlink-session --lib client::` -> **34 passed, 0 failed**, including the
  new `rpc_call_with_timeout_bounds_a_silent_server` (asserts the call ERRORS within the
  bound against a listener that accepts, drains the request, then goes silent forever —
  the exact half-open-hub shape; without the bound it hangs and the outer 5s guard trips).
- Load-bearing: temp-reverting `termlink_ping` to the raw form re-fires the check on
  exactly `tools.rs:11938 (in fn termlink_ping)`; restoring returns it to clean.
- Ledger correction: `6b587e826` carried the check and its allowlist to main but touched
  **no `.rs` file**, so the header's "After this session migrated the 5 fast-read sites …
  183 scanned / 156 unbounded" described a tree that did not exist here. Corrected in
  place, with the correction stated rather than the sentence quietly rewritten.

**What is NOT claimed:** the 156 ledgered sites are not safe. 16 are intentional
long-polls; **140 can still hang forever**. A clean check means the ratchet holds, not
that the tree is bounded.

## Evolution — 2026-08-30 (slice 1 of N)

**What was not known at filing.** The task body states 36 unbounded functions. Measured
with the check on 2026-08-28: **188 call sites scanned, 161 unbounded** — a ~4.5x
undercount, because the analytics tool family in `tools.rs` grew substantially after
this task was filed on 2026-08-12 and every one of those handlers took the unbounded
form. The remediation is far larger than filed, which makes the per-verb timeout
judgment AC#2 needs *more* load-bearing, not less: it is now a policy decision over
~140 verbs, not a dozen.

**What this slice landed.** AC#1 and AC#5 in full, plus 5 of AC#2.
`client::rpc_call_with_timeout(&Path, …)` — the missing socket-path sibling of
`rpc_call_addr_with_timeout` — now exists with a bounded-against-a-silent-server unit
test, and the 5 fast-read sites whose timeout needs no judgment (`termlink_ping`,
`termlink_status`, `termlink_hub_governor_status`, `termlink_channel_cv_keys`,
`termlink_channel_list`) are migrated onto it at 30s. Tree: 183 scanned, 0
unacknowledged. Load-bearing proven by temp-reverting `termlink_ping` — the check
re-fires on exactly that site and restoring returns it to clean.

**The finding that made this slice necessary.** The check and its ledger were committed
to main in `6b587e826` (T-2850, landing guards stranded on the charter-review worktree)
— but that commit touched **no `.rs` file at all**. The migration had been done on the
worktree and did not cross. So the ledger's header asserted "After this session migrated
the 5 fast-read sites … the tree measures 183 scanned / 156 unbounded" while those 5
sites were still on the raw unbounded form and the check was FIRING on precisely them.

The header's numbers were a forecast of the migrated tree, not a measurement of the
shipped one — and they were *correct*, which is what made the divergence hard to see:
the check fired, the ledger explained the fire away, and nothing reconciled the two. A
ratchet's baseline is only meaningful if it describes the tree it ships with, and a
baseline written ahead of its code is indistinguishable from one written behind it. The
header is corrected in place with the correction stated rather than silently rewritten.

**Why this task stays open.** AC#2 is 5/~145. The remaining ~140 are CLASS 2 in the
ledger — acknowledged so the check can ratchet from day one, explicitly NOT a claim they
are safe; each can still hang forever. Closing here would convert "140 sites waiting on
a decision" into "task complete", which is the exact reading the ledger's two-class split
exists to prevent.

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

## Evolution — 2026-08-31 (slice 2)

**What changed: 7 fast RPCs were hiding inside the CLASS 1 long-poll exemption.**

This slice set out to do AC#3 — write the in-code one-liners at the Bucket-B sites.
Before copying the ledger's reason into 16 code comments, I checked it against the
handlers. It was false for 7 of the 16.

The check keys a site by ENCLOSING FUNCTION (`sig="${file}::${fname}::unbounded-rpc-call"`),
so one CLASS 1 line exempts every `rpc_call` in that function. All 16 entries carried the
same generated reason — "dispatches a blocking event/watch RPC whose bound is passed
server-side as timeout_ms" — which is a statement about the METHOD NAME, not about what
the handler does. Read against the handlers:

- `handle_event_emit` (session `handler.rs`) reads **no** `timeout_ms`. It takes the bus
  lock, appends, returns. A fast RPC — the T-2641 hang class exactly. It sits in
  `cmd_agent_ask`, `cmd_agent_negotiate`, `cmd_request`, `termlink_agent_ask`,
  `termlink_request`.
- `handle_event_poll` never blocks either — lock, read, return. It sits in
  `termlink_wait`, and it was the **only** call in `termlink_event_poll`: a function
  filed as an intentional long-poll that never long-polls.

So 7 genuinely-unbounded fast calls sat under a permanent "justified" exemption. That is
strictly worse than CLASS 2 — CLASS 2 is an open ledger awaiting a decision, CLASS 1 says
no action is needed — and it is the same failure this task already had once: a ledger
asserting a code state nobody had checked. The reason text was uniform because it was
generated per function from method names, and `event.emit` / `event.poll` happen to live
in functions whose *other* call really is a long-poll.

**What this slice landed.** All 7 bounded at 30s via `client::rpc_call_with_timeout` —
the same vacuous-judgment criterion slice 1 used (a lock-and-return RPC has no per-verb
timeout question). This did **not** need the human policy decision, which is about the
CLASS 2 verbs where a bound can truncate real work. `termlink_event_poll` left the ledger
entirely (no unbounded call remains); the other 6 keep their line for the `event.subscribe`
long-poll, and their reason is now true rather than aspirational. AC#3 then landed in
form: 20 call sites across the 15 genuine CLASS 1 functions carry an accurate, per-method
in-code one-liner.

**Measured.** Tree 183 scanned / 156 acknowledged -> **182 / 155**; check clean, exit 0.
`cargo build -p termlink -p termlink-mcp` Finished. `cargo test -p termlink-session --lib
client::` 34 passed / 0 failed. Load-bearing proof: temp-reverting `termlink_event_poll`
to the raw form re-fires the check on exactly `tools.rs:12362 (in fn termlink_event_poll)`
— proving both the migration and the ledger removal are load-bearing — and restoring
returns it to clean. Verified separately that the 20 added comments do not contain the
check's `tokio::time::timeout` token, so no site was spuriously cleared: counts held.

**What is NOT claimed.** The remaining 140 CLASS 2 sites are untouched and can still hang
forever. AC#2 is 12 of ~145. This slice fixed a misclassification; it did not answer the
timeout-policy question, which is still the human's.

**Follow-up found, not fixed:** `scripts/check-unbounded-rpc-call.sh` has no stale-entry
detection. Had this slice not removed the `termlink_event_poll` line by hand, the ledger
would have kept acknowledging a site that no longer exists, silently. That and the
per-function keying are the same blind spot from two sides. Noted in the ledger header;
neither blocks the sweep.


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

### 2026-08-29T22:41:46Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
