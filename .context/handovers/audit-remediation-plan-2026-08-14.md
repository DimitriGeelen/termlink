# Audit remediation — status after pass 4 (2026-08-15)

**Trajectory: 30 WARN (pre-pass-1) → 20 → 10 → 6 → 6.** 0 FAIL throughout.

Pass 4 did not lower the count, and that is the finding. Two of the six were
proven **unclearable by construction** rather than merely hard, one dropped from
four sub-items to one named decision, and the pass turned up a sixth defect in
the audit itself — a Tier-0 control that reports clean on a populated log.

Re-run `.agentic-framework/bin/fw audit` before acting on anything below. It
takes **>25 minutes**: CTL-013 re-runs the verification suites of recently
completed tasks, including `cargo test`. Use `--section <name>` for a targeted
check. Do **not** pipe it through `tail` — the summary and all WARN lines appear
early, so `| tail -60` silently discards exactly the part you wanted.

---

## The headline: "zero" is now proven unreachable, not merely asserted

Earlier passes said an agent could not reach 0. Pass 4 replaced that judgement
with an arithmetic proof for the largest warning:

```
audit.sh:1541    if [ "$fabric_unenriched" -gt 10 ]; then warn ...
```

The threshold is an **absolute count, not a ratio**. Clearing it requires 185 of
the 195 edgeless cards to gain edges, on a graph where 71 cards' only real
dependency is the compiled `termlink` binary — a thing a file→file model cannot
name. No parser fix, no enrichment, and no honest work reaches it.

Worse, being absolute, it degrades with growth: every newly registered leaf card
pushes the number further from 10, so the *better* a registry's coverage becomes,
the louder the warning gets. That is why the audit's own trend analysis reports
this finding recurring 11 times in 14 days — not a backlog nobody got to, but a
threshold nobody can meet. Filed as **U-006**, raised to severity **high**.

---

## What pass 4 changed

### Fabric uncovered cards: 4 → 1, honestly (T-2722)

`watch-patterns.yaml` had recorded all four as deliberate exclusions on the
grounds that "inventing globs to chase them would make the pattern list
meaningless". That is right for one of the four and wrong for three. The rule
applied to separate the cases:

> **Widen to a natural category boundary and register every file inside it.
> Never hand-shape a glob around a single file to dodge the check.**

| card | category | files | verdict |
|---|---|---|---|
| `install.sh` | root shell sources | 1 | widened — `*.sh` is a real boundary that happens to hold one file |
| `docs/guides/upstream-reporting.md` | `docs/guides/` | 1 | widened — same |
| `systemd-templates/…worker@.service` | `systemd-templates/` | 3 | widened, and the 2 uncarded siblings registered |
| `.claude/commands/capture.md` | slash commands | **34** | left open — 33 siblings are uncarded |

Verified in a live audit run: `Fabric: 1 card(s) point at files no watch pattern
covers`. `fw fabric drift` reports 0 unregistered, 0 orphaned.

**Why the last one was not forced to zero.** Registering all 34 was considered
and rejected *for now*: `fw fabric register` emits stub cards with no edges, so it
would push the sibling no-edges warning from 195 to ~229. Clearing one warning by
worsening another is moving a problem. Registering all 34 *with their real edges*
(each slash command wraps a script) would be genuinely valuable and is worth doing
on its own merits — but as a registry decision, not as audit cleanup.

### A third defect class found in the audit itself (T-2724 → U-009, **high**)

`.context/bypass-log.yaml` has **two writers with incompatible schemas**:

```
agents/git/lib/bypass.sh:103            -> commit: $commit          (--no-verify path)
agents/context/check-tier0.sh:265,371   -> timestamp/tier/risk/…    (Tier-0 approvals)
```

CTL-010 counts `grep -c "commit:"`. Measured here: **0 matches, 15 actual
entries** — force pushes and `pkill -9` authorizations. It prints, as a PASS:

