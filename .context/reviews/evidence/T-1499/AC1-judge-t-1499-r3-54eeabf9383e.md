# T-1499 AC1 — --msg-type filtering output is operator-readable

Reviewer: judge-t-1499-r3-54eeabf9383e, revision 4751214057145e2dfca984587d8bc96e2801321f
Binary: /opt/termlink/target/release/termlink (termlink 0.12.1195), run from /opt/termlink.
Target fp: d1993c2c3ec44c94 (the local identity; its 1-year history has 190 chat, 3 status and 1 note posts).

## agent recent
- No filter, `--window-secs 86400`: header `# agent recent d1993c2c3ec44c94 (peer_fp=...) | window=86400s | n=10`, 10 chat posts (T-1438 heartbeats).
- `--msg-type note`, 86400s: header gets the suffix ` msg_type=note`; output is "(no posts found in window)". That is correct, because the 24h window holds only chat posts.
- `--msg-type note`, wide window (clamped to 604800s, which the header shows): 1 post shown, @2564 msg_type=note (T-139). The 10+ chat heartbeats are hidden.
- `--msg-type status,note`: header suffix ` msg_type=status,note`; 4 posts (1 note + 3 status); no chat posts leak through.
- `--json --msg-type chat,note`: top-level `"filter_msg_types": ["chat","note"]`; posts all chat. `--json --msg-type note`: `"filter_msg_types": ["note"]`, `posts: []`.

## agent on-thread
- T-076, no filter: 3 status posts; with `--msg-type note,status`: same 3, and the header shows ` msg_type=note,status`.
- T-1438 (30 chat posts in 24h), `--msg-type note,status`: header ` msg_type=note,status`, "(no posts found on thread=T-1438 in window=86400s)". All chat posts are excluded.
- T-139 `--msg-type note --json`: `filter_msg_types: ["note"]`, 1 note post. T-076 `--msg-type note --json`: `filter_msg_types: ["note"]`, 0 posts.
- Without the flag, the JSON has no `filter_msg_types` key, so the field appears only when the filter is set.

## Verdict: green
All three expected properties hold for both commands: only the listed types are shown, the header carries the `msg_type=<csv>` suffix, and the JSON carries `filter_msg_types`.

## Non-blocking observations / suggestions
- Matching is exact (case- and whitespace-sensitive): `--msg-type ' note , STATUS'` returns nothing, and the header echoes the raw string ` msg_type= note , STATUS`. Trimming and lowercasing each CSV item would be friendlier.
- Unknown types (e.g. a typo like `notes`) silently return empty. A stderr hint listing the known msg_types seen in the window would help.
- The window cap (604800s) is applied silently, though the header does show the effective window.
