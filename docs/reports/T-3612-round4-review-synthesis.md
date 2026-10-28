# T-3612 round 4 — outside review of draft v0.2: synthesis

**Seats:** claude (`T-3612-seats/round4-claude.md`), codex (`round4-codex.md`), antigravity (`round4-antigravity.md`).
opencode (GLM) failed for the third time: its result stops mid-way with no verdicts (`round4-opencode.md`). It is
counted as missing. This is review round 1 of the 2 the operator allowed.

**Verdicts:** all three say *change* on every section, and none rejects the draft. Codex rejects section 6 only as an
adoption checklist.

## A. Agreed by all three (to fold into v0.3 without a new decision)

1. **The scoring tooling cannot take the draft as written.**
   - 1a. D2 = 10 cannot be set; `fw bvp weight --set` accepts 0–9 (`.agentic-framework/lib/bvp.sh:962`). Use 9.
     (claude, codex)
   - 1b. Reusing the id F-RECALL runs AEF's hard-coded memory scorer (`estimator.py:2631`, and the audit's
     `recall_signals()`). The product driver needs a new id, e.g. F-CONV-RECALL. (claude, antigravity; codex: old
     scores would change meaning)
   - 1c. F-TRUST and F-SURFACE need `scoring:` blocks, or they score 0 and the audit WARNs. (all three)
2. **P-1 dropped real measures from O-2.** Restore the retention-gap notice, hub responsiveness under load (T-3488)
   and restart-safe post dedupe (G-088). (all three)
3. **P-5 overclaims.** Agents are not isolated today: there is a shared uid 0 and a shared root key on .107, and
   fleet hosts still hold .107's full secret. Move isolation to "not yet". (all three)
4. **The bounded record needs saying.** "Find by peer" must mean choosing a conversation, never a cross-topic query.
   "Hands what must be kept to a long-term store" goes beyond the T-3304 ruling: keeping long-term records is the
   consumer's job. (all three)
5. **Section 6.2 omits DP-5.** (all three)
6. **The charter-sentence drift guard extracts ONE sentence** between a hard-coded anchor and the word "machines"
   (`scripts/check-charter-sentence-drift.sh`). Adopting a new sentence must update the guard and its test in the
   same change, or the release gate fails closed. (codex, antigravity)
7. **The strict-star tripwires** (`no_spoke_mesh_tripwire.rs`, `no_federation_tripwire.rs`,
   `check-strict-star.sh`) conflict with OD-1 direct circuits and OD-11 card exchange. They must be reworded to
   protect "no message relay", not removed. (codex, antigravity)
8. **Dropping F-ORCH fits Q2.** Claude adds that AEF itself retired F-ORCH (T-2511).

## B. Agreed by two, not contested

9. **P-7's "removed on a schedule" decides Q6 before the data.** Defer the schedule and thresholds too. (claude, codex)
10. **DP-3 should read "a P- or DP-objective".** Development guards are legitimate under Q7. (claude, codex)
11. **Per-worker workspaces stay in TermLink (P-3).** T-3469 already ruled clone-per-claim, and
    `dispatch --isolate` ships. Isolation policy stays with AEF. (codex, antigravity)
12. **Non-goal 1 should name OD-11:** hubs exchange directory cards, never messages. (antigravity; matches the
    rulings summary)
13. **Non-goal violations in the code today** (antigravity in full, claude and codex in part):
    - 13a. social tools still compiled (polls, emoji stats, top-repliers);
    - 13b. 75 forever topics without owners (T-3314);
    - 13c. `orchestrator.route`, `bypass_status` and `bypass_invalidate` still in the hub router (`router.rs:78-80`;
      antigravity; verified 2026-10-28);
    - 13d. AEF and ring20 chores still in `runme.sh` (known, Q7).

## C. Where they differ (the operator's open points)

14. **Purpose sentence (open point 1).**
    - claude: the main sentence, plus "or is loudly stuck", and name the operator as a user.
    - antigravity: the main sentence, rewritten as ONE sentence that adds waking the terminal and "recording who
      acted"; the scope sentence moves to the paragraph after it.
    - codex: the draft promises too much. TermLink cannot guarantee that an agent *acts*, only that delivery stages
      are evidenced and failures visible. Its wording: "TermLink lets agents and their operator reach and control
      live terminals, exchange durable messages and hand off work across machines, with evidenced delivery stages
      and visible failures."
15. **D1 weight and canary cap (open point 3).**
    - D1: codex 5 (provisional), antigravity 6, claude no number.
    - Cap: antigravity a hard number (35 cron canaries, 75 source guards, then one in, one out); codex "cap at today's
      inventory, then reduce by measured redundancy and cost"; claude no number.
16. **Which driver set the live file holds (open point 4).**
    - claude: keep the development set in `policy/value-drivers.yaml` for now. About 92% of commits are development
      work, so scoring them on product axes would be wrong. Product arcs can use the existing arc-scoped drivers.
    - antigravity: write a second file for the development set now, and ask AEF for per-layer scoring.
    - codex: document the second set now, and ask AEF before claiming separate scoring.
17. **Product recall's meaning and retirement (open point 5).**
    - codex: "positive reuse" only (bounded per-thread reads plus explicit export); retire once those acceptance tests
      pass.
    - antigravity: new id F-CONV-RECALL; retire when arc-012 closes.
    - claude: tie it to a new P-8 "bounded record" objective, and retire when P-8 is green.
18. **F-SURFACE independence.** Codex says it overlaps D3 (fewer, clearer verbs) and D2 (maintenance) unless it is
    defined as "bounded resource and maintenance cost per capability". Antigravity says it passes as is.
19. **Where `check-unacted-mail.sh` belongs.** Antigravity: it is development-fleet tooling (it calls
    `fw sidecar`), so it measures DP-2, not P-1. Codex: an empty detector is not proof for P-1 either. Claude:
    silent.

## D. Plan

20. Fold A and B into draft v0.3 (no decision needed: they are corrections, or follow from earlier rulings).
21. Take C to the operator one at a time: 14 (purpose), 15 (D1 and cap), 16 (driver file), 17 (recall), then 18–19
    if still open.
22. A second review round, the last allowed, only if the operator's answers change the draft materially.
