# T-1384: Multi-Agent Conversation Readiness — Inception

**Status:** spikes-running
**Phase:** inception / exploration
**Date opened:** 2026-04-28

## Problem Statement

We have shipped a 59-task agent-conversation arc (T-1325 → T-1383) that
mirrors Matrix client-server primitives onto termlink topics: mentions,
threads, edits, redactions, reactions, pins, receipts, snapshots,
incremental sync, snapshot diffs, per-event relations. All commands are
unit-tested (531 tests) and end-to-end-tested locally on a single hub
(`tests/e2e/agent-conversation.sh`, 55 numbered steps).

**The unanswered question:** every test runs on a single local hub
against two synthetic identities. We have not yet validated whether
these primitives compose correctly under the conditions they were
designed for — multiple real agents on distinct machines.

## Spike S1 — Fleet Inventory + Version Census

Run: 2026-04-28 17:16Z

### Reachability

```
local-test           127.0.0.1:9100         UP    14 sessions   86ms
ring20-management    192.168.10.122:9100    UP     1 session    43ms
ring20-dashboard     192.168.10.121:9100    AUTH FAIL — TOFU violation + secret rotated
testhub              1.2.3.4:9100           DOWN  (test fixture, ignored)
```

**Healing attempt for ring20-dashboard:** `fleet reauth ring20-dashboard
--bootstrap-from ssh:192.168.10.121` failed — root login over SSH not
configured. Skipped for this spike. (Could be fixed by configuring SSH
keys to .121 — separate operational task, not a blocker for the
inception decision.)

