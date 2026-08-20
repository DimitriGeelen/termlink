---
id: T-2803
name: "One-line remediation script for the main-checkout steps, executed via TermLink"
description: >
  The three mechanical main-checkout steps (track the vendored framework subset, track the
  static-check allowlists, clear stale task files) required the operator to hand-run six
  commands and eyeball a file list for secrets. Ship one idempotent script that performs the
  safety review mechanically, refuses on anything suspicious, never touches the Tier 0 push,
  and writes a machine-readable report — then execute it in the main checkout through a
  TermLink session rather than handing it back as homework.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [governance, remediation, termlink, operator-ux]
components: []
related_tasks: [T-2694, T-2698, T-2690, T-559]
created: 2026-08-20
last_update: 2026-08-20
date_finished: null
---

# T-2803: One-line remediation script for the main-checkout steps

## Context

Three fixes shipped on this branch are only half-done, because each needs a commit in the
main checkout at `/opt/termlink` and this session is confined to a worktree by T-559:

1. **T-2694** — the `.gitignore` rule was narrowed so the vendored framework subset is
   trackable, but the files themselves (`lib/bvp.sh`, `policy/`) are still uncommitted. Until
   they are, `fw bvp` fails in every worktree and every clean clone, which is why the BVP
   estimator could not be run this session despite being asked for repeatedly.
2. **T-2698** — same shape for `.context/working/.*-allowlist`. Until committed,
   `check-alloc-sink-clamps` and `check-drain-sink-caps` fire in any fresh checkout on eleven
   sites a human already reviewed and cleared.
3. Stale task files in `/opt/termlink/.tasks/active/` that make the task gate refuse to let
   an agent work in the main checkout at all.

The operator was handed six commands and an instruction to "read that list — confirm nothing
machine-local or secret-bearing appears". That review is the right requirement and the wrong
mechanism: a human scanning a 30-file list for secrets is exactly the kind of check that gets
skimmed on the third repetition. Their reply — *"can you put all in one script i can run from
one line"*, then *"or use termlink to get it done"* — is correct on both counts.

## Approach

**Automate the review, not away the review.** The script scans every file it is about to
stage for high-confidence secret markers (PEM blocks, bare 64-hex secrets matching the
`hub.secret` format, populated `password:`/`api_key:`/`token:` fields) and **refuses to commit
anything if it finds one**, naming the file and line. Softer signals (absolute home paths,
host addresses) are reported for the operator's eye but do not block, because this repo
legitimately discusses `192.168.10.x` throughout. The human judgement is preserved as a
mechanical gate that cannot be skimmed.

**Never touch the push.** Item 4 on the operator's list — `git push --no-verify` past the
failing pre-push audit (T-2690) — is Tier 0 and stays out. The script reports the unpushed
commit count and prints the command; it does not run it, and no flag makes it.

**Idempotent.** Every step is a no-op when already applied, so re-running is safe and the
report tells you which steps did nothing.

**Executed through TermLink, not handed back.** `termlink spawn --cwd /opt/termlink` runs the
script in a session rooted in the main checkout — the path the T-559 hook itself names for
legitimate cross-boundary work, precisely because the target enforces its own governance in
its own process. The script is still committed here as a reviewable, re-runnable artifact
rather than a one-off command stream, so what ran is auditable afterwards.

**Report where the agent can read it.** The run writes JSON into this worktree's
`.context/working/`, which this session can read directly — so the outcome does not depend on
the operator copying terminal output back.

## Acceptance Criteria

### Agent
- [ ] One script performs all three mechanical steps and is safe to re-run (idempotent)
- [ ] It scans every file it would stage and REFUSES to commit if a high-confidence secret
      marker is found, naming file and line
- [ ] Soft signals (home paths, host addresses) are reported but do not block
- [ ] It never pushes, and no flag makes it push — Tier 0 stays with the human
- [ ] It reports the unpushed commit count and prints the push command without running it
- [ ] `--dry-run` shows exactly what would happen and changes nothing
- [ ] It writes a machine-readable JSON report this session can read back
- [ ] Each step reports applied / already-done / skipped-with-reason — never silent success
- [ ] Verification of the outcome is part of the run (the tracking-drift and static checks are
      re-run afterwards and their exit codes recorded)
- [ ] Fixtures prove the secret-refusal path on a scratch repo, host-independent (PL-213)
- [ ] Executed against the real main checkout via TermLink, with the report read back here

## Verification

bash tests/remediate-main-checkout-fixtures.sh
bash scripts/remediate-main-checkout.sh --dry-run --root .

## Decisions

### 2026-08-20 — Mechanise the secret review rather than dropping it

- **Chose:** A blocking scan for high-confidence markers; soft signals reported only.
- **Why:** I asked the operator three times to eyeball a file list. A review that is requested
  repeatedly and never fails is a review that has already stopped happening. Encoding it makes
  it real; keeping the soft signals visible keeps the human judgement where it is actually
  needed.
- **Rejected:** Blocking on any occurrence of the word "secret". This repo's framework code is
  full of legitimate discussion of `hub.secret` and secret rotation; a scanner that cried wolf
  there would be turned off immediately.

### 2026-08-20 — TermLink executes; the script is still committed

- **Chose:** Ship the script as a repo artifact, then run it through a TermLink session rooted
  at `/opt/termlink`.
- **Why:** Doing it as an ad-hoc command stream would get it done and leave nothing to review;
  committing the script means the next person can see exactly what was run, and re-run it.
- **Note:** this is the path the T-559 boundary hook itself names — the target directory
  enforces its own governance in its own process, so nothing is bypassed.

### 2026-08-20 — The push stays out

- **Chose:** Report it, print the command, never execute.
- **Why:** Tier 0 is not delegable by a broad "get it done" instruction. The framework's own
  rule is explicit that a structural gate exists precisely for moments when an agent has been
  told to proceed. The operator can approve it in one command; I cannot approve it for them.
