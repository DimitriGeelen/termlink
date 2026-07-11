# MCP tool-description policy (arc-005 mcp-slimming)

Every `description = "..."` on a termlink MCP tool (in `crates/termlink-mcp/src/tools.rs`)
is loaded into **every agent's context, every session**. Baseline at arc start:
**273 tools, ~156KB (~39k tokens)**, worst single description 11,751 chars, 24 over 1000,
94 over 600. This is a fleet-wide context tax and — with many long, prose-heavy
descriptions — raises the odds a model malforms the argument object (observed: an agent
looped 9× on a rejected `recent_dm` call).

The `scripts/test-mcp-desc-budget.sh` guard enforces this policy mechanically (per-tool
ceiling + total ceiling), tightened as each slice lands.

## What STAYS (agent-critical — never cut)

- **One-line purpose.** What the tool does, in a sentence. The single most important line.
- **Non-obvious param semantics.** Only what the JSON schema does NOT already convey:
  mutual-exclusivity (`peer` XOR `topic`), substring-vs-exact match behavior, clamps that
  bite (`limit` 1..=200), units (`since_hours` vs `since_ms`), what an empty string means.
- **Critical safety / footgun notes.** "WRITES state", "Tier-0", "one-shot snapshot on
  first call only", "read-only" — anything whose omission causes a wrong or destructive call.
- **Return-shape hint** when the caller must destructure it (`{ok, parsed:{...}}`).

## What GOES (move it, don't lose it)

- **Task-ID archaeology.** `T-1862 wrapper from T-1863`, `PL-188 seek-to-tail + PL-189
  timeout + PL-191 sender priority`, lineage chains. Provenance belongs in the task files /
  `docs/`, not in the per-call context of every agent forever. Relocate genuinely-useful
  rationale to a doc and drop the ID soup.
- **Restatement of the schema.** Param types, defaults, and ranges are already in the
  JSON schema the model receives. Do not repeat `(default 20, clamped 1..=200)` in prose
  when the schema field says the same — keep prose only for behavior the schema can't state.
- **Sibling-tool cross-references.** "Read-side asymmetric to X, completes the trio with
  Y and Z" — orientation prose. One short "see also" at most; the rest goes to a doc.
- **Redundant re-statement.** Descriptions that say the same thing three ways. Say it once.

## Target shape (example)

Before (~1500 chars): `recent_dm` — full T-XXXX/PL lineage + trio prose + every param
re-described + federation caveats.

After (~200-300 chars):
> Per-peer DM history. Discovers `dm:*` topics by SUBSTRING match on `peer` across all
> hubs in `hubs.toml` (so generic strings match many topics; response lists all matched).
> `peer` XOR `topic` required. Read-only. Returns `{ok, parsed:{summary, topics, posts}}`.
> DM topics may not federate — the per-hub walk surfaces that rather than hiding it.

## Process per slice

1. Trim descriptions in the slice's band per the keep/cut rules above.
2. `cargo build -p termlink-mcp` passes; tool COUNT is unchanged (trim text, never remove tools).
3. **Lower the guard ceiling** (`MAX_DESC_CEILING` / `TOTAL_DESC_CEILING` defaults in
   `test-mcp-desc-budget.sh`) to just above the new max/total — a slice that trims but does
   not tighten the ceiling has not locked its win.
4. Report bytes reclaimed.

## Slices

- **S1 (T-2406):** this policy + the guard + trim the worst offenders (the 11,751-char
  tool-catalog meta-tool + the 24 over 1000).
- **S2 (T-2407):** the 600–1000 char band (~70 tools).
- **S3 (T-2408):** long tail + relocate any genuinely-useful archaeology to docs + tighten
  the guard to the final ceiling.
