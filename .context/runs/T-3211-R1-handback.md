# T-3211 — procAsFit round 1 of 9 — handback

**Status: IN PROGRESS (skeleton written early per carried fix 3; sections fill as the round proceeds).**
Worker: `claude -p` (Fable 5.1), started 2026-09-29. Run record: `.context/runs/T-3211-R1-*`.

## Baseline at run start (for later rounds to measure against)

- HEAD: `5802c0ed9` (T-3211: procAsFit x9 harness — and the dispatch defect that would have faked all nine)
- Focus: `T-3211` (focus.yaml). Focused arc: `arc-010` (file `.context/arcs/arc-011.yaml`, slug `arc-011`,
  name "Agent-to-agent message delivery: mailbox to prompt", status in-progress).
- Budget at start: 153,209 tokens (~19% of CONTEXT_WINDOW=800000), read via
  `.agentic-framework/agents/context/checkpoint.sh status` (NOT `.budget-status`, T-3127).
  Stop threshold by NAME: `TOKEN_WARN` = 75% = 600,000.
- arc-010 slices: 12 (S1 unbuilt/T-3135, S2–S4 built, S5 partial/T-2300, S6–S11 built, S12 unbuilt/T-3135).
  8 sovereign questions, ALL RESOLVED (SQ-1..SQ-8). No open SQ blocks the arc.
- arc-010 active member tasks: exactly ONE — `T-3135` (status captured, owner agent, Q2 hv-hc,
  BVP 57 proposed / cost 4.4 proposed). All other slice tasks are in `completed/`.
- BVP quadrant census (`fw bvp --quadrant ... --include-proposed`): Q1 hv-lc = 39 tasks, Q2 hv-hc = 14 tasks,
  245 tasks total, 36 unassessed, 102 without cost.

## Selection

**Unit 1 — objective → arc → task → quadrant.**
- **Objective:** charter verb 2, "exchange durable messages", as stated by arc-010's headline
  mechanic (sender learns RECEIVED and INJECTED as two confirmed events without either side
  attached). Nothing else in the backlog names a project objective more directly than the one
  in-flight arc.
- **Arc:** arc-010 (slug arc-011) — already in flight, focused, NOT blocked (all 8 SQs resolved).
  Mandate: prefer an in-flight arc over opening a new one.
- **Task:** T-3135 "Sidecar API: local control surface + portable respawn supervisor" — the arc's
  ONLY active task (every other slice task is in completed/). It is the unbuilt half of the arc
  (S1 + S12).
- **Quadrant:** Q2 hv-hc (BVP 57 proposed, cost 4.4 proposed). There is NO Q1 task in this arc,
  so Q2 is worked; per mandate Q1 is exhausted-by-absence, not skipped.
- **Why this over the next candidate:** the next candidates are Q1 tasks OUTSIDE the arc
  (T-3132, T-2203, T-2644 …). Level-2 gating says stay in the in-flight arc while it has Q1/Q2
  work. T-3135 is scored (not by me — estimator rows dated 2026-09-25 are in its frontmatter).
- **First activity (unit 1):** the task shipped with placeholder ACs (`[First criterion]`). G-020 refuses
  source edits until real ACs exist, so activity 1 was writing ACs from T-3075's Scope Fence +
  SQ-1 + SQ-8 + design §6. No scope was invented: every AC traces to a ruled item.

**Unit 2 — level-2 re-entry after arc-010 exhausted.**
- **Arc:** arc-008 "Audit and doctor finding remediation" — in flight, 83% closed (T-3117), headline
  mechanic = an operator reads a clean `fw audit`/`fw doctor` report. Its completion is the nearest
  of any in-flight arc, and it is the arc the mandate's own auditability clause leans on.
- **Task:** T-3103 "Fabric: 10 cards point at files no watch pattern covers" — Q1 hv-lc (BVP 57,
  cost 1.9, captured, agent, build). Why this over its siblings: T-3132 (Q1, 85) is PARKED — its
  last AC is recorded as not achievable (G-095: CTL-029 flags the designed end state and its
  remediation would destroy a G-053 reminder); T-2958 and T-3130 (Q1, 57) have reached their
  terminal state (root cause filed upstream per G-062, remaining ACs answered-negative); T-3127 /
  T-3128 (Q1/Q2) are the vendored budget-status class already filed upstream. T-3103 is the one
  arc-008 Q1 whose fix is local (`.fabric/watch-patterns.yaml`) and whose AC is a re-run of the
  audit — exactly the arc's own verification rule.
- **Finding for the operator (not decided here):** the highest-BVP agent-owned Q1 tasks in the
  whole backlog — T-2644 (78), T-2573 (76), T-2606 / T-2656 / T-2662 (73) — belong to NO arc, so
  the mandate's project→arc→task ladder can never reach them. T-2573 additionally has a decide-
  the-wire-contract AC that is a Sovereign question in its own right. Listed under Sovereign
  questions below.

## Objectives advanced

**Unit 1 — T-3135 CLOSED (verb-gated: `fw task update T-3135 --status work-completed`, P-010 6/6,
P-011 8/8; commits `244d7d03f` + `bcc8905bb`).** Against run start:
- arc-010 went from 1 active task / 2 unbuilt slices (S1, S12) to **0 active tasks / S1 built /
  S12 partial**. Every remaining arc-010 slice is built or partial; the arc's ONLY open item is
  external (framework-agent-systemd `--allowed-commands`, T-559 boundary) plus the arc-close
  decision, which is a human verb (`fw arc close` needs `--demo` evidence; the arc's prover is
  `scripts/notify-rail-e2e.sh`, not run this round — see Sovereign questions).
