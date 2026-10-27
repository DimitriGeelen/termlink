<!-- T-3612 round 1, seat opencode, source /tmp/tl-dispatch/t3612-r1b-opencode-2c26b7e09df2/result.md, exit 0 -->
I have the core intent documents. Now let me mine the saved directory listings for recent activity (last two months ≈ T-3xxx tasks, S-2026-10 handovers) and read the arc YAMLs.
Now reading the arc YAMLs (initiative definitions) — these show the direction over the last two months.
I have enough evidence. Here is my independent review.

---

# T-3580 — Independent read of TermLink @ 50dece865 (seat: GLM)

## 1. What TermLink IS today

TermLink is a Rust workspace (nine crates under `crates/`: `termlink-protocol`, `-session`, `-hub`, `-bus`, `-mcp`, `-cli`, `-sidecar`, `-doctor`, `-test-utils`) implementing a hub-mediated message bus whose endpoints are real PTYs: sessions register with a star-topology hub (Unix socket same-host, HMAC+TOFU-TLS TCP cross-host), exchange durable messages on retention-bounded append-log "channel" topics with offsets, `(sender, client_msg_id)` dedupe, acks/dead-letters and an offline queue, claim/lease work units with transfer and expiry, and can exec, stream, attach to and inject keystrokes into each other's terminals — surfaced through ~30 CLI command groups and a 276-tool MCP server (`README.md`). What the code is *used* for, beyond what any doc claims, is visible in the repo itself: an always-on sidecar crate and `scripts/notify-*` rail that deliver agent mail into idle prompts (arc-011 slices S1–S12), ~11 daily canaries under `.context/cron/`, a 116KB `runme.sh` of operator one-command actions, and a git log whose recent commits are authored by `fw worker (termlink-dispatch)` and multiple `reviewer-judge-*` identities (e.g. commits on 2026-10-27). TermLink is both the product and the substrate its own agent-driven development runs on.

## 2. What it is FOR

Its users are one human operator (Dimitri Geelen) and a fleet of AI coding agents — primarily Claude Code workers, with codex, antigravity and GLM appearing as reviewer seats in commit authorship (T-1498 "codex seat", T-1498/1506 reviewer verdicts, 2026-10-27) — spread over a small multi-host fleet (hosts .107/.121/.122/.141, a laptop, the "ring20" management node; T-1015, T-2389, T-3557). The jobs: dispatching parallel workers over isolated workspaces and collecting them (`dispatch`, T-3469/T-3477); durable agent-to-agent DMs with receipts and push-wake into terminals (arc-003/004, arc-011); remote observation and control of live coding sessions (`pty attach`, `scripts/tl-claude.sh`); claiming work (channel claims); and fleet health telemetry riding the bus itself (T-3466, O-5). If it disappeared, what is lost is not a chat tool but the entire operating model: the AEF-governed process that produced ~2,000 commits in two months, its task ledger, handovers, canaries and operator decision queue all ride TermLink — the repo is the proof of load.

## 3. What it is BECOMING (2026-08-27 → 2026-10-27)

- **From bus to trusted delivery contract.** arc-011 "Agent-to-agent message delivery: mailbox to prompt" (created 2026-09-22) dominates the window: an operator interview (2026-10-04..06, T-3344) fixed a stage chain SENT→RECEIVED→STORED→HANDED_OVER→REPLIED, "urgent never injects into a busy prompt" (SQ-4), a hub-independent sidecar with portable respawn (SQ-8, 2026-09-24), and "accepted means stored" became the O-2 wording (T-3371 ruling 2026-10-21).
- **Hub as authoritative record.** arc-012 "Hub as the authoritative coordination record" (T-3304, created 2026-10-01): gap notices instead of silent skips, hub-side topic records and backfill, retention ceilings, forever-topic review (T-3314) and a rewording of charter non-goal 2 (T-3315) — i.e. the non-goal is being *edited to match the direction*, not the direction blocked by the non-goal.
- **A shared dispatched-worker contract.** T-3484, T-3501..T-3504 (Oct 2026): one authoritative dispatch record TermLink and AEF/ring20 agree on, conformance vectors (T-3517, T-3530), clone-per-claim workspaces (T-3469 IW-3), container isolation (T-3477), and sev-1 cross-project environment-leak hardening (T-3549..T-3553).
- **From implicit trust to attribution.** New O-7 (2026-10-21): signature verification on read (T-3486), per-agent identity keys (T-2292), signed inject + spawn-time allowlists (T-3534), fail-closed on missing keys (T-3467).
- **Governance turned on itself.** arc-008 audit remediation (2026-09-09) and arc-009 five-run value-review execution (2026-09-19); objectives authored and twice operator-ruled (T-3371, 2026-10-20/21); staged reviewer sweeps of old human-ticked tasks (T-3592, 2026-10-27); routine external multi-vendor consults (T-3335, T-3344 review panels; T-3586).
- **Deepening AEF interlock.** A constant re-vendor chain (T-3554..T-3598) — TermLink increasingly co-evolves with the framework rather than as a standalone library.

