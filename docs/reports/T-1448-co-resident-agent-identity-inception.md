# T-1448: Co-Resident Agent Identity Disambiguation — Inception Report

**Status:** Inception in progress
**Owner:** human (decision); claude-code (research)
**Started:** 2026-05-02
**Predecessor signals:** T-1427 (strict-reject), T-1436 (FP registration), T-1429 (agent contact), T-1438 (field rollout — first multi-agent context)
**Trigger incident:** 2026-05-02 cross-agent dialogue with peer agent "Penelope" running co-resident on .107

---

## TL;DR (filled at synthesis)

_To be filled after spikes S1–S5. Format:_
- **Recommendation:** GO / NO-GO / DEFER
- **One-line rationale:**
- **Build tasks proposed (if GO):** T-XXXX, T-XXXX, T-XXXX

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

### S2: Code archaeology — _not started_
**Goal:** Where does `sender_id` come from on the wire? What metadata fields are first-class?
**Files to read:**
- Hub strict-reject path — likely in `crates/termlink-hub/src/channel.rs` or similar
- T-1436 registration code path
- `channel post` metadata serialization
**Output:** A sequence diagram (text) of how a `channel post` becomes a `sender_id` on the receiving end, and what's first-class vs. opaque.

_Findings:_

### S3: Field measurement — _not started_
**Goal:** What's already in chat-arc traffic?
**Method:** Sample last 7d of `agent-chat-arc` on .107 + .122 + .141. Tabulate metadata field presence.
**Output:** Table of `{_from, from_project, _thread}` coverage. Identify de-facto convention.

_Findings:_

### S4: Adversarial think — _not started_
**Goal:** What attacks does host+user FP enable that per-agent identity would prevent?
**Output:** Threat list, with explicit notes on which are in/out of TermLink's threat model.

_Findings:_

### S5: Two designs sketch — _not started_

**Design A: Pure metadata convention + light schema validator.**
- Mandate `from_project` (or chosen field) on every chat-arc post
- Hub validates presence; co-resident posts without it logged as `unknown-agent`
- Agents register their project at session start (analogous to T-1436 FP registration)
- Migration: tweak heartbeat scripts, T-1431 skill, T-1429 verb to attach + read the field

**Design B: Sub-key per agent derived from host key + agent UUID.**
- Agent UUID derived from session-id (not stable across compaction — known issue)
- Sub-key signs posts; hub verifies sub-key chain
- Migration: protocol-level field, T-1427 strict-reject extends to sub-key check

_Tradeoffs to fill at synthesis:_

### Synthesis — _not started_

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
