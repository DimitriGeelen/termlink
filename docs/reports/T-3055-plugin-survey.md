# T-3055 — Which plugins actually help this project?

**Honesty note on method (C-001):** the research artifact should be created BEFORE the
research. It was not — four governance gates fired on the way in and the scan ran first.
Recorded rather than backdated.

## What is installed

19 plugins: **13 enabled, 6 disabled**.

## The measurement that matters

Grepping the repo for each plugin's own subject, to separate "installed" from "used":

| plugin | repo references | enabled | reading |
|---|---|---|---|
| **playwright** | **330 files** | yes | mandated by CLAUDE.md §Visual Verification |
| context7 | 23 | yes | genuinely referenced |
| qdrant-skills | 9 | yes | some relevance (Penelope, not TermLink) |
| **rust-analyzer-lsp** | 4 | yes ×2 | **BROKEN — see below** |
| chrome-devtools-mcp | 0 | yes | overlaps playwright |
| data-engineering | 0 | yes | purpose mismatch |
| frontend-design | 0 | yes | purpose mismatch |
| superdesign | 0 | yes | purpose mismatch |
| modern-web-guidance | 0 | yes | purpose mismatch |
| ralph-loop | 0 | yes | governance risk — see below |

Project shape for context: **7 Rust crates**, a bash guard layer, a small Flask dashboard.
No TypeScript outside a rollback directory.

## Finding 1 — rust-analyzer is enabled, duplicated, and BROKEN

The highest-value plugin for a 7-crate Rust workspace does not run:

```
LSP workspaceSymbol -> "LSP server plugin:rust-analyzer-lsp:rust-analyzer
                        crashed with exit code 1"
```

It is also listed **twice** in `claude plugin list`, which is a plausible cause (two
installs shadowing each other) and is worth ruling out first.

This is the value-review prompt's non-use **Reading A: BROKEN — wanted, does not work**,
not "unused". Intent evidence is strong: someone installed it deliberately, it is enabled,
and the codebase is overwhelmingly Rust. The right class is **ADD (REPAIR)**, and the
non-use is the cost of the defect rather than evidence against the tool.

Fixing it would make `goToDefinition` / `findReferences` / call-hierarchy available across
the crates — exactly the operations that were done by grep all session.

## Finding 2 — the DISABLED set is correctly chosen, and two must stay off

This is worth stating because "enable everything" would actively break governance:

- **github** — CLAUDE.md: *"NEVER push to GitHub. Only push to OneDev."* GitHub is a
  read-only mirror. Enabling a GitHub plugin invites precisely the prohibited action.
- **commit-commands** — commits here must carry a task reference and go through
  `fw git commit` (P-002 traceability). A generic commit helper routes around that gate.
- **typescript-lsp** — no TypeScript in the project. Correctly off.

## Finding 3 — four enabled plugins are a purpose mismatch

`data-engineering`, `frontend-design`, `superdesign`, `modern-web-guidance`: zero repo
references and no plausible fit for a Rust CLI/daemon plus an internal Flask dashboard.

The positive reason is purpose mismatch, not absence of use — which is the only kind of
reason that supports removal. They are not free: every enabled plugin competes for tool-list
budget and attention.

## Finding 4 — ralph-loop is a governance question, not a tooling one

An autonomous-loop plugin sits directly on top of what AEF already governs: work selection,
gates, stop conditions, Sovereign questions. Two loop controllers with different rules is
not additive. **Surfaced as a question, not a recommendation** — it touches authority.

## Recommendation, ranked

1. **Repair rust-analyzer** (de-duplicate first). Highest value, measured defect, Rust-heavy repo.
2. **Keep** playwright (mandated), context7 (referenced).
3. **Consider disabling** the four purpose-mismatch plugins, and chrome-devtools-mcp as a
   playwright overlap.
4. **Leave github / commit-commands disabled** — deliberately, on governance grounds.
5. **Decide on ralph-loop** — Sovereign.

## What this does NOT establish

Zero repo references is not proof a plugin is useless; it is exactly the "absence of use is
not a verdict" trap. For the four mismatch plugins the argument rests on positive purpose
mismatch, not on the reference count. No plugin was disabled or enabled by this survey.


---

# ADDENDUM — the standalone constraint (operator, 2026-09-22)

> *"Key dependency for plugins is that they are independent and can run stand alone and
> don't have any dependency on a service commercial or non-commercial."*

This is a sharper filter than "is it useful", and it reverses one recommendation above.
Measured with `claude plugin details`, which reports each plugin's component inventory —
an **MCP server** is the tell for a process that may contact something external; **skills**
are inert prompt text; an **LSP server** is a local binary.

| plugin | components | always-on | standalone? |
|---|---|---|---|
| **rust-analyzer-lsp** | 1 LSP, 0 MCP | **~0 tok** | **YES — local binary** |
| **context7** | **1 MCP** | ~0 tok | **NO — remote docs service** |
| playwright | 1 MCP | ~0 tok | yes — drives a LOCAL browser |
| chrome-devtools-mcp | 7 skills, 0 MCP | 806 tok | yes (skills only) |
| modern-web-guidance | 2 skills | 759 tok | yes |
| feature-dev | 1 skill, 3 agents | 240 tok | yes |
| superdesign | 1 skill | 190 tok | yes |
| skill-creator | 1 skill | 114 tok | yes |
| ralph-loop | 3 skills, 1 Stop hook | 86 tok | yes |
| frontend-design | 1 skill | 80 tok | yes |
| qdrant-skills / data-engineering | **not reported** | — | UNMEASURED |

