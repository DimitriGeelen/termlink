# T-3319 — Learn from message traffic

Status: inception, exploration only (no production code). Started 2026-10-02.
Related: T-3309 (per-topic activity tracking), T-3310 (retention rulings), T-3321
(compaction, DEFERRED behind this one), T-3304 (hub storage model consult), T-2548
(off-charter analytics tools pending removal).

## Question

The hub carries a large flow of messages and events. Can we **derive knowledge** from
that flow (recurring failure classes, repeated filings, stalled threads, who reads
what, reply latency, noise vs signal) rather than only retaining or pruning it — and
if so, **who does the deriving, where does the result live, and what must be captured
before the new retention rules trim it?** AEF (the vendored governance framework) is to
be involved, there is an external review below, and the operator decides after a
one-question-at-a-time discussion.

## Raw material inventory (measured 2026-10-02, read-only)

Method: `termlink channel list --json`; a byte copy of `/var/lib/termlink/bus/meta.db`
opened `?mode=ro` (the live file was locked by the hub's journal, so a snapshot copy
was queried — no write to the live DB); `termlink channel subscribe <topic> --json
--limit N` on `framework:pickup` (all 299), `channel:learnings` (all 407),
`agent-chat-arc` and one `inbox:*` topic. Payloads characterised by `msg_type`,
`metadata.from_project`, size and leading ids; no payload is quoted beyond short,
non-secret excerpts.

### Totals

