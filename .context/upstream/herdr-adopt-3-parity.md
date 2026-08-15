# herdr adoption #3 — CLI/MCP parity as a structural invariant (2026-08-15)

**READ-ONLY reconnaissance. No source modified.**

**Licence: every recommendation below is category (a) — IDEA ONLY. No herdr code
is copied, quoted structurally, or vendored. No attribution/NOTICE obligation
arises.** (Apache-2.0 as of v0.8.0 per `herdr-external-findings.md` §1 — not
relied on here.)

---

## Tooling constraints — read before trusting anything

- `Bash` outside the project root is **blocked** by the `check-project-boundary`
  hook (T-559). I could not check for a local herdr checkout at `/opt/herdr`,
  `/tmp/herdr`, or `~/herdr`. **There is no herdr source on this machine that I
  can reach**, so every herdr claim below is from `herdr.dev` docs + the GitHub
  file tree, never from reading code.
- One count in this file was initially **zero and wrong**: my first clap-enum
  census returned "0 enums" because my regex was `(pub )?enum` and the source
  says `pub(crate) enum` (`crates/termlink-cli/src/cli.rs:16`). I printed the
  raw lines, found my own error, and re-ran. The corrected numbers are below.
  Flagging it because the brief asked me to.

---

## 1. What herdr actually does — and what I could NOT verify

**Verified [VERIFIED-PAGE]:**

- `herdr.dev/docs/cli/`, verbatim: *"The CLI and the socket API are the same
  surface agents drive."*
- `herdr.dev/docs/socket-api/`, verbatim: *"The layers share the same control
  surface"* — describing Agent skill / CLI wrappers / raw socket as three
  approaches to one surface.
- Repo layout is a **single crate, `src/` layout** — no `crates/` dir
  (`/tree/master/crates` → HTTP 404). `src/` contains **`api/`, `cli/`,
  `client/`, `server/`, `protocol/`, `ipc.rs`** side by side in one binary.
- `CHANGELOG.md [0.5.0]` (cited in `herdr-external-findings.md` §4): *"herdr now
  defaults to a persistent server/client session model."*

**NOT verified — stated as inference, not fact:**

I found **no evidence of codegen, no shared schema file, no macro-generated
dispatch table**. Both doc pages assert the shared surface as a property and
neither explains a mechanism. I did not read `src/api/` or `src/cli/`.

The mechanism the layout *suggests* — and I am labelling this **INFERRED**, at
moderate confidence — is far more boring than codegen and far more interesting
for us:

> herdr's CLI is **a client of its own server**. `herdr pane split` does not
> contain an implementation of splitting a pane; it serialises a `pane.split`
> request onto the socket. There is exactly **one** implementation of each
> operation, so "CLI/socket drift" is not a bug class that can exist.

If that inference is right, the answer to *"is their approach structurally
better at preventing drift?"* is **yes, decisively — but not because of
tooling.** It is better because they have one implementation and we have two.
Codegen would be a way to keep two implementations honest; herdr appears not to
need one.

**Caveat that matters:** this is the same architecture that produced the two
architectural NOs in the synthesis (§3: no exit codes, no signals). Their CLI
cannot return an exit code either, for the same reason. Single-surface is not
free — it means the CLI can only express what the socket can express.

---

## 2. Our surface, measured

All figures from this worktree, commands shown so they can be re-run.

| Measure | Value | How |
|---|---|---|
| MCP tools (unique names) | **261** | `grep -oP 'name = "\Ktermlink_\w+' crates/termlink-mcp/src/tools.rs \| sort -u \| wc -l` |
| `#[tool(` attributes | 270 | same file (9 more than unique names — not investigated) |
| CLI variants, 22 clap `Subcommand` enums | **262** | AST-lite python census over `crates/termlink-cli/src/cli.rs` |
| — of which top-level `Command` | 54 | includes ~20 *group* nodes (`Agent`, `Channel`, `Fleet`, `Hub`…), so leaf verbs ≈ 242 |
| `tools.rs` | **45,799 LOC** | `wc -l` |
| `cli.rs` | 6,985 LOC | `wc -l` |
| Duplicated `*_mcp` helper fns | **94** | `grep -cP 'fn \w+_mcp\b' crates/termlink-mcp/src/tools.rs` |
| Parity test fns | **24** | `crates/termlink-mcp/tests/parity.rs` — 23 real pairs + `parity_negative_self_test` |

**Coverage: 23 asserted pairs against 261 tools ≈ 8.8%.** The file's own header
(`parity.rs:14-15`) says: *"As of T-2683/T-2689 that expansion has still not
happened: 24 cases against 68 distinct `*_mcp` parallel helpers."* My count of
those helpers is 94, not 68 — I did not reconcile the difference and am not
asserting theirs is wrong; both are grep-shaped and may count different things.
Either way the ratio is under 10%.

### The two surfaces are already siblings, not parent/child

`crates/termlink-mcp/Cargo.toml:10-27` — **there is no `termlink-cli`
dependency.** Both crates depend on `termlink-protocol`, `termlink-session`,
`termlink-hub`. So the *operations* are already shared: both sides RPC the same
session socket and the same hub.

**The duplication is not in the operation. It is in the response envelope.**
Verified on the exact T-2624/T-2687 defect:

- CLI: `crates/termlink-cli/src/commands/events.rs:1210-1213` — hand-written
  `serde_json::json!` literal with `total_sessions`, `sessions_unreachable`, …
- MCP: `crates/termlink-mcp/src/tools.rs:14033-14038` — a **second**
  hand-written `json!` literal with the same key names.

Two literal maps, two authors, one implicit contract. T-2624 edited one. That is
the whole bug, and it will recur exactly as often as someone edits one literal.

