# T-2683 — TermLink purpose review #3: is the guard layer itself executed by anything?

**Type:** inception (exploration → go/no-go)
**Status:** exploration complete, recommendation GO on the in-authority subset
**Predecessors:** T-2468 (product-vs-charter), T-2678 (charter-vs-guards)

---

## The question

TermLink's purpose and goals have now been critically reviewed twice:

| Review | Question asked | Answer | Response |
|---|---|---|---|
| T-2468 | Does the **product** match the charter? | Over-built in breadth | P4 pruned 52 social-analytics tools |
| T-2678 | Does anything **enforce** the charter? | Verbs 4/4, non-goals 2/5 | Built 5 guards (T-2569, T-2679–T-2682) |

Both concluded *add more guards*. A third review that concludes the same thing is a
review that has stopped working — it would be measuring the same axis a third time
and calling the reading a finding.

So this pass asks the next question down, and the only one that makes the prior two
meaningful:

> **The guard layer now has 28 check scripts, 10 fixture suites, 24 crontabs, and
> 2055 workspace tests. What executes it? And when a guard reports green, is that
> evidence?**

This is the same question T-2680 asked of one canary — *is its green measuring
anything?* — asked of the layer as a whole. PL-271 (from T-2483) already states the
governing principle: **a recurring human "go review X" mandate is itself the symptom
of a missing structural check.** This review applies PL-271 to the reviewers.

---

## Method

1. Enumerate every automated execution path in the repo (CI workflows, OneDev
   buildspec, git hooks, cron) and ask what each actually runs.
2. Grep the entire tree for *any* reference to the four static checks outside their
   own implementation — a guard nothing calls is inert.
