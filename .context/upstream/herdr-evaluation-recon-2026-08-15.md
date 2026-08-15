# Herdr evaluation — reconnaissance only, NOT a recommendation (2026-08-15)

**Status: INCOMPLETE.** This is the first 15 minutes of an investigation that was
stopped by the context budget gate at ~100%. It contains verified reconnaissance
and a strategic reframe of the question. **It does not contain a recommendation,
and nothing here should be acted on as one.**

Filed in `.context/` rather than `docs/reports/` because the budget gate confines
writes to `.context/` `.tasks/` `.claude/` at critical. C-001 wants the research
artifact in `docs/reports/T-XXXX-*.md` — move it there when the investigation
resumes under a real inception task.

---

## The question as asked, and why it is the wrong shape

Asked: *"will we benefit from adopting herdr.dev, replacing our current tmux
solution?"*

That frames herdr as a **backend swap** — tmux out, herdr in, behind
`termlink spawn --backend tmux`. The reconnaissance suggests that framing is
wrong, and the real question is materially bigger.

**Herdr is not a terminal multiplexer.** It is a background server providing
persistent PTY sessions *purpose-built for coding agents*, written in Rust,
with:

- automatic per-agent state tracking — `working` / `blocked` / `done` / `idle`
- a **socket API + CLI as the same surface agents drive**
- agents able to "spawn panes, prompt each other, and wait until another agent
  is genuinely blocked"
- client-server over a local Unix socket, or an SSH tunnel for remote
- ~20 agent CLIs detected (Claude Code, Codex, Cursor, opencode, Grok, …)
- survives lid-close, network drop, SSH disconnect

Read that list against TermLink's charter verbs:

| Charter verb | TermLink | Herdr |
|---|---|---|
| 1. discover | `agent-presence`, `find-idle`, cv_index | agent state tracking (`idle`/`working`/`blocked`) |
| 2. exchange durable messages | topics, retention, offline queue, acks | "prompt each other" (durability unknown) |
| 3. claim work | `channel claim`/`renew`/`release`, leases | "wait until another agent is genuinely blocked" (not a lease) |
| 4. control terminal sessions | `spawn`/`exec`/`inject`, tmux backend | **its entire core** |

**So the overlap is not with tmux. It is with TermLink itself** — squarely on
verb 4, and partially on verbs 1 and 3. Adopting herdr as "a tmux replacement"
would quietly import a second, competing answer to three of the four charter
verbs. That is a strategic decision about what TermLink *is*, not a backend swap,
and it deserves an inception task with a go/no-go — not an implementation task.

The genuinely interesting version of the question is the opposite one: **does
herdr's existence mean TermLink should narrow its charter to the three verbs
herdr does not do well (durable messaging, lease-based claims, fleet/hub
federation) and delegate verb 4?** That is worth asking seriously. It is also
exactly the "over-built in breadth" concern the T-2468 purpose review raised.

---

## Verified facts

- Rust, single binary (~10MB), no Electron
- macOS + Linux; **Windows beta**
- Client-server; background session server owns the PTYs
- Socket API documented at `herdr.dev/docs/socket-api/`
- Does **NOT** embed or depend on tmux — it is an alternative, not a wrapper
  (it offers "tmux-style prefix keys" as a familiarity feature only)
- Very fast traction: #1 GitHub Trending 2026-06-30; project is young (~105 days
  at time of the secondary reports)

## UNRESOLVED — sources contradict each other

Do not proceed past this table without settling it. Every row is
decision-critical and at least two sources disagree.

| Fact | herdr.dev landing | GitHub repo page | Secondary/search |
|---|---|---|---|
| **License** | Apache 2.0 | Apache 2.0 | **AGPL-3.0-or-later**, commercial license sold separately |
| Version | v0.8.0 | v0.4.0 | — |
| Stars | — | 29.2k | ~15k |
| Repo identity | — | `herdrdev/herdr` | also `ogulcancelik/herdr` |

**The license row is the gate.** Apache 2.0 and AGPL-3.0 are opposite answers for
a project that ships binaries to users:

- Apache 2.0 → link/ship freely, no reciprocal obligation.
- AGPL-3.0-or-later → strong copyleft with a **network clause**. If TermLink
  linked herdr, or arguably if a TermLink hub exposed herdr-backed sessions over
  the network, the combined work's source obligations propagate. The existence of
  a *paid commercial license* is itself evidence that the AGPL reading is real
  for some version of the project — nobody sells an exception to Apache 2.0.

Plausible explanations, none yet confirmed: a relicense during the project's
short life (0.4 → 0.8), a fork under different terms, or a secondary source
simply being wrong. **Resolve by reading the `LICENSE` file at a pinned tag in
the canonical repo**, not a badge, not a blog, not a landing page.

The version conflict (0.4.0 vs 0.8.0) matters independently: a ~105-day-old
project at v0.x, moving fast enough to disagree with its own docs, is
pre-stability by definition. That is a real risk for something proposed to sit
under the founding charter verb.

---

## What the investigation still needs (none of it done)

1. **Settle the license** at a pinned tag. Gate on it — if AGPL, most of the
   rest is moot for a shipped binary and the question becomes "optional local
   dev tool?" rather than "replace the backend".
2. **Map our actual tmux surface.** Known entry points from this session:
   `termlink spawn --backend tmux`, `scripts/session-selftest.sh` (the verb-4
   prover: SPAWN → EXEC → CLEANUP), `scripts/tl-claude.sh`, `termlink exec`,
   the Terminal.app/`osascript` backend. Quantify: how much code, how isolated
   is the backend seam, is there already a backend abstraction to slot into?
3. **Read the socket API** against what TermLink needs from a session backend —
   inject, capture, exit codes, PTY resize, signal delivery.
4. **Check Directive #4 (Portability).** T-2693's platform-lock check exists
   precisely because README claims macOS support five times. Herdr's Windows is
   beta; confirm macOS is first-class, since our own CI for macOS (T-2692) is
   still non-blocking.
5. **Weigh the strategic question above** — backend swap vs charter narrowing.
   This is the human's call, not the agent's.
6. **Consider the null option honestly.** tmux is ~20 years old, ubiquitous,
   packaged everywhere, and our integration already works and is proven daily by
   the T-2557 session-control canary. "Boring and proven" is a real position,
   and the burden of proof is on the replacement.

## Anti-recommendation

Do **not** let this be decided by the traction numbers. 29k stars in 105 days is
evidence of interest, not of fitness for a load-bearing dependency under a
charter verb. The relevant questions are the license, the API fit, and whether we
want a second answer to verbs 1/3/4 inside the codebase at all.
