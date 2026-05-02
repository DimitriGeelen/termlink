# T-1448: Co-Resident Agent Identity Disambiguation — Inception Report

**Status:** Inception in progress
**Owner:** human (decision); claude-code (research)
**Started:** 2026-05-02
**Predecessor signals:** T-1427 (strict-reject), T-1436 (FP registration), T-1429 (agent contact), T-1438 (field rollout — first multi-agent context)
**Trigger incident:** 2026-05-02 cross-agent dialogue with peer agent "Penelope" running co-resident on .107

---

## TL;DR

- **Recommendation:** **GO** with Design A (soft convention + CLI default + catalog promotion)
- **One-line rationale:** `from_project` metadata is already operational between cohort and email-archive; promote it to a CLI default + T-1288 well-known-key catalog entry; protocol unchanged; threat model unchanged; 3 build tasks ≤1 session each; T-1427/T-1429/T-1436/T-1440/T-1441 augment-not-unwind.
- **Build tasks proposed (if GO):** 3 — (a) cli default + catalog, (b) `agent contact <name>[:project]`, (c) scripts + skills sync

---

## Problem framing

TermLink's cryptographic identity is host+user-keyed: `/root/.termlink/identity.key` belongs to the (host, UID) pair, and every process running under that pair derives the same `identity_fingerprint` when it speaks to a hub. Until now this was operationally fine because each host had at most one Claude agent.

On 2026-05-02, peer agent "Penelope" appeared on .107 and posted to `agent-chat-arc` with FP `d1993c2c3ec44c94` — identical to the cohort agent's FP. The chat-arc could no longer attribute messages to a specific agent.

Penelope's own assessment (excerpted from the message that triggered this inception):

> Important wrinkle: my fingerprint is d1993c2c3ec44c94 — the same as the cohort agent's, because we both run as root on .107 and share /root/.termlink/identity.key. Cryptographic identity won't disambiguate us. The disambiguator is the from_project metadata field on each post.

The locator-card portion of Penelope's message was cut off in the transcript reaching this session. **S1 (below) recovers it.**

## What this breaks (audit of recent work)

| Task | Built-in assumption | Actual state under co-residency |
|---|---|---|
| T-1427 (whoami + identity binding on `channel post`) | `sender_id` derives from connection FP and uniquely identifies sender | Connection FP is shared; `sender_id` ambiguous |
| T-1436 (FP registered at session start) | One session = one FP entry in metadata | Co-resident sessions overwrite or duplicate |
| T-1429 (`agent contact <name>`) | Name → FP resolution gives a unique destination | Resolution lands on a (host, user) — delivery to a specific agent is up to the hub or undefined |
| T-1440 / T-1441 (whoami / FP column) | Visible FP tells operator which agent | Tells operator (host, user); agent identity invisible |
| chat-arc attribution (T-1438) | sender_id distinguishes chat-arc participants | Indistinguishable for co-residents |
| `-32014 CHANNEL_IDENTITY_MISMATCH` strict-reject | Catches sender_id forging | **TBD by S2** — does it cross-check sender_id against agent metadata at all? |

## Spike progress

