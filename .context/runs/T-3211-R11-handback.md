# T-3211 — procAsFit round 11 of 11 — handback

**Status: COMPLETE.** Stop condition 1. The operator's SQ-22 option C build is done: all three
units are built, proven and closed under **T-3269**. Every other Q1/Q2 item is still gated, as
R10 found; nothing changed that.

**Budget at stop:** ≈235K of 800K (~29%), under `TOKEN_WARN` (600K). Read on stdin from my own
transcript `94cb16ef-eed6-40b8-86d1-226ac5a7aebb.jsonl`; its recent entries are my turns. Readings:
164K → 232K → ≈235K.

**Result:**
- 1 task filed, scored before start, and closed by verb (T-3269).
- 5 commits on `main` since `62aed55df`: `6ad984122`, `a14cd49de`, `992c26374`, `972c78b7b`, plus this handback is uncommitted.
- 1 outward post: `framework:pickup` offset **245**, read back.
- Nothing pushed, nothing tagged, no crontab installed, no tier changed.

## ⚠ Read first
1. **The helper commits under the WRITER's task id, and the T-1730 focus-drift hook does not see it.**
   - That hook reads only the literal Bash command, and `bash scripts/commit-pending.sh commit` contains no `git commit -m "T-X:"`.
   - I used **no bypass**: no FW_SWITCH_FOCUS and nothing else. Git's own commit-msg and pre-commit hooks still run on every helper commit (fixture C6: a refusing hook keeps the entry, rc 1).
   - What replaces the drift check is the attribution rule: the helper only ever commits under an id that a writer recorded next to its own files.
   - I think this meets "find the supported path", but it is structurally *outside* that gate rather than *through* it, so I am raising it as **SQ-23**. The upstream filing suggests an explicit allow-list for the verb instead.
2. **Live proof:** the helper committed the R10 ledger residue (`.context/checks/guard-warn-first-red`, +check-audit-warning-acknowledgement) as `6ad984122`.
   - It was attributed to T-3260, the ledger's task (commit-msg warns that T-3260 is closed; it is a warning, not a block).
   - The commit contains exactly **1 file**, while ~25 other files were dirty in the worktree.
   - The manifest is now empty (`commit-pending.sh list` → "nothing pending").
3. **Not patched, by design:**
   - The vendored `lib/templates/resume-md.md`: only the project copy `.claude/commands/resume.md` was changed. The template change went upstream.
   - The user-level `~/.claude/commands/resume.md`: it differs from the project copy and was left alone. Which copy Claude Code actually loads for `/resume` when both exist is unverified. If it is the user-level one, `/resume` will not show pending entries.
   - `post-compact-resume.sh` still has no pending-commit awareness. The handover `--commit` hook covers the pre-compact commit; the post-compact reinjection does not name pending entries. Possible follow-up; not filed.

## Selection
- **Objective:** the operator's SQ-22 ruling (option C), made structural locally and proposed to the framework.
- **Arc:** guard-layer severity / WARN escalation, the successor of T-3267 (whose filings are the first unattended writes).
- **Task:** T-3269, operator-directed.
- **Score** (estimator + `cost-one`, before any edit): D1=4 D2=0 D3=3 D4=2; blast_radius=5, tier 2, effort 8. This time blast_radius **was** measurable.
- **Why only this:** nothing else in the pool is ungated; see the table below.

## Work log

