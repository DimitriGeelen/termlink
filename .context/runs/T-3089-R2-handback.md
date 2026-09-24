# T-3089 Round 2 (R2) handback

**Started:** HEAD `b1e80d46a`. **Ended:** HEAD `8de32b7fc` (a further run-record
bookkeeping commit follows this handback file's own commit).
**Stopped because:** Q1 arc-009 work that was safely actionable this round (no live
shared-infra touch, no second ungated structural change) was exhausted at a clean
task boundary — not a context-budget stop. Context usage stayed well under the
run's ~300K threshold throughout.

## Objectives advanced, and by how much

Directive #2 (Reliability, "no silent failures") remained the operative objective,
continuing R1's selection: arc-011's only two Q1/Q2 candidates (T-3075, T-3076) are
still Tier-0-blocked on a human `fw inception decide` (re-confirmed instantly this
round — `status: started-work` on both, unchanged since R1), so arc-009's backlog
of ~23 `captured`/`owner:agent` tasks remained the correct level-2 re-entry point,
exactly as R1's handoff note predicted.

1. **T-2976 (S-2/C-12 check-half)** — investigated and closed as **already-satisfied**,
   not built. The value-review's F9 finding ("preflight comparator structurally
   unsatisfiable") was itself stale: `crates_unchanged_since_binary()` in
   `scripts/substrate-preflight.sh`, which implements exactly the tag/mtime-aware,
   feature-relevant comparator the finding asked for, landed via **T-2226 on
   2026-06-14** — over 3 months before the 2026-09-19 review ran, and is documented
   in `PL-220`. Verified by reading the function, confirming the commit predates
   the review, and running a live preflight check. Recorded the general lesson as
   **PL-384**: a REPAIR-class finding filed from a canary firing-streak alone (no
   source check) can duplicate work a prior task already closed.
2. **T-2979 (S-5/C-22 triage)** — closed. Of the 11 filings unprocessed at the
   task's filing time, only 3 remained by execution (prior sessions had cleared the
   rest incidentally): a synthetic test fixture (no action), an out-of-scope AEF
   proposal from a peer project (informational, no action), and **one genuine,
   locally-reproducible defect** — `agents/designer/designer.sh` ships mode 644 in
   the vendor drop, so `bin/fw:5578`'s direct `exec` on it dies "Permission denied"
   for every consumer. Independently reproduced before fixing (not just trusted the
   peer's report), cross-checked their scope claim (1 broken of 19 direct-exec call
   sites) against this repo's own `bin/fw` and got the same answer, applied the
   mode-only `chmod +x` unblock, registered it in `.vendor-divergence.yaml` under
   the established T-2812 `mode-only` convention (so a re-vendor silently reverting
   it gets flagged, not rediscovered cold), and replied on framework:pickup
   (offset 145) with the independent confirmation. Canary now clean: 0 unprocessed.
3. **T-3039 (upstream filing: inception decide -> related_tasks gap)** — closed.
   This is literally item (2) of a **human-recorded GO decision already on file**
   (T-3003, 2026-09-20): item (1) — local `check-go-propagation.sh` detector — was
   already built and is green; item (3) — surfacing the 4 genuine orphan
   inceptions to the human — was already filed as T-3040. Re-confirmed the vendored
   `lib/inception.sh` still lacks any `related_tasks`-writing mechanism at decide
   time, filed the defect + proposed `--follow-on` flag to framework:pickup
   (offset 146, `pickup-bug-report`), and registered it in `.vendor-divergence.yaml`
   (`status: filed-upstream`, with a `reverify` step for the pre-re-vendor
   checklist) rather than patching the vendored file (G-062).

**Net: 3 tasks fully closed (T-2976, T-2979, T-3039), 0 code lines changed in
`crates/` — all three closures were investigation/verification/filing work that
turned out to be the actual size of the task once the real state was checked,
smaller in every case than the heuristic `effort=8` estimate implied.**

## Arc state: tasks by status and quadrant

**arc-011** (in-progress, not blocked): unchanged from R1's snapshot. T-3075/T-3076
still `started-work`, still the only two Q1/Q2 items, still waiting on the same
Tier-0 human gate (`fw inception decide`, confirmed refused again this round via
direct status read rather than re-attempting the blocked verb).

**arc-009** (in-progress): 3 more `captured`/`owner:agent` tasks closed this round
(T-2976, T-2979, T-3039), on top of R1's T-2986. Remaining `horizon: now`,
`owner: agent`, `captured` candidates, triaged this round:

| Task | Quadrant read | This round |
|---|---|---|
| T-2977 (reinstall binary, resolve 3-binary PATH ambiguity, restart hub) | Q1 by heuristic BVP-57 | **Deliberately not executed** — live shared-infra risk judgment, see Sovereign questions |
| T-2978 (re-arm push-wake, relaunch unwakeable agents) | Q1 by heuristic BVP-57 | **Deliberately not executed** — same class of risk as T-2977 |
| T-2980 (bind claim-verb identity to verified fingerprint + negative-auth tests) | **Reclassified Q2** after investigation (see below) | Investigated, not started |