### How parity is asserted today

`parity.rs` runs both sides against a shared fixture, strips non-deterministic
keys (`strip_fields`, `parity.rs:94-109`), and diffs the JSON
(`diff_json`, `:114-138`). It is a good harness. Its weakness is that it is a
**hand-curated allowlist of 23 pairs** — the other ~238 tools are not "passing",
they are **unexamined**, and nothing distinguishes those two states in the
output.

That is the T-2680 lesson replayed one layer up: a guard that reports green over
a surface it never looked at. It is worth being precise about the recent failure,
though — `parity_topics` **was** covered. It was written in T-1910 and it caught
T-2624 correctly. What failed from 2026-08-12 was that **nothing ran it**;
T-2686 fixed that by wiring `cargo test --workspace` into CI. The coverage gap
and the execution gap are different bugs, and only the execution one is closed.

### Verified absence — checked, not assumed

- `ls scripts/ | grep -i parit` → **NONE**. No parity script exists.
- `grep -l "guard-layer: source" scripts/check-*.sh` → **11 members**, none
  about parity.
- `ls .context/checks/` → **7 allowlists** (alloc-sink, busy-spin,
  charter-drift, drain-sink, error-code-emission, platform-lock, silent-exit).
  No parity allowlist.

So the guard layer — twelve canaries and six static checks deep — has **no
member that knows the MCP and CLI surfaces are supposed to correspond.**

---

## 3. Recommendations

### R1 — Enumerating parity census check (adopt the *idea*, not herdr's shape)

**What to adopt.** Invert the harness's polarity. Today `parity.rs` enumerates
what is *checked*; nothing enumerates what *exists*. Add
`scripts/check-mcp-cli-parity.sh` carrying the `# guard-layer: source` marker
that (a) extracts the 261 MCP tool names, (b) extracts the CLI verb list, (c)
maps `termlink_channel_post` ↔ `channel post` by the obvious underscore→space
rule, and (d) fires on any pair that is neither covered by a `parity_*` case nor
listed in `.context/checks/parity-allowlist` with a cited reason.

This is a pattern the repo already runs seven times. It does not prevent drift;
it converts *unexamined* into *acknowledged*, which is the precondition for ever
closing the other 238.

**Evidence in herdr.** Indirect only — herdr needs no such check because it has
one implementation. The adoptable idea is "the correspondence should be
enumerable", not anything herdr built.

**Do we already have it?** No — `ls scripts/ | grep -i parit` → NONE (§2).

**Effort.** ~1 day. **Directive: D2 Reliability.**

**Caveat, stated because it is the honest weakness:** name-mapping is heuristic.
`parity.rs:194-197` already documents a real naming divergence (CLI `ping` takes
a positional `[TARGET]`; MCP uses a `target` param, while CLI `--target` means
something else entirely). Expect a first run that is mostly allowlist entries.
That is still strictly better than a blank.

### R2 — Shared envelope types in `termlink-protocol` (the actual structural fix)

**What to adopt.** The *principle* behind herdr's single surface: one
implementation, rendered twice — never two implementations agreeing by
convention. Concretely, for each converged operation define
`#[derive(Serialize)] struct TopicsResponse { … }` in `termlink-protocol` and
have both `events.rs:1210` and `tools.rs:14033` return it. Adding a field then
lands on both sides in one edit, and forgetting one becomes *impossible* rather
than *untested*. This is the compile-time invariant the brief is asking for.

It is incrementally adoptable per-verb, which matters at 45,799 LOC. Start with
the pairs `parity.rs` already covers — they are the ones whose shapes are
already known to agree.

**Do we already have it?** No, and it is a **deliberate standing decision**, not
an oversight: `CLAUDE.md` states the T-2069 convention as *"no cross-crate
sharing for these tiny pure helpers"*, and 94 `*_mcp` duplicate helpers exist
because of it. **That convention is the drift generator.** R2 is a proposal to
revisit T-2069, which makes it a decision for a human, not an agent.

**Effort.** ~2–3 days per subsystem; weeks for the full 261. Do not attempt
wholesale. **Directive: D2 Reliability, D3 Usability.**

### R3 — Do NOT adopt herdr's single-binary shape

Making our MCP server a client of the CLI (or vice versa) would mean rewriting
45,799 LOC of `tools.rs` as a subprocess or socket shim. It would also import
herdr's own constraint — the CLI can only say what the socket can say — into a
codebase whose `exec` path deliberately keeps a direct-child `status.code()`
(`herdr-internal-tmux-surface.md` §4). **D4 Portability** cuts against it too:
the MCP surface is the portable one and should not be downstream of a CLI's
argument grammar.

---

## 4. Answering the brief directly

> **Is their approach structurally better at preventing drift?**

Yes — but the mechanism is *one implementation*, not codegen, and I could not
verify even that from source (§1). Their advantage is architectural and was
available to them because they started as a server with a client attached. Ours
grew two edges over one core.

> **Could we adopt it?**

Not the architecture (R3). The *invariant* is adoptable, and R2 is the way —
share the serialised type, not the dispatch. Our core is already shared
(`Cargo.toml` shows both crates over `termlink-session`/`-hub`); only the
envelope is duplicated. That is a much smaller problem than "we maintain two
implementations", and it is worth saying plainly: **the fix is smaller than the
brief assumed.**

> **Highest-leverage item?**

R2 is the one that converts the bug class into a compile-time invariant, and it
is blocked on a human decision about T-2069. R1 is the one that can ship this
week and tells us how big R2 actually is. Do R1 first — not because it is the
fix, but because right now nobody knows the size of the surface R2 has to cover,
and 8.8% coverage reported as green is how we got here.
