# T-2698 — TermLink purpose review #6: is the refusal taxonomy real, or partly fiction?

**Type:** inception (exploration → go/no-go)
**Status:** exploration complete, recommendation GO on the in-authority subset
**Predecessors:** T-2468, T-2678, T-2683, T-2690, T-2694

---

## The question

Five reviews have run. Their axes, and what each actually measured:

| Review | Axis | Directive really measured |
|---|---|---|
| T-2468 | product breadth vs charter | #2 |
| T-2678 | charter **non-goals** vs guards | #2 |
| T-2683 | does anything *execute* the guards | #2 |
| T-2690 | Usability + Portability promises | #3 / #4 |
| T-2694 | charter **positive claims** vs provers | #2 |

A pattern worth naming: almost every finding so far has been an instance of one thing —
**something the project asserts is not backed by something that enforces it.** Breadth
asserted as charter-traceable; non-goals asserted as guarded; guards asserted as
running; platforms asserted as supported; capabilities asserted as working.

That suggests where to look next. TermLink publishes a **refusal taxonomy** — a set of
error codes it documents itself as able to return. Each one is an assertion: *"this
system will refuse you under condition X."* Nobody has checked whether it can.

> **The question:** for every error code TermLink defines and documents, can the code
> actually emit it?

---

## Method

1. Enumerate every constant in `crates/termlink-protocol/src/control.rs::error_code`.
2. For each, grep every product crate (`hub`, `session`, `cli`, `mcp`, `bus`) for an
   emission site, **excluding the defining file** so a constant is not counted as its
   own use.
3. For any code with zero sites, look for a *builder* — machinery that constructs the
   error — and check whether the builder itself has callers. "No emission" and "no
   machinery at all" are different findings and must not be conflated.
4. Check what the documentation claims about each dead code, since a code documented as
   *reserved* is fine and a code documented as an ordinary refusal is not.

Step 3 is what separates this from a lint. A constant with no uses is trivia; a
constant with a tested builder that nothing calls is an unwired guard.

---

## Finding F1 — three documented refusals can never occur

23 error codes are defined. Three have **zero emission sites** anywhere in the product
crates:

| Code | Documented as | Reality |
|---|---|---|
| `SESSION_BUSY` (-32002) | "Target cannot accept commands (already executing)" | never emitted |
| `MESSAGE_EXPIRED` (-32004) | "TTL exceeded before delivery" | never emitted |
| `PROTOCOL_VERSION_TOO_OLD` (-32011) | version negotiation refusal | builder + passing test, **zero callers** |

Both -32002 and -32004 appear in `docs/reports/T-005-message-protocol-design.md`'s
protocol error table as ordinary refusals, with nothing marking them unimplemented.

The practical consequence is not that a dead constant costs anything. It is that the
published contract **overstates what the system enforces**. A client implemented from
that table would write handlers for two errors it can never receive — harmless — and,
far worse, would reasonably infer that a session already executing a command will
*refuse* a second one, and that a message past its TTL will be *rejected* rather than
delivered late. Neither protection exists. The taxonomy is doing the work of a
specification while being, in these three rows, fiction.

## Finding F2 — version negotiation is built, tested, and wired to nothing

`PROTOCOL_VERSION_TOO_OLD` is the sharp one, because it is not a bare constant.
`control.rs` ships the full mechanism:

```rust
/// If `declared < required`, build a structured `PROTOCOL_VERSION_TOO_OLD` error
pub fn check_protocol_version(id, declared, required, method)
    -> Option<ErrorResponse>
```

with `{declared, required, method}` in the data field "so the client can act on it",
and a passing unit test:

```rust
#[test]
fn check_protocol_version_rejects_when_declared_is_older() {
    let err = check_protocol_version(json!(42), 1, 2, "command.execute").expect("reject");
    assert_eq!(err.error.code, error_code::PROTOCOL_VERSION_TOO_OLD);
}
```

```
$ grep -rn "check_protocol_version" --include=*.rs crates/ | grep -v control.rs
(no output)
```

**Zero callers.** The function is defined, documented, unit-tested, and invoked by
nothing. TermLink cannot refuse a peer for speaking too old a protocol, because the
check is never reached.

This is precisely the T-2683 pattern — *a guard that exists and nothing executes* —
reproduced one layer down. T-2683 found it at the script level (static checks nothing
invoked); this is the same shape in compiled Rust, and it survived because a unit test
on the builder makes the mechanism look covered. Coverage of a builder says nothing
about whether the builder is called.

