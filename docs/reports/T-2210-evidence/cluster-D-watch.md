# Cluster D — Watch/Poll live-dashboard verbs (G-008 fresh re-smoke)

Generated: 2026-06-13T13:45:46Z by resmoke-agent (T-2210). Binary: target/release/termlink (built Jun 13 12:14).
All Human [REVIEW] ACs remain UNCHECKED — evidence captured for batch-confirm; sovereignty preserved.

## T-1557
`agent typers --watch` — live typing-presence dashboard.
- `timeout 6 termlink agent typers --watch` → exit=124 (streams forever, timeout-terminated = success-with-partial). ANSI clear-screen each 1s tick wipes the capture buffer = clean re-render confirmed.
- `--watch` + `--watch-interval` (clamp [1,60], default 1s) flags present in --help.
- single-shot: `No active typers on topic 'agent-chat-arc'.` exit=0.
- Verdict: **partial-stream(timeout) — ok**.

## T-1558
`agent inbox --watch` — live cross-topic unread monitor.
- `timeout 6 termlink agent inbox --watch` → exit=124 (streams, timeout = success-with-partial).
- `--watch` + `--watch-interval` (clamp [1,300], default 5s) flags present.
- single-shot exit=0, 3 topics: agent-chat-arc 1587 unread, t-1358-inbox 2, dm:9219671e… 1.
- Verdict: **partial-stream(timeout) — ok**.

## T-1559
`agent dms --watch --unread` + `agent unread --watch` — DM-only & chat-arc-only live monitors.
- `timeout 6 termlink agent dms --watch --unread` → exit=124; captured clean partial frame: header `# agent dms --watch | interval=5s | <ts>` + DM rows (28 topics, unread=2 each).
- `timeout 6 termlink agent unread --watch` → exit=124 (streams).
- Both --watch flags present; single-shots exit=0 (unread: chat-arc 4 unread, offsets 3195-3198).
- Verdict: **partial-stream(timeout) — ok**.

## T-1570
`termlink_agent_poll_*` MCP family (start/vote/end) — MCP tools, parse-confirmed via CLI equivalents.
- All 3 MCP tool names present in crates/termlink-mcp/src/tools.rs (grep count 6 = 2 refs each).
- CLI write equivalents resolve: poll-start, **vote** (CLI verb is `vote`; MCP tool is `termlink_agent_poll_vote`), poll-end; read-side poll-results. All --help exit=0.
- `termlink version --json` reports mcp_tools=270.
- Verdict: **parse-confirmed-only — ok** (MCP tool, no live MCP client invoked).
