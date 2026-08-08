# T-2540 — Decision Brief: `orchestrator.route` policy-in-substrate (subtract or grandfather?)

**Owner:** human (sovereignty — product-identity + charter decision)
**Prepared by:** agent (T-2468 purpose-review campaign, non-goal-adherence lens)
**Status:** awaiting human decision
**Agent recommendation:** **GO to SUBTRACT**, gated on one external-consumer check (IW-1) that this session cannot run.

---

## 1. TL;DR

The hub ships **`orchestrator.route`** — a JSON-RPC method backed by **1,981 lines
of adaptive routing *policy*** (specialist preference by task type, a
confidence-scored learning cache, a circuit breaker, and Tier-3 "bypass"
promotion). This is exactly the orchestration policy that **charter non-goal #4
reserves for the AEF layer, not the substrate.** It is **not deprecated**, it **is
advertised** in the hub's capability set, and it has **zero first-party callers**
in this repository. The only references are 2024 *pre-charter* design reports and
hub-internal tests.

The decision is yours because it is (a) a **product-identity** call (does this
capability define what TermLink is?) and (b) **consequential** (removing a live
advertised RPC is breaking for any unknown external caller). The agent's evidence
points to **subtract**; the honest counter-case for **grandfather** is laid out in
§5.

**One thing blocks a clean GO:** the fleet-wide *external*-consumer check (does any
peer or AEF process issue the raw `orchestrator.route` RPC?). That check is
cross-project and is structurally blocked from this session by the T-559
project-boundary. It must be cleared before any removal lands — see §6.

---

## 2. What `orchestrator.route` actually is

A three-layer adaptive router that, given a `method` + a specialist `selector`,
discovers a specialist session, forwards the call, and *learns* from the outcome:

| Layer | Module | Lines | What it does (policy) |
|-------|--------|-------|-----------------------|
| Handler | `router.rs:1160-1600` (`handle_orchestrator_route`) | ~440 | Orchestrates the three layers below; forwards + relays. |
| Layer 1 — bypass registry | `bypass.rs` | 669 | Promotes a `method` to "Tier-3 bypass" after `success_count >= 5 && fail_count == 0`, so future calls skip discovery (`router.rs:1507` "command promoted to bypass registry"). |
| Layer 2 — circuit breaker | `circuit_breaker.rs` | 488 | Opens a circuit on a failing candidate and skips it (`router.rs:1481` "circuit open — skipping candidate"). |
| Layer 3 — route cache | `route_cache.rs` | 824 | Confidence-scored cache of successful routes; serves cache hits, ages out stale entries (`router.rs:1249/1360/1526`). |
| **Total policy** | | **1,981** | |

The dispatch is live at `router.rs:78`
(`ORCHESTRATOR_ROUTE => handle_orchestrator_route`). The method constant is
`crates/termlink-protocol/src/control.rs:96`.

**Full surface to subtract is three advertised methods, not one:**
`orchestrator.route`, `orchestrator.bypass_status`, `orchestrator.bypass_invalidate`
(all three in the capability list, `router.rs:1023-1027`), plus the three modules
above.

---

## 3. Why this is a charter non-goal #4 violation

`docs/CHARTER.md:43-45`, non-goal #4 (verbatim):

> **Not a workflow or orchestration engine.** TermLink provides the primitives
> (claim/lease, DM, doorbell, presence); the Agentic Engineering Framework (AEF)
> builds orchestration *policy* on top. **The substrate stays mechanism, not
> policy.**

