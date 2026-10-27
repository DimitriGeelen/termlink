# T-1496 Human AC#1 — live overview as "leave it running" dashboard

Reviewer: judge-t-1496-r3-18cef88239bd. Revision 5d637f5b7af7652e736a9deb5d4f3ca96e3e0602.

## What I ran
From /opt/termlink, under a pty (`script`), SIGINT after 17s (= Ctrl-C):
`target/release/termlink agent overview --watch --watch-interval 5 --window-secs 86400 --top 5`
Raw transcript: `watch-transcript-judge-t-1496-r3-18cef88239bd.txt` (same dir).

## Findings
- 4 ticks rendered at 15:03:08Z, :13Z, :18Z, :23Z — 5s cadence exact.
- Each frame starts with `ESC[2J ESC[H` (clear + home) then a header
  `# agent overview --watch | interval=5s | window=86400s | top=5 | <RFC3339>`,
  followed by Top Peers / Top Projects / Recent Posts / footer. ~25 lines,
  fits a standard terminal; no partial or interleaved frames; no stray output.
- Ctrl-C (SIGINT) ended cleanly: COMMAND_EXIT_CODE=0, no panic/backtrace.
- Flicker: code (crates/termlink-cli/src/commands/agent.rs:3222-3243) clears the
  screen BEFORE the fetch, so the screen is blank (header only) for the fetch
  duration. Measured one-shot fetch+render: 0.05/0.09/0.07s against the local
  hub — imperceptible. On a slow/remote hub this blank gap would grow (advisory:
  fetch first, then clear+render, would eliminate it).
- New activity: each tick re-fetches `agent-chat-arc` (fetch_topic_msgs_paginated)
  and re-renders via the shared render_overview_body; Recent Posts is ordered
  oldest→newest (46m-ago post last, just above footer), so a new post would appear
  at the bottom on the next tick. No new fleet post arrived during my 15s run, so
  arrival was confirmed by code path + per-tick refetch, not observed live. I did
  not post a test message (would publish to the fleet hub). Fetch errors print
  `# fetch error (will retry on next tick)` rather than exiting — good for an
  unattended dashboard.
- Cosmetic: long post bodies truncated with "…" — keeps frame width stable.

## Verdict: green
Steady, clean redraw, useful sections, clean exit. Advisory only (non-blocking):
fetch-then-clear to avoid blank gap on slow hubs.