The remaining ~20 `horizon: next`/`later` arc-009 tasks were not surveyed in depth
this round — `now`-horizon work took priority per the framework's own horizon
ordering, and was not exhausted until the infra-risk and Q2-reclassification calls
above.

**Other in-progress arcs** (arc-008, arc-parallel-substrate, arc-substrate-fitness,
comms-loudness, mcp-slimming): not re-surveyed this round — R1's finding (all have
zero populated slice registers, arc-009's backlog remains denser and more
concretely scoped) had no reason to have changed in one round and re-checking
would have cost cycles for no new information.

## What remains in Q1/Q2, per task, with reason not done

- **T-3075 / T-3076** (arc-011, Q1, BVP70) — unchanged from R1: Tier-0/human-only
  `fw inception decide` gate. Re-checking status cost one cheap read; re-attempting
  the verb was not worth it (confirmed refused in R1 already).
- **T-2977** (Q1 by heuristic score) — **not a scope block, a risk judgment.** The
  task's own scope includes "restart hub via systemd" on the SAME local hub this
  session was actively using for `termlink channel post`/`subscribe` calls all
  round. `agent-presence` showed only one unrelated LIVE listener (`penelope`) at
  check time, but that doesn't rule out other live consumers this session can't
  see (the R1/orchestrator dispatch path, or R3/R4 about to spawn). CLAUDE.md
  documents hub restarts as a standard, low-risk, systemd-supervised remediation
  — this is very likely fine to just do — but "very likely fine" on a shared,
  partially-observable system is exactly the case the "Executing actions with
  care" guidance asks to flag rather than assume. Left for R3 with this reasoning
  attached so it's a quick go/no-go read, not a re-investigation.
