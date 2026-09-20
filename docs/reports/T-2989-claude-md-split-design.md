# T-2989 — CLAUDE.md split design (clobber-safe, <20k token preload)

**Status:** exploration complete, recommendation written
**Task:** T-2989 (inception, `owner: human`) · **Arc:** arc-009
**Origin:** `docs/reports/VALUE-REVIEW-repo-2026-09-19-consolidated.md`, finding C-04 / S-15

## Why this artifact exists

C-001: the thinking trail is the artifact. Written before the research, filled as
measurements landed.

The task arrived carrying a **pre-filled `## Recommendation: GO`**, written at
creation time by the value review — before any exploration existed. It is treated
here as a hypothesis, not a conclusion. (It survives, but for different reasons
than it states, and with a materially different scope.)

## Measurements

All taken 2026-09-20 against the working tree.

### Size (IW-1)

| quantity | value |
|---|---|
| lines | 3,283 |
| bytes | 276,749 |
| **estimated tokens** | **~69,000–77,000** (bytes ÷ 4 … ÷ 3.6) |
| stated target | < 20,000 |
| **overshoot** | **~3.5–3.8×** |

The filing said preload cost "is measured as excessive". That is correct and
**understated**: on a ~300k window this single file is roughly a **quarter of the
entire context** before the session has read one line of code.

### The clobber boundary (IW-3)

`## Core Principle` is at **line 2461**. `fw upgrade` replaces everything from
there to EOF with the framework template.

| half | lines |
|---|---|
| above boundary — survives, project-owned | 2,460 |
| below boundary — **destroyed on upgrade** | 823 |

**CLAUDE.md's own record of this is stale.** Line 1149 states: *"Measured
2026-08-20: this file is 2493 lines, `## Core Principle` is at 1650, so 844 lines
are in the destroyed half."* Actual today: 3,283 lines, boundary 2461, 823
destroyed. The file has grown **+790 lines** since that measurement and
**essentially all of it landed above the boundary** (surviving half +811;
destroyed half −21).

That is not incidental. It is the mechanism.

### Where the bytes actually are

| section | bytes | ~tokens | share |
|---|---|---|---|
| Canary + guard-layer documentation block (L40–1948) | 134,340 | ~37,300 | **48.9%** |
| `### TermLink Substrate + Skills Reference` (L1951) | 64,643 | ~17,950 | 23.5% |
| `### Hub Auth Rotation Protocol` (L2012) | 21,671 | ~6,020 | 7.9% |
| everything else | ~54,000 | ~15,000 | ~19.7% |

**Two blocks are 72% of the file.** The single largest section is ~18k tokens —
by itself it nearly equals the entire <20k target.

### On-demand mechanisms (IW-2)

- `@`-imports used in CLAUDE.md today: **0**.
- Skills present in `.claude/commands/`: **34**.

## Findings

### F1 — The clobber-safety advice is the bloat mechanism

T-2015 correctly established that anything below `## Core Principle` is destroyed
by `fw upgrade`, and instructs: *"Add project-specific rules HERE, never below
`## Core Principle`."* Following that instruction is what produced +811 lines in
the surviving half in one month.

So the two problems are in **direct tension**: the remedy for clobber-safety
drives content into the preload, and the remedy for preload cost drives content
out of the clobber-safe region. Any design that treats them as independent will
trade one for the other. This is the finding the filing did not have.

### F2 — The largest section duplicates a mechanism that is already on demand

`### TermLink Substrate + Skills Reference` (~18k tokens) is a table describing
operator verbs — and **34 of those verbs already exist as individual skill files**
in `.claude/commands/`, which the harness surfaces by name and description without
preloading their bodies. The table is a second, eagerly-loaded copy of a catalogue
that the skills mechanism already provides lazily.

This is the T-2839/PL-362 shape: *a deduplication that adds a shared definition
without deleting the copies leaves both.* Here the copies were never deleted
because nobody measured what they cost.

### F3 — The canary block has a better-placed home that already exists

The ~37k-token canary/guard block is reference material read *when a canary fires*,
not every session. Each canary's remediation text **already exists** in its own
check script header and in `docs/operations/`. The CLAUDE.md copy is a third
location.

### F4 — The load-bearing unverified fact

Everything above is measured. One fact is **not**, and the whole design turns on it:

> Does a `@path` import in CLAUDE.md defer loading, or is it inlined into the
> preload at session start?

If imports are **inlined** (my expectation, confidence 2 of 3), then "split into
imported files" saves **zero tokens** and the only real lever is moving content to
genuinely on-demand surfaces — skills, `docs/operations/`, script headers. If
imports are **lazy**, a much cheaper split is available.

A design that assumes the wrong answer here produces a refactor that moves 55k
tokens between files and reduces preload by nothing. **This must be confirmed
before any build work starts.** It is cheap to confirm and expensive to assume.

## Recommendation

**GO — with a scope materially different from the filing's.**

The filing's rationale ("preload cost measured on every session; the upgrade
clobber boundary makes the layout fragile") is confirmed. But its implied remedy —
reorganise the file around the clobber boundary — would make the preload problem
*worse*, because the clobber-safe half is the preloaded half (F1).

Recommended scope, in dependency order:

1. **Confirm F4 first.** One question, cheap, and it decides whether steps 2–3 are
   a token reduction or a no-op. Do not start the refactor before this answer.
2. **Reduce `### TermLink Substrate + Skills Reference` to a pointer** (~18k → <1k).
   Highest ratio of tokens-saved to risk, because the content is already available
   on demand through the skills the table describes (F2).
3. **Move the canary/guard block to `docs/operations/`, leaving a short index**
   (~37k → ~2k). Remediation text already lives in each check script's header (F3).
4. Re-measure. Steps 2–3 alone target ~77k → ~22k; the remaining gap to <20k comes
   from the Hub Auth Rotation Protocol (~6k), which has the same treatment.

**Explicitly OUT of scope:** relocating content across the `## Core Principle`
boundary as a goal in itself. That is the tension in F1 and solving it is a
separate decision — and arguably a Sovereign one, since it interacts with the
pending re-vendor decision (T-2950/T-2949): a re-vendor rewrites the destroyed
half, so what lives there is a governance question, not a token-budget question.

### Answer to IW-4 — what stops moved content from never being read?

Not nothing, and this is why steps 2–3 are safe while a naive split is not:

- The skills table's content is reached by **invoking the skill**, which is how
  operators reach it anyway.
- The canary block is reached when `/canaries` fires, and `/canaries` already
  names the failing canary — whose script header carries the remediation.

Both have an **existing, triggered path to the reader**. Content with no such path
must not be moved out; it must be deleted or kept. That is the discriminator the
build task should apply section by section.

## Side finding (not this task's scope)

CLAUDE.md's Context Budget section documents an absolute ladder — 120k warn, 150k
urgent, **170k = critical BLOCK**. The live gate classified a measured **182,786
tokens as `level: ok`**. The documented ladder and the enforcing gate disagree.
Filed as evidence for **T-2986** ("Fix stale CLAUDE.md factual claims"), together
with the stale 2493-line measurement at line 1149.
