---
id: T-3304
name: "Hub storage model: should the hub be a queryable source of truth? (retention,
  pruning, query, topic discovery)"
description: >
  Inception: Hub storage model: should the hub be a queryable source of truth? (retention,
  pruning, query, topic discovery)

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-10-01T18:13:59Z
last_update: 2026-10-01T18:22:01Z
date_finished:
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
bvp_scores_proposed:
  - ts: '2026-10-01T18:14:36Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 2
      D3: 2
      D4: 2
      F-RECALL: 2
      F-ORCH: 2
    rationale: D1=2 (no-signal); D2=2 (no-signal); D3=2 (no-signal); D4=2 
      (no-signal); F-RECALL=2 (no-signal); F-ORCH=2 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-3304: Hub storage model: should the hub be a queryable source of truth? (retention, pruning, query, topic discovery)

## Problem Statement

While walking SQ-11 (T-2573, subscribe deadline drops collected messages) the
operator raised the principle underneath it: does the hub store messages as a
**queryable source of truth**, or is it a transport where agents keep their own
history? That determines retention/pruning defaults, the query model (cursor +
filter today; no time-range, cross-topic or metadata index), and how agents
discover and select topics. Charter non-goal #2 currently says "Not a durable
database or system of record" — this inception tests that stance. The operator
asked to consult three non-Anthropic agents first ("there's real value in there").
Research artifact: `docs/reports/T-3304-hub-storage-model.md`.

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

<!-- T-2190 (T-2186 Slice 4): every IW-N question must be disposed before
     --status work-completed. Disposition gate (agents/task-create/update-task.sh
     check_disposition_gate) refuses on under-disposed inceptions.

     Per-question shape:

       - **IW-1: <question text>**
         confidence: 0-3      (your confidence in your current answer; 0=guess, 3=verified)
         disposition: answered | deferred | dissolved
         rationale: <one-line evidence — file:line, decision id, dialogue ref>

     Never bare yes/no — the gate refuses bare checkboxes. See 050-Inceptions.md
     §Disposition Gate. Bypass: --skip-disposition-gate "rationale" (direct) or
     FW_SKIP_DISPOSITION_GATE=1 (env-var, T-1890 producer/consumer parity).
-->

- **IW-1: Should the hub be a queryable source of truth, or a transport where agents keep their own history?**
  confidence: 3
  disposition: answered
  rationale: Operator ruled option B on 2026-10-01 — the hub is authoritative for coordination within a declared retention window; long-term records live elsewhere; "forever" becomes an owned exception. Evidence: docs/reports/T-3304-hub-storage-model.md (Synthesis; 2 of 3 consulted agents converge on B; scoring A -20 / B +48 / C 0).
- **IW-2: What retention and pruning model (defaults, who decides, automatic vs explicit sweep)?**
  confidence: 3
  disposition: answered
  rationale: Operator ruled option C on 2026-10-01 ("C then") — bounded default for new topics, forever only with owner+reason, existing hub sweeper (T-2427) on everywhere we can, plus a ceiling checked on post (bounded topics past 2x their limit trim oldest, loudly; forever topics warn, never delete). Default window 14 days = agent recommendation, assumed accepted (operator did not name a window). Evidence: report § IW-2.
- **IW-3: Is cursor + optional filter enough, or are time-range / cross-topic / metadata-index queries needed?**
  confidence: 3
  disposition: answered
  rationale: Operator ruled option B on 2026-10-01 — keep the cursor model; add, in order, the retention-gap signal, the SQ-11 deadline fix (c2), page end reasons, then hub-side time-range reads on hub receive time. No query language, no cross-topic queries, no new indexes now (worst filtered read measured 79 ms). Evidence: report § IW-3.