- Charter verb 2 ("exchange durable messages") gained the local control surface design §6 called
  for, with its bright line enforced by a tripwire rather than remembered; SQ-8's portable
  respawn requirement is met with a bash-only core and host-native unit emission.
- Live evidence (read-only, real rail, this host): `status` → listener ALIVE 5s, hub reachable,
  fqdn/ip recorded; `queue` → 91 pending above local watermark -1 (labelled — see caveat);
  `--emit-unit auto` → chose systemd and said so.

## Arc state (tasks by status and quadrant)

**arc-010 (focused at start):** active tasks = 0 (was 1). Completed this round: T-3135 (Q2 hv-hc).
Slices: S1 built (was unbuilt), S2–S4 built, S5 partial (T-2300, unchanged), S6–S11 built,
S12 partial (was unbuilt). Q1/Q2 in this arc: **exhausted** — no active member task remains.

**Other in-flight arcs (level-2 re-entry candidates, measured after unit 1):**
- arc-008 "Audit and doctor finding remediation" (73 tasks): Q1 T-3132 (BVP 85, cost 2.0,
  captured, agent, build), Q2 T-3128 (BVP 85, cost 4.4, started-work, agent). T-3117 notes the
  arc hit its 80% closure threshold.
- arc-009 (47), arc-001 (45), arc-002 (12), arc-007 (4), arc-005 (3): not yet censused for Q1/Q2
  this round.
- Q1 tasks outside any arc: T-2644, T-2573, T-2606, T-2656, T-2662 (agent-owned, captured);
  T-2203, T-2576, T-2577 are `owner: human` (not eligible for an agent worker).

## What remains in Q1/Q2, per task, with reason not done

_(pending)_

## Sovereign questions raised (unresolved, priority order)

_(none yet)_

## Gates that refused, and what was done instead

1. **G-020 scope-aware task gate** (`check-active-task`) BLOCKED a Bash command after
   `fw work-on T-3135`, because T-3135 had placeholder ACs and the command's grep pattern
   contained `>` (matched as a redirect). The blocked command was a read (`sed -n`/`grep`), so
   the match was a false positive on the write-detector — but the gate's premise (no source
   edits on an unscoped build task) was correct. Did NOT route around: wrote real ACs into the
   task file (the documented unblock), then re-ran the read without the `>` character.
2. **`fw work-on T-3135`** warned: 76 other tasks already in started-work. Not a refusal; noted
   as fleet hygiene, not this round's scope.

## Cost-vs-estimate deltas

- **T-3135:** estimator said cost 4.4 (blast_radius 5 / tier 2 / effort 8 — "lines=204, acs=4"
  measured on a body that was still placeholder). Actual: ~1 round-unit of a single worker,
  ~130K context tokens (153K→~283K), 5 files created/changed + 1 arc yaml, 2 gate refusals
  satisfied. The estimate was made on a task with NO real ACs, so its effort term was reading
  template boilerplate. Feedback for calibration: a `captured` build task with `[First criterion]`
  should be flagged UNSCORABLE for effort rather than scored from template line count (the
  T-3185 "unassessed ≠ low value" lesson applied to cost).

## Checks recorded (traceability ledger)

| # | check | result |
|---|---|---|
| 1 | `checkpoint.sh status` at start | 153,209 tokens (~19%) |
| 2 | `fw work-on T-3135` | captured→started-work, focus set (rc 0) |
| 3 | G-020 gate on first source read after work-on | BLOCKED (placeholder ACs) — satisfied by writing ACs |
| 4 | `bash tests/notify-sidecar-api-fixtures.sh` (run 1) | 87 passed / 6 failed — `QUEUE_WM` unbound in subshell |
| 5 | same, run 2 after fix | 92 / 1 — static tripwire false-positive on usage text `--remote —` |
| 6 | same, run 3 after regex tightened to `[^a-z-]remote ` | **93 passed / 0 failed** |
| 7 | `bash tests/notify-sidecar-supervisor-fixtures.sh` | **17 passed / 0 failed** (pre-existing suite, unchanged behaviour) |
| 8 | each Verification line under `bash -c 'set -eo pipefail; …'` | 8/8 PASS (one failed on the first rehearsal, fixed by #6) |
| 9 | live `notify-sidecar-api.sh status --agent-id claude-termlink` | rc 0, ALIVE, hub_reachable=true |
| 10 | live `queue --agent-id claude-termlink` | rc 0, 91 pending above local watermark -1 (caveat documented) |
| 11 | live `--emit-unit auto` | chose systemd, printed unit |
| 12 | `fw task update T-3135 --status work-completed` (attempt 1) | P-010 6/6, P-011 8/8, REFUSED by T-1718 (empty Evolution) |
| 13 | same (attempt 2, after Evolution entry) | completed, episodic generated, file moved to completed/ |
| 14 | `python3 yaml.safe_load(.context/episodic/T-3135.yaml)` | parses to a mapping (T-2805 class clean) |
| 15 | `fw git commit` ×2 | `244d7d03f` (rename only — git add aborted on stale pathspec) + `bcc8905bb` (content) |
