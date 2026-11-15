# T-1432 Human AC#1 — evidence report (rung-5-panel:claude-code)

Revision reviewed: `9eebc403b7d1773914434e939d3d02275f5b0f9d` (HEAD at review time).
CLI: `termlink 0.12.13+32.g023fbc762`. Date: 2026-11-15.
Verdict: **amber**.

## Step 1 — baseline on the live fleet (run, read-only)

`termlink fleet doctor --legacy-usage`:

```
=== T-1166 cut-readiness (7d window) ===
Verdict: UNCERTAIN
  total legacy invocations across fleet: 0
  CLEAN (7d): greenfield, local-hub, local-test, ring20-dashboard, ring20-management, workstation-107-public
  UNSUPPORTED (pre-T-1432, upgrade to measure): node1-pool
Fleet summary: 8 hub(s), 7 ok, 0 warn, 1 fail
```

laptop-141 failed the connectivity probe (TOFU fingerprint changed, T-3801) and is
correctly left out of the legacy verdict.

## Steps 2–3 — increment and reset (run on a throwaway hub, NOT on a fleet hub)

I did not fire a legacy call at any fleet hub: that is a mutating action on shared
infrastructure and restarts that hub's 7-day clean window. Instead I started an
isolated hub from the same binary (`HOME` and `TERMLINK_RUNTIME_DIR` under `/tmp`,
TCP 127.0.0.1:19177, its own `hubs.toml` with one profile `iso`), exercised it, then
stopped it and removed the directory.

| Action | `hub.legacy_usage` / doctor result |
|---|---|
| fresh hub, no calls | `total_legacy: 0`, `audit_present: false` |
| sent `inbox.push`, `file.send`, `inbox.list` (from=`t1432-reviewer`) | `total_legacy: 2` — `file.send` 1, `inbox.list` 1. `inbox.push` was written to `rpc-audit.jsonl` but NOT counted |
| doctor | `Verdict: WAIT`, `iso: 2 legacy invocation(s) — last call 16s ago (ACTIVE)`, `2× t1432-reviewer` |
| one more `file.send`, doctor | `3 legacy invocation(s) — last call 0s ago (ACTIVE)` |

So the count increments and the recency resets. Two literals of the criterion do not
match the product:

- `inbox.push` is not in the tracked set (`LEGACY_METHODS` in
  `crates/termlink-hub/src/rpc_audit.rs`: `event.broadcast`, `inbox.list`,
  `inbox.status`, `inbox.clear`, `file.send`, `file.receive` + chunked variants), and
  no `inbox.push` method exists in `crates/`. A reviewer who follows Step 2 with
  `inbox.push` sees no increment.
- `days_since_last` exists nowhere in `crates/`. The doctor prints
  `last call <N>s/h/d ago`; the JSON field is `last_legacy_ts_ms`.

## Step 4 — flip to CUT-READY after 7 clean days

Simulated on the same throwaway hub by ageing its own audit rows:

| Audit rows aged to | Doctor verdict |
|---|---|
| 8 days old | `CUT-READY`, `total 0`, `CLEAN (7d): iso` |
| 6d23h old | `CUT-READY-DECAYING`, `3 legacy invocation(s) — last call 6d ago (decay residue)` |

The window logic is right. On the LIVE fleet, however, the verdict today is
`UNCERTAIN`, not `CUT-READY`, so Step 4 as written is not met.

## Finding — the UNCERTAIN verdict gives a wrong reason and a wrong remedy

`node1-pool` reports hub version `0.12.1084`, yet the doctor says
`UNSUPPORTED (pre-T-1432, upgrade to measure)` with the hint
`upgrade to >=0.9.1640`.

- `remote.rs:5364` maps EVERY failure of the `hub.legacy_usage` call
  (`Ok(_) | Err(_)`) to `audit_unsupported` + "hub predates T-1432".
- `hub.legacy_usage` is not in the Observe arm of `hub_method_scope`
  (`server.rs:759-767` lists only `hub.version`, `hub.bus_state`,
  `hub.governor_status`), so it falls through to `auth::method_scope` → `Execute`.
- The doctor itself prints for node1-pool: `hub token ... carries scope 'interact',
  below the 'execute' this command asks for; the hub will refuse methods above 'interact'`.

The likely real cause is therefore a permission refusal, not an old hub, and upgrading
the hub would not clear it. I did not prove it from the hub's side (I have no
execute-scope token for node1-pool), so this is inferred from the code and the
doctor's own scope note. The task's 2026-08-27 capture shows `CUT-READY` with four
hubs; node1-pool joined the fleet since.

## Also out of date

"agent operators trust it enough to flip T-1166's `LEGACY_PRIMITIVES_ENABLED=false`":
the cut already landed (T-1166 `work-completed` 2026-08-20; `router.rs:1154`
"legacy_primitives is now hardcoded false (cut landed 2026-05-31)"). The doctor says
so itself: "verdict is informational". There is no flag left to flip.

## Not run

`cargo build` / `cargo test -p termlink-hub rpc_audit::tests::summarize` — not run
(read-only seat; the behaviour was measured end to end on the installed binary instead).
