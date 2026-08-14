# T-2690 — TermLink purpose review #4: the two Directives no review has examined

**Type:** inception (exploration → go/no-go)
**Status:** exploration complete, recommendation GO on the in-authority subset
**Predecessors:** T-2468 (product-vs-charter), T-2678 (charter-vs-guards), T-2683 (guard execution)

---

## The question

The framework names four Constitutional Directives, in priority order:

1. **Antifragility** — system strengthens under stress; failures are learning events
2. **Reliability** — predictable, observable, auditable; no silent failures
3. **Usability** — joy to use/extend/debug; sensible defaults; actionable errors
4. **Portability** — no provider/language/environment lock-in; prefer standards

Three critical reviews have now run:

| Review | Question | Directive |
|---|---|---|
| T-2468 | Does the product match the charter? | #1 / #2 |
| T-2678 | Does anything *enforce* the charter? | #1 / #2 |
| T-2683 | Does anything *execute* the enforcement? | #1 / #2 |

**All three sit under Directives #1 and #2.** Nobody has reviewed #3 or #4 — not
partially, not once. That is a blind spot in the *review series*, and it is exactly
the shape the series keeps finding in the product: a mechanism that looks thorough
because it is repeatedly exercised along the axis it already covers.

So this pass asks the only question that hasn't been asked:

> **Are TermLink's Usability and Portability claims true, and does anything check
> them?**

---

## Method

1. Enumerate what the project *claims* about platforms — README, charter, docs — so
   the review measures against a stated promise rather than an invented standard.
2. Enumerate every CI runner, and ask which platforms actually execute code.
3. Grep the product crates (not just scripts) for platform-locked primitives —
   `/proc`, `ss`, `systemctl`, `/sys` — and check each for a guard or a fallback.
4. For each unguarded site, trace what the *user* sees on the unsupported platform:
   a crash is a Reliability bug, but a silent wrong answer is worse and is invisible
   to every existing guard.

Step 4 is the one that distinguishes this from a lint. "Does it compile for macOS" is
already answered (yes — release.yml cross-builds it). The question that matters is
what happens when someone *runs* it.

---

## Finding F1 — macOS is a recommended platform that never executes a test

### What the project promises

`README.md` carries a formal Platform Support table:

| | macOS | Linux | Windows |
|-|-------|-------|---------|
| Core binary | **Yes** | Yes | No |
| PTY operations | **Yes** | Yes | No |
| Terminal.app spawn | **Yes** | — | — |
| tmux spawn | **Yes** | Yes | — |
| TCP hub | **Yes** | Yes | — |

Five explicit `Yes` claims. Elsewhere the README calls Homebrew the *recommended*
install for macOS, offers `brew install termlink`, and lists "macOS or Linux" under
build requirements. `release.yml` cross-builds `aarch64-apple-darwin` and
`x86_64-apple-darwin` and publishes both to GitHub Releases, from which the Homebrew
formula downloads.

### What CI actually runs

```
.github/workflows/install-check.yml:25   runs-on: ubuntu-latest
.github/workflows/doc-lint.yml:34,45,74  runs-on: ubuntu-latest
.github/workflows/release.yml:33         runs-on: ubuntu-latest   ← the test job
.github/workflows/release.yml:71         runs-on: ${{ matrix.os }} ← build only
.github/workflows/release.yml:116,167    runs-on: ubuntu-latest
```

Every runner is Linux except the build matrix. The macOS entry compiles a binary and
uploads it. **No test, no install-check, no smoke test, and no guard-layer run ever
executes on macOS.**

So the 3470-test workspace suite — the one T-2686 just made blocking for releases —
has never run against a macOS build. A macOS user installs via the *recommended*
path and receives a binary that has had zero behavioural verification.

### Why this is the same class as G-069, on a new axis

T-2683's finding was "shipped ≠ executed" for guards. T-2359/G-069 was "shipped ≠
live" for fleet binaries. This is **"built ≠ verified" for a platform** — and it is
sharper than either, because unlike an internal guard, this one has a documented
public promise attached to it and a package manager distributing the result.

Note the self-implication: the `test` job I added in T-2686 is `ubuntu-latest`. The
previous review closed the "nothing runs the tests" gap and reproduced the platform
blind spot inside its own fix.

---

## Finding F2 — identity resolution silently degrades on macOS, on both surfaces

`termlink whoami` resolves "which session am I?" through a fallback ladder. When no
`--session`/`--name` flag and no `TERMLINK_SESSION_ID` env var is present, it walks
the process ancestor chain and picks the closest registered session owning one of
those PIDs (T-1303):

```rust
// crates/termlink-cli/src/commands/metadata.rs:782
fn read_ppid_from_proc(pid: u32) -> Option<u32> {
    let raw = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    parse_ppid_from_stat(&raw)
}
```

`/proc` does not exist on macOS. `.ok()?` turns that into `None`, `walk_ancestor_pids`
breaks out of its loop immediately, and the chain collapses to `vec![start]` — the
termlink process's own PID, which is never a registered session's PID. Execution falls
through to the "no hint" branch and prints **every** live session as an ambiguous
candidate.

The same code exists a second time on the MCP surface
(`crates/termlink-mcp/src/tools.rs:7935`, `mod whoami_helpers`), deliberately
duplicated with a comment noting "no behavioural drift expected". Correct — the defect
is identical on both.

