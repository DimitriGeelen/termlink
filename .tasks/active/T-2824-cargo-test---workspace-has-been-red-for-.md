---
id: T-2824
name: "cargo test --workspace has been red for 8 days — MCP termlink_topics lost parity with the CLI at T-2624"
description: >
  parity_topics fails: CLI `topics --json` emits sessions_probed/skipped/unreachable/bad_result (added by T-2624, 2026-08-12, correctly — no silent partial inventory) and the MCP termlink_topics tool was never updated to match. Pre-existing, not from this branch: zero files touched in termlink-mcp or termlink-cli here. The parity harness is doing its job; nobody was running it.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [mcp, parity, directive-2]
components: []
related_tasks: [T-2624]
created: 2026-08-20T19:30:47Z
last_update: 2026-08-20T22:00:11Z
date_finished: null
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
- [ ] **NOT DONE — stopped deliberately.** `parity` is 23 passed / 1 failed, the same count as
      before the change (no regression), and `parity_topics` is now much closer: every field
      matches except the reachability counts. See the 2026-08-20 decision below.
- [ ] **NOT DONE** — blocked on the same remaining delta.
- [x] The remaining divergences are recorded, not silently left — both the empty-registrations
      one and the reachability asymmetry found while fixing this

## Verification

# The change compiles. `cargo test --workspace` is NOT listed: parity_topics still
# fails on the reachability delta described in the update below, and listing a
# command known to fail would either block this task forever or invite --force.
cargo build -p termlink-mcp
# The four counters are emitted by the handler.
f=$(mktemp); grep -n 'sessions_unreachable' crates/termlink-mcp/src/tools.rs > "$f" 2>/dev/null; n=$(wc -l < "$f"); rm -f "$f"; test "$n" -ge 1
# No regression: parity is no redder than before (23 passed / 1 failed).
f=$(mktemp); cargo test -p termlink-mcp --test parity > "$f" 2>&1 || true; grep -q '23 passed' "$f"; rc=$?; rm -f "$f"; test $rc -eq 0

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
