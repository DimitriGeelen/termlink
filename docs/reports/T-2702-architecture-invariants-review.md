# T-2702 — TermLink purpose review #7: are the architecture doc's invariants enforced?

**Type:** inception (exploration → go/no-go)
**Status:** exploration complete, recommendation GO on the in-authority subset
**Predecessors:** T-2468, T-2678, T-2683, T-2690, T-2694, T-2698

---

## The question

Six reviews have run. Review #6 named the pattern connecting all of them: every finding
was **an assertion without an enforcer**. Breadth asserted as charter-traceable;
non-goals asserted as guarded; guards asserted as running; platforms asserted as
supported; capabilities asserted as working; refusals asserted as issuable.

Applying that template needs a surface of assertions nobody has checked. There is one
left, and it is the one the charter itself points at:

> The authoritative statement of the substrate design and its invariants is
> [`docs/architecture/parallel-execution-substrate.md`].
> — `docs/CHARTER.md`, *Origin and evolution*

The charter **delegates authority** to that document. Its §10 is titled *"Invariants
(must not be violated)"* and lists five. Six reviews audited the charter, the guards,
the platforms, the capability claims and the error taxonomy — none opened §10.

> **The question:** for each invariant the authoritative design document declares must
> not be violated, what would stop it being violated?

---

## Method

1. Read §10 and decompose it into individually checkable invariants.
2. For each, find the artifact that would enforce it, and read what that artifact
   *actually* covers — not what its name suggests.
3. Where an invariant is about governance rather than code, ask what evidence its
   enforcement would produce, and check whether that evidence exists.
4. Actively try to clear the invariants that look weak, so the surviving findings are
   not artifacts of wanting to find something.

Step 2 is the one that mattered. Two invariants *appear* covered by existing tripwires
and are not; the appearance comes from shared vocabulary, not shared scope.

---

## Finding F1 — the decisive invariant is guarded by nothing

§10's first invariant, restated plainly in §3:

> **The hub mediates all coordination; spokes never connect to one another.**

§3 spends forty lines defending it, rejecting an agent-to-agent mesh on three arguments
and calling the third **decisive**: a star has one failure point that is visible,
diagnosable and — because channel logs and the inbox spool are durable — *recoverable*;
a mesh distributes fragility across N² links with **no central durable replay** and,
worst, **silent partial-partition divergence**, where A↔B and C↔B survive while A and C
hold inconsistent state with no authority to reconcile against. On a flaky homelab,
partial partitions are the normal failure mode.

Nothing enforced it.

### Why it looked enforced

`crates/termlink-hub/tests/no_federation_tripwire.rs` (T-2569) is a topology tripwire
using the vocabulary "strict star" and "no peer-to-peer". T-2678 built it and recorded
charter non-goal #1 as closed. Both are true — and irrelevant here, because they cover
a **different edge**:

| | edge | guarded by |
|---|---|---|
| charter non-goal #1 | hub ↔ hub (**federation**) | T-2569 ✅ |
| architecture §3/§10 | spoke ↔ spoke (**mesh**) | nothing ❌ |

T-2569 scans `CARGO_MANIFEST_DIR/src` of the *hub* crate and forbids the hub building a
hub-speaking client. The entire client side was unguarded, and
`crates/termlink-session/src/client.rs` ships a generic RPC client that dials any unix
path or TCP `host:port` with nothing constraining the target. A direct agent-to-agent
channel could be added in `termlink-session` or `termlink-cli` and no test would fail.

The two edges share a vocabulary, so a guard on one reads as a guard on both. That is
precisely why the gap survived a review series that had already built the non-goal
matrix: from either side it looked covered.

## Finding F2 — "producer ≠ judge" is unfalsifiable as written

§10's fourth invariant is **"Producer ≠ judge at the seam."** §9 defines it:

> The substrate does not declare a hard-dependency primitive complete unilaterally —
> the AEF layer … validates it is actually usable for dispatch before it is accepted.
> … **Neither side self-certifies the boundary.**

