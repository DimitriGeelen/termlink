<!-- T-3612 round 4, seat codex, source /tmp/tl-dispatch/t3612-r4-codex-f311e7b0b140/result.md, exit 0 -->
1. **Section 1 — CHANGE. Purpose is directionally right, but promises too much.** Q1 explicitly combines delivery and terminals; Q2 assigns routing and handoff mechanics, not successful execution. Neither sentence should imply guaranteed agent action. Prefer a revised main sentence:

   > “TermLink lets agents and their operator reach and control live terminals, exchange durable messages and hand off work across machines, with evidenced delivery stages and visible failures.”

   Keep the scope sentence. The alternative’s “until an agent acts” exceeds what transcript evidence establishes. Sources: `docs/reports/T-3612-purpose-rederivation.md`, Dialogue Log 5–7; `crates/termlink-sidecar/src/deliver.rs:16–19`; `crates/termlink-session/src/pty.rs`.

2. **Section 2 — CHANGE. Keep all seven subjects, revise their contracts and measures.**

   - **P-1: change.** “Accepted messages are durably stored; senders can distinguish handover, response/no-action acknowledgement, and unresolved failure.” Being loudly stuck is not successful completion. Require crash/restart recovery tests and per-rail stage evidence, alongside fresh, successfully executed canaries. An empty detector is insufficient: `scripts/check-unacted-mail.sh:17–40` describes bounded, partly heuristic coverage and explicit NOT CHECKED outcomes. Sources: T-3618; `deliver.rs`; `README.md:50–66`.
   - **P-2: keep, clarify measurement.** The outcome is sound, including exact-instance conversation binding. Add a specifically planned address-resolution matrix covering all five levels, dead instances, grants, and fallback. Pinned-main resolution alone proves only a subset. Sources: `crates/termlink-sidecar/src/book.rs:7–77`; T-3360; T-3619; Dialogue Log 7, 9.
   - **P-3: change.** “Work ownership and outcomes remain unambiguous across retries and worker failure: one authoritative dispatch record, fenced attempts, and idempotent terminal transitions.” Drop “done once”: the implementation explicitly permits subsequent attempts. The conformance adapter already exists; distinguish unfinished coverage from an unbuilt measure, and require zero unexplained skips/not-implemented cases for applicable requirements. Sources: `crates/termlink-bus/src/worker_contract.rs:105,238–253`; T-3517; T-3530, reporting 15 pass, nine not implemented, one N/A.
   - **P-4: keep.** Outcome-shaped and supported by concrete terminal operations. Name the self-test command, supported-platform matrix, cleanup deadline, and fresh execution requirement rather than merely empty logs. Sources: `crates/termlink-session/src/pty.rs`; draft §2 P-4; `.github/workflows/release.yml`.
   - **P-5: keep, qualify present coverage.** Signatures prove attribution, not permission. Add negative tests for unauthorised post, inject, and lifecycle writes, plus isolation tests. “Every act” remains a target; existing checks do not establish universal coverage. Sources: `crates/termlink-hub/src/channel.rs:883–925`; `worker_contract.rs:456–463`; T-2422; T-3423; Dialogue Log 10.
   - **P-6: keep, complete measurement.** Add a specifically planned clean-host install→upgrade→rollback acceptance test on Linux/macOS. Publication and a macOS test gate do not prove those transitions. Source: `.github/workflows/release.yml`; draft §2 P-6.
   - **P-7: change.** “The supported surface earns its maintenance and context cost through demonstrated coordination use or a documented requirement; removal decisions follow representative fleet evidence.” Defer the removal schedule and thresholds, not just the cut rate. Sources: Dialogue Log 11; T-3615; `policy/bvp-scoring-rubric.md`, D3.

   Missing measurable outcomes: bounded coordination-record retrieval and responsiveness under load. Add these to P-1/P-7 rather than necessarily adding objectives. Sources: T-3304 IW-3; T-3488; `docs/reports/T-3612-seats/round1-codex.md` §5.

3. **Section 3 — CHANGE. Priorities are plausible; independence and implementation need work.**

   D2 highest, D3 raised, D4 retained are defensible priorities, not empirically established numbers. D1=5 is a reasonable provisional choice, but lowering it does not remove incentives for canaries: D2’s rubric also rewards audit checks. Moreover, **D2=10 cannot be set through the current command**, which accepts 0–9. Use 9 unless the tooling range is deliberately changed. Sources: `policy/bvp-scoring-rubric.md`, D1–D4; `.agentic-framework/lib/bvp.sh:962–964`.

   **F-TRUST passes conditionally:** permission boundaries and isolation are distinct from reliable/auditable execution. Define it as “least-privilege authority and isolation”; score attribution-only visibility under D2. Retirement must include the isolation target, not merely two tasks landing. Sources: Dialogue Log 10; T-3351 OQ-2; D2 rubric.

   **F-SURFACE does not yet demonstrate independence.** Fewer, clearer verbs overlap D3; maintenance reliability overlaps D2. Define the separate axis as “bounded resource and maintenance cost per supported coordination capability,” with measured context, process, storage, and maintenance budgets. Otherwise reweight D3. Sources: `policy/value-drivers.yaml`, free-driver distinction; D3 rubric; round-2 synthesis §1e.

   **F-RECALL passes only as positive reuse**, not generic auditability or easier retrieval. Wording: “Retained conversation evidence can be reused in subsequent work through bounded, per-thread reads and explicit export.” Keep both layers as ruled. Dropping F-ORCH is consistent with Q2; its existing rubric rewards framework executor expansion. Sources: Dialogue Log 6, 14; `policy/value-drivers.yaml`, F-RECALL/F-ORCH.

