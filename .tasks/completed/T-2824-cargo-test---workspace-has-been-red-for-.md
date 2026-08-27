---
id: T-2824
name: "cargo test --workspace has been red for 8 days — MCP termlink_topics lost parity
  with the CLI at T-2624"
description: >
  parity_topics fails: CLI `topics --json` emits sessions_probed/skipped/unreachable/bad_result
  (added by T-2624, 2026-08-12, correctly — no silent partial inventory) and the MCP
  termlink_topics tool was never updated to match. Pre-existing, not from this branch:
  zero files touched in termlink-mcp or termlink-cli here. The parity harness is doing
  its job; nobody was running it.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [mcp, parity, directive-2]
components: [crates/termlink-bus/src/claim.rs, crates/termlink-bus/src/lib.rs, crates/termlink-bus/src/meta.rs, crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/commands/dispatch.rs, crates/termlink-cli/src/commands/events.rs, crates/termlink-cli/src/commands/execution.rs, crates/termlink-cli/src/commands/metadata.rs, crates/termlink-cli/src/commands/mirror_grid.rs, crates/termlink-cli/src/commands/pty.rs, crates/termlink-cli/src/commands/session.rs, crates/termlink-cli/src/commands/substrate.rs, crates/termlink-cli/src/main.rs, crates/termlink-cli/src/util.rs, crates/termlink-hub/src/artifact.rs, crates/termlink-hub/src/channel.rs, crates/termlink-hub/src/server.rs, crates/termlink-hub/tests/no_federation_tripwire.rs, crates/termlink-mcp/src/tools.rs, crates/termlink-mcp/tests/parity.rs, crates/termlink-protocol/src/control.rs, crates/termlink-session/build.rs, crates/termlink-session/src/ansi.rs, crates/termlink-session/src/claim_client.rs, crates/termlink-session/src/executor.rs, crates/termlink-session/src/handler.rs, crates/termlink-session/src/lib.rs, crates/termlink-session/src/pty.rs, crates/termlink-session/src/registration.rs, crates/termlink-session/src/scrollback.rs, crates/termlink-session/tests/no_spoke_mesh_tripwire.rs, scripts/agent-chat-arc-recent.sh, scripts/agent-conversation-selftest.sh, scripts/canary-status.sh, scripts/check-alloc-sink-clamps.sh, scripts/check-busy-spin.sh, scripts/check-canary-log-hygiene.sh, scripts/check-canary-log-isolation.sh, scripts/check-charter-drift-freshness.sh, scripts/check-charter-sentence-drift.sh, scripts/check-cron-install-drift.sh, scripts/check-drain-sink-caps.sh, scripts/check-env-var-docs.sh, scripts/check-error-code-docs.sh, scripts/check-error-code-emission.sh, scripts/check-mcp-parity-census.sh, scripts/check-platform-lock.sh, scripts/check-preflight-doc-set-drift.sh, scripts/check-release-artifact-drift.sh, scripts/check-silent-exit.sh, scripts/check-stuck-claims-freshness.sh, scripts/check-verification-legs.py, scripts/check-version-derivation.sh, scripts/fabric-register-workflow.sh, scripts/fleet-adoption-snapshot.sh, scripts/lib/reap-topic.sh, scripts/run-guard-layer.sh, scripts/session-selftest.sh, scripts/substrate-preflight.sh, scripts/substrate-smoke.sh, scripts/substrate-worker-pickup.sh, scripts/sweep-test-debris.sh, scripts/test-agent-conversation-list.sh, scripts/test-agent-conversation-status.sh, scripts/test-agent-respond.sh, scripts/test-agent-send-auto-discover.sh, scripts/test-agent-send-orchestration.sh, scripts/test-agent-send.sh, scripts/test-agent-send-transport.sh, scripts/test-journal-mirror.sh, scripts/test-sidecar-auto-confirm.sh, tests/agent-send-grace-window.sh, tests/agent-send-idle-gate.sh, tests/canary-log-hygiene-fixtures.sh, tests/canary-log-isolation-fixtures.sh, tests/canary-status-worktree-fixtures.sh, tests/charter-drift-check-fixtures.sh, tests/chat-arc-recent-fixtures.sh, tests/cron-drift-firing-fixtures.sh, tests/cron-install-drift-fixtures.sh, tests/error-code-emission-fixtures.sh, tests/guard-layer-runner-fixtures.sh, tests/mcp-parity-census-fixtures.sh, tests/platform-lock-check-fixtures.sh, tests/reap-topic-fixtures.sh, tests/relay-b1-doorbell-rail.sh, tests/relay-b2-send-hops.sh, tests/release-artifact-drift-fixtures.sh, tests/silent-exit-check-fixtures.sh, tests/stuck-claims-check-fixtures.sh, tests/substrate-preflight-hubs-toml-fixtures.sh, tests/substrate-preflight-runtime-dir-fixtures.sh, tests/sweep-debris-census-fixtures.sh, tests/version-derivation-check-fixtures.sh, tests/wake-confirm-reply-match.sh]
related_tasks: [T-2624]
created: 2026-08-20T19:30:47Z
last_update: 2026-08-27T12:52:13Z
date_finished: 2026-08-27T12:52:13Z
bvp_scores_proposed:
  - ts: '2026-08-20T22:14:42Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=3 
      (body:portability-abstraction); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-08-20T22:14:47Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: e4a00f38e801
