# T-2875 — Cross-session reach into a running Claude session

**Status:** exploration complete, no decision recorded
**Date:** 2026-09-01
**Origin:** a peer agent on `.122` reported, with evidence, that reaching a running
Claude TUI session on `.107` was structurally impossible from outside.

---

## The claim under test

> *"Reaching that specific session's chat window requires an action **inside** it —
> an external process (me over SSH, or a spawned agent) structurally cannot. That's
> a Claude Code security boundary, not a config gap."*

## Verdict

**False as stated, true from where the peer stood.**

The target session is addressable from any `.107` session right now, with no setup.
The peer is on `.122`, and its own toolset was the evidence it reasoned from: it had
no `ListAgents`, so it inferred the capability does not exist. It generalised a
property of its vantage point into a property of the system.

## What the peer got right

Each verified here independently rather than taken on trust:

| claim | verdict | evidence |
|---|---|---|
| TIOCSTI keystroke injection is dead | **correct** | `dev.tty.legacy_tiocsti = 0` |
| no `claude send`/`message`/`peer` subcommand | **correct** | `claude --help` |
| Remote Control cannot be retrofitted | **correct** | `--remote-control` is a *launch-time* flag |
| a bare `claude -p` one-shot lacks the peer tools | **correct** | consistent with the above |

The retrofit limit is not unique to Remote Control. TermLink's own doorbell has it
too — PL-237: *"running headless claudes cannot be retrofitted — arm at relaunch."*
Three mechanisms, one shared structural truth: **reachability is decided at launch.**

## The measurement

Two rails exist and they see different worlds.

| | population | scope | enrolment |
|---|---|---|---|
| Claude Code `ListAgents` | **14** sessions | **`.107` only** | automatic |
| TermLink `agent-presence` | **3** listeners | **3 hosts** (`.107`/`.122`/`.121`) | opt-in at launch |

Scope was established by **absence**, which is the stronger direction: `ring20-concierge`
(`.122`) and `ring20-dashboard-agent` (`.121`) are LIVE on TermLink's rail and appear
in none of the 14 `ListAgents` rows. So `ListAgents` here is same-machine-only, and the
peer on `.122` genuinely cannot see `.107`.

## Where the peer's recommendation would have hurt

It proposed `termlink register --name dimitri-mint-dev --shell`, claiming *"then I own
it completely… and I inject straight into it."*

`register --help` is explicit: `--shell` = *"Start a PTY-backed session (full
bidirectional I/O)"*. It **starts** a PTY; it does not attach to the caller's terminal.
`--self` is *"event-only endpoint (no PTY)"*.

T-2873 measured what injecting into such a session actually does: the bytes landed at a
**bash prompt**. So that advice hands out a remote shell on the host — materially more
dangerous than a chat channel — and still would not put text into the conversation.

## End-to-end test of delivery (IW-2)

Run on a session spawned and owned for the purpose. No live session was touched.

1. `claude --bg` a target in a scratch dir, with a standing instruction to write any
   received message to `receipt.txt`.
2. Confirm it is addressable — `cross-session message relay [51d6ec]`, `bg`, `idle`.
3. `SendMessage` carrying sentinel `HARNESS-PROBE-T2875-9f3c1a`. Returns `success:true`.
4. Assert **independently of that claim**, in the target's own transcript JSONL.

**Result: delivery works.** The sentinel appears twice in
`~/.claude/projects/<slug>/<sessionId>.jsonl` — once as `queue-operation`, once as a
**`user` turn** — and the target woke from `state: done` and acted on it.

### The finding that matters more than the pass

`receipt.txt` **never appeared.** Not because delivery failed, but because the target
blocked on a permission prompt:

```
"status": "waiting", "waitingFor": "permission prompt", "state": "blocked"
```

`--permission-mode acceptEdits` did not cover creating that file. **From the sender's
side this is byte-identical to non-delivery** — `success:true`, no observable effect.

This is the same class of error as T-2873, where a hub reported `injected` and only a
PTY read proved arrival, and as T-2874, where a config looked authoritative and did
nothing. A layer reporting success is not evidence that the next layer acted. Three
instances in two days is a pattern, and it is the reason the harness below asserts on
the *receiver's* state rather than the sender's return value.

Three outcomes look the same from outside and must be distinguished:

| observed | actual state | how to tell |
|---|---|---|
| no effect | **delivered**, target blocked on permission | `claude agents --json` → `waitingFor` |
| no effect | **delivered**, target chose not to act | sentinel is a `user` turn in transcript |
| no effect | **not delivered** | sentinel absent from transcript |

## Evaluation: is there value in building anything?

**For the stated problem, no.** It already works from `.107`, today, with no setup.

**For the general cross-host problem, also no** — the mechanism exists and demonstrably
works across three hosts. What does not work is **enrolment**: 14 Claude sessions live on
`.107`, exactly 1 on the TermLink rail. That is an adoption question, not a transport one,
and this repo already instruments it (`fleet-adoption-snapshot` cron).

Building a third rail would be precisely the breadth accretion T-2483 exists to prevent.

## Recommendation

**NO-GO on new transport. GO on the prover** (delivered as T-2876), because the thing
actually missing was never a mechanism — it was the ability to tell a working rail from
a broken one without guessing.

## Open questions still open

**IW-2 is answered for `bg`→`bg` on one machine.** Not tested: an `interactive` TUI
target (the original case), and cross-machine delivery with Remote Control connected —
neither is available to test from here without touching a session in use.
