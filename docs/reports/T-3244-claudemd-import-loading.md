# T-3244 — Do CLAUDE.md `@path` imports load on demand, or are they inlined?

**Answer: INLINED AT LAUNCH.** An `@`-imported file is expanded into the preloaded
context when the session starts, whether or not anything ever needs it. An `@import`
split of CLAUDE.md therefore **cannot reduce preload**. This is the gate T-2989 put
ahead of its steps 2–3 (T-3245, T-3246).

## Measurement (2026-09-29, Claude Code 2.1.285)

Scratch dir outside the repo (`/tmp/r8-import.*`), two sibling projects, identical
except for one character in CLAUDE.md:

| variant | CLAUDE.md | `big.md` present |
|---|---|---|
| A | `# Project` / `See @big.md for reference.` | yes, 137,239 bytes |
| B | `# Project` / `See big.md for reference.` (no `@`) | yes, identical file |

The same prompt ran in each: `claude -p "Reply with exactly: OK" --output-format json --max-turns 1`.
Both replied `OK`, `is_error: false`. Usage from the `result` record:

| variant | input | cache_creation | cache_read | **total input** |
|---|---|---|---|---|
| A (`@big.md`) | 2 | 83,982 | 9,362 | **93,346** |
| B (plain path) | 2 | 23,319 | 9,362 | **32,683** |

**Delta: 60,663 tokens**, which is the whole imported file (~2.3 bytes/token for this
synthetic word list). The model never read `big.md` in either run; it answered `OK` in
a single turn. So the cost in A is paid at launch, not on use. The identical 9,362
cache_read and the near-identical baseline show the rest of the context was the same.

## What this means for T-2989

- **An `@import` is not a lazy include.** Moving the ~18k-token skills reference (T-3245)
  or the ~37k-token canary/guard block (T-3246) into a file that CLAUDE.md `@`-imports
  would save **nothing**; the bytes still land in every session's preload.
- **The only split that reduces preload is a plain reference**: a path the agent `Read`s
  when it needs it (e.g. "See `docs/operations/guard-layer.md`"), with no leading `@`.
  T-3245 and T-3246 should be scoped that way. The trade-off is that an un-imported doc
  is only seen when the agent chooses to open it, so the pointer text left in CLAUDE.md
  has to say *when* to open it.
- The **clobber boundary** is unaffected either way (T-2015): the pointer text must still
  sit above `## Core Principle`.

## Scope and limits

- One paired run. The delta equals the imported file's size, so it is not noise, but
  this measures one CLI version. Re-run the two-variant probe if a future version
  advertises lazy imports.
- It measures the top-level CLAUDE.md in the working directory. Nested/recursive imports
  and `~/.claude/CLAUDE.md` imports were not measured. Nothing here suggests they differ.
- The repository's own CLAUDE.md was not modified.