- **IW-4: How should agents discover and select the topics relevant to them?**
  confidence: 3
  disposition: answered
  rationale: Operator ruled C-prime on 2026-10-01 ("C, as suggested and recommended") after a research round (Kafka, Pulsar, NATS, RabbitMQ, Redis, MQTT, Matrix, Pub/Sub; 27 sources) and five consultants (Codex, GLM-5.3, qwen3, gpt-oss:20b, gemma4) — 5/5 chose C; two catches reshaped it. Evidence: report § IW-4 consultation.
- **IW-5: Does the answer change the SQ-11 (T-2573) fix choice?**
  confidence: 3
  disposition: answered
  rationale: No — SQ-11 is a slice of IW-3 and both serious reviewers asked for exactly c2. Ruled c2 with IW-3 on 2026-10-01; recorded on T-2573.

## Exploration Plan

1. Write the brief with measured facts (`docs/reports/T-3304-consult/brief.md`).
2. Send the identical brief to three non-Anthropic agents, read-only: Codex (OpenAI,
   ChatGPT subscription), GLM-5.3 via opencode (Z.AI Coding Plan), qwen3:14b local
   via Ollama. Answers saved verbatim under `docs/reports/T-3304-consult/`.
3. Synthesise agreements/disagreements; walk the operator through it one question at a time.

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

<!-- What's IN scope for this exploration? What's explicitly OUT? -->

## Acceptance Criteria

### Agent
<!-- @auto-tick-on-decide -->
- [ ] Problem statement validated
<!-- @auto-tick-on-decide -->
- [ ] Assumptions tested
<!-- @auto-tick-on-decide -->
- [ ] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- The operator chooses a storage stance (transport vs queryable store) and the
  charter non-goal #2 is either reaffirmed or amended by the operator
- The resulting changes (retention defaults, query additions, topic discovery)
  decompose into bounded build tasks

**NO-GO if:**
- The current stance (retention-bounded coordination log) is reaffirmed and no
  query/discovery change is warranted

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).
#
# Toolchain hint (L-291): if a GO decision will mean editing *.vbproj/*.csproj/*.xaml,
# *.go, Cargo.toml, tsconfig.json, or pom.xml in the build task, plan to add the
# matching build command (dotnet build / go build / cargo check / tsc --noEmit /
# mvn compile) to that build task's ## Verification — P-011 only runs what you write.

## Recommendation

**Recommendation:** DEFER

**Rationale:**

No evidence yet; operator asked to consult three non-Anthropic agents (Codex, GLM-5.3 via opencode, local qwen3) before forming a view. Charter non-goal #2 currently says the hub is not a system of record; this inception tests that.

**Evidence:**

<!-- Add evidence bullets as exploration progresses (file paths,
     commit hashes, test results). The filing-time recommendation
     can be revised before fw inception decide. -->

## Decisions

### 2026-10-01 — IW-1: what is the hub? (operator ruling)
- **Chose:** B — authoritative for coordination within a declared retention window. The hub is the trusted, queryable record of coordination (who is doing what, recent messages, current state) for a bounded period; long-term records (decisions, learnings, archives) live in purpose-built stores (git, files, a database); retention "forever" becomes a deliberate exception with a named owner.
- **Why:** states honestly what the fleet already does (75 of 111 topics are "forever"); gives one trusted answer to "what happened while I was down"; keeps the hub small; gives SQ-11 and an append-time retention cap a rule to follow. Codex and GLM-5.3 converged on it independently.
- **Rejected:** A transport only (agents are unreliable record-keepers; it degrades into an unguaranteed archive — how the 75 forever topics happened); C queryable archive (fills the disk of the machine everyone coordinates through; every query becomes a contract; rebuilds what git/files do better).
- **Follow-on (not yet decided):** charter non-goal #2 is reworded to match — exact wording comes back to the operator for approval before docs/CHARTER.md changes (three-copy canary, T-2484). IW-2..IW-4 still open, taken one at a time.