| unit | outcome | evidence | commit |
|---|---|---|---|
| 1 helper + writers | `scripts/commit-pending.sh add\|list\|commit [--dry-run]` (details below); `warn-escalation-file.sh` records each FILED task (`record_pending`, only for files inside this repo's `.tasks/`); the release canary records the ledger when the `--record-warn` refresh changed it (attributed T-3260; a record failure FIRES) | `tests/commit-pending-fixtures.sh` **17/17, 4 mutants killed**; filer fixtures 34/34; release-publication fixtures 30/30 (both unchanged after wiring) | `a14cd49de` |
| 2 wiring | `/resume` Step 1 item 7 lists entries **by name** and forbids folding them into the git-status count; Current State gains a pending line; Step 3 has a standing first step that runs the helper; procAsFit prompt has carried-fix 5 (list, then commit, first); `handover.sh --commit` runs the helper **before** its pathspec commit (`[ -x ]`-guarded, never fatal); divergence registered for T-3269 | fixtures W1–W4 (W3 asserts the helper call precedes the handover commit line); `check-vendor-divergence.sh` rc 0, "34 commit(s) … all registered" | `a14cd49de` |
| 3 upstream | `framework:pickup` **offset 245**: kind feature-proposal, measured reasons (T-3090/T-3231, the ~20-file baseline, the R10 residue), the full helper text, and the handover, resume and writer diffs. Divergence moved to `filed-upstream` with the offset | read-back: 24,202 bytes, sha256 `08193fa194bc8049`, identical to the source except the trailing newline `$(cat)` strips (same as T-3198) | `992c26374` |
| close | P-011 **6/6**; reviewer **CONCERN**, 1 heuristic finding (F42) | `fw task update --status work-completed` rc 0 | `972c78b7b` |

### What the helper does
- **Commit shape:** `git add -- <paths>`, then `git commit -- <paths>`, one commit per recorded task id. Message: `"<T-ID>: commit unattended write(s) — <reasons>"` plus a body listing the paths.
- **Scope:** only `.tasks/` and `.context/` paths are accepted, checked at `add` and again at `commit`. An out-of-scope entry is refused loudly and kept.
- **Handled cases:**
  - missing file → reported DROPPED;
  - identical to HEAD → dropped;
  - HEAD is verified to carry each file before its entry is removed;
  - a failed commit keeps its entries;
  - `flock` around manifest writes;
  - dedupe on (path, id);
  - rc 0 / 1 / 2.

### Fixture cases
- A1 list by name.
- A2 refusal: src/, absolute paths, traversal and bad ids all refused, and nothing is written.
- **C1 sweep case:** another session's files staged → they stay staged; only the listed paths are committed, 2 commits for 2 ids.
- C2 idempotent.
- C3 missing file.
- C4 out-of-scope entry smuggled into the manifest.
- C5 dry-run.
- C6 hook refusal.
- C7 dedupe / already at HEAD.
- **Mutants:** drop the `--` → C1 red; scope always true → A2 red; no prune → C2 red; no missing-file branch → C3 red.

## What remains in Q1/Q2, and why it was not done
Unchanged from R10; all gated:

| task | why |
|---|---|
| T-2573, T-2606, T-3091 | SQ-11/12/13 |
| T-2886, T-2911, T-3227 | R7 gated pool (T-3227 has a human AC) |
| T-3177 | R6 closure request pending |
| T-2016, T-2015 | waiting on an upstream `upgrade.sh` fix |
| T-2669 | human per-verb timeout decision |
| T-2532 | its own Decisions say "Do NOT guess-and-ship" |
| T-2398 | launches live agents on the fleet rail (SQ-6 shape) |
| T-3245/46/47, T-3250–53, T-3255 | SQ-17, human-owned, or redeploy |
| T-2644, T-3139 | human ACs pending |

Possible small follow-ups (not filed, not scored):
- a post-compact-resume listing of pending entries;
- a Verification leg naming the manifest path directly (F42).

## Sovereign questions (priority order)
1. **SQ-23 (new):** is "helper commits under the recorded writer id, outside the T-1730 hook's view, no bypass" acceptable?
   - The alternative is requiring focus on each id first. That is impossible for completed ids, because T-2874 refuses to focus them.
2. **SQ-9 (carried, approved but not installed):** `release-publication-canary.crontab`. Until it is installed nothing unattended is written, so the manifest only ever holds hand-recorded entries.
3. **Re-file rule (T-3267, carried):** "new 14-day window from the close date".
4. **SQ-19:** check-receiver-ack-lag files its first task on **2026-10-13**. It will now be recorded in the manifest and committed at the next session.
5. **Which `/resume` does Claude Code load** when a project and a user-level command share the name? (Point 3 under "Read first".)
6. Carried unchanged: SQ-20, SQ-21, SQ-17, SQ-15, SQ-16, SQ-1/2, SQ-6/7/8, SQ-10–13.

## Gates that refused, and what I did
1. **check-active-task** refused a read-only `run-guard-layer --list` after the finalize had cleared focus, because the command had a redirect. I refocused T-3211 via `fw context focus`; the next command ran.
2. **`git add` pathspec abort** (the R10 F-note again): adding the vanished `.tasks/active/T-3269-*` path aborted the whole add. The rename was already staged. I added only the episodic, then committed both sides of the rename by pathspec (`972c78b7b`). Afterwards git status showed no T-3269 residue.
3. **Bypass log:** 0 entries dated 2026-09-30. No `--force`, `--skip-*`, `--no-verify` or FW_SWITCH_FOCUS.

## Findings
- **F42:** the P-011 reviewer's CONCERN on T-3269 AC#1 is `AC-verify-mismatch`: no Verification line names `.context/working/pending-commit.list` directly. Coverage is real (fixture A1 asserts the list output), but the finding is correctly heuristic. I recorded it and did not reopen the task.
- **F43:** the focus-drift gate is command-string-level, so any script that commits internally is invisible to it. The gate is narrower than its name suggests. This is now stated in the helper header, in CLAUDE.md, and upstream.
- **F44:** the manifest-writer seam in the release canary (`RELEASE_PUB_PENDING_CMD`) has no fixture of its own. W4 only checks the call by grep, because the canary fixtures run with the test seam and never take the host refresh path.

## Cost-vs-estimate
- Effort 8 / tier 2 was about right: one round-fraction of ≈70K tokens covered three units, including the upstream filing.
- The largest single cost was reading the handover, check-active-task and canary code to find the attribution path.

## Checks recorded

| # | check | result |
|---|---|---|
| 1 | own transcript budget (stdin) | 164K → 232K → ≈235K |
| 2 | BVP estimate + cost-one T-3269 | D1=4 D2=0 D3=3 D4=2; br=5 tier 2 eff 8 |
| 3 | commit-pending fixtures | 17/0, 4 mutants killed |
| 4 | filer / release-publication fixtures after wiring | 34/0 / 30/0 |
| 5 | live helper commit | `6ad984122`, 1 file; manifest empty after |
| 6 | check-vendor-divergence | rc 0, 34 commits all registered (before and after the filed-upstream edit) |
| 7 | pickup post + read-back | offset 245, delivered; 24202 B, sha 08193fa194bc8049, suffix-only delta |
| 8 | P-011 T-3269 | 6/6 PASS; reviewer CONCERN (F42) |
| 9 | guard-layer discovery / marker checker | fixture suite listed (FAIL tier) / clean 93/23/1 |
| 10 | bypass log 2026-09-30 | 0 |