120 topics on the local hub; **4,367 records, 12.4 MB of payload** (meta.db `records`,
`sum(length)`); 49 MB of segment files on disk under `/var/lib/termlink/bus/topics`
(the gap is reclaimed-but-not-compacted space — T-3322's territory). Oldest record
2026-08-16.

### By topic family

| family | topics | records | payload bytes | oldest | retention today |
|---|---|---|---|---|---|
| `framework:pickup` | 1 | 299 | 3.82 MB | 08-16 | messages |
| `agent-*` (chat-arc, presence, listeners) | 10 | 2,238 | 2.23 MB | 08-17 | messages |
| other / ad-hoc (`xfer-*`, `aef-*`, demos, probes) | 26 | 211 | 1.89 MB | 08-23 | mostly forever |
| `inbox:*` project mail | 32 | 486 | 1.59 MB | 09-22 | forever (→ newest 1000, T-3310) |
| `dm:*` | 24 | 509 | 1.45 MB | 08-23 | mostly messages |
| `channel:learnings` | 1 | 407 | 0.81 MB | 08-16 | forever |
| `sidecar:*` | 18 | 53 | 0.34 MB | 09-21 | forever |
| `health:*`, `cockpit`, `seq`, `broadcast`, `proof`, `sprind` | 6 | 164 | 0.26 MB | — | mixed |

### What the payloads are

- **`framework:pickup` — the richest material.** 299 filings, median 3.3 KB, max 47 KB.
  `msg_type`: note 131, bug-report + pickup-bug-report 72, reply 35,
  proposal + pickup-feature-proposal 18, finding 10, pickup-learning 9. Senders by
  `metadata.from_project`: 010-termlink 133, 999-AEF 37, 055-cockpit 35,
  832-Workflow-designer 32, `root` 28, 050-email-archive 11, unattributed 10, five more
  projects. Content is structured prose: corrections, bug reports with file:line,
  acks, triage replies. One thread (AEF T-3639 triage of 055's vendored fixes) is ~69
  messages. Two `msg_type` spellings exist for the same thing (`bug-report` vs
  `pickup-bug-report`) — a first sign that a classifier must normalise.
- **`channel:learnings`** — JSON envelopes `{origin_project, origin_hub_fingerprint,
  learning_id, learning, task, source, date}`. **~108 of 407 records are test-fixture
  pollution**: `PL-001` posted 146 times, with placeholder text ("Test learning",
  "First learning", task `T-9999`/`T-123`) — test runs publishing to the live hub.
  150 records carry no `from_project` metadata (the origin is inside the JSON).
- **`agent-chat-arc`** — 1,000 records (at its cap), overwhelmingly identical hourly
  "vendored-arc heartbeat … Binary: termlink 0.11.1716" lines. Signal content: binary
  version drift per host over time, nothing else.
- **`agent-presence`** — 1,222 heartbeats; latest-per-agent is useful, history is not.
- **`inbox:*`** — e.g. the 010-termlink inbox: `sidecar.consult` envelopes with rich
  metadata (`conversation_id`, `from_agent`, `from_circuit`, `from_project`,
  `client_msg_id`); several are repeated E2E test pings.
- **Sender identity is per HOST, not per agent.** Nearly every record carries the same
  `sender_id` fingerprint; real attribution lives only in free-form metadata
  (`from_project`, `_from`, `from_agent`), filled inconsistently. Any "who said what"
  analysis inherits that weakness.

### Activity tables (T-3309)

`topic_activity` tracks 67 topics **since 2026-10-02 13:28 UTC only** (hours of data):
551 writes vs 12,685 fetches. Example: `inbox:…/010-termlink` 6 writes / 1,564
fetches — polling dominates read traffic. `topic_readers` has 6 rows (4 distinct
readers); `cursors` 2 rows; 119 claim rows (largely expired debris, cf. T-2709). So the
"who reads what" signal exists but is days, not weeks, deep.

## What AEF already learns from, and how

| Mechanism | Where | Input → output |
|---|---|---|
| Learnings / patterns / decisions | `.agentic-framework/agents/context/context.sh` (`add-learning`, `add-pattern`, `add-decision`) | agent-recorded → `.context/project/{learnings,patterns,decisions}.yaml` (407 learnings here) |
| Episodic memory | `agents/context/context.sh generate-episodic`, run on task completion | completed task → `.context/episodic/T-*.yaml` (2,705 here) |
| Audit discoveries | `agents/audit/audit.sh` sections `discovery` / `discovery-trends` (cron, every 30 min / hourly) | task corpus → `.context/audits/discoveries/LATEST.yaml` (D1..D11, e.g. "68 human-review tasks waiting >30d") |
| GO-scope-not-propagated scan | `audit.sh` ~L1856 (T-2096/T-3099) | completed GO inceptions with no follow-up → `.context/audits/go-scope-unpropagated/` |
| Harvest / graduation | `lib/harvest.sh` | learning seen in 1 project = local, 2+ = candidate, 3+ = practice |
| Consolidation | `agents/context/consolidate.py` | detects duplicate / stale learnings |
| Learning bus bridge (out) | `lib/publish-learning-to-bus.sh` | each new learning → `channel:learnings` |
| Learning bus bridge (in) | `lib/subscribe-learnings-from-bus.sh` | `channel:learnings` → `.context/project/received-learnings.yaml` (de-dup) — **here it holds 1 entry** against 407 on the topic |
| Pickup bridge | `lib/pickup.sh` + `lib/pickup-channel-bridge.sh` | inbound envelope → `framework:pickup` mirror (SHA-256 dedup) |
| Message router (upstream, T-3044/T-3046) | `lib/message_router.py` | static `msg_type` → disposition table over a **47,879-message archive** recovered from hubs; written after "a bug report sat unread for three months"; identity = content hash because `(topic, offset)` collided 3,593 times across hubs |

So AEF already has: a graduation rule (2+/3+ projects), a dedup engine, an
audit-to-task pipeline, and — upstream — an archive and a disposition router. What it
does not have is anything that reads the *flow* (threads, latency, readership,
repetition across filings) as opposed to individual artefacts.

## Candidate knowledge to derive

1. **Recurring failure classes across projects** — cluster `bug-report`/`finding`
   filings in `framework:pickup`; feed the existing harvest rule (seen in 3+ projects
   → practice).
2. **Repeated filings of the same bug** — near-duplicate detection (same defect re-filed,
   or independently re-fixed — the P-043 / T-2304 story in CLAUDE.md is the cost).
3. **Unanswered / stalled threads** — a filing with no `reply`/ack within N days; a
   `conversation_id` whose last message is a question.
4. **Reply latency per project pair** — how long a filing waits for its first response.
5. **Readership** — topics written but never read (G-063 "write-only sink"), readers
   that poll 1,500× for 6 writes (cost), topics nobody reads that are kept forever.
6. **Noise vs signal** — heartbeat-shaped topics (`agent-chat-arc`) that carry one bit
   of information (version drift) at 1,000 records; test pollution on live topics.
7. **Learnings that should graduate** — `channel:learnings` entries arriving from 2+
   projects that the receiving side never ingested (received-learnings has 1 entry).

## Constraints

- **Retention now bounds the data (T-3310).** Mail topics keep the newest 1,000; other
  new topics 14 days; ceilings on post. `framework:pickup` (299, messages-retention) and
  `channel:learnings` (forever) are currently safe; `agent-chat-arc` is already at its
  1,000 cap and rotating. The trim risk is mainly to *new* topics (14 days) and to the
  T-3309 activity history, which has no retention policy of its own yet. T-3321
  (compaction) stays deferred until this inception decides what raw material matters.
- **Charter non-goal #2** — "not a durable database / system of record". A
  capture-everything archive on the hub would violate it; so would re-growing the
  analytics tool surface that T-2548 is removing (the charter-drift canary,
  T-2483/T-2680, would fire on new leaderboard-shaped tools).
- **Privacy / secrets.** Payloads include file paths, host names, hub fingerprints,
  and occasionally commands; T-2806 already found a sha256 pin that pattern-matched as
  a secret. Any export or LLM summarisation must run a secret scan first and must not
  leave the operator's machines without a decision.
- **Attribution is weak.** Host-level `sender_id`; free-form `from_project`. Derived
  knowledge about "who" is only as good as metadata discipline.
- **Test pollution** pollutes the corpus (≥108 fake learnings). Producers need fixing
  regardless of what is learned.
- **Per-hub state (G-060).** Each hub sees only its own traffic; a fleet-wide view needs
  a collector (AEF's upstream archive is exactly that, and is where the 3,593 offset
  collisions came from).

## Open questions for the operator

Each answerable on its own, one at a time. Q0 was added after the consultation and is
the recommended first question (see Synthesis):

0. **Goal:** make sure filings get *answered* (routing / accountability), or *learn
   patterns* from them (mining)?
1. **Where does derived knowledge live?** (a) AEF project memory (learnings, patterns,
   concerns, audits, tasks), (b) small derived views on the hub, (c) a separate store.
2. **Who runs the miner?** AEF (it already owns learnings, harvest, the upstream
   message router) or TermLink?
3. **Capture before trim:** should anything be snapshotted before the T-3310 rules start
   trimming (e.g. a one-off export of `framework:pickup` + `channel:learnings`), or is
   loss of old raw material acceptable?
4. **First question to answer from the data** — which one item of the candidate list
   above is worth a first, measured pass?
5. **Producers vs miners:** fix the noisy producers first (test pollution, heartbeat
   chat, missing `from_project`) before learning anything, or in parallel?
6. **LLM involvement:** may thread summaries / clustering be done by an LLM, and if so,
   local-only (Ollama) or remote models?
7. **Activity-table retention:** should `topic_activity` / `topic_readers` history be
   kept long enough to learn from (it is hours deep today)?

## Dialogue Log

<!-- C-001: record questions posed by the operator, answers, course corrections, outcomes. -->

## Consultation

Brief: `T-3319-consult/brief.md` (identical text to every reviewer). Method as T-3304:
each reviewer ran in isolation, answers written to a scratch directory and copied in
only after all had finished, so no reviewer could read another's answer.

| Reviewer | Harness | Result | File |
|---|---|---|---|
| OpenAI Codex (`gpt-6-astra`) | `codex exec -s read-only -C /opt/termlink` | OK, ~1 min. Ran no shell commands; answered from the brief alone | `T-3319-consult/codex.md` |
| Z.AI GLM-5.3 | `opencode run -m zai-coding-plan/glm-5.3` from an empty scratch folder (`--dir` = that folder, which stayed empty) | OK, ~1 min | `T-3319-consult/glm.md` |
| qwen3:14b (local) | `ollama run qwen3:14b --hidethinking < brief.md` | OK, ~2 min. Terminal redraw codes stripped, nothing else changed | `T-3319-consult/qwen.md` |
| gemma4:latest (local) | `ollama run gemma4:latest --hidethinking < brief.md` | OK, ~1 min. Same cleaning | `T-3319-consult/gemma4.md` |

All four ran at the same time, wrote to a scratch directory outside the repo, and were
copied into `T-3319-consult/` only after all four had finished. No reviewer failed.

**Two corrections to the brief, which reviewers spotted or tripped on:**
- Codex: *"'~108 records' and 'posted 146 times' may use different scopes."* That is
  correct. `PL-001` appears 146 times. By placeholder text, about 108 records across
  several placeholder ids are test fixtures ("Test learning" 19, "First learning" 19,
  `T-9999` pattern 32, `T-123` 19, "Source learning" 19). The two numbers measure
  different things. The fixture count still needs a proper reconciliation before
  anything quotes it.
- GLM says *"The 14-day rule threatens `framework:pickup` — most of its 299 records are
  already older than that."* That misreads the brief. T-3310 applies 14 days to **new**
  topics. `framework:pickup` already exists and has `messages` retention, so it is not
  being trimmed today. GLM's practical advice (export pickup once) still holds: it is
  the irreplaceable corpus, and nothing guarantees its policy will stay as it is.

## Synthesis

### Where they converge

| Point | Codex | GLM-5.3 | qwen3 | gemma4 | Convergence |
|---|---|---|---|---|---|
| Worth doing? | "Yes—but as a bounded AEF experiment, not a new hub analytics subsystem" | "Yes, but far narrower than the framing suggests" | "Yes, but with strict boundaries" | "Proceed with extreme caution" | **4/4 qualified yes: narrow and bounded** |
| Where knowledge lives | "The hub owns transport facts; AEF owns interpreted knowledge" | "Not on the hub … Durable knowledge goes into AEF memory" | "AEF project memory, not the hub or a separate store" | "in AEF Project Memory … actionable, formalized findings" | **4/4: AEF memory; hub keeps only regenerable live state; no separate store** |
| Biggest unasked mistake | repetition taken as "independent corroboration" (provenance) | "The per-host sender fingerprint … every 'who' … is fiction" | "sender fingerprint is per host, not per agent" | "relying on the sender fingerprint (per host)" | **4/4 on attribution/provenance. 3 name the host fingerprint; Codex names the deeper version: repeats inflate the 2+/3+ harvest rule** |
| Start corpus | "Start with … `framework:pickup`" | "Export pickup in full, once, now" | preserve `framework:pickup` threads + non-fixture learnings | "Signal (Priority 1): bug-report, feature-proposal …" | **4/4: pickup filings are the signal; heartbeats and fixtures are noise to let go** |
| Capture rule | one-shot, fixed 30-day review deadline, delete the rest: "an evidence lifecycle rather than an archive" | "one-way, one-shot … export only what exists nowhere else" | scoped snapshot, "avoid capturing raw logs" | "Do not build an archive" | **4/4: no rolling archive. 2/4 (Codex, GLM) give a concrete one-shot export of pickup** |
| Root problem | "a delivery and accountability problem disguised as a knowledge problem" | "an attention/routing failure, not a knowledge-extraction failure" | — | — | **2/4 strong models: the motivating incident (unread for 3 months) is routing, not mining** |
| Fix producers | producer fixes "plus a one-shot batch analysis" | "D now, A thin, C for pickup only" | lists it, no priority | "does not solve the underlying problem" | **3/4 put producer fixes first or in parallel** |

### Where they disagree

1. **Hub-side views: in or out?** GLM wants a thin hub layer: "Extend the activity
   table: oldest-unread, unanswered-thread age … charter-legitimate". Codex lists hub
   views but warns of "gradual return of off-charter social analytics" and does not
   choose them. qwen rates hub metrics "High" cost and says they "could bloat the hub,
   violating its 'not a system of record' charter". gemma4 calls them low to medium
   cost. **This is the real fork:** whether "unanswered age" is coordination state (it
   belongs on the hub, GLM) or analytics (it belongs in AEF only, qwen and implicitly
   Codex).
2. **What to measure first.**
   - Codex: *"Incremental actionable yield"*, meaning reviewer-accepted, previously
     unknown findings that lead to an assigned action, plus reviewer minutes per
     finding. It warns that "1,564 fetches establishes polling, not reading".
   - GLM: hand-code all 299 pickup records into duplicates, unanswered >30d and new
     cross-project classes, with a kill threshold: "If fewer than ~2 repeated
     cross-project classes exist, there is nothing to mine".
   - qwen: thread resolution rate and learnings redundancy.
   - gemma4: (writes+replies)/fetches on inbox/dm. Codex's warning directly undercuts
     this measure.

   The two strong models agree on a human-adjudicated yield measure over pickup with a
   threshold set **before** reviewing. They differ on whether the threshold is
   "accepted new findings per reviewer-minute" (Codex) or "≥2 recurring cross-project
   classes" (GLM).

A lesser split: gemma4 alone proposes capture **at trim time** ("a small 'summary
record' … when a retention policy event is triggered"). That puts a capture hook into
the hub's sweep path. No other reviewer goes that way.

### Strongest arguments against (worth putting to the operator verbatim)

- GLM: *"299 substantive records can be read by a human in an afternoon."*
- Codex: *"If the experiment mostly rediscovers existing findings, fix producers and
  handling workflows instead."*

### My recommendation

I agree with the convergence: knowledge lives in AEF, there is no hub archive, a
one-shot pickup export goes under a named owner with an expiry date, and attribution
has to be fixed before any "who" question is trusted. I would add one point that none
of the reviewers had. AEF's inbound learning consumer has ingested **1** entry against
407 on `channel:learnings`. So the existing wire is broken before any new miner is
built, which supports Codex and GLM's "routing, not mining" reading.

**Recommended first question for the operator:**

> **Is the goal to make sure filings get *answered* (routing and accountability), or
> to *learn patterns* from them (mining)?**

Both strong reviewers diagnose the motivating failure as a routing problem. The answer
decides everything that follows:
- If **routing**: the work is an AEF "unanswered pickup" audit plus producer fixes, and
  no miner.
- If **mining**: the work is a one-shot pickup export plus a hand-coded yield pass with
  a pre-set kill threshold.

The other open questions (capture before trim, hub views, LLM use, attribution) follow
from that answer, and should be asked one at a time after it.
