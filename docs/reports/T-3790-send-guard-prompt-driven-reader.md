# T-3790 — Send guard refuses a second message to a prompt-driven reader

Filed upstream to AEF (999-Agentic-Engineering-Framework). Vendored framework v1.8.16-be.49.
No local patch (G-062): the guard lives in vendored `lib/`.

## Symptom

2026-11-14: mail went to `cacc73ea32b121dd/1409-sprind/design`. That is the 1409 design session,
Claude Code inside Claude Desktop. It reads its inbox through the `sidecar-inbox` hook on each
operator prompt (T-3786/T-3788). The first post went through. A second post, sent before the
operator's next prompt, was refused:

```
send: REFUSED, nothing posted — no evidence anything reads inbox:cacc73ea32b121dd/1409-sprind/design
on hub cacc73ea32b121dd: not in the peer directory, no receiver known on this host, and the hub
reports no fetch since its last post. ...
```

We sent twice with `--force`. Both messages were read on the next prompt.

## Where it comes from

1. `lib/sidecar_cli.py:149-173` `_explicit_circuit_ok` refuses unless
   `addressing.explicit_circuit_evidence` returns `known`.
2. `lib/sidecar/addressing.py:536-578` `explicit_circuit_evidence` accepts one of three kinds of
   evidence:
   1. the circuit is in the peer directory;
   2. a receiver for it is known on this host;
   3. `topic_is_read(row)` is true for its hub topic.
3. `lib/sidecar/addressing.py:496-508` `topic_is_read` is true only when both hold:
   1. the last fetch is **after the last write** (`fetch < write` returns False);
   2. that fetch is within `READ_WINDOW_MS` (7 days, :473).

So check 2.3 answers "has the reader caught up?", while the guard needs "does a reader exist?".
A reader that fetches on human prompts is always behind right after our own post. Our first post
makes every follow-up look unread until the operator next types.

## Live evidence (hub row, read 2026-11-14 ~20:30Z)

| field | value |
|---|---|
| fetches_total | 63 (tracking since ~15:05Z) |
| writes_total | 53 |
| last_fetch_ms | 1794687730751, after last_write_ms 1794687483141, so it reads as read now |

63 fetches in about 5 hours is clear evidence that a reader exists. Between a post and the next
prompt the same row shows `fetch < write`, and the guard refuses.

Peer-directory learning does not cover this case either. `learn()` (addressing.py:105) records a
circuit only from mail that circuit itself *sent*, and keys it by `circuit_name`, which is the
agent part (here `design`). That name is generic: a second project's `design` agent would be
refused as a conflict (:133-142).

## Proposed shapes (AEF's call)

A. **Split the two questions.** Count a fetch within READ_WINDOW_MS as evidence that a reader
   exists, whenever it happened relative to the last write. Keep `fetch < write` only as a
   warning, e.g. "the reader has N unread; this will queue".
B. **The reader declares itself prompt-driven.** The hook's subscribe registers a marker on the
   topic, and the guard accepts that marker.
C. **Status quo plus guidance.** The refusal text names `--force` for prompt-driven readers.
   This is cheapest, but it trains senders to use `--force`, which defeats the guard.

We recommend A. It is the smallest change and needs no new protocol. It removes no protection
against the T-4418 class: a nickname circuit that nothing has ever fetched still has no fetch at
all, so it is still refused.