This is a governance invariant over eight contracted hard dependencies (claim/exclusive
delivery, idle/busy registry, pull/assign verb, reconnect/outbound queue, symmetric
auth, persistent presence, typed git surface).

The same section removes the only artifact that could evidence it:

> A heavier shape was considered and rejected: a standalone collaboration-protocol
> document **with formal sign-off ceremony**. For a homelab where the "two parties" are
> agents the same operator orchestrates, that is enterprise scaffolding for a workshop.

Grep finds **zero sign-off records** anywhere in the repository. The only occurrence of
the phrase in the document is the sentence rejecting the ceremony. And CLAUDE.md
separately records **G-063**: the cross-repo seam channel `framework:pickup` sat at
**36-sent / 0-received** — a write-only sink nobody noticed for long enough to warrant
a canary (T-2231).

So the invariant declares a rule and, three paragraphs earlier, deliberately deletes the
evidence that would show whether it was followed. **An invariant whose only possible
evidence was designed away is not an invariant; it is a value.**

To be fair to the decision: rejecting sign-off ceremony for a homelab was reasonable,
and the doc argues it well. The defect is not the rejection — it is keeping the word
*invariant*, in a section headed *must not be violated*, for something simultaneously
made unverifiable. Those are different commitments and the document makes both at once.

This is also self-implicating for the review series. Every prior pass closed substrate
tasks as `work-completed` on the producer's own say-so. Under this invariant, none of
those closures had the consumer's sign-off — because there is no mechanism to obtain
one.

---

## What is NOT wrong — two theses investigated and cleared

- **Append-log durability and ordering (§10's second invariant).** `termlink-bus`
  carries 88 tests in `lib.rs`, and monotonic cursor semantics are documented and
  asserted across `meta.rs` (the delivery frontier takes a monotonic MAX). Ordering is
  not the unguarded invariant.
- **The CLI dialling a local session socket.** `cmd_topics` and friends connect
  straight to `reg.socket_path()`, which superficially looks like a peer-to-peer
  surface. It is not what §3 forbids: the invariant is that *spokes* — agents — do not
  mesh with each other; an operator tool reaching a session on the same host is the
  operator surface working. Counting it would have inflated the finding and made the
  resulting guard unusable, since any tripwire banning it would fire constantly.

Both were pursued far enough to be settled, then dropped. Recorded at equal length
because the value of a seventh consecutive "ultra-critical review" depends entirely on
whether it can still return *not guilty*.

---

## Gap register

| # | Gap | Severity | Authority | Disposition |
|---|---|---|---|---|
| **G1** | The spoke↔spoke mesh invariant — §3's decisive one — is enforced by nothing | **high** | agent | **BUILD** (T-2703) |
| **G2** | "Producer ≠ judge" is declared an invariant and made unfalsifiable in the same section | **high** | **human** | **FILE** (T-2704) — amending an authoritative design doc, and cross-repo governance (G-062) |
| **G3** | `termlink-cli` also dials sockets; a mesh introduced purely there is still unguarded | medium | agent | **FILE** — needs its own predicate, since local-session dialling is legitimate there |

---

## Recommendation

**GO on G1.** It is buildable, testable here, and closes the invariant the document
itself calls decisive.

**FILE G2.** Two reasons, both about authority rather than difficulty. It amends a
document the charter designates authoritative — the same sovereignty boundary that made
T-2470's charter sentence human-blessed. And the underlying question (what evidence a
cross-repo seam should produce) is G-062 territory: the AEF layer is a different
repository, and deciding its obligations unilaterally from this side would itself
violate "neither side self-certifies the boundary". Recording it is the only move
consistent with the invariant it is about.

**FILE G3** rather than bolt it onto G1: `termlink-cli` legitimately dials local
sessions, so the same predicate would false-positive there, and a guard that fires
constantly gets deleted.

---

## Assumptions registered

