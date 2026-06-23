# T-2249 — R1 re-scope: should `termlink register` publish presence? (inception)

**Arc:** arc-substrate-fitness (arc-002). **Anchor of finding:** T-2242 ingestion
plan §1 FLAG 1. **Workflow:** inception (Sovereign design decision — human decides
GO/NO-GO; agent advisory only). **Agent recommendation: NO-GO** (keep presence
opt-in; the `count=0` was operational, not a register code gap).

## 1. Problem

The arc-002 ingestion plan (`docs/plans/T-2242-substrate-fitness-ingestion.md`)
re-scoped **R1** (FLAG 1) to: *"extend `cv_key` emission to the `register`/PTY
presence producer path"* — described as a **minor build** ("small once R3 lands").
That framing assumes `termlink register` already posts heartbeats to the
`agent-presence` topic and merely omits the `cv_key` metadata field.

**Source verification (this task) reverses that assumption — a third
plan↔source contradiction, after FLAG 1 and FLAG 2.** The binding rule from the
plan §1 (§7 of the handoff) applies: *"if the repo contradicts a finding, the repo
wins."*

## 2. Verified source findings (the repo wins)

| Claim | Evidence (file:line) | Verdict |
|---|---|---|
| `register`'s heartbeat loop only touches a local JSON file | `crates/termlink-cli/src/commands/session.rs:294-307` — loop calls `ctx.registration.touch_heartbeat(&path)` and nothing else | confirmed |
| `touch_heartbeat` is local-only | `crates/termlink-session/src/registration.rs:325-329` — sets `heartbeat_at = now_iso8601()` + `write_atomic(path)`; no envelope, no post | confirmed |
| `register` command never references `agent-presence` / channel-post / presence | `grep` over `session.rs` → **zero** matches for `agent-presence`/`channel_post`/`.post(`/`presence` | confirmed |
| The `--self` twin behaves identically | `crates/termlink-session/src/endpoint.rs:175-190` (`heartbeat_loop`) — touch only | confirmed |
| `agent-presence` is fed ONLY by opt-in producers | `scripts/listener-heartbeat.sh:152-175` (`channel post … --metadata cv_key=$agent_id`, T-2107) and the MCP `listener_heartbeat` tool — both shell out to `channel post` | confirmed |
| The bus-side `agent-presence` references are read/retention only | `crates/termlink-bus/src/lib.rs:459,559` (consume + `LatestPerCvKey`), not a producer | confirmed |

**Conclusion:** the binary's `register` command is **not** a presence producer.
The frozen-husk canary (T-2239) tracks `register` sessions via their session-JSON
`heartbeat_at` files — a *separate* liveness channel from the `agent-presence`
topic. The two were conflated in the discovery.

## 3. Why the discovery saw `cv_index count=0`

Not a missing code path on `register`. The cv_index is in-memory (cleared at the
Jun-17 hub restart) and repopulated only by a `cv_key`-emitting heartbeat — i.e.
the `/be-reachable` producer (`listener-heartbeat.sh`). That producer was **not
running** (`~/.termlink/be-reachable.log` = "Terminated"; no `listener-heartbeat`
in `ps`). The running `register --shell` PTYs never fed `agent-presence` *by
design*. So `count=0` is **operational** (run `/be-reachable`), folding into R7,
not a build.

## 4. The actual design question (Sovereign)

R1-as-described does not exist. The genuine question the human must decide:

> **Should `termlink register` gain a new role as an `agent-presence` producer?**

This is a substrate-semantics decision, not mechanical work, because it changes
what "presence" *means* and adds a producing role to a currently local-only
command:

- **Today:** `agent-presence` = "agents that explicitly opted in via
  `/be-reachable`." Discovery (`find-idle`, `/peers`) shows opted-in workers.
- **If `register` auto-publishes:** presence = "every registered session,"
  including `--shell` PTYs never meant to advertise as dispatchable workers.
  This would also create a **third** presence producer overlapping the shell
  script + MCP tool, and add periodic hub traffic to a command that currently
  does only local file I/O (a portability/footprint cost — cuts against
  AS_RESOURCE_FOOTPRINT, the w4 driver).

## 5. Options (for the human)

1. **NO-GO (agent recommendation).** Keep presence opt-in; the dedicated
   `/be-reachable` producer owns `agent-presence` + cv_key. Re-scope R1 to
   operational ("ensure `/be-reachable` runs on hosts that should advertise") —
   folds into R7. Cheapest; preserves clean opt-in semantics; matches the
   already-shipped `LatestPerCvKey` model (one record per *opted-in* agent).
2. **GO — opt-in flag.** Add `register --publish-presence` (default OFF) so a
   session *can* feed `agent-presence` with `cv_key` without changing default
   semantics. Middle path; small surface; preserves opt-in default while letting
   an operator make a long-lived `register` session discoverable without a
   separate `/be-reachable`.
3. **GO — always-on.** `register` always publishes presence. Largest semantic
   change; not recommended (presence becomes a side-effect, not a choice;
   triple-producer overlap; footprint cost on every register).

## 6. Recommendation

**NO-GO** on always-on auto-publishing (option 3). Between option 1 and the
opt-in flag (option 2), the agent leans **option 1** (operational re-scope) as
the HV/LC close for arc-002: it requires no code, preserves the clean opt-in
model the substrate already encodes, and correctly attributes the `count=0` to
operations. Option 2 is a reasonable small build if the human wants `register`
sessions to be optionally discoverable — but that is the human's call, and it is
a *new feature*, not the "minor cv_key" residue the plan implied.

## 7. Sovereign boundary — what this task did NOT do

- Did **not** decide GO/NO-GO (agent advisory only; `fw task review T-2249` is
  the human's gate).
- Did **not** modify any source, hub state, or config — verification was
  read-only.
- Did **not** mint a build task — none is authorised until the human picks an
  option.

*Research is not authorization. The repo won the contradiction; R1's "minor
build" framing is retired in favour of this surfaced design decision.*