4. **Section 4 — CHANGE narrowly.** Non-goals 1, 2, 4, 5 and 6 match Q2–Q5/Q7. Explicitly add **“no cross-topic queries”**: T-3304 IW-3 forbids these as well as joins and query languages. “By peer” must select conversations, not silently introduce a hub-wide query.

   Social analytics remain implemented: `termlink_channel_emoji_stats` and `termlink_agent_emoji_stats` in `crates/termlink-mcp/src/tools.rs:21391,24581`. They exemplify the acknowledged surface tension. However, their existence alone does not violate the draft’s conditional “removal candidates” wording; Q6 expressly postpones disposition. Do not label every reaction or poll a proven violation. Sources: Dialogue Log 11; `docs/CHARTER.md:43–45`.

5. **Section 5 — CHANGE.** The split is useful, but DP-3’s requirement that *every* guard prove a product objective excludes legitimate development-governance guards retained by Q7. Use:

   > “Every guard names the product or development objective it protects, its owner, cost and retirement condition.”

   Explicitly retain D′’s requirement that AEF/ring20 chores leave `runme.sh`; “development project” must not become their permanent home. Product usability also needs a product measure for avoiding human message relays; DP-1 covers this repository’s operator workflow. Sources: Dialogue Log 12; draft §5; round-2 synthesis §1d.

6. **Section 7 — CHANGE; recommendations on all five points.**

   - **Purpose:** revised main sentence in finding 1; Q1 chooses both capabilities.
   - **Workspace:** retain a narrowly bounded worker-owned isolation provision in P-3 pending an explicit superseding decision. T-3469 already rules clone-per-claim and worker-side CLI/MCP mechanics; moving everything to AEF is not a neutral cleanup.
   - **D1/cap:** provisionally 5; cap at the inventoried active baseline, then reduce by measured redundancy/cost. No defensible absolute number follows from script counts alone. Sources: synthesis §2 D6; Dialogue Log 12.
   - **Layer tooling:** document the second set now; request AEF support before claiming operationally separate scoring. Source: `bvp.sh:286–301`.
   - **Recall:** retain bounded reuse, forbid cross-topic queries, and make long-term-store selection the consumer’s responsibility. Retire the temporary driver after accepted per-thread/time-range retrieval, gap reporting, and explicit export acceptance tests pass. Sources: T-3304 IW-1/IW-3; Dialogue Log 14.

7. **Section 6 — REJECT as a sufficient adoption checklist.** Add these migration requirements:

   - Update the charter canary’s hardcoded anchor and `machines` terminus, not only its three documents. Source: `scripts/check-charter-sentence-drift.sh`, `ANCHOR`/`extract_sentence`.
   - Reconcile strict-star documentation and tripwires with permitted circuits/directory cards without removing the no-message-relay protection. Sources: `crates/termlink-session/tests/no_spoke_mesh_tripwire.rs`; `crates/termlink-hub/tests/no_federation_tripwire.rs`.
   - Preserve explicit O→P/DP aliases: O-1/O-2→P-1, O-3→P-3, O-4→P-4, O-5→P-6/DP-2, O-6→DP-1, O-7→P-5; verify every arc reference. Add **DP-5**, omitted from §6.2.
   - Version scoring by layer/rubric and rescore affected tasks. Reusing F-RECALL’s ID otherwise gives historical scores a new meaning; new drivers need scoring handlers. Sources: `bvp.sh:286–301,345–362,1756–1767`; draft §§3,6.

8. **Three priority changes:** first, executable adoption/scoring migration (7); second, evidenced delivery and attempt semantics (1–2); third, bounded recall and defensible free-driver separation (3–4).

**Confidence: medium overall; high on cited tooling defects.** Read the draft, dialogue, synthesis, four earlier reviews, and sampled code/tasks. The local snapshot lacks `.context`; Git metadata could not verify `26bccc7bf`, and GitHub DNS failed. Objectives/arcs were therefore assessable only through quoted repository reports, not directly. No builds, tests, live-fleet checks, or repository edits were performed.