## The one that fails the constraint

**context7 is the only enabled plugin that depends on an external service.** It ships an
MCP server that resolves library documentation from a remote, commercially-operated API.
I endorsed it in the main survey on the strength of 23 repo references — under this
constraint that endorsement does not stand, and it is the one enabled plugin to reconsider.

The trade is real, so it is stated rather than decided: context7 exists to stop an agent
answering from stale training data about a library. Dropping it removes a service
dependency and restores that failure mode. That is a judgement about which risk this
project would rather carry, which makes it the operator's.

## Cost, now measured (closes IW-3, previously deferred)

**~2,275 tokens always-on, every session**, across the skills-bearing plugins.

Of that, **1,835 tokens — 81% — belongs to four plugins with zero repo references and a
purpose mismatch** for a Rust daemon plus a small Flask dashboard: chrome-devtools-mcp
(806), modern-web-guidance (759), superdesign (190), frontend-design (80). That is the
cost figure the main survey said it did not have; the disable recommendation no longer
rests on fit alone.

## Two not measured

`qdrant-skills` and `data-engineering` returned no inventory from `claude plugin details`.
Recorded as UNMEASURED rather than assumed absent — the tool was asked and did not answer,
which is not the same as a plugin with no components.

## What changed on disk

`rust-analyzer` was the survey's top repair candidate and is now **fixed** (T-3056): the
rustup component was missing behind a shim. It is also the cleanest fit for this
constraint — local binary, no MCP server, ~0 tokens. It immediately reported dead-code
diagnostics in `crates/termlink-mcp/.../inbox_channel.rs` that nothing else had surfaced.

---

# ADDENDUM 2 — context7: use, exposure, value (operator question, 2026-09-22)

## First, a correction to this document

The main survey said *"context7: 23 files — genuinely referenced"*. That figure was
inflated and the inference was wrong. Deduplicated, the references are **~4 distinct
documents**: a governance design report (T-908), a spec (T-532), the `.mcp.json` /
`settings.local.json` config, and a completed task (T-1111) — the other ~19 hits are
copies inside five `.claude/worktrees/` snapshots, plus this survey citing itself.

More importantly: **none of those references are usage.** They are configuration and
policy mentions. "Referenced" was doing work in that sentence that the evidence does not
support.

## How it would be used

`resolve-library-id` then `query-docs`: the agent names a library, the server returns
current documentation. Its purpose is to stop an agent answering from stale training data
about a fast-moving API.

## Exposure

**A. Unpinned remote code execution at every session start — the material one.**

```json
"context7": { "command": "npx", "args": ["-y", "@upstash/context7-mcp"] }
```

`npx -y` with **no version pin**: whatever is published on npm at that moment is fetched
and executed, auto-confirmed, with the session's privileges — root on this host. A
compromised publish of that package runs here immediately, with no review step and no
version to audit. And `.mcp.json` is **git-tracked**, so this applies to every clone, every
developer and every CI checkout, not just this machine.

This is not unique to context7: `playwright` is `@playwright/mcp@latest`, the same class.
Two of the four configured MCP servers fetch remote code at startup; `termlink` and `fw`
run local commands and do not.

**B. Query egress to a commercial third party.** Library name and query text go to
Upstash's API. Not source code — but the query text is agent-composed and can carry
intent and architecture ("how do I do X in rmcp for a hub that does Y").

**C. Availability coupling.** A remote service sits in the startup path of a project whose
own charter prizes working when things are down.

## Value — honestly, UNMEASURED

No usage trace exists: nothing under `.context/working/` or `.context/audits/` names
context7, and per-tool invocation telemetry is confirmed absent (T-3044 Q4 — T-2996 shipped
the writer while the running MCP server predates it, so the sink file does not exist).

So the correct reading is **D: UNMEASURED**, not E: not wanted. I cannot say it is unused;
I can say nobody can currently tell. That gap is itself the finding.

**What its value would be, structurally, for THIS repo:** lower than for a typical
JS/Python project. The codebase is 7 Rust crates, and Rust already ships a better answer
locally — `cargo doc` generates documentation from the *exact versions pinned in
Cargo.lock*, which is strictly more accurate than any third-party index, works offline, and
costs nothing. Where context7 could genuinely help is the Flask/Watchtower side and fast-
moving crates like `rmcp`.

## The asymmetry

The exposure is **certain and continuous** — unpinned third-party code fetched and executed
every session, on every clone. The value is **unmeasured** and structurally reduced by a
local toolchain that already answers the same question more accurately.

That asymmetry, not the standalone constraint alone, is the argument.

## Recommendation

1. **If context7 goes:** use `cargo doc` / docs.rs for Rust. Removes a service dependency,
   a commercial one, and one of the two unpinned-npx vectors. Consistent with the operator's
   standalone constraint.
2. **Regardless of that decision — pin playwright.** It passes the standalone constraint
   (local browser) and CLAUDE.md mandates it, so it stays; but `@latest` should become a
   pinned version. This is a small, high-value change that is independent of the context7
   question and should not wait on it.
3. **Before deciding on value:** if the answer matters, instrument first. Per-tool telemetry
   (T-2996) is built and not live — that is the ADD that converts this from judgement to
   measurement.