### 2026-10-01 — IW-2: retention defaults and enforcement (operator ruling)
- **Chose:** C — (1) new topics default to bounded, **14 days** (assumed; operator did not name a window — Codex said 7, GLM 14); "forever" only when set explicitly with an owner and a reason; the four operator-durable topics keep forever. (2) Turn on the existing hub retention sweeper (T-2427, `TERMLINK_SWEEP_INTERVAL_SECS`) everywhere we run hubs — on at .107 (hourly, verified), off at .122 and .121. (3) A safety ceiling checked on post: a bounded topic past 2x its limit trims its oldest records on the spot and logs it loudly; forever topics get a size warning, never deletion.
- **Why:** all three reviewers want the default flipped from forever; Codex and GLM both want growth bounded even when the periodic mechanism is off or broken (it is off on .122/.121 today).
- **Rejected:** A keep as is (contradicts IW-1: "authoritative for a declared window" cannot hold with forever-by-omission); B defaults + sweeper only (trusts one mechanism completely).
- **Not decided here:** what happens to the 75 existing forever topics (separate review, nothing cut automatically); the inbox "don't drop unacknowledged" guard needs a design check.
- **Process note:** the operator challenged the first IW-2 framing as biased toward the conversation; re-checking the reviewers and the code found the sweeper already existed and was on — the first framing's "35 bounded topics are never swept" was false.

### 2026-10-01 — IW-3: what to add to reads (operator ruling)
- **Chose:** B, in this order: (1) **retention-gap signal** — a reader whose cursor fell behind the oldest surviving record is told so explicitly ("records before X were swept"); (2) **SQ-11 fix, option c2** (T-2573) — a deadline hands back the collected messages plus a resume point that always advances (last scanned + 1); (3) **every page says why it ended** (limit / end of topic / deadline); (4) **hub-side time-range reads** using the existing (topic, ts) index and the hub's own receive time.
- **Why:** (1) is silent loss today and IW-2 makes sweeping routine on every hub; (2) both Codex and GLM asked for exactly this; (4) serves IW-1 — a hub authoritative for a window should answer questions about the window; the index already exists.
- **Rejected:** A without time-range (leaves "what happened while I was down" to client-side download-and-filter); C extra indexes now (worst filtered read measured 79 ms; add only when measured); D nothing (silent gaps become routine after IW-2).
- **Never (3 of 3 reviewers):** a query language, cross-topic queries or joins.

### 2026-10-01 — IW-4: topic metadata, discovery, "is anyone reading?" (operator ruling)
- **Chose:** C-prime — (1) a hub-side record per topic, outside the log (retention cannot erase it): owner taken from the creator's signed identity, optional purpose/description shown as "incomplete" when missing, last writer + write rate, per-reader last fetch and last fetch that returned messages (coalesced, not written per poll); (2) never refuse creating a topic; `--ensure-topic` takes the same path; existing topics get an owner only where derivable, else "owner unknown"; (3) two review flags, never deletion — **idle** (no posts and no fetches for 30 d) and **unread** (posts continue, no fetch for 30 d); ensure/metadata touches are not activity; per-topic override of N; (4) build order: tracking + flags first, catalog second; (5) deferred: wildcard subscriptions, relevance queries, tags.
- **Why:** matches every researched system (metadata outside the log; reader activity tracked server-side); catches the real incident (the ring20 probe is "unread", which a no-read-and-no-write rule never fires on — Codex); owner from identity cannot be junk, and never blocking creation avoids routing-around (GLM; 4 of 5 against a hard purpose requirement).
- **Rejected:** A nothing; B optional catalog (optional is how 12/113 happened); C original (required free-text purpose becomes junk and blocks writes; its single dead-topic rule misses the ring20 case); D relevance queries + wildcards now (5/5 said later; no measured need).
- **Dissent recorded:** gemma4 wanted auto-deletion at 90 d (rejected: Pulsar's auto-delete wipes topic settings; four consultants against). qwen3 and gpt-oss raised access control — out of scope here; belongs to T-2422 (per-agent authorization, revisit ripe).

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-10-01T18:14:36Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
