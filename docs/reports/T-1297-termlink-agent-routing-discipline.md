# T-1297: TermLink agent-routing discipline — research artifact

**Status:** Inception (pre-spike)
**Created:** 2026-04-26
**Owner:** human
**Related:** T-1291 (declarative heal), T-243 (multi-turn agent dialog)

## The incident that triggered this inception

2026-04-26: email-archive agent on .107 was coordinating cross-host work for the email-archive cutover to Penelope LXC on Ring20. It needed to delegate two pieces of infrastructure work:

- T-1191 — provision Penelope LXC (CT 250 on proxmox2)
- T-1194 — create Garage S3 bucket + RW key for email-archive blobs

It emitted both delegation events to **`framework-agent`** on the .107 hub (192.168.10.107:9100). framework-agent is a governance-role session — it handles `channel:learnings`, framework pickups, and cross-project framework coordination. It is NOT the email-archive product session.

Operator caught it ("why do put framework-agent as source agent to contact SILLY?"). Agent re-emitted both events on the right bus (`email-archive`, session `tl-pljpkait`). The wrong-bus traffic stayed on the hub at framework-agent's seq 906/907 — no harm, just nobody listens on that channel for `infra.*` topics.

Cost so far: one human-cycle of catch-and-correct. Cost projected as more product agents come online: linear with session count.

## Dialogue Log

**Q (operator):** how can .121 contact you?

**A (agent):** [walked through the TermLink hub address card — gave `framework-agent` as the target on .107]

**Q (operator):** [tested the path; 5-min poll, no reply]

**A (agent):** [diagnosed PTY inject vs bus events, recommended event.broadcast on bus topics — framework-agent topic]

**Q (operator):** why do put framework-agent as source agent to contact SILLY?

**A (agent):** [acknowledged routing mistake; identified `email-archive` (id `tl-pljpkait`, role `email-archive`) as the correct session; re-emitted T-1191 + T-1194 delegations on email-archive bus seq 213/214; left framework-agent emits at seq 906/907 in place as orphans]

**Q (operator):** do we need a structural fix?

**A (agent):** Yes — recurring class. Three options proposed:
1. `termlink whoami` — agent self-lookup, removes the guess.
2. Topic↔role lint at emit time — catches misroute as a warning.
3. framework-agent rejects non-framework topics — destination-side hard reject.

Initial pick: (1) + (3).

**Q (operator):** 3 not sure can that any agent is asked to relay topics

**A (agent):** Right — destination-rejection breaks legit relay. Strike (3). Revised pick: (1) + soft hub warning on self-emit role mismatch.

**Q (operator):** why not 2 also??

**A (agent):** Conflated (2) with (3) and threw both out. (2) actually survives the relay test: when framework-agent forwards `infra.lxc.reply` to .107, the destination IS email-archive (role match), not framework-agent. (2) catches *originator confusion*, not *forwarder traffic*. Revised pick: (1) + (2).

**Q (operator):** [authorize inception capture] yes

**A (agent):** [this artifact + T-1297 task]

## Three-option matrix (post-dialogue)

| Option | What it does | Relay-safe? | Maintenance cost | Verdict |
|---|---|---|---|---|
| 1. `termlink whoami` | Read-only RPC: returns caller's session identity on hub. | Yes (read-only). | Low — derived from existing session registry. | **In** — root-cause fix. |
| 2. Topic↔role soft-lint at emit | Warns when self-emit topic doesn't match self-role tags. `relay_for` declarations suppress false positives. | Yes (relay = different destination role, not destination=self). | Medium — small mapping table grows slowly with topic catalog. | **In** — defense in depth. |
| 3. framework-agent rejects non-framework topics | Hard reject at destination. | **No** — breaks legit relay where framework-agent forwards cross-project traffic. | Low. | **Out** — destination-rejection is the wrong layer. |

## Why both (1) and (2), not just (1)

Single-mechanism fixes have a known failure mode: the mechanism becomes optional, agents skip it, regression returns silently.

- (1) alone: agents call `whoami`, get the right answer, emit correctly. But a stale memory entry, a buggy agent, or a copy-pasted command from another project skips `whoami` — and we're back to the original failure with no detection.
- (2) alone: catches misroutes, but every misroute is a warning event the operator has to read and triage. Without (1), agents have no cheap way to do the right thing pre-emit; they get warned but can't easily self-correct.
- (1) + (2): (1) provides the easy path to right; (2) catches when agents skip (1) or when (1)'s answer is wrong. Failures are caught at both layers.