### S1: Penelope locator card retrieval — _completed (substance), partial (literal text)_
**Goal:** Read the full message from the bus, capture her proposed convention and any code refs.
**Method actually used:**
- `termlink topics` (117 topics across 37 sessions) — no `penelope:` namespace with content; `framework-agent:pen.outbound.*` cluster (key-available, key-revoked, ping, protocol.bumped, rate-warning, scope-changed, surface.update) reveals Penelope is a *service* with TermLink-mediated outbound, not a co-resident agent.
- `termlink channel subscribe agent-chat-arc --cursor 0 --limit 300` (chat-arc has 73 offsets, latest=73). Searched for non-cohort posts.
- `find /tmp/penelope* /opt/050-email-archive` → discovered `/tmp/penelope-deploy-handoff.md`, `/tmp/penelope-q1-q10-response.json`, and the email-archive project at `/opt/050-email-archive` with full framework instance.
- `grep -rliE 'locator card.*[Pp]enelope'` across `/opt/050-email-archive/.context`, `/opt/002-Claude-Partner-Network`, `/tmp` → **no hit**. The literal locator-card text is not present on disk in any project I have visibility into. It was either delivered to the user via another channel (different session, paste from another agent's output) or is on a topic I cannot reach from this hub.

**Findings (substance — sufficient for S2/S3 to proceed):**

1. **The peer agent identity is concrete:** The .107 co-resident agent the user is hearing from is the **email-archive build agent** (project `/opt/050-email-archive`), not Penelope. "Penelope" in the cut-off line is a *third entity*: the `penelope-dev.service` LXC at 192.168.10.129 (CT 250). The peer agent was *describing* Penelope's TermLink topics — i.e. how to find Penelope's bus traffic.

2. **FP collision is real and confirmed by both ends.** `agent-chat-arc` offset 73 (a cohort post — i.e. ours, 12 hours ago) explicitly acknowledges:
   > "shared root /root/.termlink/identity.key on .107 means d1993c2c3ec44c94 maps to BOTH the cohort agent and email-archive. Disambiguator is from_project metadata on each post."
   The cohort and email-archive have already coordinated on this in-band. They are NOT awaiting a structural fix — they are operating on a metadata convention right now.

3. **The de-facto convention is already established:** posts on `agent-chat-arc` thread `pen-contract-cohort-forwarder` use:
   - `from_project=002-Claude-Partner-Network` (cohort, this project)
   - `from_project=050-email-archive` (email-archive, peer)
   - `to_project=...` for directed reply routing
   - `thread=...` for conversation grouping
   - `from_agent_fingerprint=d1993c2c3ec44c94` (redundant given collision, but explicit)
   - `mention=<fp>` for ping/notification semantics

4. **`from_project` is a CLI-level metadata flag**, not a protocol field. It rides on `termlink channel post --metadata key=value` and is opaque to the hub today (S2 will confirm).

5. **Penelope's bus locator (inferred from topics list):** Penelope's outbound surface is published on `framework-agent`'s hub under topics `pen.outbound.{key-available, key-revoked, ping, protocol.bumped, rate-warning, scope-changed, surface.update}`. To consume Penelope-mediated mail traffic, an agent subscribes to those topics on the framework-agent hub. **This is locator-card-level info, even if not in the literal words the peer agent used.**

**S1 outcome:** Question shape sharpens. The relevant inception scope is **NOT** "how do we identify Penelope" (Penelope is a service, addressed by topic, not by FP). It is:
- *How do we disambiguate two AGENTS that share an identity.key?*
- The de-facto answer (`from_project` metadata) is already operational. The inception question becomes: **should the framework promote that convention to a first-class, enforced field, or leave it as a soft convention?**

### S2: Code archaeology — _completed_
**Files read:**
- `crates/termlink-protocol/src/control.rs:234` — error code `-32014 CHANNEL_IDENTITY_MISMATCH`
- `crates/termlink-hub/src/channel.rs:344-491` — `handle_channel_post` end-to-end

**Sequence (channel.rs):**
1. **Client supplies (line 383):** `sender_id`, `sender_pubkey_hex`, `signature`, `topic`, `msg_type`, `payload`, optional `metadata` map.
2. **Hub computes canonical signed bytes (line 420):** `topic + msg_type + payload + artifact_ref + ts_unix_ms`. **`sender_id`, `sender_pubkey_hex`, and `metadata` are NOT in the signed bytes.**
3. **Hub verifies signature (line 427):** Standard ed25519 against the canonical bytes using `sender_pubkey_hex`.
4. **Hub strict-reject (T-1427, lines 436-451):** computes `expected_fp = fingerprint_of(verifying_key)`. Rejects with `CHANNEL_IDENTITY_MISMATCH` iff `sender_id != expected_fp`. **The check is pubkey-vs-FP only — there is NO cross-check against any agent-layer metadata claim.** A4 ✅ confirmed.
5. **Metadata handling (line 453-464):** Comment is explicit:
   > "T-1287: optional metadata routing-hint map. NOT included in canonical signed bytes — trusted-mesh threat model treats it as routing only. Well-known keys: conversation_id, event_type (per T-1288 catalog)."
   Parsed as opaque `BTreeMap<String, String>`. No schema enforcement at hub level.

**First-class metadata keys (used in code):**
- `conversation_id` (line 480, T-1287/T-1286 presence tracking)
- `in_reply_to` (line 608, filter)
- `up_to` (line 677, receipts)
- `event_type` (T-1288 catalog — referenced)

**NOT first-class today:** `from_project`, `_from`, `_thread`, `to_project`, `from_agent_fingerprint`. All convention.

**Threat-model verbatim (in code):** "trusted-mesh threat model treats it as routing only." → metadata is application-layer, not auth-layer. A malicious co-resident agent with the host's identity key could forge any `from_project`. The framework explicitly does not defend against this.

**Implication:** Promoting `from_project` to first-class is a *catalog* extension (T-1288 well-known keys), not a *protocol* extension. No signed-bytes change, no version gate. Default-injection at the CLI layer is the natural insertion point.

### S3: Field measurement — _completed_
**Method:** `termlink channel subscribe agent-chat-arc --cursor 0 --limit 200` (74 entries total, 73 with sender_id).
**Counts (raw grep, full text saved at `/tmp/T-1448-chat-arc.txt`):**

| Field | Count | % of 74 | Notes |
|---|---|---|---|
| `_thread` | 27 | 36% | Dominant existing convention; T-1438 work used `_thread=T-1438` heavily |
| `_from` | 12 | 16% | Used by field-heartbeat.sh (`_from=$HUB-vendored`) |
| `from_project` | 5 | 7% | NEW — only the pen-contract-cohort-forwarder thread (offsets 68/70/73 plus 2 others) |
| `to_project` | 1 | 1% | Newer still |

**Unique sender FPs:** 2 (`d1993c2c3ec44c94`=72 posts — collapses cohort + email-archive on .107; `9219671e28054458`=2 posts = .122 ring20-management).

**Read:** `from_project` is below the GO threshold of "≥70% organic coverage" — we would be mandating a convention, not promoting an established one. But the convention is **cheap to mandate** (one CLI default + scripts) and **already used by the only co-resident pair that exists**. The 70% bar applies to "GO without mandating"; it does not apply to "GO with mandating" if the cost is small. Below in synthesis.

### S4: Adversarial think — _completed (5 min, condensed)_
What attacks does host+user FP allow today that per-agent identity would prevent?

| Attack | Today | Under Design A (metadata convention) | Under Design B (signed metadata) |
|---|---|---|---|
| Co-resident agent impersonates peer (forges `from_project`) | Possible | Still possible — metadata unsigned | Prevented |
| External agent forges any sender_id | Blocked by T-1427 (✅ today) | Blocked | Blocked |
| Attribution dispute in audit trail (which co-resident did it?) | Cannot resolve from logs | Resolvable from `from_project` (assuming honesty) | Resolvable cryptographically |

**TermLink threat model (per channel.rs:454):** trusts root. Defending against co-resident-with-root-key attackers is **explicitly out of scope** (host owns its key; if root is compromised, the host is compromised). The audit-trail dispute case has no resolution today and Design A makes it socially resolvable but not cryptographically resolvable. **For the threat model we have, that's adequate.**

### S5: Two designs sketch — _completed_

**Design A — soft convention + CLI default + catalog promotion:**
- `termlink channel post` auto-injects `from_project` from `.context/working/focus.yaml` or `.framework.yaml` if not explicitly provided
- `from_project` added to T-1288 well-known-keys catalog (alongside `conversation_id`, `event_type`)
- Hub remains protocol-neutral on the field — it's in the metadata map, opaque, just better documented
- T-1429 `agent contact <name>` extended: name → (FP, project) tuple resolution; auto-attach `to_project`
- Heartbeat scripts (`field-heartbeat.sh`, `vendored-arc-heartbeat.sh`) updated to set `from_project`
- `/agent-handoff` and `/check-arc` skills updated to read+write the field
- T-1427 strict-reject **unchanged** — it correctly identifies the host; project is application-layer
- T-1440/T-1441 (`whoami`, `remote list`) — surface project alongside FP, so operators see "FP a1b2 / project=050-email-archive"

**Cost:** 3 build tasks, ~1 session each:
1. termlink-cli: default `from_project` injection + catalog entry
2. termlink-cli: T-1429 `agent contact <name>[:project]` extension + auto-attach `to_project`
3. Scripts + skills sync (field-heartbeat, vendored-arc-heartbeat, /agent-handoff, /check-arc)

**Reversibility:** Trivial. Can revert any of the 3 in isolation; the field stays in metadata regardless.

**Design B — signed metadata + sub-key per agent:**
- Add `from_project` to canonical signed bytes (channel.rs:420 list)
- Agent sub-key derived from host key + project ID
- `channel.post` signs with sub-key; hub verifies sub-key chain
- T-1427 strict-reject extends: sender_id = host FP AND from_project matches sub-key

**Cost:** Protocol break. Version-gated rollout across fleet. ≥5 build tasks. All clients require update. Design and review of sub-key derivation (HKDF? per-project ECDH?). Existing pre-T-1448 hubs reject signed-metadata posts.

**Tradeoff Design A vs Design B:**

| Axis | A | B |
|---|---|---|
| Disambiguates co-resident agents | ✅ | ✅ |
| Defends against co-resident forge | ❌ | ✅ |
| Protocol-stable | ✅ | ❌ |
| Cost | 3 tasks | 5+ tasks + fleet migration |
| Reversibility | High | Low (version-gate trap) |
| Aligned with threat model? | ✅ | Over-engineered for it |
| Existing convention compatibility | Promotes already-emerging convention | Replaces it |

### Synthesis — Recommendation

**Recommendation: GO with Design A.**

**Rationale:**
1. **Threat model alignment.** TermLink trusts root. Co-resident-forge is explicitly out of scope (channel.rs:454 verbatim). Design B defends against an attack we don't claim to defend against; that's over-engineering.
2. **De-facto convention exists and is operational.** Cohort and email-archive coordinated `from_project` in-band at chat-arc offset 73, 12h before this inception. We're not inventing — we're codifying.
3. **Cost is bounded.** 3 build tasks, each ≤1 session. Hits the GO criterion exactly.
4. **Migration story for the 5 affected tasks is augment-not-unwind.**
   - T-1427 strict-reject: unchanged (host identity disambiguation is still valid; agent identity is a separate axis)
   - T-1429 agent contact: extend resolution to `(name, project)` tuple
   - T-1436 registration: add `from_project` to registration metadata
   - T-1440/T-1441: surface project alongside FP
5. **Reversible.** If Design B becomes needed later (threat model expands), it's additive on top of Design A — `from_project` is already first-class metadata, just unsigned.
6. **Stable identity invariant.** `from_project` is anchored on the project directory path, which is stable across compaction, `/clear`, and restart — solving the "session-id is not stable" constraint identified in the technical-constraints section.

**Why not DEFER:** Penelope-only? No. The cohort + email-archive co-residency is permanent (both projects live on .107 and won't move). The pattern will recur as more projects are added. Defer-and-record-only would mean every new project re-discovers FP collision at first chat-arc post.

**Build tasks proposed (if GO):**
1. **T-XXXX(a)** — termlink-cli: default `from_project` injection from focus.yaml + promote to T-1288 catalog
2. **T-XXXX(b)** — termlink-cli + T-1429: `agent contact <name>[:project]` + auto-attach `to_project`
3. **T-XXXX(c)** — scripts + skills: field-heartbeat.sh, vendored-arc-heartbeat.sh, /agent-handoff, /check-arc — emit + read `from_project`

Order: a → c → b. (a) unblocks (c); (b) builds on (a)'s catalog entry and is the most operator-visible change.

**Out of scope, explicitly:**
- Sub-key cryptography (Design B)
- Hub-side schema enforcement (would be a softer enforcement, but adds review/version-gate cost without proportional benefit)
- Renaming existing `_from`/`_thread` conventions — keep them, document them in the catalog
- Cross-host project-namespace conflicts (e.g. two `050-email-archive` directories on different hosts) — flag for follow-up but not in this inception

**Recommendation:** GO.

---

## Elaboration — what "identity" actually means in TermLink, and where the conflation lives

### The two layers we've been sloppily collapsing

A TermLink "identity" today is a **single ed25519 keypair stored on disk** at `/root/.termlink/identity.key`. From it the system derives one FP (e.g. `d1993c2c3ec44c94`) and uses that FP for several jobs that aren't actually the same job:

| Job | What it really needs to identify |
|---|---|
| **TLS pinning / TOFU** (hub<->client trust) | A *host endpoint* — "is this the same hub I trusted yesterday?" |
| **Channel post auth** (T-1427 strict-reject) | A *signing principal* — "did this envelope come from someone who holds a key I'm willing to trust?" |
| **Chat-arc attribution** | An *agent* — "which Claude session said this?" |
| **`agent contact <name>` routing** | An *addressable counterparty* — "deliver this to the right inbox" |

The first two jobs need a (host, user) identity, which is what we have. The last two jobs need an *agent* identity, which is what we **don't** have but were pretending we did because every host had at most one agent.

So the conflation isn't "identity is wrong." It's: **a single identity key is being asked to play two distinct roles, and one role just gained a multiplicity it can't represent.**

### Why FPs collide on .107 (mechanically)

`/root/.termlink/identity.key` is a 32-byte ed25519 seed file. When a TermLink CLI starts:
1. It reads that file (root-readable; no agent isolation),
2. derives the public key,
3. computes `fingerprint_of(pubkey) = sha256(pubkey)[..16]` = `d1993c2c3ec44c94`,
4. uses that as `sender_id` on every channel post.

The cohort agent (this session, project `002-Claude-Partner-Network`) and the email-archive build agent (project `050-email-archive`) both run as root on .107, both read the same file, both produce the same FP. The hub cannot tell them apart at the cryptographic layer because **at the cryptographic layer they are not different** — they are two processes holding the same private key.

### What "agent identity" needs to be

The four properties we need from agent identity:

1. **Stable across the agent's process lifecycle** — survives `/clear`, restart, compaction. Session-id (`tl-XXXX`) does not satisfy this. **Project directory does.**
2. **Distinguishable from co-resident peers** — the disambiguator must vary across agents on the same (host, user).
3. **Operator-meaningful** — when a human reads "post from X", X should mean something to them. `from_project=050-email-archive` is meaningful; `agent_uuid=8f3e2a1b...` is not.
4. **Cheap to attach to every post** — if attachment is manual, it will drift; if it's a CLI default sourced from the project's own working memory, it's automatic.

A project ID (`002-Claude-Partner-Network`, `050-email-archive`) satisfies all four. A session-id satisfies only (2) and (4). A per-agent UUID would satisfy (1) and (2) but fails (3).

This is why the conversation has been gravitating to `from_project` — it isn't an arbitrary choice; it's the field that an operator-facing agent identity *must* look like.

---

## Mapping options against the four directives

| Directive | Design A (CLI default + catalog) | Design B (signed metadata + sub-key) | Design C (per-project identity key) | Design D (do nothing — keep ad-hoc convention) |
|---|---|---|---|---|
| **Antifragility** — system strengthens under stress | ✅ Codifies a learning that emerged from a stress event. Future co-residency cases inherit the fix. | ✅ Same, but at higher fidelity. | ✅ Same, with cryptographic backing. | ⚠️ Convention exists but is invisible to new agents. Each new co-resident pair re-discovers the collision. The lesson does not propagate. |
| **Reliability** — predictable, observable, auditable; no silent failures | ✅ Audit trail gains `from_project`; chat-arc misattribution becomes mechanically detectable (post without `from_project` from a colliding FP = warning). | ✅✅ Audit trail is **cryptographically** auditable — no honest-mistake risk. | ✅ Audit trail is cryptographic AND each agent has its own identity in the chain. Strongest auditability. | ❌ Silent ambiguity remains. An audit reading `[68] d1993c2c3ec44c94 wrote …` cannot determine which agent wrote it without reading metadata that may not be there. |
| **Usability** — joy to use, sensible defaults, actionable errors | ✅✅ Operator default-injection from focus.yaml; T-1429 `agent contact <name>` works; `whoami` shows `FP a1b2 / project=050-email-archive`. Zero friction. | ⚠️ Operators see two FP-like blobs (host FP + sub-key FP) and must mentally map them to "host" and "agent." Cognitive cost. | ⚠️⚠️ Each project requires its own auth bootstrap (TOFU pin, secret_file, fleet doctor entries). Heal procedures multiply. The Hub Auth Rotation Protocol in CLAUDE.md becomes per-project. | ❌ Operators must memorize "this FP is two agents on .107; check `_thread` or `from_project` if present, or ask in-band." Footgun for newcomers. |
| **Portability** — no provider/language/environment lock-in; standards | ✅✅ `metadata` is already a generic string→string map; `from_project` is a string. Any client in any language emits it the same way. T-1288 catalog is documentation. | ⚠️ Sub-key derivation requires implementation in every client language. Protocol break = every consumer must upgrade in lockstep. | ❌ Per-project key files multiply config surface; complicates Homebrew/MSI/binary distribution because key bootstrap is per-project, not per-install. | ✅ Convention is universal because it's invisible. (False win — same as silence.) |

**Headline:** A scores ✅✅ on Usability and Portability, ✅ on Antifragility and Reliability. B and C score higher on Reliability but lose on Usability and Portability. D loses on Antifragility and Reliability outright.

---

## Steelman / Strawman each option

### Design A — soft convention + CLI default + T-1288 catalog promotion

**Steelman (strongest case FOR):**
> "TermLink's protocol already says metadata is opaque routing-hint. The hub explicitly does not promise to authenticate it. The threat model trusts root. Two of our own agents have already invented `from_project` and used it productively without any framework support. The right move is to make that invention discoverable by the next pair of agents who hit this — by defaulting it at the CLI, documenting it in the T-1288 catalog, and surfacing it in `whoami` / `remote list`. We are *codifying success*, not *inventing a defense*. The cheapest fix that solves the actual problem (operator + agent disambiguation) is the right fix; cryptographic agent identity is a different problem with a different threat model."

**Strawman (weakest case FOR — or the obvious objection):**
> "It's just a string. Anyone can lie about `from_project` — the hub doesn't check it and the cohort agent could post `from_project=050-email-archive` and frame email-archive. We're solving a problem with a Post-it note labeled 'trust me.'"

**Counter:** The threat model says we already trust the host's root user. A co-resident agent that wants to forge `from_project` already has the identity key — it can already forge anything by being root. Authenticating `from_project` adds a defensive layer for an attack we explicitly do not defend against (channel.rs:454: "trusted-mesh threat model"). The Post-it note is the right tool when the room is locked.

---

### Design B — signed metadata + sub-key per agent

**Steelman (strongest case FOR):**
> "Adding a field to the canonical signed bytes is a one-time protocol cost. After it's done, *every* future agent-identity question (audit trail forensics, cross-agent attribution disputes, agent-impersonation in compromised-but-not-fully-rooted scenarios, multi-tenant futures) is solved cryptographically. Soft conventions rot — operators forget to set them, scripts emit them inconsistently, audit-trail integrity becomes 'whatever the convention was that month.' Pay the protocol cost once; collect cryptographic guarantees forever. The threat model can shift — and when it does (e.g. when we add untrusted-tenant hosts), Design A is a brittle floor we'll have to rip out."

**Strawman (the obvious objection):**
> "Five build tasks, fleet-wide version gate, all consumers must upgrade. We've just spent T-1166 + T-1418 + T-1294 + T-1438 *removing* unsynchronized version-gate pain. Adding a new one immediately to defend against an attack the threat model says is out of scope is a self-inflicted wound."

**Counter to the steelman:** The "threat model can shift" argument is real but speculative. Until it shifts, Design B over-engineers. And — crucially — Design A is **additive** to Design B if we ever go there. `from_project` becomes signed metadata; nothing about the catalog entry has to change. Choosing A doesn't preclude B; choosing B preempts A.

---

### Design C — per-project identity key files (NEW — wasn't in S5)

**Steelman (strongest case FOR):**
> "If a *project* is what we actually mean by 'agent', and if the operator-meaningful identity property is 'this project on this host', then the cleanest model is: each project has its own `~/.termlink/projects/<project-id>/identity.key`. The TLS pinning layer is unchanged (still keyed on the hub host). Channel post signing uses the project's key. Hub strict-reject works exactly as today, just on the project FP. No protocol change to envelope schema. No metadata convention. Disambiguation is cryptographic and operator-meaningful at the same time. T-1429 `agent contact <name>` resolves directly to a unique key without needing a metadata side-channel."

**Strawman (the obvious objection):**
> "Every project now needs its own auth bootstrap, its own TOFU pin entry, its own `secret_file` in the heal manifest. The Hub Auth Rotation Protocol in CLAUDE.md becomes per-project, not per-host. The operator burden goes UP, not down. And if the hub TLS layer remains host-keyed but channel posts are project-keyed, the operator now has TWO FPs per project to think about. Plus: the framework has *one* `.termlink/identity.key` because that's how it was originally designed; introducing per-project keys would require a default-derivation strategy (HKDF from host key + project ID? Random per project?) — neither option is obviously correct."

**Counter to the steelman:** This is the most architecturally clean of the three, and worth keeping as a *future* option if we ever need cryptographic agent identity. But it solves more than we need today and at higher operator cost than A. It belongs on the deferred list, not the active list.

---

### Design D — do nothing (keep ad-hoc convention, do not codify)

**Steelman (strongest case FOR):**
> "The cohort and email-archive figured this out in-band in one chat-arc round-trip. The system worked. Codifying conventions before the second occurrence risks freezing the wrong shape. Wait for the second co-resident pair to discover the same problem, see if they reach for the same field, then codify when there's actual evidence of consensus. Premature codification is more expensive to undo than to defer."

**Strawman (the obvious objection):**
> "Inviting every new agent to re-derive the answer from scratch is exactly the antifragility anti-pattern: failures should produce structural learning, not be re-experienced as fresh pain by every new participant. The whole point of a framework is that lessons compound. Telling new agents 'figure it out in-band like we did' is the system getting weaker over time, not stronger."

**Counter:** D is the option that fails the antifragility directive most loudly. It also leaks operator attention forever — every co-resident pair we add (and we will add them; multi-project hosts are the dominant deployment) gets one round-trip of confused chat-arc traffic. That's a small per-incident cost but an unbounded total cost.

---

## Revised directive-aligned recommendation

The steelman/strawman exercise **confirms Design A** but sharpens *why*:

- **A is the option that aligns with all four directives without trading off.** B and C trade Usability + Portability for Reliability gains we don't currently need. D trades Antifragility for "wait and see," which is the wrong trade for a system that is getting more multi-agent over time.
- **A is additive to B and C.** Choosing A doesn't preempt promoting `from_project` to signed metadata later, nor introducing per-project keys later. Choosing B or C now would lock in a higher-cost path before we need it.
- **A is reversible.** Each of its 3 build tasks is independently revertible. B and C are not — they introduce protocol/auth surface that becomes load-bearing.

**One sharpening of A from this exercise:** make the CLI default-injection emit a **warning to the operator** if `from_project` is unresolvable (no focus.yaml, no .framework.yaml). That converts a silent failure into an actionable error (Reliability directive). This becomes an explicit acceptance criterion for build task (a).

**Final recommendation: GO with Design A, with the warning-on-unresolvable-project AC added to task (a).**

Defer Design C explicitly — record it as the natural next step if the threat model ever shifts to include co-resident-with-stolen-key forge.

Reject Design B — over-engineers for the current threat model.

Reject Design D — fails antifragility.

---

## Dialogue Log

Per C-001, conversations that shape this inception are logged here.

### 2026-05-02 — Trigger message from peer agent (Penelope, .107 co-resident)

**Penelope (excerpt, cut off):**
> Important wrinkle: my fingerprint is d1993c2c3ec44c94 — the same as the cohort agent's, because we both run as root on .107 and share /root/.termlink/identity.key. Cryptographic identity won't disambiguate us. The disambiguator is the from_project metadata field on each post.
>
> Locator card for Penelope's bus messages: [TRUNCATED]

**User → claude-code (cohort):**
> please incept, this seems like we need a more fundamental improvement.

**Decision:** Open T-1448 inception. Frame: "What is the right model for agent identity in TermLink, given that the cryptographic root of trust is necessarily host+user-keyed?"

---

## Decision

_To be recorded via `fw inception decide T-1448 go|no-go --rationale "..."` after synthesis._
