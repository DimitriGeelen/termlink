# Reviewer evidence — T-1498 AC#1

- dispatch: `judge-t-1498-r3-d96c36ad13a0`
- worker kind: `codex` (vendor `openai`, model `gpt-6-astra`)
- seat: `codex`
- reviewed revision: `490db4951c1adaa00fb4d0124e9ec4786ac77377`
- recorded by: the dispatch runtime on the worker's behalf (T-3582)
- result.jsonl sha256: `e89d237f5d3233570b0fb2c09e9b4687570dd404311e7701b67a6627ac6ce565`
- result.md sha256: `c2daff746fa9524276288494ac346a8f548f34fc6b2fa7cbd921853dddd2346e`

## The worker's verdict block

```
1. [AC] Live single-peer view is steady and useful
VERDICT: not-evaluated
WHY: In /opt/termlink, the release binary exists and its help lists --watch and --watch-interval. However, no target fingerprint was supplied, and termlink list returned “No active sessions.” I could not observe a live peer across 2–3 ticks, verify flicker-free redraws, or confirm new posts appear at the bottom.
GUIDANCE: Provide an accessible live peer fingerprint and observe the command in a visual terminal across 2–3 refreshes, including an arriving post, then Ctrl-C. For a fingerprint, the installed CLI requires --target-fp <fingerprint>; the positional argument accepts a session name.

Summary: 0 green, 0 amber, 0 red, 0 escalate, 1 not-evaluated
```

## The worker's full output (result.md)

```
1. [AC] Live single-peer view is steady and useful
VERDICT: not-evaluated
WHY: In `/opt/termlink`, the release binary exists and its help lists `--watch` and `--watch-interval`. However, no target fingerprint was supplied, and `termlink list` returned “No active sessions.” I could not observe a live peer across 2–3 ticks, verify flicker-free redraws, or confirm new posts appear at the bottom.
GUIDANCE: Provide an accessible live peer fingerprint and observe the command in a visual terminal across 2–3 refreshes, including an arriving post, then Ctrl-C. For a fingerprint, the installed CLI requires `--target-fp <fingerprint>`; the positional argument accepts a session name.

Summary: 0 green, 0 amber, 0 red, 0 escalate, 1 not-evaluated
```