- **A1** — Confining connection construction to an enumerated set of transport modules
  is a meaningful proxy for "no mesh". *Confidence: medium.* It cannot see a *runtime*
  target, so a caller passing a peer's address still meshes; that needs per-agent
  authorization (T-2422 / G-064) and is stated as residual risk in the test itself.
- **A2** — Test-module connections are not mesh paths. *Confidence: high.* They dial
  scratch sockets to exercise the transport, which is why the scanner excludes
  `#[cfg(test)]` items — carefully, see the correction below.
- **A3** — The absence of sign-off records in *this* repository is evidence about the
  seam. *Confidence: medium.* Sign-off could in principle live in the AEF repo, which I
  cannot read. Stated explicitly so F2 is not over-claimed: what is certain is that
  **this** side records none, and that the document rejected the artifact that would
  carry them.

---

## Outcome

| Gap | Task | Result |
|---|---|---|
| G1 | **T-2703** | shipped — `crates/termlink-session/tests/no_spoke_mesh_tripwire.rs`, three checks: connections are constructed only in enumerated transport modules, each module's site count is pinned, and the scan is provably non-vacuous. |
| G2 | **T-2704** | filed, `owner: human` — amends an authoritative design doc, and is cross-repo governance (G-062). |
| G3 | **T-2704**-adjacent | recorded in the tripwire's own residual-risk section: `termlink-cli` needs its own predicate, since local-session dialling is legitimate there. |

**What the tripwire pins.** Production connections in `termlink-session` are confined
to four transport/probe modules, each read and confirmed during the build:

| module | sites | dials |
|---|---|---|
| `client.rs` | 5 | a local session control plane; the hub (plain + TLS); hub-router forwarding to a local proxy |
| `transport.rs` | 3 | `Transport` trait impls + a sync liveness probe (generic plumbing) |
| `tofu.rs` | 1 | a hub, to capture its TLS leaf cert for TOFU pinning |
| `ws_consumer.rs` | 1 | the hub's unix socket (WebSocket consumer) |

A mesh appears either as a new site inside one (count check) or — far more likely — as
a socket opened in a fifth module (containment check).

**Verified:**

```
cargo test --workspace  → 3483 passed · 0 failed · 24/24 suites ok
guard layer             → 25/25 clean
no_spoke_mesh_tripwire  → 3 passed
```

**Load-bearing, proven by injection:** a simulated peer-to-peer `UnixStream::connect`
added to `discovery.rs` fails `connections_are_constructed_only_in_transport_modules`
with the offending file and line named; removing it returns to green. A *comment*
mentioning a connect does not trip it.

### The tripwire caught two defects in itself

Both are the failure mode this series keeps finding elsewhere — a guard that silently
reads less than it claims — so they are recorded rather than quietly fixed.

1. **A false premise from a truncated grep.** The test was first written asserting all
   connects live in `client.rs`. That came from a `grep … | head -10` and was wrong; the
   check failed on its own first run against `transport.rs`, `tofu.rs` and
   `ws_consumer.rs`. The corrected four-module enumeration is a *stronger* invariant
   than the guess, because it is the actual answer to "what can a spoke dial".
2. **A scanner that read half a file.** Test code was excluded by truncating at the
   first `#[cfg(test)]`. `discovery.rs` places its test module at line 89 of 176, so
   that discarded **half the file unscanned** — and the load-bearing probe appended at
   the end did not fire. Replaced with brace-counting that skips only the guarded
   item's body. Without the injection probe this would have shipped as a guard that
   passes forever.

Same class as T-2680's scope over-report and T-2699's comment-as-emission and
digit-blind regex. Three consecutive reviews have now found this defect *inside the
guard they were building*, which is itself the argument for always proving a new guard
by making it fail.

### A note on self-implication

F2 is not only about the document. Every prior pass in this series closed substrate
tasks as `work-completed` on the producer's own assessment. Under "producer ≠ judge",
none of those closures carried the consumer's sign-off — because no mechanism exists to
obtain one. The reviews have been operating inside the gap they just found.
