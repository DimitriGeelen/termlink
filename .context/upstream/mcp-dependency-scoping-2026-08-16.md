# MCP dependency scoping — rmcp update + mcpo adoption (2026-08-16)

Captured at 99% context budget so the next session starts from facts rather than
re-deriving them. **Neither question is answered here** — this is the scope, plus
the one measured fact I could get before the gate closed.

## Measured fact

`crates/termlink-mcp/Cargo.toml`:

```toml
rmcp = { version = "~1.3", features = ["server", "transport-io", "macros"] }
```

`~1.3` means `>=1.3.0, <1.4.0`. That is a **deliberate pin from T-1056**, so
`cargo update` will never move us across a minor. Any upgrade is an explicit
constraint change, which is why nothing has drifted on its own.

Dev-dependencies pin the same `~1.3` with `client` + `transport-async-rw` added —
so an upgrade touches the parity test harness too, not just the server.

## Question 1 — rmcp upgrade (confirmed by operator as the intended target)

Unknown until measured. What the next session must establish, in order:

1. What version is current, and how far is `~1.3` behind.
2. What breaks. `tools.rs` carries **260 `#[tool]` registrations** — the largest
   single blast radius in the repo. The `#[tool]`/`#[tool_router]` macro surface
   and `schemars` interaction are the likely break points (`schemars = "1"` is
   itself a loose constraint and moves independently).
3. Whether `parity.rs` still holds. It asserts MCP↔CLI output equality on 24
   tools; an envelope-shape change in rmcp would surface there first. Note the
   census: **24 asserted, 236 acknowledged** (T-2747) — so parity passing is NOT
   evidence the other 236 survived.
4. Whether the guard layer + `cargo test --workspace` stay green (now gated in CI
   by T-2686, so a regression blocks the build rather than shipping).

**Do not bump the pin without running the full workspace suite.** T-1056 pinned it
for a reason; the reason should be re-read before it is overridden.

## Question 2 — mcpo (operator asked "why not also use mcpo?")

`mcpo` is the OpenWebUI project that wraps an MCP server and re-exposes it as an
OpenAPI/REST surface, so OpenAPI-speaking clients can reach MCP tools. It is
**not a dependency of this repo today** — adopting it is new work, not an update.

Honest position: I do not have grounds to recommend for or against it yet, and my
knowledge of its current release is stale (the operator says there is a new one).
What makes it a genuinely open question rather than an obvious yes or no:

**Argument for.** Constitutional Directive #4 (Portability) names the target
standards explicitly — *"prefer standards (MCP, LSP, **OpenAPI**)"*. OpenAPI is
not an outside idea being imported; it is already written into the directives.
An OpenAPI face on the existing MCP server is arguably the directive being
honoured rather than breadth being added, and it costs no new tool surface —
it re-projects the surface that already exists.

**Argument against.** T-2468 found TermLink **over-built in breadth**, and P4
(T-2471/T-2478) pruned 52 charter-untraceable tools. A new distribution surface
must trace to the charter's four verbs (discover / exchange durable messages /
claim work / control terminal sessions). It also adds a second transport to keep
in parity — and the parity census (T-2747) is already only 9.2% covered on the
ONE pairing that exists. A third surface with no parity guard would be the same
coverage gap again, one layer out.

**The sharp question** to settle first: does mcpo serve a real consumer we have,
or is it capability-for-its-own-sake? If something concrete (an OpenWebUI
instance, a non-MCP client on the fleet) needs REST access to termlink tools,
that is a charter-traceable need and the answer is probably yes. If not, it is
exactly the accretion the charter-drift canary (T-2483) exists to prevent.

**Recommended shape:** an inception task (one question, go/no-go), not a build
task. Per CLAUDE.md inception discipline the research artifact goes in
`docs/reports/T-XXXX-*.md` and is written *before* the spikes.

## Session-state note

At the time of writing, this branch was merged to `main` (`19ba70a33..447b8b638`,
fast-forward, 225 commits) on operator authorisation — so the guard layer and
T-2709's stuck-claims predicate are now live in cron, which was the T-2764
finding.