3. Run every static check and every fixture suite by hand to establish whether the
   tree is currently green (distinguishing "green because guarded" from "green
   because nobody looked").
4. Read the live canary log state on this host, not just the scripts, because the
   scripts' exit-code discipline and the *crontab's* logging discipline are two
   different things and only one of them is what the operator reads.
5. Size the live tool surface against the charter's four named verbs to find
   anything the charter cannot classify.

Step 4 is the one prior reviews skipped. Both earlier passes read the canary
*scripts* and confirmed their exit-code contracts were correct. Neither read the
*crontab line* that captures their output, which is where the contract is actually
consumed — and where it is broken.

---

## Finding F1 — the source-level guard layer is executed by nothing (headline)

### What runs automatically

| Path | What it actually executes |
|---|---|
| `.github/workflows/release.yml` | `cargo build --release` — **no `cargo test`** |
| `.github/workflows/doc-lint.yml` | `check-error-code-docs.sh`, `check-env-var-docs.sh` — 2 of 28 checks |
| `.github/workflows/install-check.yml` | build + binary smoke + command-hint lint |
| `.onedev-buildspec.yml` | mirror push to GitHub only |
| pre-push audit | `Sections: structure` — task/YAML/arc structure only |
| 24 crontabs | the 17 runtime canaries (these *are* automated) |

### What that leaves unexecuted

- **2055 workspace tests.** Nothing runs `cargo test`. The release pipeline builds a
  binary and publishes it; a test regression ships.
- **4 source-level static checks** (`alloc-sink`, `drain-sink`, `silent-exit`,
  `busy-spin`). Tree-wide grep for any reference outside their own implementation
  returns only: their own scripts, their own fixture suites, `.context/episodic/*`
  (records of the tasks that *created* them), and `learnings.yaml`. **No CI job, no
  cron entry, no git hook, no `fw doctor` step, no aggregate runner invokes them.**
- **10 fixture suites** under `tests/*.sh`. Nothing references `tests/` in any CI
  config.

### Why this is the finding and not a nitpick

The split is exactly along the axis that matters. The **runtime** guard layer — the
17 cron canaries watching hubs, wakers, queues, claims — is genuinely automated and
fires daily. The **source-level** guard layer — the checks that guard *the code* —
is entirely manual. And the code is the thing that changes on every commit.

The four static checks exist specifically because a convention enforced by
discipline is not enforced. From `check-alloc-sink-clamps.sh`'s own header:

> The repo has a strong "clamp every numeric caller param" convention … but the
> convention is **by discipline, not enforced** — two instances in one window means
> the mechanism recurs.

The check was built to convert discipline into structure. But the check itself is
invoked only by discipline. The guard layer has the disease it was built to cure —
one level up, where nothing is watching.

Current state on this tree (all run by hand for this review):

```
alloc-sink   rc=0   0 unacknowledged (101 sink calls scanned)
drain-sink   rc=0   0 unacknowledged (6 sink calls scanned)
busy-spin    rc=0   0 unacknowledged (14 long-poll loops scanned)
silent-exit  rc=0   0 unacknowledged (39 non-zero-literal exits scanned)
fixtures     6 + 6 + 8 + 7 = 27 assertions, 0 failed
```

Green — because the previous session ran them by hand. Note `alloc-sink` now scans
**101** sink calls where CLAUDE.md documents 98: three new allocation sites entered
the tree since that paragraph was written, and the only reason we know they are
clamped is that this review happened to run the check. That is the mechanism, caught
mid-act.

### Scope note on responsibility

This is not an argument that CI should run everything. `cargo test --workspace` on
this tree is minutes of compute and the release workflow is cross-compiling for
several targets. The finding is narrower and harder to argue with: **there is no
single command that runs the guard layer**, so neither a human, an agent, nor a CI
job can execute it even when they want to. That is the gap worth closing first,
because it is the precondition for every other answer.

---

## Finding F2 — every canary merges tooling errors into its findings log

### The defect

All 17 runtime canaries share one operator contract, stated identically in CLAUDE.md
seventeen times:

> Empty log = healthy. Any entry = *(the specific fault)* needs operator action.

And they share one exit-code contract, stated identically:

> exit 0 = healthy · exit 1 = firing · exit 2 = tooling error

The exit-2 class exists deliberately, and T-2557 spells out why:

> This split keeps a firing log meaningful: it fills **ONLY** when session control
> genuinely broke, never on a transient hub-down.

**The crontab throws that split away.** Every canary job line is:

```
… bash scripts/check-<x>.sh --quiet >> .context/working/.<x>-canary.log 2>&1
```

`2>&1` merges stderr into the findings log. A check that *cannot run* writes into the
log that means *the thing you are watching is broken*. **19 of 19 canary job lines
across the 24 crontabs use this idiom** — it is universal, not an oversight in one
file.

### It has already happened

Live on this host right now:

```
$ cat .context/working/.release-mirror-canary.log
error: origin HEAD empty
```

`check-mirror-freshness.sh` itself is correct — it exits 2 on this condition (line
60) and 1 on real drift (line 174), and run by hand today it reports `rc=0`,
`GitHub mirror: synced`. The script did its job. The crontab captured its stderr
into the drift log anyway.

Per CLAUDE.md, an operator reading a non-empty release-mirror log is directed to:

> inspect OneDev job log, rotate `github-push-token` if expired, re-fire the mirror job

So the current state sends an operator to rotate a GitHub credential in response to a
check that simply could not determine `origin` HEAD. Wrong diagnosis, real work,
zero underlying fault.

### The compounding harm

Worse than one false positive: **the signal is now permanently destroyed for that
canary.** "Empty log = healthy" is a one-bit channel. Once a tooling error dirties
the log, a subsequent *genuine* mirror drift appends to an already-non-empty file and
changes nothing an operator can see. The canary that exists to prevent G-058 — a
16-day silent mirror failure — is currently in a state where a real mirror failure
would be indistinguishable from the noise already sitting in its log.

`/canaries` (T-2172) partly compensates by surfacing signal-bearing lines, but it
classifies this canary as FIRING, which is exactly the wrong verdict: nothing is
wrong with the mirror.

### Charter trace

Directive #2, Reliability: *"Predictable, observable, auditable execution; no silent
failures."* A monitoring layer that cannot distinguish "the thing broke" from "I
couldn't look" is not observable, and its failure to look is silent.

---

## Finding F3 — the charter cannot classify 32 live tools

Live tool surface, by category, from the binary's own `help --json` (214 live of 260
total):

| Charter verb | Categories | Live tools |
|---|---|---|
| 1 · discover | `agent_presence`, `tofu` | 22 |
| 2 · exchange durable messages | `channel`, `agent_read`, `channel_admin`, `agent_thread`, `agent_inbox`, `channel_threading`, `channel_moderation`, `agent_chat`, `events` | 96 |
| 3 · claim work | (claim/renew/release/transfer within `channel`) | ~8 |
| 4 · control terminal sessions | `session`, `execution`, `batch`, `files`, `dispatch` | 23 |
| — acknowledged off-charter | `agent_stats`, `agent_thread_health`, `agent_rankings`, `channel_engagement` | **28** |
| — **traces to no named verb** | `fleet` 12, `diagnostics` 12, `hub` 8 | **32** |

Two observations, in order of importance.

**The unnamed block is bigger than the acknowledged off-charter block.** 32 tools of
pure fleet/diagnostics/hub operations sit outside every verb the charter names. They
are almost certainly *legitimate* — Directive #2 demands observability, and this
review is itself only possible because those tools exist. But the charter does not
say so. That has a concrete consequence: the charter-drift canary cannot ask about
them (they are not analytics-shaped), and T-2548's subtract-or-keep decision does not
cover them (it is scoped to the 28). They are in a blind spot that no review — this
one included — has authority to close, because the charter's wording is
human-sovereign.