## Pre-spike inclination

GO on combined fix. Decomposable into:

- **Build A** — `termlink whoami` RPC + CLI subcommand. ~½ day.
- **Build B** — Topic↔role mapping format + soft-lint hub-side. ~1 day.
- **Build C** — `relay_for` per-session declaration + integration with (B). ~½ day.

Total: ~2 dev-days. Reversible (lint can be disabled per-emit, RPC is additive).

Locked AFTER spikes 1-3 in T-1297.

## Spike scope refresher

- **Spike 1 — Quantify.** Walk recent emit history. Count misroutes. Goal: confirm >1 incident or de-prioritize.
- **Spike 2 — `whoami` prototype.** Verify lookup is unambiguous in current 7-session bus.
- **Spike 3 — Mapping table format.** Centralized vs distributed; opt-in `relay_for` shape.

## Out-of-scope (deferred to follow-up inceptions)

- **Auto-rewrite of misrouted emits.** If we can detect, we could in principle rewrite. But auto-rewriting traffic without human-in-the-loop is a sovereignty concern. Detection first.
- **Cross-hub topic propagation rules.** Single-hub first.
- **Hub-driven session capability advertisement.** A larger change that subsumes the topic↔role mapping into a general capability protocol. Worth its own inception if (B) feels too rigid.

## Spike 1 — Quantify (executed 2026-04-26)

**Method.** Used `termlink topics` to enumerate live topics across all 7 sessions on the .107 hub, then `termlink events --target framework-agent --topic <name>` to inspect payloads of suspect topics on the governance session. Each event was classified by inspecting the payload for originator/target markers (`_from`, `from`, `requesting_session`, `needs`, `relay_target`).

**Findings.** Five confirmed misroutes on `framework-agent` (governance session) — events whose payloads explicitly name a non-framework originator AND a non-framework intended destination, yet were emitted on framework-agent's bus:

| seq | topic | originator | declared target | task |
|---|---|---|---|---|
| 224 | `infra.qdrant.down` | email-archive | ring20-management-agent | T-1042 |
| 231 | `infra.qdrant.down` | email-archive | ring20-management-agent@.122 | T-1042 |
| 688 | `oauth.redirect-uri.help-requested` | email-archive@.107 | ring20-management-agent | T-1184 |
| 906 | `infra.lxc.delegate` | S-2026-0426-resume-post-1316 (.107) | ring20-management-agent (`relay_target` field) | T-1191 |
| 907 | `infra.s3.bucket.delegate` | S-2026-0426-resume-post-1316 (.107) | ring20-management-agent (`relay_target` field) | T-1194 |

framework-agent topic catalog (73 topics) also shows pervasive product-prefix leakage — `email-archive.t11{74,77,78,79}.*`, `dashboard.{rekey,sibling,gap}.*`, `penelope.cutover.*`, `gpu.coordination.*`, `outage.qdrant`. Each is a separate originator-confusion case, not counted in the 5 above because they could plausibly be intentional framework-relay broadcasts. Conservative lower bound is what's tabulated.

Volume context: framework-agent next_seq=914 over 8 days (~114 emits/day); 5 confirmed product/infra misroutes is ≥3-incident threshold. Approximate misroute rate among product/infra-prefixed events: 5 / ~30 product-prefixed emits sampled ≈ 17% (very rough — the catalog suggests this is an underestimate).

**Bug bonus.** One topic name on framework-agent is literally `learning.shared</topic>\n<parameter name="from">email-archive` — a malformed XML emit from an agent that interpolated parameter syntax into the topic name. Independent of misrouting, this points to insufficient validation of topic strings on emit. Out-of-scope for T-1297; flag for separate follow-up.

**Verdict.** GO criterion 1 ("≥3 misroute incidents in last 30 days") **satisfied** — 5 distinct events with unambiguous payload-level evidence, spanning 4 distinct topics and 3 distinct originating tasks (T-1042 / T-1184 / T-1191+T-1194). Pre-spike inclination upheld; no surprises that demand redesign.

