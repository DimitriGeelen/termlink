# T-3247 — CLAUDE.md preload after T-3245 and T-3246

T-2989 GO step 4. Re-measures what CLAUDE.md costs at session launch after step 2 (T-3245, the
substrate + skills reference table) and step 3 (T-3246, the canary and static-check block) moved
their content to plain-path docs under `docs/operations/`.

## Method (same as T-3244)

Three scratch projects outside the repo, identical except for CLAUDE.md, no `.claude/` settings,
no hooks. The same prompt in each: `claude -p "Reply with exactly: OK" --output-format json --max-turns 1`.
All three replied `OK`, `is_error: false`. Usage from each run's `result` record, 2026-10-24:

| variant | CLAUDE.md bytes | input | cache_creation | cache_read | **total input** |
|---|---|---|---|---|---|
| empty (`# Project`) | 10 | 2 | 21,821 | 10,341 | **32,164** |
| before T-3245 | 316,310 | 2 | 147,690 | 10,341 | **158,033** |
| now (after T-3246) | 101,808 | 2 | 61,564 | 10,341 | **71,907** |

CLAUDE.md's own cost is the total minus the empty baseline:

| | tokens |
|---|---|
| before | **125,869** |
| now | **39,743** |
| saved | **86,126 (68%)** |

This file tokenises at about 2.5 bytes per token (dense markdown, backticks, IDs), not the usual
~4. Estimates made from byte counts at 4 bytes per token undercount it by about a third, which is
why earlier estimates in T-2989 (~18k + ~37k) were below the measured cut.

## Gap to the 20k-token target

**About 19.7k tokens over.** Where the remaining 101,808 bytes sit:

| part | bytes | approx. tokens | owner |
|---|---|---|---|
| `## Core Principle` to end (framework governance) | 47,731 | ~18.6k | **framework template**: replaced wholesale by every `fw upgrade` |
| Hub Auth Rotation Protocol | 21,941 | ~8.6k | this project |
| "TermLink Substrate + Skills Reference" heading (now the pointer, plus "Session rules the governance template does not carry") | 14,061 | ~5.5k | this project; the session rules are operator rules |
| CI / Release Flow (release flow, guard index, six rules) | 7,755 | ~3.0k | this project |
| Four prover sections + Shipped==Live + Channel Topic Semantics | ~8,900 | ~3.5k | this project |
| Project overview and smaller headings | ~1,400 | ~0.5k | this project |

## What this means

1. **The 20k target cannot be met from this repo alone.** The framework governance section
   costs about 18.6k tokens by itself, and `fw upgrade` rewrites it from the template on every
   re-vendor (T-2015). Every project-owned byte could go and the total would still be about
   18.6k plus whatever stays.
2. **The next cut on our side is the Hub Auth Rotation Protocol (~8.6k tokens)**, as T-2989
   expected. It is an incident runbook (rotation symptoms, heal tiers, R1–R3), read when
   `fleet doctor` reports auth-mismatch or a TOFU violation, not on every turn. Moving it to
   `docs/operations/hub-auth-rotation.md` with a pointer that names those triggers would bring
   CLAUDE.md to about 31k tokens.
3. **The four prover sections (~3.5k)** are the same shape (what a script proves, how to read
   it) and could follow the same route to `docs/operations/`.
4. **Below ~25k needs the framework.** The governance template would have to slim itself, or offer
   its own plain-path split. That is upstream work (G-062) and is not proposed here.

Nothing in CLAUDE.md was changed by this task.