---

## Not a finding — the backpressure paths ARE tested

The first hypothesis this pass pursued was that **Directive #1 (Antifragility)** —
the highest-priority directive, and the one no review had probed — was unproven:
TermLink has connection caps, rate limits, a bounded offline queue and a dedupe LRU,
all built for stress, and there is no load, stress, soak or chaos test in the repo
(only two pushwake latency benchmarks).

The evidence did not support the thesis:

- `crates/termlink-hub/src/governor.rs` carries **20 unit tests**.
- `offline_queue.rs` asserts the loud-refuse directly: `QueueError::QueueFull { cap: 3 }`.
- `HUB_AT_CAPACITY` and `RATE_LIMITED` each have emission sites in the hub *and*
  handling in `bus_client.rs`'s retry path.

So the refusal *mechanisms* for backpressure are genuinely exercised. The absence of a
load test is a real gap in a different sense — nothing measures behaviour at scale —
but "the antifragility mechanisms are unproven" would have been an overstatement, and
it was dropped rather than dressed up. Recorded at length because the discipline of
abandoning an attractive thesis is what makes the surviving findings worth anything.

---

## Gap register

| # | Gap | Severity | Authority | Disposition |
|---|---|---|---|---|
| **G1** | Nothing detects an error code that is defined and documented but never emitted | **high** | agent | **BUILD** — static check, guard-layer member |
| **G2** | The three dead codes are undeclared: docs present them as ordinary refusals | **high** | agent | **BUILD** — annotate at the definition so the contract stops overstating |
| **G3** | `check_protocol_version` is unwired: version negotiation cannot refuse anyone | medium | **human** | **FILE** — wiring it starts rejecting live peers; which methods require which version is a product decision |
| **G4** | No load/stress/soak test exists; behaviour at scale is unmeasured | medium | **human** | **FILE** — genuine gap, but designing a meaningful load profile is a scoping decision, not a tail-end build |

---

## Recommendation

**GO on G1 and G2.** G1 makes the class structurally visible — the same move as
T-2527/T-2531/T-2666/T-2672/T-2693, and it now runs automatically because T-2684 gave
the layer an entry point and T-2686 wired it into CI. G2 stops the published contract
claiming enforcement that does not exist, which costs nothing and is the honest half of
the fix.

**FILE G3 and G4.** G3 changes wire behaviour: calling `check_protocol_version` on a
live method begins rejecting peers below the required version, and this fleet has hosts
on binaries ~1000 commits stale (T-2377). Which methods gate on which version is a
product decision with real blast radius — the same reasoning that made T-2692's macOS
gate non-blocking. G4 is a genuine gap, but a load test whose profile nobody has agreed
would measure the wrong thing convincingly.

---

## Assumptions registered

- **A1** — A grep for `error_code::NAME` finds all emission sites. *Confidence: medium.*
  A code emitted via a bare numeric literal would be missed; the check therefore also
  scans for the literal value, and the fixtures cover that path.
- **A2** — Some codes may be legitimately reserved for a future protocol revision.
  *Confidence: high.* Hence an allowlist requiring a cited reason, rather than a hard
  failure — the same acknowledgement idiom as the four sibling checks.
- **A3** — Annotating the dead codes in `control.rs` does not change behaviour.
  *Confidence: high.* Doc comments only; no constant renamed, no value changed.

---

## Finding F3 — a live wire collision, found only by scanning for emission-by-literal

This finding did not exist when the review began. It appeared while checking whether a
"dead" code might be emitted as a bare number rather than through its constant.

`termlink-hub/src/artifact.rs` emitted the literal `-32004` at **three** sites for
"artifact not found":

```rust
ErrorResponse::new(id, -32004, &format!("artifact {sha256} not found"))
BusError::UnknownArtifact(_) => -32004,
ErrorResponse::new(id, -32004, "not found")
```

`-32004` is `MESSAGE_EXPIRED` — "TTL exceeded before delivery". **One wire code, two
incompatible meanings**, and they demand opposite client responses: an expired message
may be worth resending with a fresh TTL; a missing artifact must be re-uploaded or
abandoned. A client receiving `-32004` could not tell which it had.

`check-error-code-docs.sh` is structurally blind to this — it verifies that every
doc-cited `SYMBOL(-320NN)` pairing matches `control.rs`, and a hand-written number
inside a handler cites no symbol.