`orchestrator.route` is *policy by any reading*: "prefer this specialist for this
task type", "learn which routes worked", "trip a breaker on failure", "promote a
proven command to bypass". Deciding *who should do what, based on learned
outcomes* is the orchestration decision the charter explicitly assigns to AEF. The
substrate's legitimate role here is the **mechanism** underneath — `session
discover` (find candidates) and `channel claim` (reserve one) — both of which
already exist and are called. `orchestrator.route` stacks the *policy* on top,
inside the hub.

---

## 4. Evidence (all verified in code, 2026-08-08)

- **Advertised, not deprecated.** Present in the capability `methods` vec
  (`router.rs:1023-1027`) — the same list from which retired methods are *removed*
  (see the comment at `router.rs:1018-1020`). No `deprecated`/`is_deprecated`
  marker exists for it (contrast the retired `remote_inbox_*` / social tools,
  which carry deprecation flags).
- **Zero first-party callers.** No references in `crates/termlink-cli`,
  `crates/termlink-mcp`, `crates/termlink-session`, or `scripts/`. The only
  non-hub references are:
  - the protocol constant definition (`control.rs:96`), and
  - **2024 pre-charter design reports** — `T-233-*`, `T-239-route-cache-inception`,
    `T-240-*`, and the `T-247-*` scenario family. These are the "smart routing"
    design lineage that predates the charter; they are *design docs, not live
    call sites*.
- **Advertised-but-uncalled** is the exact shape of dead-but-visible surface: the
  hub tells capability consumers the method exists, but nothing in-tree uses it.
- **Un-remediated adversarial surface (argument *for* removal).**
  `docs/reports/T-247-scenarios-adversarial.md` documents that the bypass registry
  promotes **arbitrary method strings** to Tier-3 with no allow-list
  (`success_count >= 5 && fail_count == 0` is the only gate), and that
  `BypassRegistry::load()`→mutate→save is not concurrency-safe under parallel
  calls. Keeping the subsystem means owning (or fixing) this policy-layer security
  debt; subtracting it retires the debt outright.

---

## 5. The three options

### Option A — SUBTRACT (agent recommendation)
Remove the three RPC methods + capability advertisements + the three modules
(`route_cache.rs` / `circuit_breaker.rs` / `bypass.rs`), and, if any routing
behavior is genuinely needed, relocate it to the AEF/client layer where non-goal
#4 says it belongs.
- **Restores** the charter's mechanism-not-policy line; removes ~2k lines and the
  T-247 adversarial debt.
- **Cost:** a scoped build task — delete methods + modules + tests, drop the
  capability advert (mirror the 2026-06-05 retirement pattern already in
  `router.rs:1018`), confirm `session discover` + `channel claim` remain the
  supported mechanism.
- **Risk:** breaks any *external* caller (unknown until IW-1 is cleared).
- **Reversible:** yes — it's a deletion behind a gate; the design reports preserve
  the intent if AEF ever wants to rebuild the policy on its own side.

### Option B — GRANDFATHER
Keep `orchestrator.route`; amend the charter to sanction an explicit exception
("the hub carries this one orchestration-policy method for historical/strategic
reasons").
- **Honest case for it:** the routing policy is already built, tested, and could be
  strategically valuable if a future first-party orchestrator wants in-substrate
  routing; deleting working code has its own cost.
- **Cost:** a human-blessed charter edit (`docs/CHARTER.md`) carving the exception,
  **plus** paying down the T-247 adversarial debt (arbitrary-string promotion,
  concurrency) if the capability stays live and advertised.
- **Risk:** normalizes policy-in-substrate — the exact breadth-accretion the
  T-2468 review exists to counter; weakens non-goal #4 as a load-bearing line.

### Option C — INTERMEDIATE (deprecate-then-remove)
Mark the three methods deprecated now (stop advertising them / warn on call, per
the `remote_inbox_*` T-1166 pattern), keep the code one release, then remove.
- **Best when** IW-1 cannot be cleared quickly and you want a safe soak: external
  callers surface as deprecation warnings before the code is deleted.
- **Cost:** two steps instead of one; a deprecation window.
- **Risk:** low — this is the standard safe-retirement path.

---

## 6. The GATE — IW-1 external-consumer check (must clear before removal)

Zero *first-party* callers is verified. What is **not** verifiable from this
session is whether any **external** process — a peer hub, an AEF orchestrator, a
vendored agent script — issues the raw `orchestrator.route` RPC. The T-559
project-boundary blocks grepping `/opt/999-AEF` and peer repos from here.

**To clear IW-1 (human or a cross-project session):**
```
# In the AEF checkout and each peer repo:
grep -rn "orchestrator.route\|orchestrator_route\|ORCHESTRATOR_ROUTE" /opt/999-Agentic-Engineering-Framework
# On live hubs, check for the method in recent RPC traffic / capability negotiation logs.
```
- **If zero external callers** → Option A (subtract) is clean; proceed to a GO
  build task.
- **If any external caller** → Option C (deprecate-then-remove) with a migration
  note to that caller, or Option B if the caller is strategic and staying.

---

## 7. Go/No-Go evaluation

**GO-to-subtract if:** non-goal #4 is to remain a load-bearing line **and** IW-1
clears (no external callers) → delete the surface, keep `session discover` +
`channel claim` as the sanctioned mechanism.

**NO-GO (→ grandfather) if:** the human judges the routing policy strategically
worth keeping in-substrate → amend the charter for a sanctioned exception **and**
fund the T-247 security paydown.

**Agent recommendation: GO to subtract.** The capability is uncalled, off-charter,
and carries un-remediated adversarial debt; non-goal #4 is a deliberate
architectural boundary the T-2468 review is defending. But this is a sovereignty
call — the grandfather case in §5B is real, and the decision is yours.

---

## 8. If GO — build-task scope (for reference, not built here)

1. Remove the three methods from the capability advert (`router.rs:1023-1027`) and
   the dispatch (`router.rs:78` + siblings), returning `-32601` (mirror the
   2026-06-05 retirement at `router.rs:1018`).
2. Delete `route_cache.rs`, `circuit_breaker.rs`, `bypass.rs` + their tests.
3. Remove `ORCHESTRATOR_ROUTE` / bypass constants from `control.rs`.
4. Confirm `session discover` + `channel claim` cover the mechanism an AEF
   orchestrator needs; document the relocation in AEF if any policy is wanted.
5. `cargo test -p termlink-hub` green; capability-negotiation test updated.

---

*Cross-refs: `docs/CHARTER.md` non-goal #4 · T-2468 purpose review · the T-233 /
T-239 / T-240 / T-247 pre-charter design lineage · T-2483 charter-drift canary
(guards the tool surface) · register memory `project_t2468_purpose_review` P65/P66.*