## 4. Proposed purpose statement

**TermLink is the coordination substrate a single operator and their fleet of AI agents run on: through a hub they discover each other, exchange durable, signed, receipted messages, dispatch and hand back work, and control terminal sessions — with every failure loud and every action attributable.** Delivery truth is the product: a message counts only when stored, and its loss is surfaced, never silent. Terminals are first-class endpoints, not transport detail — the fleet wakes, observes and types into real sessions. It is deliberately mechanism: orchestration *policy* lives above it (AEF), and it refuses to be a database, a federation, a social platform or a multi-tenant security boundary.

## 5. Proposed objectives

1. **No silent loss:** every posted message is stored or loudly failed, with a sender-visible stage chain. *Measure:* `scripts/notify-rail-e2e.sh` full-stage PASS daily; dead-letter/unconfirmed canary logs empty; the known restart-dedupe gap (G-088, T-3515) closed by test.
2. **Trivially small onboarding for a new machine/agent:** install → hub join → first receipted message in one command. *Measure:* timed cold-start canary on a fresh Linux and macOS host, ≤5 min, run per release.
3. **Dispatched work done-and-recorded exactly once** across TermLink and AEF. *Measure:* shared conformance vector file green against the hub backend (T-3517); zero double-counts in 30 live days.
4. **Failure loudness:** every comms failure mode surfaces in-band at the moment it happens. *Measure:* arc-007's loud-contract checklist as an executable prover; count of failure modes discovered only by out-of-band canary → 0.
5. **Attribution:** every message, injection and lifecycle write traceable to a verified identity; no self-signoff. *Measure:* signed-inject allowlist enforced (T-3534); supervisor token live (T-3501); audit finds no unsigned lifecycle writes.
6. **The fleet runs what is shipped:** releases published and every host's binary/capabilities and daily health collected. *Measure:* release-publication, fleet-binary and telemetry-fleet-view canaries green (O-5's existing measures).
7. **The substrate stays cheap:** MCP tool catalog and context footprint bounded. *Measure:* tool-description total bytes ≤ target from arc-005 (was ~156KB/39k tokens), anti-regrowth guard green.

## 6. Proposed value drivers

Weights 0–10, judged per unit of work on TermLink:

