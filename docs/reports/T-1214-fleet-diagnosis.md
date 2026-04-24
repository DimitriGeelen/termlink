# T-1214: Fleet termlink diagnosis (T-1210 S1) + converge-vs-federate decision

**Date:** 2026-04-24
**Task:** T-1214 (build, agent-owned)
**Parent inception:** T-1210 (fleet termlink version-divergence, GO)
**Related:** T-1165 (pickup→channel bridge), T-1168 (channel:learnings)

## Purpose

Execute T-1210 Spike 1: probe every reachable termlink peer for binary lineage
and classify as **same-lineage / forked / stranger**. Decide converge-vs-federate.

## Capability matrix

| Peer | Addr | termlink version | `channel` subcmd | Binary path | Binary mtime | Source at `/opt/termlink`? | Lineage |
|---|---|---|---|---|---|---|---|
| local | 192.168.10.109 | **0.9.206** | **absent** (`unrecognized subcommand`) | `/root/.cargo/bin/termlink` | 2026-04-20 09:53 | yes, HEAD=dc111330 (**0.9.398**) | same-lineage, **install 192 commits behind source** |
| ring20-management | 192.168.10.122 | **0.9.844** | absent (T-1210 probe) | (not captured) | (not captured) | **no** (T-1210 probe) | **stranger** — version string > ours, no source, missing `channel`, hub lacks `command.exec` method |
| ring20-dashboard | 192.168.10.121 | **unknown** | unknown | unknown | unknown | unknown | **unprobed** — hub reachable (fleet-doctor pass, 43 ms), but `remote_list` returns 0 sessions → no agent to run commands on |

### Supporting evidence

**Local probe (2026-04-24T~10:10Z):**
```
$ which termlink       → /root/.cargo/bin/termlink
$ termlink --version    → termlink 0.9.206
$ termlink channel -h   → error: unrecognized subcommand 'channel'
$ stat termlink         → mtime 2026-04-20 09:53:32
$ git rev-parse HEAD    → dc111330  (source version 0.9.398)
```

**ring20-management (.122) probe:**
- Fleet-doctor: OK, 42 ms latency
- `remote_list ring20-management` → 1 session (`tl-schnqg3a`, state=ready)
- `remote_exec` → `Method not found: command.exec` on the remote hub
- T-1210 prior findings: version 0.9.844, no `channel`, no `/opt/termlink`

**ring20-dashboard (.121) probe:**
- Fleet-doctor: OK, 43 ms latency (G-013 appears healed vs. the concerns-register entry from earlier — auth-mismatch streak broken)
- `remote_list ring20-dashboard` → 0 sessions
- No agent process registered → cannot capability-probe binary

## Classification

- **.109 (local)** — same-lineage but install stale. Our own binary doesn't have the
  `channel` subcommand that the T-1155 channel bus is being built on. Any consumer
  that uses `termlink channel` via CLI will fail even locally until we rebuild.
- **.122 (ring20-management)** — stranger. Its version string (0.9.844) does not
  appear anywhere in our git history (our tip is 0.9.398); `channel` is absent;
  source tree is not at `/opt/termlink`; its hub rejects the `command.exec` method
  that our fleet-local hub serves. This is either a fork, a mirror from a different
  upstream, or an older binary with a forked version-numbering scheme. Treat as
  black-box until proven otherwise.
- **.121 (ring20-dashboard)** — unknown lineage. No runnable agent; can only
  capability-probe the hub itself via supported methods. Not actionable for
  convergence without a dashboard-side agent installing itself first.

## Converge-vs-federate

### Option A — Converge (unify installs across the fleet)

**Approach:** Establish `/opt/termlink` + matching binary at every peer from one
authoritative source. Ship a bootstrap that installs/updates the binary via ssh
or agent-driven self-update.

**Cost:**
- Need reliable install channel for each lineage (.122 is stranger — no source;
  would need to either push our source there or replace its forked binary).
- N peers × O(platform) install matrix.
- Destructive on .122 — overwriting an existing forked install risks breaking
  whatever depends on it.
- Even with convergence, version drift between commits will re-emerge unless we
  also ship a pull-update loop.

**Benefit:** `termlink channel` (and every future subcommand) works uniformly.

### Option B — Federate (capability-probe at call time)

**Approach:** The T-1155 channel bus layer does not rely on the `channel` CLI
being present at peers. Instead:
1. Probe each peer's supported JSON-RPC methods at hub-join time.
2. Build the adapter shim (T-1165) against the **JSON-RPC layer** (which is
   version-independent) rather than shelling out to `termlink channel`.
3. Peers without the method fall through to legacy `event.broadcast` +
   `inbox.*` primitives, which we already know are universally present.
4. Log capability mismatches to the concerns register so operators see what's
   missing where.

**Cost:**
- More code (shim has a fallback branch).
- Slightly heavier hub-join handshake.

**Benefit:**
- Zero install destruction on foreign lineages.
- Works with `.122` (stranger) without needing to touch its host.
- Works with `.121` (no agent) by querying hub methods directly.
- Drift-tolerant: new peers can join the federation at whatever lineage they run.
- Aligns with Directive 4 (portability / no lock-in).

## Recommendation

**Recommendation:** GO Option B (federate) — **DEFER** Option A.

**Rationale:**
- Directives ranking: Antifragility (stranger lineages are a given, not a bug to
  eliminate) + Portability (don't force uniformity) pull toward B. Usability
  (consistent CLI surface) pulls toward A, but only mildly — the CLI is already
  non-uniform *today* and nothing is catastrophically broken, so the pressure
  is low.
- Evidence: .122 is stranger; we have no safe way to converge it without risking
  its existing workload. .121 has no agent; we cannot converge what we cannot
  reach. Even our own .109 install is stale — convergence would not be a
  one-shot fix; it would be a recurring maintenance cost.
- T-1165 (pickup→channel bridge) is the next build that *needs* this decision.
  It can proceed against the JSON-RPC layer immediately under Option B; Option A
  would block it on a fleet-wide install refresh.

**Scope of B:**
- T-1165 implements the bridge against `channel.post` / `channel.subscribe` JSON-RPC
  methods directly.
- Hub-side capability probe: add a `hub.capabilities` method to our hub impl and
  have clients cache the supported-method list per peer at connect time.
- Fallback adapter: when `channel.post` is absent, route through
  `event.broadcast` + `inbox.post` (legacy primitives that all known lineages
  speak).
- Rebuild local (.109) install so the CLI is at least parity with source — this
  is a one-shot (`cargo install --path .` or equivalent), not a fleet operation.

**Deferred (Option A):**
- Fleet-wide unified install is a future decision. Revisit if (a) a second
  stranger-lineage peer appears and shim maintenance becomes painful, or (b) a
  feature ships that the JSON-RPC layer fundamentally cannot express.

## Follow-ups

1. Rebuild local (.109) termlink from current source (`cargo install --path crates/termlink-cli`) so `channel` subcommand exists where T-1155 is being tested.
2. Update T-1165 description to reference Option B (federate): build against JSON-RPC, not CLI.
3. Add `hub.capabilities` method + client-side cache to hub impl — separate task (capture).
4. Register capability-mismatch logging into concerns register on first observed divergence (structural memory, so drift is surfaced not silent).
5. G-013 appears healed (ring20-dashboard fleet-doctor OK) — verify concern can be downgraded.
