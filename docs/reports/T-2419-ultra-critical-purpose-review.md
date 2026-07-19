# T-2419: Ultra-Critical Review — TermLink Purpose vs Current State

**Status:** in progress (research phase)
**Type:** inception (one question → one decision)
**Question:** Where has termlink drifted from — or failed to deliver — its core purpose,
and which gaps warrant scoped correction work now?

**Method (C-001):** artifact created before research; updated incrementally as evidence
lands. Four parallel evidence sweeps (stated purpose / feature-surface metrics /
failure-class history / field-usage reality), then synthesis, then per-gap GO/NO-GO
with scoped follow-up tasks.

---

## 1. Stated purpose and goals (evidence sweep A)

**Two eras, unreconciled:**
- **Era 1 (front door):** README.md:3-4 + Cargo.toml:18 + crate descriptions: "Cross-terminal
  session communication — message bus with terminal endpoints … discover, message, and
  control each other over Unix sockets." Use-cases lead with remote observation / PTY
  control. README advertises **37 MCP tools**; the live server exposes **276**.
- **Era 2 (the real mission):** docs/architecture/parallel-execution-substrate.md §1:
  "Move AEF from single-agent … to multiple agents executing tasks concurrently across
  ring20 … coordinated through TermLink" — a strict-star, hub-mediated, durable
  append-log coordination substrate with an 11-primitive work-stealing manifest (§6).
  All five recent arcs (substrate, substrate-fitness, reliable-comms, push-transport,
  mcp-slimming) pursue Era 2. README never mentions substrate, agents, claims, or
  federation.

**Missing from any purpose statement (should be headline for a comms substrate):**
(a) delivery guarantees (exactly-once via client_msg_id dedupe + receipt frontier —
buried in ADR §6.5); (b) trust model (two non-equivalent models: UDS UID-trust vs
HMAC+cert-pinned TCP, ADR:66-67, unification unshipped §7); (c) federation stance (open
question §8 — "more hubs in a star, never a spoke mesh" is stated but not committed);
(d) ordering/retention semantics (deep in §2/§6.10). The constitutional directives are
used as scoring axes in old reports, but no doc states what D1-D4 mean *for termlink*.

**Verdict (A):** crisp if you read the ADR; misleading if you read the README. A
front-door reader and an ADR reader would not recognize the same system.

## 2. Feature surface and sprawl metrics (evidence sweep B)

- **LOC:** ~164K total. CLI 66,819 + MCP 49,804 = **71%**; the actual
  transport/session/protocol/hub core = **29%** (46,911).