- **Delivery truth — 9.** The load-bearing promise; everything else is built on it. (Promote arc-011's `AS_DELIVERY_TRUTH` to global.)
- **Loud failure observability — 8.** arc-007's axis; the difference between a fleet that runs unattended and one that silently rots.
- **Antifragility — 7.** Keep (existing D1): hub blips, restarts, dead workers must make the system stronger; proven by provers like `demo-ws-push.sh`.
- **Operator burden — 7.** One human is the scalability constraint; decisions-not-courier (O-6, ruled 2026-10-21).
- **Attributable trust — 6.** New O-7's axis; rising, will need weight as multi-agent signoff lands.
- **Substrate leanness — 5.** Context tax per agent (arc-005's 39k tokens) and surface-area discipline (keep `mcp-slimming` honest).
- **Portability — 3.** Keep (D4): macOS/Linux parity, install paths, no systemd-only assumptions (SQ-8 already ruled this).

**Vs `policy/value-drivers.yaml`:** keep D1–D4 as the protected chassis but re-weight as above (Reliability D2 is absorbed into Delivery truth; Usability D3 splits into Operator burden); **change** F-ORCH to a temporary driver to be retired when the worker contract (T-3484 ff.) lands — the file's own `retire_when` already says so; **drop** F-RECALL (recall/memory is the AEF framework's concern, not TermLink-the-product's — it scores substrate work 0 for reasons that have nothing to do with TermLink). Note the file itself is largely framework-inherited (it cites `lib/bvp.sh`, `web/blueprints/*`, which live in the vendored `.agentic-framework/`); the genuinely TermLink-specific criteria currently live as arc-scoped `AS_*` drivers (arc-002, arc-011) — consolidating those globally would be my main change.

## 7. Non-goals

Keep all five charter non-goals (no federation, no durable DB/system-of-record, no social platform, no orchestration engine, no multi-tenant security boundary — `docs/CHARTER.md`). Add: **not an agent runtime or harness** (it carries agent mail, it does not host agents); **not a product for third-party teams** — the trust model and coarse authorization are for a single-operator fleet, and pretending otherwise invites the wrong features. Drift to stop: (i) the chat-social surface already built (edit/redact/pins/typing presence, T-1530..T-1557) contradicts non-goal 3 and should be pruned or explicitly justified as coordination-serving; (ii) README drift ("Four crates" and a 5-row table vs nine crates; "276 tools, trimming in progress" for an arc idle since July — T-3125 flags zero task commits); (iii) governance meta-work (ledgers, reviewer sweeps, re-vendoring) consuming the majority of recent task slots without a bound — fine as investment, but it should be a visible, capped budget, not an accident.

## 8. Tensions and contradictions

- **Charter non-goal 2 ("not a system of record") vs arc-012 ("hub as the authoritative coordination record")** — a direct contradiction, handled honestly in-repo by rewording the non-goal (T-3315) rather than silently violating it; I believe the direction (record) is what the operator actually wants, given the 2026-10-21 O-2/O-3 rulings.
- **Non-goal 4 ("not an orchestration engine") vs arc-011's injector, priority queues and role leases** — resolved by SQ-1 (2026-09-23: "primitive, not engine"), but the boundary is re-litigated with each slice (S9's title said "bypass the wait"; the ruling inverted it). The non-goal is doing real work and surviving, which is a good sign.
- **README vs code:** crate table and MCP tool count both drifted; exactly-once is honestly caveated (G-088) while "Durability" headline-claims more.
- **Stated priority vs effort:** arc-005 `mcp-slimming` (2026-07-11) claims a 39k-token/agent tax yet has sat in-progress with no commits (T-3125), while enormous effort went to governance hygiene (T-3592 sweeping August-era task ticks on 2026-10-27) and AEF re-vendoring (T-3554..T-3598). The value-review machinery (arc-009) ranks framework health above this user-facing cost.
- **Trust model vs O-7:** "treat every authenticated peer as trusted" (README) coexists with an objective demanding per-action authorization; O-7's own measure admits the pieces are "not yet" built.
- **CLAUDE.md is 101KB** — the same context-tax problem arc-005 exists to fix, inside the repo that diagnosed it.

## 9. Confidence

- Q1 **high**, Q2 **high**, Q3 **high** — consistent evidence across README/CHARTER/objectives, all ten arc YAMLs, task-name corpora (~450 T-3xxx names in `.tasks/`), and sampled git log.
- Q4–Q7 **medium-high**: these are judgments grounded in the evidence above, but they are mine, not the repo's.
- Q8 **high** on the specific contradictions cited (each has a file/task citation); **medium** on the effort-distribution claim (I sampled commit messages, not a full statistical pass over ~2,000 commits).
- Not read/verified: any Rust source file contents (no local checkout, nothing executed); the 740-line arc-011 body beyond ~half; contents of handovers, `docs/reports/*`, `docs/design/*` and runme.sh (listings only); one task file in full (T-3371). Claims about behavior (latency numbers, canary greenness) rest on the repo's own reports, not independent verification.

*Note: I ran under a read-only constraint, so I could not write `.context/working/t3612/seat-glm.md`; this printed answer is the deliverable.*
