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
