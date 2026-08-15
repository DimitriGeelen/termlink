# Herdr adoption research — ready-to-run dispatch briefs

**Why this file exists.** The operator approved dispatch for up to 8 termlink
agents on 2026-08-15. The `check-agent-dispatch` gate was thereby cleared — but
the **budget gate** (116% of context window) independently blocks all Bash
except git commit/push and `fw handover`, so `fw termlink dispatch` could not be
run from that session. These briefs are written out so a fresh session fires
them immediately without re-deriving anything.

**5 workers, within the 8 approved.** Each is independent — no shared files, no
sequential dependency. Worker 6 (synthesis) runs only after 1–5 return.

## Shared preamble — prepend to EVERY worker prompt

```
Working dir: /opt/termlink/.claude/worktrees/charter-review-2026-0814 (a git
worktree — do not cd out of it). READ-ONLY reconnaissance: do not modify source.

READ FIRST for settled context, do not relitigate:
  .context/upstream/herdr-evaluation-synthesis-2026-08-15.md
  .context/upstream/herdr-external-findings.md
  .context/upstream/herdr-internal-tmux-surface.md

Settled: we are NOT replacing our session layer with herdr. Its socket API
cannot return exit codes or deliver signals; TermLink owns its PTYs natively
(crates/termlink-session/src/pty.rs:103-120). herdr is Apache-2.0 as of v0.8.0
(relicensed 2026-07-22 from AGPL-3.0-or-later + commercial).

LICENCE RULE: adopting an IDEA is always fine. Copying CODE needs attribution +
NOTICE handling. Flag any category-(b) recommendation explicitly.

METHODOLOGY — this repo spent a week cataloguing findings that were wrong
because one measurement was generalised past what it measured. Cite a source for
every claim. When you say "TermLink lacks X" you must have LOOKED — cite
file:line for where you looked. An unverified absence is the exact error we keep
making. If a count surprises you, especially a zero, suspect your own query and
print the matching lines before reporting it. If a hook blocks your tools, say
so plainly and mark which claims are therefore unverified.

For each recommendation give: what to adopt, the evidence in herdr, whether
TermLink already has it (cite our file:line — VERIFY), effort estimate, and
which Constitutional Directive it serves (D1 Antifragility, D2 Reliability,
D3 Usability, D4 Portability).
```

---

## Worker 1 — terminal correctness (highest expected value)

```
fw termlink dispatch --name herdr-w1-terminal --prompt '<preamble>

SCOPE: herdr has ~142 open issues, largely terminal edge cases: ConPTY mouse
loss, CSI-u leakage, OSC responses printed to screen, an 80x24 pane floor.
These are the hard-won lessons of a PTY implementation under heavy real use.
THEIR BUG REPORTS ARE PROBABLY WORTH MORE TO US THAN THEIR CODE — each is a
test case we may be missing.

Go through their open and closed issues systematically. For each terminal-
correctness class, determine whether TermLink handles it: read
crates/termlink-session/src/pty.rs and crates/termlink-cli/src/commands/pty.rs
(1263 lines, already known to contain the inject/output/resize/attach/stream/
mirror surface). Produce a table: issue class -> does TermLink handle it ->
evidence (file:line or "no handling found, searched X") -> severity for us.

End with a ranked list of concrete regression tests we should add.
Write findings to .context/upstream/herdr-adopt-1-terminal-correctness.md
Return under 500 words: top gaps only.'
```

## Worker 2 — agent-state detection as a fallback

```
fw termlink dispatch --name herdr-w2-state --prompt '<preamble>

SCOPE: herdr classifies agents working/blocked/done/idle by screen-scraping the
bottom buffer against TOML manifests, with ~20 agent CLIs detected. We judged
this weaker than our protocol-based presence (and it is — their docs concede new
prompts misclassify as idle rather than blocked). BUT it works on agents that
never opted in, and ours does not.

Question: is a HYBRID worth having — protocol presence when available,
heuristic fallback when not? Read their TOML manifest format in detail and
report exactly how it works. Then assess our side: where would a fallback slot
in (agent-presence topic, find-idle, /peers)? What would it cost in false
positives, and does that break the liveness contract our canaries depend on?

Be genuinely critical — a heuristic that lies is worse than an absence, and
T-2387s waker-liveness canary depends on presence meaning something exact.
Write findings to .context/upstream/herdr-adopt-2-agent-state.md
Return under 500 words with a clear recommend/do-not-recommend.'
```