```
[PASS] CTL-010: Bypass log exists but empty (no bypasses detected)
```

That is a false statement about the highest-privilege audit trail the framework
keeps. And independently: **both arms of the count test are `pass`**, so once the
file exists no content can make CTL-010 fire. Its only failing path is the file
being absent.

Stated fairly — CTL-010's header scopes it to `--no-verify` commits, so counting
`commit:` matches its intent. The defect is the *phrasing* ("empty", "no bypasses
detected" — a claim about the whole file from a partial match) combined with the
unreachable failure. Either alone is minor; together they make Tier-0 report
clean by construction.

### Warning #5 re-read: the framework is its own largest bypass generator (T-2723 → U-008)

The warning counts 26 safety bypasses in 7 days. Grouped by task rather than
counted:

```
13  T-1452   <- the session-handover task
 4  T-1166
 3  T-2567
 3  T-1291
 1  T-2672
 2  (inception filings, different flag)
```

**More than half are one task: the handover.** At session end, focus is correctly
on the working task while the mandatory handover commit references T-1452, so the
focus-drift gate fires on a step the framework itself prescribes — and the block
message offers three escapes, two of which are Tier-2 bypasses.

Corrects pass 3's reading in both directions: pass 2 said these were "the
framework's own handover flow" (wrong — no script sets `FW_SWITCH_FOCUS`), pass 3
said all 24 were "deliberate agent-initiated" drift (true but incomplete — half
are the framework's prescribed wrap-up). Demonstrated avoidable: this session hit
the gate three times and cleared it every time with `fw context focus <task>`,
never the bypass.

---

## The 6 remaining warnings

| # | Warning | Clearable by an agent? | What actually clears it |
|---|---|---|---|
| 1 | Arc `arc-substrate-fitness` stale | **No — sovereign** | Human resolves the 21-day-overdue T-2250 revisit, or `fw arc approve-driver … --i-am-human` |
| 2 | Fabric 195/346 no edges | **No — proven impossible** | Nothing. Threshold is absolute `>10`; see headline. U-006 (high) |
| 3 | Fabric 1 card uncovered | **Not without a scope decision** | Register all 34 slash commands with real edges, or drop the singleton card |
| 4 | Uncommitted changes | Yes, transient | A commit. The audit writes its own report, so any run dirties the tree it then warns about |
| 5 | Gate-bypass 26 in 7 days | **No — and 13 of 24 are the framework's own step** | U-008's direction 1: make `fw handover --commit` refocus around itself |
| 6 | Learnings ready for promotion | **No — curation** | Human runs `fw promote PL-XXX --name "…" --directive DN` |

### Prepared work for #6 (proposal only; the human decides)

The check fires on **application count alone**, so it cannot distinguish a
reusable principle from an incident note that happens to be referenced often.

**Practice-shaped — worth promoting:**
- `PL-168` (17) — *a detector without a trigger is not prevention*. Generalizes
  well past its origin incident (G-058). Suggest **D1**.
- `PL-213` (23) — *proof scripts must assert the property they claim, not the
  happy-path outcome*. Suggest **D2**.
- `PL-206` (17) — *fenced code in docs is inert and lapses silently*. Suggest **D2**.

**Not practice-shaped — recommend NOT promoting:**
- `PL-209` (18) — an incident observation carrying an open "Next: investigate…".
  That is a task, not a practice.
- `PL-195` (11) — a specific bug, already filed as T-1874.
- `PL-172` (13) — a narrow MCP-parity recipe; useful, but a technique note.

---

## The pattern this whole review has been tracing

Every finding across four passes is the same shape: **a guard whose verdict rests
on an assumption about its input that no longer holds.** The catalogue now runs
T-2680, T-2709, T-2714, T-2715, T-2718, T-2719, T-2720, and T-2724 — and it
includes this review's own work three times.

The fabric measurement in T-2718 needed **three** attempts. The original claim
("0 of 149 scripts invoke a sibling") was one grep over three variable idioms,
generalised into a statement about invocation. The first correction ("18 files")
counted `echo` hint-text and heredoc usage blocks as calls, and cited
`run-guard-layer.sh` as proof a parser could fix it — the one file that resolves
its ~20 members by runtime glob with no literal name anywhere, and so is provably
the case a parser *cannot* reach. The settled figure, verified by printing and
reading every matching line, is **17 files / 28 edges**, hidden by two small
parser gaps (`$HERE` unrecognised; known variables matched only after
`source`/`.`, never after `bash`/`exec` or in a bare `VAR=` binding).

Two of those three readings were written down, and one was filed outbound in
U-006 before it was checked.

**The rule that falls out:** a count of zero deserves more suspicion than a count
of a few, and any count headed somewhere outbound deserves its lines printed and
read first. Recorded as PL-341's second-order lesson.

## Why "zero" would now be a red flag

Reaching 0 from here requires **fabrication** (#3 fake globs, #4 impossible while
the audit dirties its own tree) or **overstepping** (#1 closing a sovereign arc,
#6 self-serving a curation decision, #5 silencing a signal that is telling the
truth) — or, for #2, arithmetic that does not exist.

The audit itself offers a fabrication as a mitigation: warning #1 suggests
`fw task update T-XXX --last-update $(date -u +%FT%TZ)` — touching a timestamp so
the staleness check stops firing. That was not done.

**If a future pass reports 0 WARN, check what it faked or narrowed to get there.**

---

## Open items

### Outward filing — needs operator authorisation

Nine upstream records under `.context/upstream/`, all YAML-validated:

| | Task | Sev | Summary |
|---|---|---|---|
| U-001 | T-2711 | med | `revisit-due-scan.sh` exits 0 after scanning nothing |
| U-002 | T-2713 | med | hook telemetry counts exit-2 blocks as failures |
| U-003 | T-2714 | **high** | audit hook checks concatenate `.git/hooks` |
| U-004 | T-2715 | med | CTL-020 worktree-blind; its mitigation is harmful |
| U-005 | T-2717 | **high** | inception decision can contradict its own rationale |
| U-006 | T-2718 | **high** | no-edges threshold is absolute; unsatisfiable and worsens with growth |
| U-007 | T-2720 | **high** | a shipped hub-side rail never moved the deploy floor |
| U-008 | T-2723 | med | handover commit collides with the focus gate it must pass |
| U-009 | T-2724 | **high** | CTL-010 calls a 15-entry Tier-0 log empty; cannot fail |

The post to the shared `framework:pickup` topic is **deliberately not made** — it
is visible to peer projects, so it is the operator's call. On T-2723 that decision
is now a `### Human` AC with a GO recommendation and evidence attached, which is
the pattern the rest should follow.

### Operator actions

- **Upgrade the three hubs** to ≥0.11.871 (install binary, restart *through* the
  systemd unit per G-070). Until then the fleet runs a known silent-data-loss bug
  and the canary names it daily — which is now the correct behaviour.
- **T-1898** revisit — 41 days overdue.
- **T-2250** revisit — 22 days overdue. This is what keeps warning #1 alive.

### Still open for an agent

- **T-2719** — reconcile the two cursor stores (`~/.termlink/cursors.json` vs the
  hub-side ack receipt). Do *not* treat the binary upgrade as the fix: with a
  correct `latest` of 11867 and a stale local cursor of 1611, the printed count
  gets *worse* (10256 against a true 30).
- Two peer-reported usability items, captured in T-2719 and needing their own
  tasks: `agent recent <topic>` resolving its positional as a session, and no
  verb mapping a fingerprint to a name.
- The two `enrich.py` parser gaps in U-006 — worth fixing upstream for graph
  correctness (`fw fabric impact` / `blast-radius` would improve for 17 scripts),
  though it does not move the warning.