**Bonus design signal.** Every misrouted event in the table carries a `relay_target` / `needs` / target-naming field in its payload — agents already encode their intended destination at emit time. This means a soft-lint at emit (option 2) has high-quality input data: it can compare `topic_prefix` against `payload.relay_target`/`payload.needs`/payload-declared `from` and warn when they don't reconcile. Strengthens the (1)+(2) pick.

## Spike 2 — `whoami` prototype shape (executed 2026-04-26)

**Method.** Inspected `termlink list` output for all 7 sessions, mapped `cwd → session(s)`, and checked existing CLI surface (`termlink info`, `termlink status`, `termlink list`) for self-lookup capability.

**Cwd collision matrix in current bus.**

| cwd | sessions | roles |
|---|---|---|
| `/opt/termlink` | framework-agent, termlink-agent, g046-mirror | [framework,pickup], [termlink,diagnostics], [] |
| `/opt/050-email-archive` | email-archive | [email-archive,pickup] |
| `/003-NTB-ATC-Plugin` | ntb-dev-test | [] |
| `/opt/3021-Bilderkarte-tool-llm` | push, push2 | [], [] |

**Finding.** Cwd alone is insufficient — 5 of 7 sessions share their cwd with at least one other session (collision rate 71%). A `whoami` that takes only cwd will return ambiguous results in the most common case (multi-role hubs are exactly the topology this primitive serves).

**Existing CLI surface.** `termlink info` reports runtime/hub state, NOT caller identity. `termlink status` queries an OTHER session by ID. There is no current `whoami`-class read primitive. So Spike 2 is additive; no existing call needs reshaping.

**Disambiguator design (sketch).**

1. **Primary:** `TERMLINK_SESSION_ID=<id>` env var, set at session creation (inside the shell `termlink register` spawns), inherited by child processes. `whoami` reads env var → calls hub for full identity card by ID. No ambiguity possible.
2. **Fallback (no env var):** hub walks caller's source-PID tree, finds nearest registered session whose `pid` field is an ancestor of the caller. Works for cron jobs, watchdog scripts, sub-shells.
3. **Tertiary:** if neither, return `{session_id: null, candidates: [...by cwd...]}` with hint `"set TERMLINK_SESSION_ID=<id> in your environment, or run 'termlink register' to claim a session"`. Caller chooses.

Hub already has all the data — sessions registry tracks `(id, display_name, roles, tags, cwd, pid, hub_address)`. `whoami` is a pure exposure of existing state, no new data model.

**Edge cases checked.**
- Multiple roles per session (e.g. framework-agent has `[framework, pickup]`) — return all roles, callers compare against topic mapping. No collapse to single role.
- Cross-host caller — out of scope per Scope Fence (single-hub first).
- Stale registration (heartbeat older than threshold) — hub already filters; whoami sees only ready sessions.

**Verdict.** Lookup is unambiguous **once disambiguation is added** (env var = canonical, PID tree = robust fallback). Without it, 71% of current sessions need user-driven disambiguation — usable but worse UX. GO criterion 2 ("no edge cases that demand redesign") satisfied — env-var injection at register-time is a small, additive change to the existing register flow, not a registry restructuring.

## Spike 3 — Topic↔role mapping shape (executed 2026-04-26)

**Method.** Drafted a mapping table from the actual topic catalog (125 distinct topics across 7 sessions, harvested from `termlink topics`) and tested whether <50 rules cover the catalog (the NO-GO threshold).

**Draft rule set (10 prefix rules).**

