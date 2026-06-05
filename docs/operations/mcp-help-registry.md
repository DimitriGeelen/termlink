# `termlink_help` / `termlink help` — paged-ranked-filtered-projected discovery API

The help registry has two co-equal surfaces:

- **MCP** — `termlink_help(...)` — the discovery tool LLM agents call first
- **CLI** — `termlink help [flags]` — the shell-side parity verb (T-2002, cycle 13 #1)

Both wrap the same internal `build_help_json` registry; shape parity is locked
by `build_cli_help_json_matches_mcp_shape` in `crates/termlink-mcp/src/tools.rs`.
**Adding an axis on one surface requires adding it on the other** — the test
catches drift at compile/test time. T-1984..T-2000 (cycle 12) expanded the
MCP shape; T-2002 (cycle 13) brought the CLI to parity.

This doc is the operator + LLM-client-author reference for the resulting shape.

## TL;DR — the canonical cold-start call

Same axis surface, both interfaces:

```python
# MCP (LLM agent path)
termlink_help(
    sort_by='required_arity',                     # cost-aware ranking
    limit=20, offset=0,                           # paginated window
    exclude_deprecated=True,                      # live tools only
    categories=['channel', 'agent_chat'],         # positive scope
    exclude_categories=['agent_inbox'],           # negative scope
    fields=['name', 'parameter_required_count'],  # row projection
)
```

```bash
# CLI (shell operator path) — same shape, kebab-case flags
termlink help --json \
    --sort-by required_arity \
    --limit 20 --offset 0 \
    --exclude-deprecated \
    --categories channel,agent_chat \
    --exclude-categories agent_inbox \
    --fields name,parameter_required_count
```

Both return ~20 rows, ~1.5KB total JSON, ranked by call cost ascending,
scoped to two namespaces minus one, projected to two keys per row. Pre-arc
the no-arg MCP call dumped the entire registry (~50KB, unranked, unscoped);
pre-T-2002 the CLI had no `help` subcommand at all (only clap's auto-generated
3KB usage banner).

### Shell-only ergonomics

The CLI also defaults to a **human-readable** render when `--json` is omitted —
categorized listing for bare `termlink help`, per-row listing for `--name-filter`
matches, drill-in render for `--tool-detail`, etc. The `--json` flag flips to
the raw envelope for `jq` piping.

Quick recipes:

```bash
termlink help --list-categories          # 29-line category index (~1.5KB)
termlink help --summary                  # 17-line stats: totals, biggest cats
termlink help --essentials               # 29-row starter set, one per category
termlink help --tool-detail termlink_channel_post   # full drill-in
termlink help --name-filter inbox        # substring search across names+descs
```

Multi-value flags accept comma-separated values: `--categories channel,agent_chat`,
`--fields name,description,parameter_count`, `--exclude-categories agent_inbox,batch`.

## Seven axes

Each axis is opt-in and additive. Omitting a param yields pre-arc behavior
(backcompat invariant — every existing test stays green at each slice).

| Param | Type | Slice | Effect |
|---|---|---|---|
| `limit` | `Option<usize>` | T-1984 | Cap `matches[]` at first N (post-sort if `sort_by` set) |
| `offset` | `Option<usize>` | T-1994 | Skip first N rows; envelope gains `next_offset` when more remain |
| `sort_by` | `Option<String>` | T-1996 | Deterministic axis: `name`, `arity`, `required_arity`, `category` |
| `fields` | `Option<Vec<String>>` | T-1998 | Strict row projection — no implicit `name` retention |
| `categories` | `Option<Vec<String>>` | T-1999 | Positive multi-namespace scope (overrides single `category`) |
| `exclude_categories` | `Option<Vec<String>>` | T-2000 | Negative multi-namespace filter (wins on overlap) |
| (signal) | — | T-1995 | Every row carries `parameter_required_count` for cost-aware ranking |

`parameter_required_count` is a row-level signal (not a param). It pairs with
`parameter_count` so an LLM client knows that `(12, 2)` is cheaper to call
than `(4, 4)` despite the lower total arity.

## Eighth axis — routing

T-1997 added one route extension that doesn't introduce a new param.
Previously the no-needle path (`termlink_help()` with no args) routed to the
legacy categories-keyed dump. With T-1997, ANY of `limit` / `offset` /
`sort_by` (or the prior `min_parameters` / `max_parameters` arity bounds)
on a no-needle call routes into the same `matches[]` flat shape as
`name_filter` mode. So `termlink_help(limit=10)` returns 10 rows instead of
200+.

When NONE of those signals is set, the legacy categories-keyed dump is
preserved exactly — that's the backcompat anchor.

## Envelope validation — the `*_applied` / `*_unknown` pattern

Every "subset-against-a-domain" param carries the same input-validation
shape so LLM clients can detect mis-typed values instead of silently
misreading the response.

| Param | Envelope on recognized input | Envelope on unrecognized input |
|---|---|---|
| `sort_by` | `sort_by_applied: <value>` | `sort_by_unknown: <value>` |
| `fields` | `fields_applied: [...]` | `fields_unknown: [...]` |
| `categories` | `categories_applied: [...]` | `categories_unknown: [...]` |
| `exclude_categories` | `exclude_categories_applied: [...]` | `exclude_categories_unknown: [...]` |

Unknown values are silently dropped from the filter (graceful degradation)
AND surfaced in the envelope (input validation). The LLM client sees its
silently-ignored input rather than misreading the row shape.

For `fields`: empty array `[]` is degenerate and treated as no projection
(envelope omits both flags). Same for `categories: []` and
`exclude_categories: []`.

## Composition rules

The pipeline runs in this fixed order:

```
filter (category, arity, deprecated, needle, categories, exclude_categories)
  → sort (sort_by, stable, registry-walk tiebreak)
  → page (offset, limit)
  → project (fields)
```

Practical consequences:

- **`limit` truncates the SORTED set.** `sort_by='name', limit=5` gives the
  5 alphabetically-first names, not the 5 registry-walk-first names sorted.
- **`offset` slices the SORTED set.** Pagination invariants hold under any
  sort axis because the sort is stable.
- **`fields` runs LAST.** The window is determined by full-row data, then
  trimmed. So `sort_by='required_arity', limit=10, fields=['name']` returns
  the 10 cheapest tools as name-only rows (the sort uses the full
  `parameter_required_count` data even though it's dropped from the output).
- **`exclude_categories` wins over `categories` on overlap.** Intersection-
  minus-exclusion semantic. `categories=['A','B'], exclude_categories=['A']`
  returns rows from B only.

## Pagination protocol

The LLM client loop:

```
offset = 0
while True:
    r = termlink_help(name_filter='X', limit=20, offset=offset, ...)
    process(r['matches'])
    if 'next_offset' not in r:
        break
    offset = r['next_offset']
```

`next_offset` is present iff more rows lie beyond the current page;
absent means exhausted. `total_matched` carries the universe size (pre-cap,
pre-offset) so the client can size the query before pulling it.

## Allowed values

### `sort_by`

- `name` — alphabetical ASC by row `name`
- `arity` — `parameter_count` ASC
- `required_arity` — `parameter_required_count` ASC (the canonical
  cost-aware ranking)
- `category` — alphabetical by `category` field; registry-walk tiebreak
  preserves the existing intra-category order

Unknown value → `sort_by_unknown: <value>` in envelope, registry-walk order
preserved.

### `fields`

Eight allowed keys (the row-shape surface area shipped by T-1960..T-1995):

```
name, category, category_tool_count, description, deprecated,
parameter_count, parameter_required_count, replacement_hint
```

Strict — no implicit `name` retention. If the caller wants `name`, they
include it.

### `categories` / `exclude_categories`

Any tool-category name registered in the help registry. Unknown values
dropped + surfaced.

## Backcompat invariant

Every slice in the cycle preserved this rule: **when the new param is unset,
the envelope shape is unchanged.** Verified empirically by the existing
tests staying green at each commit (test count grew 764 → 834 across the
8 slices, 0 failed throughout).

The macro doc-comment drift test (`tools.rs::help_macro_description_documents_*`)
locks the shape-vs-doc contract — adding a new envelope field MUST also add
a token to the drift table or the test fails. This catches the "shipped a
field but forgot to document it" failure mode at compile time.

## Slice T-IDs (for git archeology)

| Commit | Slice | Capability |
|---|---|---|
| `aac3219d` | T-1984 | `limit` cap |
| `4d398554` | T-1994 | `offset` cursor + `next_offset` |
| `85b61987` | T-1995 | `parameter_required_count` signal |
| `bf1538a9` | T-1996 | `sort_by` axis |
| `0cc9f35d` | T-1997 | bulk-flat routing for no-needle paging |
| `3efc2b9b` | T-1998 | `fields` projection |
| (T-1999) | T-1999 | `categories` array |
| (T-2000) | T-2000 | `exclude_categories` array |
| `59cdc224` | T-2002 | **CLI parity** — `termlink help` subcommand + shape-parity test (cycle 13 #1) |

See learning PL-202 in `.context/project/learnings.yaml` for the slice
recipe (Python depth-tracking caller-patch script, drift-table token,
invariant test layout, etc.) — reuse it for the next tool-registry
shape-expansion arc.