---

# T-2824: MCP termlink_topics lost parity with the CLI

## Context

Found by running the full suite during the landing pass. One failure out of ~2880:

```
thread 'parity_topics' panicked at crates/termlink-mcp/tests/parity.rs:220
  MCP: {ok, sessions:[], total_sessions:0, total_topics:0}
  CLI: {ok, sessions:[], total_sessions:0, total_topics:0,
        sessions_probed:1, sessions_skipped:1, sessions_unreachable:1, sessions_bad_result:0}
```

**T-2624 (2026-08-12) was right.** It changed the CLI's `topics` loop from silently
`continue`-ing past a session that timed out or returned garbage, to classifying each probe
and reporting the counts — so a consumer can tell that the topic inventory is PARTIAL rather
than complete. That is a Directive #2 fix.

The MCP tool was not updated with it. Its loop is still the silent form:

```rust
if let Ok(Ok(resp)) = tokio::time::timeout(timeout, rpc_future).await
    && let Ok(result) = client::unwrap_result(resp)
    && let Some(topics) = result["topics"].as_array()
{ ... }
```

Every failure mode — timeout, transport error, error response, missing `topics` array —
falls through the same chained condition and vanishes. An agent calling `termlink_topics`
gets a topic list with no way to know a session was skipped.

**The workspace suite has therefore been red for 8 days** and nothing surfaced it. The parity
harness exists precisely to catch this and it did; it just was not being run.

## Approach

Mirror the CLI's classification in the MCP handler. The CLI's `TopicsProbe` /
`aggregate_topics_probes` helpers live in `termlink-cli`, and the established convention for
small pure helpers is to duplicate rather than share across crates (T-2069, recorded in
CLAUDE.md) — so the classification is written inline here, matching the CLI's arms exactly:

| probe outcome | CLI arm | count |
|---|---|---|
| `Ok(Ok(resp))` → `unwrap_result` Ok → `topics` array present | `Topics` | contributes topics |
| `topics` not an array, or `unwrap_result` Err | `BadResult` | `sessions_bad_result` |
| timeout, or transport `Err` | `Unreachable` | `sessions_unreachable` |

with `sessions_skipped = unreachable + bad_result` and `sessions_probed = registrations.len()`.

## Scope boundary

Adds the four counters to the MCP handler's main path. Does **not** touch the CLI. Does **not**
change the MCP tool's existing fields. Does **not** reconcile the empty-registrations early
return — see Decisions; that is a second, separate divergence which the parity test does not
exercise and which cannot be fixed from the MCP side alone.

## Acceptance Criteria

### Agent
- [x] `termlink_topics` emits `sessions_probed`, `sessions_skipped`, `sessions_unreachable`,
      `sessions_bad_result` with the same semantics as the CLI