### What the user experiences

On Linux with one obvious ancestor session: `whoami` resolves it. On macOS, the same
command, same setup, same everything: `ambiguous: true` with the full candidate list.
Nothing distinguishes "your platform cannot do this" from "you genuinely have several
sessions and must disambiguate". The function's own doc comment even lists
`non-Linux` among its failure modes — the limitation was known at authoring time and
simply never surfaced to the user.

This is a three-directive violation in one line:

- **#4 Portability** — a platform-locked primitive on a supported platform
- **#2 Reliability** — "no silent failures"; this fails silently and returns a
  plausible wrong answer rather than an error
- **#3 Usability** — "actionable errors"; the output is *unactionable*, because the
  action it implies (pick from these candidates) is not the action required (set
  `TERMLINK_SESSION_ID`, because auto-resolution will never work here)

### Scope check — what is *not* broken

`pgrep` (`dispatch.rs:903`, `execution.rs:478`) exists on macOS; not a defect. The `ss`
call (`infrastructure.rs:343`) sits inside a `ufw`-output-parsing block, and `ufw` is
Linux-only, so that path is unreachable on macOS rather than broken — low severity,
noted but not fixed. Only the two `/proc` sites are genuine live defects.

---

## What is NOT wrong

- **The product compiles and cross-builds for macOS cleanly.** This is not a "macOS is
  broken" finding; it is a "macOS is unverified, and one known path degrades silently"
  finding. The distinction matters for how much to claim.
- **Windows is honestly excluded.** The README states plainly that Windows is not
  supported and gives the reason (POSIX PTY requirements). That is exactly the
  Portability discipline this review is asking for, applied correctly.
- **The duplication between CLI and MCP whoami helpers is documented and deliberate**,
  with a stated rationale and a note that a future task may extract it. That is a
  legitimate engineering trade, not drift.
- **Directive #4's "prefer standards" half is well served** — MCP is a standard, the
  protocol is JSON-RPC, and there is no provider lock-in in the coordination layer.

---

## Gap register

| # | Gap | Severity | Authority | Disposition |
|---|---|---|---|---|
| **G1** | macOS is documented + recommended but no CI job executes anything on it | **high** | agent | **BUILD** — non-blocking macOS job first |
| **G2** | `/proc`-based identity resolution silently degrades on macOS (CLI **and** MCP) | **high** | agent | **BUILD** |
| **G3** | Nothing detects the next platform-locked primitive added to the product crates | medium | agent | **BUILD** — static check, guard-layer member |
| **G4** | The 214-tool live surface as a Usability question | medium | **human** | **OUT OF SCOPE** — T-2548 already owns it |

---

## Recommendation

**GO on G1–G3.**

G2 and G3 are unambiguous: name the limitation where it bites, and make the convention
load-bearing so the next `/proc` read is caught by a check the T-2684 runner already
executes.

G1 is deliberately staged. The honest constraint is that **I cannot execute macOS from
this host**, so I do not know whether the suite passes there. Adding a *blocking*
macOS gate on an unverified suite would violate T-2686's own acceptance criterion —
"the gate is only added once the suite is actually green" — and would break releases
on an assumption. The correct first move is a non-blocking macOS job that makes the
answer visible. If it goes green, promoting it to blocking is a one-line change; if it
goes red, that result is itself the most valuable output of this review.

**OUT OF SCOPE G4** — re-litigating the tool surface under a Usability banner would be
T-2548's question in new clothes.

---

## Assumptions registered

- **A1** — The workspace suite's macOS result is *unknown*, not assumed-passing.
  *Confidence: high (it is a statement of ignorance).* This is why G1 ships
  non-blocking.
- **A2** — A runtime platform check is preferable to `#[cfg(target_os = "linux")]` for
  the whoami fix. *Confidence: medium.* A cfg would make the fallback unreachable on
  Linux and therefore untestable from this host; a runtime check on `/proc`
  availability is testable on Linux by pointing the probe at a path that does not
  exist.
- **A3** — Adding a macOS CI job does not meaningfully increase release cost.
  *Confidence: high.* It runs on tag push only, in parallel with the existing Linux
  job.

---

## Outcome

*(Filled at close — see the task's Decision section for the formal go/no-go.)*

---

## Dialogue Log

### 2026-08-14 — framing

**Human mandate (verbatim, 4th issuance):** *"please ultra critically review
termlink's purpose and goals and identify gaps or needed adjustement, incept these and
build these and test these, drive to comopletion"*

**How the axis was chosen.** Three prior passes had all measured Directives #1 and #2.
Rather than find a fourth question inside those, the series itself was treated as the
object of review: which Directives has it never touched? That produced #3 and #4
immediately, and the first evidence query (does anything run on macOS?) returned a
finding. PL-271 again — the recurring mandate is the signal that a structural axis is
uncovered, and this time the uncovered axis belonged to the reviewer rather than the
product.

**Self-implication noted rather than buried.** T-2686 — shipped in the previous pass,
hours earlier — added the `cargo test --workspace` gate on `ubuntu-latest`. It closed
"nothing runs the tests" and simultaneously reproduced the platform blind spot inside
its own fix. That is recorded in F1 rather than quietly corrected, because a review
series that hides its own misses is the failure mode this whole arc exists to prevent.
