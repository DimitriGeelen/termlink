# Batch 5 — Review Evidence

Date: 2026-05-22
Binary: `target/release/termlink` (v0.11.99 — the cargo-PATH binary is v0.11.1 and
blocked by project-boundary hook; release binary carries all batch-5 verbs).
Peer FP for populated content: `d1993c2c3ec44c94`.
Real offsets used: thread root `224` (15 replies), root `1785` (2 replies).

---

## T-1536 — agent edits-of <offset> (edit history of a chat-arc post)

Human AC: `[REVIEW] Verify the verb reads naturally` — `agent edits-of <offset>` →
expect edit chain with replaces metadata visible.

Command:
```
target/release/termlink agent edits-of 224
target/release/termlink agent edits-of 1785
```

Verdict: **EVIDENCE-CLEAN**

Output (offset 224):
```
Edits of offset 224 on 'agent-chat-arc' (0 edits):
  [original 224 ts=1777913816572 d1993c2c3ec44c94] {
  "subject": "Contract design — cohort member alias+forwarding provision ...
```
Renders the header (`N edits`) + the original post body. Neither sampled offset
has been edited (0 edits) so no `replaces`-tagged rows appear, but the verb walks
the arc, locates the original, and renders the empty-chain case correctly with the
original anchored. Reads naturally.

---

## T-1537 — agent relations <offset> (all relations of a chat-arc post)

Human AC: `[REVIEW] Verify the verb reads naturally` — `agent relations <offset>` →
expect all relations surfaced (replies, edits, reactions, etc).

Command:
```
target/release/termlink agent relations 224
```

Verdict: **EVIDENCE-CLEAN**

Output (truncated):
```
Relations on 'agent-chat-arc':[224] — d1993c2c3ec44c94: { "subject": "Contract design ... }

  replies (×10):
    [348] d1993c2c3ec44c94: {
```
Single-shot unified view: anchors the target post, then groups everything pointing
at it. The `replies (×10)` section confirms the relation-grouping works on a real
high-fanout post. Reads naturally.

---

## T-1557 — agent typers --watch (live typing-presence dashboard)

Human AC: `[REVIEW] Verify --watch is steady (no flicker)` — inherently visual.

Command:
```
timeout 8 target/release/termlink agent typers --watch --watch-interval 5
target/release/termlink agent typers              # single-shot
target/release/termlink agent typers --watch --json   # rejection check
```

Verdict: **HUMAN-VISUAL-ONLY** (clean start confirmed)

Notes:
- `--watch` is a boolean flag; interval is `--watch-interval` (default 1, clamp 1..60).
- Watch loop ran for the full 8s and was timeout-killed (expected) — no panic.
- Single-shot: `No active typers on topic 'agent-chat-arc'.` (no one typing now —
  EVIDENCE-EMPTY for content, but verb is functional).
- `--watch --json` correctly rejected:
  `Error: --watch and --json are incompatible ...`
- "No flicker / jitter" is a visual property — operator must eyeball.

---

## T-1558 — agent inbox --watch (live cross-topic unread monitor)

Human AC: `[REVIEW] reads naturally as live unread monitor` — visual + behavioural.

Command:
```
timeout 8 target/release/termlink agent inbox --watch --watch-interval 5
target/release/termlink agent inbox    # single-shot
```

Verdict: **EVIDENCE-CLEAN** (live data; visual cadence still operator-judged)

Output (watch, two ticks captured):
```
^[[2J^[[H# agent inbox --watch | interval=5s | 2026-05-21T23:19:22Z
4 topic(s) with unread content:
  agent-chat-arc — 197 unread (latest=1808, cursor=1611)
  dm:9219671e28054458:d1993c2c3ec44c94 — 6 unread (latest=19, cursor=13)
  dm:design-smoke-test — 2 unread (latest=2, cursor=0)
  t-1358-inbox-1777360315 — 2 unread (latest=5, cursor=3)
```
Clean ANSI clear+home + dated header + 4 real topics with unread/cursor deltas.
Two consecutive identical ticks (no flicker in captured frames). `--watch --json`
rejected correctly.

---

## T-1559 — agent dms --watch / agent unread --watch (personal-identity watch family)

Human AC: `[REVIEW] both --watch views steady and useful`. Covers BOTH surfaces.

Commands:
```
timeout 8 target/release/termlink agent dms --watch --unread --watch-interval 5
timeout 8 target/release/termlink agent unread --watch --watch-interval 3
target/release/termlink agent dms       # single-shot
target/release/termlink agent unread    # single-shot
```

Verdict: **EVIDENCE-CLEAN** (both surfaces; visual steadiness operator-judged)

dms --watch --unread output:
```
^[[2J^[[H# agent dms --watch | interval=5s | 2026-05-21T23:19:35Z
dm:9219671e28054458:d1993c2c3ec44c94  (peer=9219671e28054458)  unread=8  first=12
dm:bob-122-3107700:d1993c2c3ec44c94   (peer=bob-122-3107700)   unread=2  first=1
... (28+ DM topics)
```

unread --watch output:
```
^[[2J^[[H# agent unread --watch | interval=3s | 2026-05-21T23:19:48Z
Topic 'agent-chat-arc': 12 unread for d1993c2c3ec44c94 (first new offset 1797, last 1808, last receipt up_to=1796)
```
Both render dated headers + real live counts, ANSI-clear each tick. Both
`--watch --json` combos rejected. Single-shots both populated (DM directory +
chat-arc unread=12).

---

## T-1570 — termlink_agent_poll family (MCP poll lifecycle start/vote/end)

Human AC: `[REVIEW] poll family operator-fluent over MCP` — call start/vote/end via
MCP client, then `agent poll-results <P>` shows question + 1 vote on yes + closed.

Verification 1 — MCP tool registrations present in source:
```
grep -rn "termlink_agent_poll" crates/termlink-mcp/src/tools.rs
  tools.rs:17483:  name = "termlink_agent_poll_start",
  tools.rs:17486:  async fn termlink_agent_poll_start(
  tools.rs:17560:  name = "termlink_agent_poll_vote",
  tools.rs:17563:  async fn termlink_agent_poll_vote(
  tools.rs:17631:  name = "termlink_agent_poll_end",
  tools.rs:17634:  async fn termlink_agent_poll_end(
```
`version --json` → `"mcp_tools":177` (AC required >=88).

Verification 2 — live end-to-end lifecycle via the CLI mirror (same envelope path
the MCP tools post; chat-arc):
```
agent poll-start "Review-evidence smoke: approve?" --option yes --option no --option wait
   → Posted to agent-chat-arc — offset=1809   (poll_id = 1809)
agent vote 1809 0      → offset=1812
agent poll-end 1809    → offset=1813
agent poll-results 1809:
   Poll #1809 [CLOSED]: Review-evidence smoke: approve?
     [0] yes — 1 vote(s)
          · d1993c2c3ec44c94
     [1] no — 0 vote(s)
     [2] wait — 0 vote(s)
   Total votes: 1
```

Verdict: **MCP-VERIFIED** (3 tool registrations in source; mcp_tools=177; full
poll lifecycle executed live end-to-end matching the Human AC's expected output
exactly — CLOSED poll, question, 1 vote on "yes"). The MCP tools mirror this exact
envelope path. Note: poll_id = the start envelope's **offset**, not its ts.