- [x] Each probe outcome is classified explicitly — the chained `if let` is replaced by the
      CLI's three-arm match, so a timeout, a transport error, an error response and a missing
      `topics` array are now counted rather than swallowed
- [x] Existing fields (`ok`, `sessions`, `total_topics`, `total_sessions`) are unchanged
- [x] `parity` is **28 passed / 0 failed** under an isolated session registry. The 1 failure
      and the >6min stall were both harness pollution, not product defects: the suite reads the
      host's live session registry, and this host had 13 registered sessions. See 2026-08-27.
- [x] Unblocked by the same measurement — the remaining delta was never in the product.
- [x] The remaining divergences are recorded, not silently left — both the empty-registrations
      one and the reachability asymmetry found while fixing this

## Verification

# The change compiles. `cargo test --workspace` is NOT listed: parity_topics still
# fails on the reachability delta described in the update below, and listing a
# command known to fail would either block this task forever or invite --force.
cargo build -p termlink-mcp
# The four counters are emitted by the handler.
f=$(mktemp); grep -n 'sessions_unreachable' crates/termlink-mcp/src/tools.rs > "$f" 2>/dev/null; n=$(wc -l < "$f"); rm -f "$f"; test "$n" -ge 1
# Parity is green under an isolated registry. TERMLINK_RUNTIME_DIR is the isolation seam:
# the suite reads the host session registry, so an ambient session makes it non-deterministic.
d=$(mktemp -d); mkdir -p "$d/sessions"; TERMLINK_RUNTIME_DIR="$d" cargo test -p termlink-mcp --test parity > /tmp/.t2824.out 2>&1; rc=$?; rm -rf "$d"; grep -q '0 failed' /tmp/.t2824.out && ! grep -q 'test result: FAILED' /tmp/.t2824.out && test $rc -eq 0

## Decisions

### 2026-08-20 — Duplicate the classification rather than share the helper

- **Chose:** Write the probe classification inline in `termlink-mcp`.
- **Why:** `TopicsProbe` and `aggregate_topics_probes` live in `termlink-cli`, and this repo's
  convention for tiny pure helpers is duplication over a cross-crate dependency (T-2069, in
  CLAUDE.md). Making `termlink-mcp` depend on `termlink-cli` to reach a three-arm match would
  be a much larger change than the defect warrants.
- **Cost, stated plainly:** two copies of the classification can now drift. The parity test is
  what stops that — it compares the two outputs directly, which is exactly the round-trip
  assertion 832-Workflow-designer argued for over comparing implementations.

### 2026-08-20 — Leave the empty-registrations divergence, and say so

- **Context:** the two sides also disagree when there are NO sessions at all. CLI returns
  `{ok, sessions, total_topics}`; MCP returns those plus `total_sessions`.
- **Chose:** Not fixed here.
- **Why:** Removing `total_sessions` from the MCP empty path to match would be a breaking
  change to a field consumers may read, and adding it to the CLI is out of this task's scope.
  The parity test does not exercise the case (it registers a session first), so this is a
  latent divergence rather than a live failure. Recording it beats fixing it silently in a
  direction nobody asked for.

## Update — 2026-08-20: partial fix landed, remaining delta is NOT the field shape

**Done and worth keeping regardless.** The MCP handler no longer swallows probe failures. The
chained `if let` became the CLI's explicit three-arm match, and the four counters are emitted.
That is a real Directive #2 improvement on its own: before this, an agent calling
`termlink_topics` could not tell a partial inventory from a complete one.

**The parity test still fails, for a different reason.** After the change:

```
MCP: sessions_probed:1  sessions_unreachable:0  sessions_bad_result:0  sessions_skipped:0
CLI: sessions_probed:1  sessions_unreachable:1  sessions_bad_result:0  sessions_skipped:1
```

Every field is present and identically named. Both clients find the SAME single registration.
The MCP client reaches it and gets an empty topics array (the fixture session registers no
topics, so `topic_list.is_empty()` and it is correctly not listed). The CLI client, probing the
same session, times out or errors and classifies it `Unreachable`.

So the two clients disagree about whether the fixture session is reachable. That is either a
real difference in how the two resolve/connect to a session socket, or an artifact of the
harness (the session is served by the test's own in-process runtime; MCP shares that process,
the CLI is a subprocess). Distinguishing those is a genuine investigation, not a field edit.