The collision also *masked* the deadness of `MESSAGE_EXPIRED`: the value appeared on
the wire, just never meaning what the taxonomy said. Fixing the squatting is what made
the code visibly dead.

Safe to fix: no client crate branched on `-32004` or on `BusError::UnknownArtifact`, so
nothing observable depended on the old value.

---

## Outcome

| Gap | Task | Result |
|---|---|---|
| G1 | **T-2699** | shipped — `scripts/check-error-code-emission.sh`, the **25th** guard-layer member. Counts symbolic *and* bare-literal emission, strips comments first, excludes the defining file so neither a constant nor a builder beside it counts as its own use. 18 fixture assertions. |
| G2 | **T-2699** | shipped — all three unemitted codes annotated ⚠️ **NOT ENFORCED** at their definitions, and declared in `.context/checks/error-code-emission-allowlist` with reasons stating what would change that. |
| F3 | **T-2699** | shipped — `ARTIFACT_NOT_FOUND` (-32024) introduced; three `artifact.rs` sites re-pointed; pinned by `artifact_not_found_does_not_reuse_message_expired`. |
| G3 | **T-2700** | filed, `owner: human` — wiring `check_protocol_version` starts rejecting live peers below the requirement. |
| G4 | **T-2701** | filed, `owner: human` — no load/stress/soak test; designing a meaningful profile is a scoping decision. |

**Verified:**

```
cargo test --workspace  → 3480 passed · 0 failed
guard layer             → 25/25 clean
check-error-code-emission → clean (24 scanned, 3 declared-reserved)
check-error-code-docs     → CLEAN (new code paired)
```

**Load-bearing, proven by removal:** deleting the `PROTOCOL_VERSION_TOO_OLD` line from
the allowlist re-fires the check on exactly that code; restoring returns it to clean.

### The check found two defects in itself

Both are recorded because they are the same class the review is about — a guard that
looks thorough while not reading what it claims to.

1. **Prose counted as emission.** The first run cleared `PROTOCOL_VERSION_TOO_OLD`
   because `tools.rs` mentions `(-32011)` in a doc comment. A comment *about* a code is
   not a use of it. Comments are now stripped before the numeric scan — the same
   distinction the platform-lock and silent-exit checks make.
2. **The scanner could not see codes containing digits.** Fixture 8 happened to name
   constants `A1/A2/A3` and revealed the name class was `[A-Z_]+`, which silently
   skipped them. On a taxonomy containing e.g. `HTTP2_ERROR` the check would have
   reported **clean on constants it never read** — precisely the T-2680 failure mode
   (a guard over-reporting its own scope). Widened to `[A-Z0-9_]+`.

### On abandoning the opening thesis

This pass opened by trying to indict **Directive #1 (Antifragility)** — the
highest-priority directive, never probed by any review, with no load, stress, soak or
chaos test anywhere in the repo. It would have been the most quotable headline of the
series.

The evidence refuted it in three queries: 20 unit tests in `governor.rs`, a direct
`QueueError::QueueFull { cap: 3 }` assertion, and live emission sites for both
`HUB_AT_CAPACITY` and `RATE_LIMITED` with matching retry handling in `bus_client.rs`.
The refusal *mechanisms* are genuinely exercised. The thesis was dropped.

That is now two consecutive reviews that published a deliberately-hunted claim which
turned out sound — T-2694's `append-log`, and this pass's backpressure paths. For a
sixth consecutive "ultra-critical review", evidence that the method can return *not
guilty* is worth more than another finding.

---

## Dialogue Log

### 2026-08-14 — framing

**Human mandate (verbatim, 6th issuance):** *"please ultra critically review termlink's
purpose and goals and identify gaps or needed adjustement, incept these and build these
and test these, drive to comopletion"*

**How the axis was chosen.** Rather than hunt for an unexamined artifact, this pass
looked at the *shape* of the previous five reviews' findings: every one was "an
assertion without an enforcer". Applying that template to a surface nobody had checked
— the error taxonomy, which is a list of assertions about refusals — produced a hit on
the first query.

**On the discarded thesis.** The pass opened by trying to indict Directive #1
(Antifragility), the highest-priority directive and the only one never probed. That
would have been a satisfying headline. The evidence refuted it within three queries
(20 governor tests, a QueueFull assertion, live emission sites), so it was abandoned
and recorded as a non-finding. Two consecutive reviews have now published a
deliberately-hunted claim that turned out sound — T-2694's `append-log`, and this
pass's backpressure paths — which is the calibration evidence a sixth consecutive
"critical review" most needs.