- **T-2978** (Q1 by heuristic score) — same risk class as T-2977 (relaunches other
  agents' sessions via `tl-claude.sh`), same reasoning, same disposition.
- **T-2980** (Q1 by heuristic score, **reclassified Q2 by direct investigation**) —
  not started. Read `channel.rs`'s existing `channel.post` identity-binding
  mechanism (T-1427: per-message ed25519 signature over canonical bytes, verified
  against a claimed `sender_id`) as the pattern "bind to verified sender fingerprint
  as channel.post already does" refers to. Confirmed claim/renew/release/transfer
  carry NO equivalent binding today — `claimer`/`by`/`to_owner` are unauthenticated
  free-string params, gated only by token PERMISSION SCOPE (Interact/Control), not
  identity. There is no existing connection-level verified-identity concept the
  claim handlers could cheaply read (grepped for one, found none) — closing the
  gap for real means extending channel.post's per-message signing convention to
  4+ RPC methods, which touches the hub RPC layer, the CLI, the MCP tool
  implementations, and needs a negative-auth test suite, i.e. a genuine multi-file,
  multi-crate change with a real (if currently low, per the value-review's own "cheap
  NOW because nothing uses V3 yet" framing) backward-compatibility question. That is
  Q2, not Q1, and starting a partial implementation with the round's remaining
  budget would have left a security-relevant change half-done and ungated —
  exactly what "One lock at a time" warns against. Left for R3/R4 (or human
  triage) with this scope note attached so the next session doesn't have to
  re-derive it from scratch.

## Sovereign questions raised, unresolved, in priority order

1. **T-3075 IW-4 / guard-layer "(seconds)" claim** (both carried over from R1,
   unchanged — still open, still genuinely the human's call, not re-litigated this
   round since nothing new bears on either).
2. **T-2977/T-2978 hub-restart risk read** (new this round) — is it safe for an
   autonomous round to restart the shared local hub / relaunch other agents'
   sessions mid-run, given imperfect visibility into concurrent consumers? This
   session's read is "probably yes, but flag rather than assume" — not a
   definitive block, an explicit judgment call surfaced for the operator or a
   round with fuller context (e.g. one that can first confirm no other round is
   mid-dispatch) to make deliberately rather than by default.
3. **T-2980 scope** (informational, not blocking) — flagged as Q2 with the concrete
   shape of the fix (extend T-1427-style per-message signing to 4 RPC methods) so
   whichever round or human picks it up next doesn't have to re-investigate to
   find that out.

## Gates that refused you, and what you did instead

- **`git commit` on T-2976's close-out — blocked once** by `check-active-task`'s
  focus-drift logic (`BLOCKED: Task T-2976 is not active`), the same class R1 hit.
  Worked around identically: `fw work-on T-3089` (re-set focus to the round task)
  immediately before the commit, no bypass flags used. This happened again for
  T-2979 and was avoided the third time (T-3039) by re-setting focus to T-3089
  BEFORE attempting the commit rather than after the first failure — cheaper once
  the pattern was known.
- **A follow-on wrinkle from the same gate class, not seen in R1:** the first
  T-2976 commit landed with only the file rename staged — the finalize-field diff,
  episodic file, and learning entry had been `git add`-ed before the blocked
  attempt but were not part of the successful commit once focus was re-set and
  the command re-run. Fixed with a small follow-up commit rather than amending.
  Root cause not fully diagnosed (plausibly some interaction between the blocked
  hook invocation and the index state — worth a closer look if it recurs a third
  time, but not investigated further this round to stay in scope).
- **`check-vendor-divergence.sh` fired twice** (after the T-2979 and T-3039 commits)
  because it keys registration off the **leading `T-\d+` token in the commit
  subject**, not the task the work closes — and this round's commit subjects lead
  with `T-3089/R2: T-XXXX — ...` (the orchestrated-round convention), so the
  checker parsed `T-3089` and found no matching registry entry. Not a bug in the
  checker relative to its own documented contract; fixed by adding
  `also_task_ids: [T-3089]` to both new `.vendor-divergence.yaml` entries (a field
  the checker already supported, per its own T-2687 precedent) rather than
  rewriting commit subjects. **Worth flagging forward:** any future orchestrated
  round whose commits lead with the round task ID rather than the content task ID
  will trip this same check on any vendored-file touch; the `also_task_ids` fix is
  cheap but easy to forget.
- **G-020 build-readiness gate blocked an unrelated read-only `termlink channel
  subscribe | python3` pipeline** while T-3039 had placeholder ACs, because the
  gate treats any command it can't prove is read-only as a potential write once a
  build task is focused with template ACs. Not routed around — wrote the real ACs
  first (the intended remediation), which also produced a better task record.

## Cost-vs-estimate deltas worth feeding back into calibration

- **T-2976**: heuristic `effort=8` (`lines=207,acs=4`); real cost was reading one
  function, one commit date, and running one script — the heuristic has no way to
  see "the described defect no longer exists," only the file's current size.
  Third instance of this exact blind spot this run (R1 saw it twice on T-3086/
  T-2986) — worth treating as a pattern, not a coincidence: value-review-sourced
  REPAIR tasks specifically seem prone to being already-resolved by the time
  they're worked, because the review methodology (per this round's PL-384) reads
  symptoms (log firing streaks) rather than current source.
- **T-2979**: heuristic `effort=8`; real cost was one grep, one `chmod`, one
  registry entry, one reply post — smaller than either T-2976 or a typical
  11-item triage would suggest, because 8 of the original 11 items had already
  been cleared by intervening sessions and 2 of the remaining 3 needed zero action
  once read closely (test fixture, out-of-scope proposal). The heuristic also has
  no way to see "most of this backlog already drained."
- **T-3039**: heuristic `effort=8`, `tier=1`; real cost was near-zero — the task
  was pre-scoped down to "file one message" by a **human's own prior GO decision**
  (T-3003) that had already done the hard part (deciding what to file and
  confirming the other two follow-on items were already handled). This is a
  distinct pattern from the other two deltas: not "the review was stale," but
  "a human already did the expensive triage, the remaining task was mechanical."
  Worth noting for BVP calibration: a task whose `related_tasks`/lineage traces to
  an already-decided GO is systematically cheaper than an unscoped `captured` task
  with the same heuristic score, and nothing in the current scoring surfaces that.
- **T-2980**: heuristic `effort=8`, same generic score as everything else with
  `components: []` — this is the one case this round where the heuristic's
  under-confidence was arguably closer to right than the other three, though
  probably still an underestimate: real investigation (reading the existing
  T-1427 signing pattern, confirming no reusable connection-identity exists)
  alone took real effort, and the remaining implementation is bigger than 8.

## Handoff note for R3

- Fresh HEAD after this round's commits: `f9285e4c9` (run-record update itself
  lands in a follow-up commit after this file is written).
- arc-011's T-3075/T-3076 are still Tier-0-blocked; don't re-spend a cycle
  re-confirming unless a human has plausibly acted between rounds.
- Best next candidates, in order: (1) a deliberate go/no-go read on T-2977/T-2978
  now that this round's risk framing exists — if R3 has any way to confirm no
  concurrent round/session depends on this hub staying up mid-restart, doing them
  is probably the highest-value remaining Q1 work; (2) T-2980 if R3 has budget for
  a genuine multi-file Q2 build (the investigation in this handback is a running
  start, not a full spec — a real implementation still needs its own design pass
  on backward compatibility for existing unsigned claim callers); (3) survey the
  `horizon: next` arc-009 tail if both of the above are still blocked/risky.
- The `also_task_ids` vendor-divergence fix pattern: if a round's commits lead
  with the round task ID (`T-3089/R2: T-XXXX — ...`) and touch anything under
  `.agentic-framework/`, remember to list the round task ID in `also_task_ids`
  alongside the content task ID, or `check-vendor-divergence.sh` will read the
  commit as unregistered even though it's fully accounted for.