- **MCP:** **276 tools** in a single 45,619-line `tools.rs` (92% of the crate) —
  ~5x fan-out over the ~54 top-level CLI verbs. ~156KB of tool descriptions
  (~39K tokens) loaded into every consuming agent (arc-005's target).
- **Slice replication:** watch/notify/log/history axes stamped across dozens of verbs
  (430 `watch` lines, 299 `log`, 105 `history`, 96 `notify` in CLI) + MCP parity as a
  fifth axis. **15-18 pure helpers verbatim-duplicated** between CLI and MCP (T-2069
  "no cross-crate sharing" convention).
- **Tests:** 1,808 sync unit tests in CLI+MCP (62% of all tests) exercise
  formatting/parse helpers; async/concurrency tests concentrate in session (137) + bus
  (64) — the core is comparatively under-tested relative to its risk.
- **Ops:** 10 canary freshness scripts, 18 crontabs; only 3 of 39 ops docs describe the
  observability machinery — it outgrew its own documentation.

**Verdict (B):** the codebase is majority scaffolding for exposing and monitoring a
comparatively small core. The multiplicative slice pattern is the sprawl engine.

## 3. Failure-class history (evidence sweep C)

**Registers:** 44 concerns (21 open/watching), 271 learnings.

**Open-concern class breakdown (21 open):**
- Silent-failure / framework-blindness: G-085, G-083, G-069, G-063, G-061, G-084, G-015, G-053, G-066, G-068 — **10 of 21**.
- Deployment / binary-staleness / host-env: G-070, G-069 (dual), G-009, G-055.
- Auth / identity / authorization: **G-064 (hub has NO per-user authorization — every authenticated caller has equal privilege)**, G-011.
- Process/governance backlog: G-008 (64 partial-complete tasks), G-059, G-062, G-065, G-067, G-010.

**Learning theme recurrence (271 learnings, keyword-clustered):**
- stale binary / deploy / restart: **68**
- sed/parsing/tooling fragility: 61
- silent-failure / blindness / canary: **49**
- auth / rotation / secret: **43**
- identity confusion (host-key vs own fp): **38**
- volatile /tmp / runtime_dir: 21
- SIGPIPE / verification gate: 17
- per-hub topic semantics / federation absence: 10

**Verdict (C):** The same four structural classes recur regardless of how many
point-fixes land: (1) the system defaults to **silent failure** and each new
instance is answered with another out-of-band canary (11 canaries now — the
canary count IS the metric of the in-band loudness deficit); (2) **deployment is
the dominant incident source** (68 learnings) because there is no managed
upgrade/restart story — binaries go stale invisibly, processes detach from
systemd, /tmp eats runtime state; (3) **identity is structurally weak** — 38
learnings about "who actually sent this" (shared host key d1993c2c vs per-agent
fp), and G-064: no authorization model at all once authenticated; (4) federation
absence (G-060/PL-176) keeps resurfacing as diagnostic confusion.

## 4. Field-usage reality (evidence sweep D)

**Topic census (local hub .107):** 1581 topics. dm:* 198, agent-conv-* 144,
agent-* 139, explicit test/smoke 150, broadcast 1, "other" 949 — of which
**617 are `t-*`/T-XXXX smoke debris, 107 `xhub-*`, 96 `stress-*`** test residue.
≈55-60% of all topics on the production hub are test debris that no retention or
cleanup path ever removes. Top real traffic: agent-presence 8865,
agent-chat-arc 8364 (heartbeat-dominated), then DM threads (127, 91, …).

**Substrate observability usage:** `~/.termlink/find-idle.log`, `governor.log`,
`claims.log`, `queue.log`, `heal.log` — **ALL ABSENT**. rotation.log: 3 lines.
The watch/notify/log/history×CLI+MCP observability arcs (≈25+ tasks across
find-idle/claims/queue/governor/cv-index) have **zero field usage** of their
log/history layers. outbound.sqlite (28KB) and awaiting_ack.sqlite (16KB) show
the resilience write-paths DO get used.

**Self-inflicted scale problem:** `claims-summary --all` walking 1581 topics
trips the hub's own rate limiter (-32008) mid-walk — the observability verb is
DoS'd by the test debris the system never cleans up.

**Task economy:** 2058 completed tasks, 124 active (59 captured). G-008: 64
stuck in partial-complete.

**Verdict (D):** Real usage concentrates in a narrow core: presence heartbeats,
DM threads (doorbell+mail), chat-arc broadcasts, file transfer, remote exec, and
the resilience write path. The wide observability/history surface is unused in
the field; meanwhile the actually-used core sits on a hub polluted with test
debris that degrades the very verbs built to observe it.

## 5. Ultra-critical synthesis

TermLink's real purpose — a reliable coordination substrate for parallel agents — is
being delivered by ~29% of its code, while 71% is presentation and observability
scaffolding whose log/history layers show **zero field usage**. The project's failure
history says the things that actually hurt are NOT what the scaffolding observes:

1. **The front door lies.** README/Cargo still sell the Era-1 terminal-control tool
   (37 tools vs 276 live). Delivery guarantees, trust model, and federation stance —
   the three questions any substrate consumer asks first — are answerable only by
   archaeology in the ADR. This is a usability-directive failure at the cheapest
   possible fix point.
2. **Silent failure is the default; canaries are the crutch.** 10/21 open concerns and
   49 learnings are the same class: something broke and nothing fired. Eleven cron
   canaries now police the gaps. Each canary is an admission the in-band path stayed
   quiet. The loud-contract arc (T-2380) is the right structural answer and is only
   partially shipped.
3. **Deployment is the #1 incident source (68 learnings)** — stale binaries, detached
   processes, /tmp-volatile runtime state — and termlink has NO managed
   upgrade/restart story. Every fleet upgrade is a bespoke musl-build + scp + kill
   ritual (T-2409 did exactly this for .121). This is where reliability actually dies.
4. **The hub cannot forget.** No topic-deletion primitive exists (only retention
   trim). ≈60% of the production hub's 1,581 topics are test debris; walking verbs
   (claims-summary --all) now self-rate-limit on debris. The substrate pollutes itself
   and offers no mop.
5. **Identity/authorization is a two-legged stool.** 38 learnings on "who sent this"
   (shared host-key signing vs per-agent fp, T-1693 designed not shipped), and G-064:
   once authenticated, every caller has equal privilege — no authorization model at
   all. Fine for a trusted homelab; fatal for the stated multi-agent mission if any
   boundary is ever crossed.
6. **The sprawl engine is still running.** The 5-slice convention (watch/notify/log/
   history/MCP-parity) multiplies every new verb by ~5 in code, tests, and agent
   context cost, and duplicates helpers across crates by policy (T-2069). Field
   evidence (sweep D) shows the marginal slices go unused. arc-005 trims descriptions;
   nothing yet stops the next arc from stamping five slices again.

## 6. Identified gaps and per-gap disposition

| # | Gap | Disposition |
|---|-----|-------------|
| GAP-1 | Front-door purpose drift (README/Cargo era-1, no guarantees/trust/federation statement) | **BUILD NOW** (T-2420): rewrite README front matter + Cargo description to the substrate identity; state delivery guarantees, trust model, federation stance, true tool count. Grep-testable. |
| GAP-2 | No topic-deletion primitive; hub self-pollutes with test debris | **BUILD NOW** (T-2421): `channel.delete_topic` RPC (auth-gated, explicit-name only, no wildcard) + CLI `channel delete` + tests; then debris sweep runbook. |
| GAP-3 | MCP context bloat (276 tools, ~39K tokens/agent) | **EXISTING ARC** — advance arc-005 (T-2408 S3 long-tail trim) rather than new work. |
| GAP-4 | No authorization model beyond authentication (G-064) | **INCEPT-DEFER** (T-2422): needs its own inception + operator threat-model decision. Not buildable blind. |
| GAP-5 | No managed deploy/upgrade story (68 learnings, #1 incident source) | **INCEPT-DEFER** (T-2423): options (self-update verb, fleet-deploy hardening, systemd-first contract) need operator input on fleet topology. |
| GAP-6 | 5-slice sprawl convention keeps multiplying | **POLICY** (recorded as learning + CLAUDE.md note via arc-005): new verbs ship core+JSON only; watch/notify/log/history slices require field-demand evidence first. |
| GAP-7 | Silent-failure default | **EXISTING ARC** — comms loud-contract (T-2380 et al.) continues; no new filing. |

**Decision:** GO — the review found actionable purpose-level gaps; GAP-1 and GAP-2
built+tested this session; GAP-4/GAP-5 filed as inceptions for operator decision;
GAP-3/GAP-6/GAP-7 routed into existing arcs/policy.

## Dialogue Log

- 2026-07-19: Operator directive: "please ultra critically review termlink's purpose and
  goals and identify gaps or needed adjustment, incept these and build these and test
  these, drive to completion." Autonomous execution authorized; scoped follow-ups to be
  built and tested this session where completable.