**The founding verb is the smallest.** "Control terminal sessions" — the thing
TermLink began as — is 23 live tools against 96 for messaging. That is not
necessarily wrong (the charter is explicit that TermLink *grew into* a coordination
layer), but it is the kind of ratio a purpose review should state out loud rather
than leave for a fourth pass to rediscover.

---

## Finding F4 — the MCP/CLI parity harness covers a quarter of its own stated scope

F1 predicted that an unrun suite would be hiding something. It was: `parity_topics`
had been red since 2026-08-12. T-2624 added four partial-inventory fields to the CLI's
`topics --json` — explicitly so "a consumer can now tell the topic set excludes
sessions that timed out or errored" — and never added them to the MCP
`termlink_topics` tool, whose `if let … && let … && let …` chain dropped every failed
probe into silence. An agent calling the MCP tool received a truncated inventory
presented as complete: Directive #2 fixed on the human-facing surface and left broken
on the agent-facing one, which has more consumers.

Fixing that surfaced a **second** defect underneath it, which the first had masked:
`parity_topics` ran on the default current-thread tokio runtime while `call_cli`
blocks the test thread in `.output()`, starving the session's accept loop so the CLI
subprocess could never be accepted. The file's own comment listed `parity_topics`
among tests that "do not need the socket" because it is *hub-less* — conflating "needs
no hub" with "needs no socket" when it starts a SESSION the CLI must RPC. With the
multi_thread flavor the test completes in 0.06s instead of burning two 5s timeouts.

The wider gap is the harness's coverage, and the harness documents it itself:

> v0.1 scope: session-control thin slice. 3 cases … **v0.2+ expands to channel_* (53
> pairs) and chat-arc agent_* (the divergence-heavy group).**

v0.2 never happened. Today: **24 parity cases against 68 distinct `*_mcp` parallel
helper functions (693 call sites)**, and the group the harness itself names as
divergence-heavy has **zero** coverage. T-2687 proves the class is live — a real
divergence sat undetected inside a *covered* pair, so the ~44 uncovered ones have no
detection whatsoever.

Filed as **T-2689** at `horizon: next` and deliberately **not built here**: expanding
parity to 50+ pairs is an arc, not a tail-end of this session, and doing it badly
would produce exactly the kind of guard this review exists to criticise.

---

## What is NOT wrong

Stated explicitly, because a review that only finds problems is not calibrated:

- **The 17 runtime canaries genuinely run.** They are wired to cron, they heartbeat,
  and the meta-canary (T-1723) watches their freshness. F2 is a defect in how their
  output is captured, not evidence that they are dark.
- **The exit-code contracts in the check scripts are correct.** Every script sampled
  implements 0/1/2 properly and fails closed. The defect is in the crontab.
- **The static checks are well-built.** Narrow anchors, drift-stable allowlists with
  cited reasons, hermetic fixtures, and each one proven load-bearing by reverting a
  fix and observing the fire. The problem is exclusively that nothing calls them.
- **T-2681's allowlist migration holds.** All five allowlists are tracked under
  `.context/checks/` and the checks resolve them tracked-first; this worktree — a
  fresh checkout with no `.context/working/` history — scans clean, which is the
  reproducibility property T-2681 was built for, now confirmed on a second checkout.
- **The charter's four verbs are each provable and each watched.** T-2678 established
  prover + canary coverage at 4/4 and nothing here contradicts it.

---

## Gap register

| # | Gap | Severity | Authority | Disposition |
|---|---|---|---|---|
| **G1** | No single command runs the source-level guard layer; nothing automatic invokes the 4 static checks, 10 fixture suites, or 2055 tests | **high** | agent | **BUILD** |
| **G2** | 19/19 canary crontabs merge exit-2 tooling errors into the exit-1 findings log; live false positive on release-mirror | **high** | agent | **BUILD** |
| **G3** | Nothing detects G2 recurring — a new canary crontab written with the old idiom reintroduces it silently | medium | agent | **BUILD** (G-019 half of G2) |
| **G4** | Release pipeline publishes binaries without running `cargo test` | medium | agent (CI edit) | **BUILD** |
| **G5** | 32 live tools trace to no charter verb; charter is incomplete as a discrimination instrument | medium | **human** | **DEFER** — charter wording is human-sovereign (T-2470 precedent) |
| **G6** | Founding verb (session control) is the smallest surface at 23 tools vs 96 for messaging | low | **human** | **DEFER** — product-shape question, T-2548 adjacent |
| **G7** | MCP `termlink_topics` returned a silently partial inventory; `parity_topics` red since 2026-08-12 | **high** | agent | **BUILD** (T-2687) |
| **G8** | `check-silent-exit` blind to a bare exit hidden behind a comment | medium | agent | **BUILD** (T-2688) |
| **G9** | Parity harness covers 24 of 68+ parallel implementations; `agent_*` group at zero | medium | agent | **FILED** (T-2689, `horizon: next`) — an arc, not a task |