## Worker 3 — CLI/socket-API unification vs our parity drift

```
fw termlink dispatch --name herdr-w3-parity --prompt '<preamble>

SCOPE: herdr deliberately makes its CLI and socket API THE SAME SURFACE. We
maintain CLI/MCP parity BY HAND, and it has drifted — parity_topics has been
failing since 2026-08-12 (T-2624 added four fields to the CLI topics --json and
never to the MCP tool; fixed in T-2687, found only when T-2686 wired cargo test
into CI).

Question: is their approach structurally better at preventing drift, and could
we adopt it? Read how they achieve the shared surface (codegen? single
dispatch table? macro?). Then examine ours: how many CLI verbs have MCP twins,
how is parity currently asserted, and what would a structural fix look like
(shared registry, generated bindings, a parity test that enumerates rather than
spot-checks)?

This is the highest-leverage item if it works — it converts a whole class of
recurring bug into a compile-time or generated invariant.
Write findings to .context/upstream/herdr-adopt-3-parity.md
Return under 500 words.'
```

## Worker 4 — reattach and persistence UX

```
fw termlink dispatch --name herdr-w4-persistence --prompt '<preamble>

SCOPE: herdr survives lid-close, network drop, SSH disconnect, and reattaches
from any terminal. Determine HOW (client-server over Unix socket, SSH tunnel
for remote — get the detail), then assess TermLinks equivalent honestly.

We have registration + heartbeat + the be-reachable/pushwaker rail, but we also
have a documented history of frozen husks (T-2230/T-2235/T-2239), dead wakers
(T-2387), and stale waker code (T-2405) — three canaries exist because this area
keeps breaking. Is their architecture structurally more robust here, and if so
what specifically would we borrow?

Do not confuse "they have fewer canaries" with "they have fewer problems" —
they may simply not be looking. Check their issues for persistence failures.
Write findings to .context/upstream/herdr-adopt-4-persistence.md
Return under 500 words.'
```

## Worker 5 — distribution and onboarding

```
fw termlink dispatch --name herdr-w5-distribution --prompt '<preamble>

SCOPE: herdr ships a single ~10MB binary, brew install, auto-detects ~20 agent
CLIs, and went 0 to 29k stars in ~141 days. Our install story involves a hub,
runtime_dir configuration with a known PL-021 volatile-/tmp footgun, secret and
TLS pinning, hubs.toml, and a preflight script with six checks.

Question: what specifically makes their onboarding frictionless, and which of it
is adoptable WITHOUT giving up the guarantees our complexity buys? Be concrete —
"simplify the install" is not a finding. Look for: what they auto-detect and
how, what they defer until first use, what defaults they pick, how they avoid
a configuration file on the happy path.

Cross-reference docs/operations/substrate-getting-started.md and
scripts/substrate-preflight.sh for our current cold-start sequence.
Write findings to .context/upstream/herdr-adopt-5-distribution.md
Return under 500 words.'
```

---

## Worker 6 — synthesis (run ONLY after 1–5 return)

```
fw termlink dispatch --name herdr-w6-synthesis --prompt '<preamble>

Read all five of .context/upstream/herdr-adopt-{1..5}-*.md. Produce a single
ranked adoption backlog: what to do, in what order, with effort and directive.

Separate firmly into: (a) ideas to adopt — free; (b) code to copy — needs
Apache-2.0 attribution + NOTICE; (c) tempting but rejected, with the reason.

Then file the ones worth doing as real tasks via fw work-on, one deliverable
each per the task-sizing rules. Do NOT create a task for anything a worker
could not verify — carry those forward as open questions instead.

Write to .context/upstream/herdr-adoption-backlog.md
Return under 600 words: the ranked list and the tasks you filed.'
```

---

## Note for whoever runs these

Two gates were involved and only one is cleared. The operator approved dispatch
(clearing `check-agent-dispatch`, limit was 2). The **budget gate** is separate
and blocks Bash above ~100% context — it is why these were written rather than
run. In a fresh session both are satisfied and workers 1–5 can fire in parallel
immediately.

Worker 3 (CLI/API parity) is the highest-leverage item and the one I would run
first if only one could be run: it targets a defect class that has already cost
us a silent CI failure lasting weeks, and a structural fix would retire the
whole class rather than one instance.