**Result:** 2 reachable agents (local + ring20-management). Per the
inception's DEFER criteria, this exact-2 state would normally trigger
DEFER. The user directive overrides ("run the test now, surface
feedback, fix and re-run") — so we proceed with what's available.

### Version census

```
local-test           termlink 0.9.1527
ring20-management    termlink 0.9.844     (683 commits behind)
```

### Critical finding — capability gap

The remote agent does NOT have the `channel` subcommand at all:

```sh
$ termlink remote exec ring20-management tl-aai6xg5o "termlink channel --help"
error: unrecognized subcommand 'channel'
```

The entire T-1325..T-1383 arc — all 59 commands, including the
canonical state primitives (state / snapshot / state-since /
snapshot-diff / relations) — is unreachable on remote agents until a
new binary is deployed.

### What this means for the readiness question

**We are NOT ready to push the arc to other agents in the literal
"agents drive the new commands" sense — the binary deploy step is a
hard prerequisite.**

But the arc CAN still be cross-validated using a different topology:
both client identities run locally (with the new binary) but point
at the REMOTE hub via `--hub <addr>`. That tests:

- Authentication against a remote hub
- Cross-network envelope storage and retrieval
- Canonical state computation on envelopes that physically live on a
  different machine

The hub's role is just to store and return envelopes — envelope shape
is stable, hub version is largely orthogonal to the new client
commands. This is the test we run in S2.

## Spike S2 — Cross-hub validation against ring20-management (.122)

**Result: BLOCKED at hub-RPC layer.**

```sh
$ termlink channel create xhub-test-... --hub 192.168.10.122:9100
Error: Hub rpc_call failed
  Caused by: I/O error: No such file or directory (os error 2)
```

The remote hub at .122 is running 0.9.844 and does not implement the
`channel.*` RPC namespace at all. Every `channel.create / info /
subscribe / post` against .122 fails identically.

**Attempted mitigation:** push the new binary to .122 via
`termlink remote send-file`. The 20MB transfer reported success
(417 chunks, SHA-256 verified) but the message
`peer lacks channel.post — falling back to legacy file.* events`
indicates the file went via the legacy event-bus fallback. No
file appeared at any inbox path on .122 — the legacy fallback
streams events but does not auto-persist the file. To complete
the deploy:

1. Either configure SSH key auth to .122 and `scp` the binary
2. Or have the agent on .122 actively assemble the file from the
   event stream (requires a handler the current 0.9.844 agent
   doesn't have)
3. Or stand up a tiny HTTP server on this host and have the
   agent on .122 curl the binary down

Even after binary deploy, the **hub on .122 needs to be restarted
with the new binary** — the running hub process holds the old
executable. Hub restart is service-impacting and warrants human
supervision; not in scope for this inception.

## Spike S3 — Local multi-session arc validation (substituted for S4 + S5)

After deploying the new binary to `/usr/local/bin/termlink` on
this host (one `cp` command), all 14 ready sessions resolved to
0.9.1527 immediately. Ran two tests:

### S3a. Re-run 55-step e2e against the system-PATH binary

```
PATH="/usr/local/bin:/usr/bin:/bin" bash tests/e2e/agent-conversation.sh
=== END-TO-END WALKTHROUGH PASSED ===
```

All 55 steps green. Confirms the system binary deploy is correct
and no regression vs the cargo-installed binary.

### S3b. Multi-session post test (3 live sessions, one shared topic)

```sh
TOPIC="multi-agent-real-$(date +%s)"
termlink channel create "$TOPIC" --retention forever
# Post from 3 distinct live sessions:
termlink remote exec 192.168.10.107:9100 tl-ismotg7j   "termlink channel post $TOPIC --payload 'hello-from-framework-agent'"
termlink remote exec 192.168.10.107:9100 tl-bkfp6hqt   "termlink channel post $TOPIC --payload 'hello-from-termlink-agent'"
termlink remote exec 192.168.10.107:9100 tl-pljpkait   "termlink channel post $TOPIC --payload 'hello-from-email-archive'"
termlink channel state "$TOPIC"
```

**Findings:**

1. `tl-ismotg7j` (framework-agent) has a session-level command
   allowlist that excludes `termlink channel post` →
   `Execution failed: command not in allowlist`. Expected security
   feature; documents that arc adoption requires reviewing each
   session's allowlist.

2. The other 2 posts succeeded (offset 0 and 1) and `channel state`
   rendered them correctly. **However:** both posts show the SAME
   `sender_id` (`d1993c2c3ec44c94`). The two distinct sessions
   share one identity because they run as the same OS user on the
   same machine; identity is per-user (per
   `~/.termlink/identity/`), not per-session.

3. **This is the real architectural finding from the spike:**
   running multiple Claude Code agents on one host as one user does
   NOT produce distinct conversational identities. Each session is
   a process, not an identity. A multi-agent conversation between
   such sessions appears as one identity talking to itself.

## Findings Summary

| Question | Answer |
|---|---|
| Are we ready to push the arc to other agents? | **NO**, not for cross-machine. Blocked at remote hub version (.122 lacks `channel.*` RPCs) and binary-deploy-without-SSH gap. |
| Is the arc itself correct? | **YES**. 531 unit tests + 55-step e2e green; passed twice in this session against both the cargo and the system binary. |
| Can we test multi-agent on the local fleet? | **PARTIALLY**. The local hub serves multiple sessions, but they all share one identity. Multi-identity testing requires per-session `TERMLINK_IDENTITY_DIR` overrides (which the existing e2e does, but the live sessions don't). |
| What's the cleanest next test we can run today? | The e2e already covers this. **There is no *additional* test we can usefully run with what we have today** beyond what's already passing. |

## Recommendation

**DEFER** full fleet rollout. **GO** on the arc itself for local-host
multi-identity scenarios (already validated). **Open follow-up tasks**
for the gaps:

1. **T-1385** (build): Per-session identity opt-in for the conversation
   arc. Either via `TERMLINK_IDENTITY_DIR` resolution from session
   metadata, or via a hub-side `inject_session_id_as_sender` flag on
   `channel post`. This is the highest-leverage single change to
   unlock realistic multi-agent conversations on shared hosts.

2. **T-1386** (deploy): Deploy `termlink 0.9.1527` to ring20-management
   (.122) and restart its hub. Includes the heal flow on .121 first
   (currently SSH-blocked). Human-supervised because hub restart is
   service-impacting.

3. **T-1387** (build, optional): Tier-2 reauth for SSH-blocked hubs —
   either a password-based fallback or a one-shot trust handshake
   triggered from a third party. Today the SSH key requirement is the
   single chokepoint blocking ring20-dashboard heal.

**Evidence summary:**
- S1: 2 reachable agents (.107=local, .122=ring20-management); .121 auth-broken; .107 has 14 sessions
- S2: cross-hub channel ops blocked by remote hub version (0.9.844 has no `channel.*` RPCs)
- S3a: 55-step e2e green twice (cargo binary + system binary)
- S3b: 3-session local-hub posting works at envelope level; identity is per-user not per-session (architectural gap)
- Local /usr/local/bin/termlink upgraded from 0.9.844 to 0.9.1527 with safety backup at `.0.9.844.bak`

## Dialogue Log

### 2026-04-28 17:14Z — opening prompt

**Human:** "are we ready to push to other agents, and what test can we set up with all the agents we have"

**Agent:** Drafted inception artifact with 5 spikes and explicit GO/NO-GO/DEFER criteria. Presented for review BEFORE running spikes.

### 2026-04-28 17:16Z — directive update

**Human:** "have right now (end-to-end test) to validate functioning, feedback issues, fix and run again"

**Agent action:** Pivoted from drafting to executing. Ran S1 (inventory), discovered 683-commit version gap on remote.

### 2026-04-28 17:21Z — fleet expansion

**Human:** "we have more agents actually running also on .107"

**Agent:** Probed .107, discovered .107 IS this machine (proxmox host) with 14 live sessions. Corrected mental model: "remote .107" = "local fleet via LAN IP". All sessions still on 0.9.844 because they were spawned before the fresh `cargo install`.

### 2026-04-28 17:30Z — deploy authorization

**Human:** "2"  (chose Option 2: local deploy + push to .122)

**Agent:** (a) Deployed locally: `cp /opt/termlink/target/release/termlink /usr/local/bin/termlink`, backup at `.0.9.844.bak`. Verified via remote exec on session — version now 0.9.1527, `channel state --help` works. (b) Started `remote send-file` to .122. Transfer succeeded (417 chunks) but the legacy event-bus fallback didn't persist the file. Halted further .122 work pending human review of the deploy strategy.

**Agent then ran:** 55-step e2e (passed twice) and 3-session local multi-post test (revealed per-user-not-per-session identity finding). Concluding inception with DEFER + 3 follow-up tasks.

## Dialogue Log

### 2026-04-28 17:14Z — opening prompt

**Human:** "are we ready to push to other agents, and what test can we
set up with all the agents we have"

**Agent:** Drafted inception artifact with 5 spikes and explicit
GO/NO-GO/DEFER criteria. Presented for review BEFORE running spikes.

### 2026-04-28 17:16Z — directive update

**Human:** "have right now (end-to-end test) to validate functioning,
feedback issues, fix and run again"

**Agent interpretation:** Reduce drafting; pivot to executing the
test. Run S1 (inventory) immediately, surface findings as feedback,
then run S2 (cross-hub validation) and report what works / what
doesn't / what we fix.

**Agent action:** Ran S1 → discovered version gap. Proceeding to S2
with the cross-hub topology that's actually testable today.