---

## Recommendation

**GO on G1–G4.** All four are mechanical, testable, and require no product decision:
they make existing guards executable and make existing signals honest. None of them
changes what TermLink is; they change whether the things that already claim to
protect it actually do.

**DEFER G5 and G6** to the human. Both are charter-wording or product-shape
questions. T-2470 set the precedent that the canonical purpose sentence is
human-blessed, and G5 is a proposal to *extend* it — that is proposal, not decision,
per the Authority Model. Recording them here rather than acting on them is the point.

---

## Assumptions registered

- **A1** — Adding a `cargo test` gate to `release.yml` does not materially slow the
  release path. *Confidence: medium.* Mitigated by gating once, as a prerequisite job,
  rather than inside each cross-compile matrix leg.
- **A2** — Splitting canary stderr from stdout will not lose diagnostic information
  the operator needs. *Confidence: high.* The stderr stream is preserved to a sibling
  `.stderr` log rather than discarded, so nothing becomes less observable.
- **A3** — No existing consumer parses the canary logs in a way that a sibling stderr
  file would break. *Confidence: high.* `/canaries` auto-discovers `*-canary.log` by
  glob; a `.stderr` suffix does not match that glob.

---

## Outcome

| Gap | Task | Result |
|---|---|---|
| G1 | **T-2684** | shipped — `scripts/run-guard-layer.sh`: one command, 21 members (8 static checks + 13 fixture suites), declared membership via a `# guard-layer: source` header marker, PASS/FAIL/**ERROR**/SKIP verdicts, findings-dominate roll-up. 27 fixture assertions. |
| G2+G3 | **T-2685** | shipped — all 30 canary job lines across all 24 crontabs migrated to `2>> <log>.stderr`; `check-canary-log-hygiene.sh` fires on `2>&1` **and** on `2>/dev/null`; 19 fixture assertions; the polluted release-mirror log truncated after independently re-confirming the mirror synced. |
| G4 | **T-2686** | shipped — `doc-lint.yml` gains a per-commit `guard-layer` job; `release.yml` gains a `test` job (`cargo test --workspace` + guard layer) that both build jobs `needs:`, so a red suite blocks the build outright. |
| G7 | **T-2687** | shipped — MCP `termlink_topics` now carries the four partial-inventory fields with CLI-identical probe classification (extracted to a unit-tested pure helper); both zero-session early-return paths emit the full zeroed field set; `parity_topics` given the `multi_thread` flavor it always needed. |
| G8 | **T-2688** | shipped — `check-silent-exit` now skips comment-only lines in all three places it reads context; 4 new fixtures pin the *rule* rather than the tree's current shapes. |
| G9 | **T-2689** | filed at `horizon: next` — parity expansion is an arc, not a task. |
| G5 | — | **deferred to human** — charter wording is sovereign. |
| G6 | — | **deferred to human** — product-shape, T-2548 adjacent. |

### The guard layer caught its first defect within minutes of existing

Not a claim about future value — an observation from this session. `T-2687`'s new MCP
handler was written with `Vec::with_capacity(total_probed)`. Running
`run-guard-layer.sh` immediately fired `check-alloc-sink-clamps` on it: a bare
identifier reaching an eager allocation. The correct resolution was not an allowlist
entry but a code fix — sizing from `registrations.len()` directly, so the capacity is
bounded by an allocation that has already succeeded, which is both true and visible to
the checker. Code written and caught inside the same session, by a check that had
existed for months and been run by nobody.

### Honest correction

This session's earlier report stated the workspace suite was "2055 tests, 0 failures".
That was wrong: `parity_topics` was red, and had been since 2026-08-12. The error is
itself an instance of F1 — a suite nothing runs is a suite whose state you are
guessing at.

---

## Dialogue Log

### 2026-08-14 — framing

**Human mandate (verbatim):** *"please ultra critically review termlink's purpose and
goals and identify gaps or needed adjustement, incept these and build these and test
these, drive to comopletion"* — the same mandate that produced T-2468 and T-2678.

**Course correction taken without being asked:** the mandate's third issuance is
itself the strongest available evidence for PL-271. Rather than run the same axis a
third time, this pass turned the instrument on the previous two reviews' output and
asked whether their deliverables execute. They largely do not. Had this pass repeated
the earlier framing, it would have added a sixth and seventh guard to a layer that
nothing invokes, and reported that as progress.

**On the Tier-0 gate:** T-2678's `fw inception decide` was blocked as Tier 0 and was
not bypassed; the same applies here. G1–G4 were built because each stands on its own
as a defect fix and needs no inception GO to justify. G5/G6 are recorded and left for
the human. The formal go/no-go on this artifact remains the human's to record.