| prefix | role | example topics from current catalog |
|---|---|---|
| `framework:*` / `framework.*` | framework | `framework:pickup`, `framework.gap` |
| `channel:*` / `channel.*` | framework | `channel:learnings`, `channel.delivery` |
| `pickup.*` | framework | `pickup.received`, `pickup.inception`, `pickup.cross_project` |
| `learning.*` | framework | `learning.captured`, `learning.shared` |
| `inception.*` | framework | `inception.kickoff`, `inception.recommendation`, `inception.review` |
| `claude.md.*` | framework | `claude.md.codification` |
| `gap.*` | framework | `gap.cross-consumer-query`, `gap.verification` |
| `peer.*` | framework | `peer.pattern.instance`, `peer.status.update` |
| `task.*` | framework or product (ambiguous) | `task.complete`, `task.progress` — split: `task.complete` framework-relay, project-tagged updates product |
| `infra.*` | infrastructure (ring20-management role) | `infra.lxc.*`, `infra.s3.*`, `infra.qdrant.*` |
| `oauth.*` | product (originator's role) | `oauth.redirect-uri.*` |
| `outage.*` | infrastructure | `outage.qdrant` |
| `<project>.*` (e.g. `email-archive.*`, `dashboard.*`, `penelope.*`) | product (matching role tag) | many |
| `agent.{request,response}` | RPC primitive — exempt from lint | |
| `session.*`, `worker.done`, `test.*`, `help.*` | system primitives — exempt | |

**Coverage check.** 10 prefix rules + 4 exempt categories cover 119 of 125 catalog topics (95%). The remaining 6 are the malformed-topic-name bug (1) and 5 ambiguous "is this a product project name?" cases (`gpu`, `data`, `export-review`, `review`, `ship`) that warrant operator review on first sight, not blanket rules.

**Format decision: centralized hub-side YAML.**

```yaml
# /var/lib/termlink/topic_roles.yaml — hub reads at startup, hot-reloads on SIGHUP
rules:
  - prefix: "framework"
    roles: [framework, pickup]
  - prefix: "infra"
    roles: [ring20-management, infrastructure]
  - prefix: "oauth"
    roles_from_originator_role: true   # match originator's product role; warn if mismatched
exempt_prefixes: [agent., session., worker., test., help., channel.delivery]
```

Centralized over distributed because: (a) prefix conflicts need a single arbiter, (b) lint must apply uniformly across all sessions on a hub, (c) operator can audit the entire policy in one file. Per-project distributed mappings push complexity into N agents that may disagree.

**`relay_for` opt-in shape.**

```toml
# ~/.termlink/hubs.toml addition (per session, or per profile)
[session.framework-agent]
relay_for = ["channel.delivery", "task.complete", "learning.*"]
# Lint suppresses warnings when this session emits these prefixes,
# even if the prefix would normally bind to a different role.
```

Per-session declaration sits with other session config (cwd-adjacent), keeps the central mapping clean, and is reviewable. The matrix already documents that A1 (framework-agent stays multi-purpose) requires this opt-in.

**Maintenance test.** 10 rules + ~5 `relay_for` declarations across active sessions = under the <50-entries threshold and well under "5 min/week" maintenance budget. A new product session adds at most 1 prefix rule and 0-1 relay declarations.

**Verdict.** GO criterion 3 ("mapping format converges on something a single human can maintain in <5 min/week") satisfied. Decomposition test: Build A `whoami` ½d, Build B mapping+lint 1d, Build C `relay_for` ½d ≤ "≤3 build tasks each ≤1 day" — satisfied.

## Summary — all three GO criteria evaluated

| # | Criterion | Verdict | Evidence |
|---|---|---|---|
| 1 | ≥3 misroute incidents in last 30d (or ≥1% rate) | **GO** | 5 confirmed in Spike 1 with payload-level evidence |
| 2 | `whoami` lookup unambiguous, no design surprises | **GO (with env-var)** | Cwd-only is ambiguous (71%); env-var injection at register-time resolves it; existing registry has all needed fields |
| 3 | Mapping <50 entries, decomposable into ≤3 ≤1-day builds | **GO** | 10 prefix rules + 4 exempt categories cover 95% of 125 topics; A=½d B=1d C=½d |

NO-GO triggers checked:
- "Existing primitives cover at <30% cost" — no, current primitives have no concept of role-topic binding; building from scratch is the only path.
- "Pain entirely auth-driven" — no, all 5 misroutes succeeded auth-wise; routing-discipline is a distinct failure class above auth.
- "Simpler alternative emerges" — naming-convention enforcement at register-time was considered but doesn't address EMIT-time confusion (which is what Spike 1 measured).

**Locked recommendation:** GO on combined fix `(1) termlink whoami + (2) topic↔role soft-lint + (3) relay_for opt-in`. Decompose into Build A + Build B + Build C as scoped in pre-spike inclination. Total ~2 dev-days, reversible.

## Notes for next session

If picking up this inception cold: the dialogue log above captures the conversation that produced the option matrix. Spike 1 is the first hard evidence step — without misroute volume data, the inclination above is just opinion.