**Stopped here on purpose.** The task was scoped as "add the four fields the CLI has"; that is
done. The remaining work is "find out why two clients see different reachability", which is a
different question with an unknown floor — and the caveat agreed before starting was to stop
rather than widen. Three things today looked two lines deep and were not.

**No regression:** `parity` was 23 passed / 1 failed before this change and is 23 passed /
1 failed after. The suite is no redder than it was.

## 2026-08-24 re-verification (partial)

The reachability delta that stopped the 2026-08-20 pass appears CLOSED:

    $ cargo test -p termlink-mcp --test parity parity_topics
    test parity_topics ... ok
    test result: ok. 1 passed; 0 failed; 27 filtered out

Field sets are now identical on both sides -- tools.rs:14184-14188 vs
events.rs:1156-1160 both emit total_sessions / sessions_unreachable /
sessions_bad_result / sessions_skipped / sessions_probed.

The two NOT DONE ACs are LEFT UNTICKED: they require the whole parity suite green,
and a full `cargo test --workspace` run could not establish that. It reached
10 suites ok / 0 FAILED and then STALLED >6min on:
    parity_version, parity_whoami_no_sessions, parity_whoami_session_match

INCONCLUSIVE, not a new failure. parity_whoami_no_sessions asserts an empty session
registry, and this run had 4 live TermLink sessions registered on the host (spawned
by the session doing this verification). Those tests shell out to the real CLI, so
the harness may be observing host state it does not control.

NEXT STEP: run `cargo test -p termlink-mcp --test parity` on a host with NO registered
termlink sessions. If 24/24, the two ACs tick and T-2824 closes. If parity_whoami_*
still hangs, that is a separate isolation defect in the parity harness and deserves
its own task -- do not re-open the topics work.

## Update — 2026-08-27: the remaining delta was never in the product

The 2026-08-24 note ended with a precise next step: *"run `cargo test -p termlink-mcp --test
parity` on a host with NO registered termlink sessions. If 24/24, the two ACs tick."* That was
the right instruction and it is now executed — with one refinement: the host does not have to be
clean, because `TERMLINK_RUNTIME_DIR` is the isolation seam. Pointing it at an empty directory
gives the suite a registry of its own without deregistering anything real.

    $ termlink list | wc -l          # 13 live sessions on this host
    $ TERMLINK_RUNTIME_DIR=<empty> cargo test -p termlink-mcp --test parity -- --test-threads=1
    test result: ok. 28 passed; 0 failed; 0 ignored; finished in 8.17s

**28/0 in 8.17 seconds.** Both symptoms were the same cause. The 1 failure (`parity_topics`
reachability delta — CLI said `Unreachable`, MCP said reachable) and the >6min stall on
`parity_whoami_*` were the suite observing ambient host state it does not control: with 13
sessions registered, `parity_whoami_no_sessions` asserts an empty registry that is not empty,
and the CLI subprocess races the in-process runtime for session sockets. Neither was a product
divergence. The 2026-08-20 pass was right to stop rather than widen — the floor was unknown and
the answer was not in `tools.rs`.

**The verification leg was rewritten, not merely re-run.** It previously asserted `23 passed`,
which is now false in the good direction; a leg that pins a known-bad count cannot witness the
fix. It now asserts `0 failed` under a `mktemp -d` registry, so it stays true as the suite grows
and fails if isolation regresses.

**The real remaining defect is the harness, not the product.** `parity` depends on ambient host
state, so it is non-deterministic between a clean CI runner and a working dev box — green here,
red or hung there, with no signal saying which. That is a distinct defect from the topics work
and is filed separately rather than folded in; do not re-open the topics work for it.

## Reviewer Verdict (v1.5)

- **Scan ID:** R-2c6efdac
- **Timestamp:** 2026-08-27T12:52:17Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** yes
- **Findings:** none

- **Layer-1 escalations:** 1
  1. **destructive-action** (high) — Destructive operation in verification or AC
     - matched: `rm -f`

### 2026-08-27T12:52:13Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
