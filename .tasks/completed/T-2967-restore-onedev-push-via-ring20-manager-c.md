---
id: T-2967
name: "Restore OneDev push via ring20-manager credential service"
description: >
  git push origin main has failed across four sessions (Authentication required, now 403) leaving 25 commits local-only. The operator directs that credentials are handled by the ring20-manager service rather than locally. Contact that peer over the TermLink rail, obtain or have applied the OneDev credential, and confirm the push lands.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
# demo_target: true               # T-2286: optional — marks task as reserved for an orchestrated demo
#                                 # worker (e.g. arc-010 HM-A dispatches via mcp__fw__work_on). When set,
#                                 # `fw work-on T-XXX` refuses unless --i-am-demo-orchestrator (CLI) or
#                                 # FW_I_AM_DEMO_ORCHESTRATOR=1 (env) is passed. Prevents the parent
#                                 # session from consuming the captured→started-work transition the demo
#                                 # worker expects to drive. Origin OBS-057.
created: 2026-09-16T18:45:32Z
last_update: 2026-09-16T19:48:32Z
date_finished: 2026-09-16T19:48:32Z
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── BVP scoring fields (T-1918, arc-006). See docs/reports/T-1915-bvp-inception.md for semantics. ──
# bvp_scores:                     # confirmed per-driver scores 0-5, set by `fw bvp confirm` (T-1924).
#                                 # Sovereignty boundary — only set after human or agent confirmation.
#                                 # Shape: {D1: <int 0-5>, D2: <int 0-5>, D3: <int 0-5>, D4: <int 0-5>, [<free-driver-id>: <int>]...}
# bvp_scores_proposed:            # estimator-proposed scores (T-1922 worker). Persists when ≥2 delta
#                                 # from bvp_scores: on any driver (M3 v2-delta). Shape: list of timestamped entries.
# cost_estimate:                  # F8 composite: 0.6×blast_radius + 0.3×tier + 0.1×effort.
#                                 # Q2 fallback: T-shirt S/M/L/XL mapped to 2/4/6/8 when blast_radius is not yet computable.
---

# T-2967: Restore OneDev push via ring20-manager credential service

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] **The failure is characterised before asking anyone for anything.** The remote URL, the auth method actually in use, and the exact server response are recorded. `Authentication required` and `403` are different failures — the first is "no credential offered", the second is "credential offered and refused" — and asking the credential service to fix the wrong one wastes a round trip on a peer.
- [x] **Ring20 Manager is contacted over the TermLink rail, with the request stated in terms it can act on.** The peer is located via presence (`agent-listeners-fleet`), its LIVE/armed state recorded, and a DM sent naming the repo, the remote, the observed error, and precisely what is being asked for. If the peer is not reachable the fact is recorded and the request is left durably on the rail rather than assumed delivered — a send that returns ok is not a receipt.
- [x] **No credential is invented, guessed, or written locally by this agent.** Whatever the service returns is applied through the mechanism it specifies. If the resolution requires a secret to be placed on this host, that is recorded as the human/service action it is, not performed by fabricating a token or editing git config with a value this agent chose.
- [x] **The push is confirmed by observation, not by the absence of an error.** `git rev-list --count origin/main..HEAD` is 0 afterwards, verified against the remote ref rather than inferred from `git push` printing nothing alarming. If the push still fails, the new server response is recorded and the task reports blocked with that evidence rather than closing.

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.

     ── Prefix routing (T-1811, T-1878): default to [REVIEWER] if Expected is grep-able ──
     If your Expected clause is grep-able / file-exists / structural (a deterministic
     shell check), prefer [REVIEWER] — that AC should be an Agent AC with the reviewer
     command in `## Verification` instead of a Human AC here. Only keep [REVIEW] if
     verification genuinely needs human taste (tone, feel, layout rhythm).
     See CLAUDE.md §AC Classification Guidance for the conversion rule.

     [REVIEW] example (genuine human judgment):
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error

     [REVIEWER] example (static-scan-verifiable — convert to Agent AC + Verification):
       - [ ] [REVIEWER] Block message names both bypass mechanisms
         **Steps:**
         1. Run `bin/fw reviewer T-XXX`
         **Expected:** Verdict: PASS; no findings on `block-message-completeness`
         **If not:** Inspect hook block-message string and add missing mechanism
       Conversion: this AC should be moved to ### Agent and
       `bin/fw reviewer T-XXX > /tmp/.rev 2>&1 && grep -q "Overall:.*PASS" /tmp/.rev`
       added to ## Verification. NEVER `... 2>&1 | grep -q ...` — that is the shape the
       Pipefail/SIGPIPE section below forbids, and this line used to prescribe it.
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
#
# ── Pipefail/SIGPIPE: grepping a command's output (L-387, T-2090, T-2743, T-2738) ──
#
# THE DEFAULT — redirect to a file, then grep the file:
#     cmd > /tmp/.out 2>&1 && grep -q "PATTERN" /tmp/.out
#     curl -sf "$(bin/fw watchtower url)/page" -o /tmp/.out && grep -q "PAT" /tmp/.out
# Correct at any output size, and `&&` keeps the PRODUCING command's exit code in
# the verdict. Reach for this first; the alternative below is the special case.
#
# NEVER `cmd | grep -q PAT` (L-387) — why: P-011 runs each line under `set -eo
# pipefail`. When grep matches it exits and closes stdin while cmd is still
# writing, cmd takes SIGPIPE, the pipeline exits 141 — verification "fails" with
# the pattern present. Captured 4× (T-1716, T-1838, T-1862, T-1863).
#
# THE EXCEPTION — capture first, grep the capture:
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"
# Valid ONLY while "$out" fits the 65536-byte pipe buffer, and it is on you to
# know that it does. Above that the form inverts and becomes the very failure
# L-387 describes: echo blocks on the full pipe, grep -q exits, echo takes
# SIGPIPE, rc=141 (T-2743 — measured on a 146,366-byte Watchtower page, 3/3 runs,
# deterministic not racy; rendered routes run 50-200KB, so anything that curls a
# page is over the line). It also discards cmd's exit code, so a 404 yields an
# empty capture that grep merely fails to match rather than a failed line.
# If you do use it: single pipe only, no intermediate tail/awk/sed stage between
# capture and grep (T-2090) — the middle stage is what `grep -q` slams its stdin
# on, and grep scans the whole captured string anyway, so the `tail -3` was
# cosmetic. `echo "$out" | grep -q PAT`, nothing between.
#
# TEST RUNNERS need a guard either way (T-2738). `set -e` is suppressed inside the
# `if` condition the gate runs each line in, so in `cmd1; cmd2` only cmd2 is the
# verdict — and the pass marker you grep for survives a partial failure: a suite
# printing "3 failed, 9 passed" satisfies `grep -q "9 passed"`, and generalising
# to `grep -qE "[0-9]+ passed"` matches the same output. Keep the exit code:
#     python3 -m pytest <file> -q > /tmp/.out 2>&1 && grep -q passed /tmp/.out
# or add the guard the exit code used to supply:
#     out=$(python3 -m pytest <file> -q 2>&1); echo "$out" | grep -q passed && ! echo "$out" | grep -q failed
#     out=$(bats <file> 2>&1); echo "$out" | grep -q '^ok 1 ' && ! echo "$out" | grep -q '^not ok'
# The close gate refuses the unguarded form. Bypass: FW_ALLOW_UNJUDGED_TEST_RUN=1.
#
# REHEARSING A LINE BY HAND DOES NOT REHEARSE THE GATE (T-2743). Your interactive
# shell has no `set -eo pipefail`. A line has returned 0 by hand and 141 under
# P-011, from the same directory, the same second. To rehearse for real:
#     bash -c 'set -eo pipefail; <your verification line>'
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

# T-2967 verification.
# Line 1 proves the credential actually works against the REMOTE (AC1 + AC4): ls-remote
# authenticates and returns a sha for refs/heads/main. Deliberately not `git push` printing
# nothing alarming — that is the inference AC4 forbids.
# Line 2 proves local HEAD is not ahead of the pushed ref.
# Line 3 proves the request to ring20-management-agent is DURABLE on the rail and readable
# back, not merely that a send returned ok (AC2 — "a send that returns ok is not a receipt").
# Line 4 proves the incidental .122 finding was captured as its own task rather than dropped.

GIT_TERMINAL_PROMPT=0 git ls-remote origin -h refs/heads/main > /tmp/.t2967-remote.txt 2>&1 && grep -qE '^[0-9a-f]{40}[[:space:]]+refs/heads/main$' /tmp/.t2967-remote.txt
test "$(git rev-list --count origin/main..HEAD)" = "0"
timeout 60 termlink channel subscribe "dm:termlink-107-landing:ring20-management-agent" --hub 192.168.10.122:9100 --cursor 0 --limit 1 --json > /tmp/.t2967-dm.ndjson 2>&1 && python3 -c "import json,base64,sys; ls=[l for l in open('/tmp/.t2967-dm.ndjson') if l.strip()]; b=base64.b64decode(json.loads(ls[0])['payload_b64']).decode('utf-8','replace'); sys.exit(0 if 'termlink-107-landing' in b and 'T-2967' in b else 1)"
test -f .tasks/active/T-2970-ring20-management-122-agent-presence-is-.md

## RCA

<!-- REQUIRED for bug-class tasks (workflow_type=build with bug-tag, OR title matches
     fix/bug/rca/broken/crash/error/regression/fail/hotfix).
     Non-bug-class tasks may leave this section empty or remove it.

     For bug-class, fill in:
       **Symptom:** what was observed (the user-facing manifestation).
       **Root cause:** the specific structural/logical gap — not "the code was wrong".
       **Why structurally allowed:** what in the framework/code/tooling let this go undetected.
       **Prevention:** what catches the next instance (test/lint/gate/doc/learning) — distinct from the fix itself.

     The completion gate (T-1550, G-019) blocks --status work-completed when
     bug-class AND this section is empty/template-only. Use --skip-rca to bypass (logged).
-->

**Symptom:** `git push origin` refused from workstation-107 for four sessions. Reported as a
credential failure; the human asked for the Ring20 Manager credential service to be engaged.

**Root cause — the diagnosis was wrong, and that is the finding.** The credential was never
the problem. A LOCAL pre-push gate (T-1599/T-1610) was rejecting every push because
`.context/project/decisions.yaml` failed to parse (repaired under T-2968). The remote was
never contacted. Once the YAML parsed, the stored credential authenticated on the first
attempt and all 26 commits landed. The earlier `403` / `Authentication required` observations
came from unrelated attempts and were carried forward as if they described the current
failure.

**Why structurally allowed:** the pre-push gate reports the file it rejected, but the operator
sees only "push failed" plus whatever the previous attempt printed. Nothing forced the
question "has the remote actually been reached?" before escalating to a peer. AC1 of this task
exists precisely to force it, and it is what resolved the task: characterising the failure cost
one command and removed the need for the service entirely.

**AC2 outcome — contacted, not delivered.** `ring20-management-agent` could NOT be located via
presence: `.122`'s agent-presence is unreadable (`channel.subscribe` wedges at 30s while
`channel.list` is fast — filed as **T-2970**). The identity was recovered from `channel list`
instead. The request was left DURABLY on the rail at
`dm:termlink-107-landing:ring20-management-agent` on 192.168.10.122:9100, offset 0, and read
back to confirm (1954 bytes). The post returned `"confirmed": false,
"status": "delivered-unconfirmed"` — recorded as such, because the hub accepting a message is
not a recipient receipt.

**AC3 outcome:** no credential was invented, guessed, written, or altered. `credential.helper`
remains `store`; `~/.git-credentials` (mode 600) was read only to enumerate HOSTS, with secrets
redacted, and was not modified. The one open credential question — whether the OneDev token is
long-lived or lapses — is stated in the durable message as a question for the service, not
resolved locally.

**Prevention:** none claimed beyond AC1 already being the right gate and having worked. The
durable-message pattern (post + read back + record `confirmed:false`) is the honest shape for
an unreachable peer and is reused from the existing awaiting-ack convention.

## Evolution

<!-- REQUIRED for arc-tagged build tasks (tags include arc:*). Captures how
     understanding evolved during build — what was learned that wasn't known at
     filing, what in the original plan no longer fits, what triggered pivots
     or new sub-tasks. Mandatory at slice boundaries (when applicable) and
     before --status work-completed.

     Origin: T-1717 grill Q4 — "the understanding of what we need and want
     evolves with the process of materialisation." Structural counter to §ACD:
     spec-vs-build divergence is logged as soon as it happens, not lost as
     folklore.

     Format (one entry per slice boundary or significant insight):
       ### YYYY-MM-DD — [topic]
       - **What changed:** [what we learned that we didn't know at filing]
       - **Plan impact:** [what in the plan no longer fits]
       - **Triggered:** [new sub-task / pivot / scope cut, with task ID if filed]

     The completion gate (T-1718) blocks --status work-completed when this
     section exists but is empty/template-only. Use --skip-evolution to bypass
     (logged Tier-2). Non-arc tasks may leave this empty.
-->

## Recommendation

<!-- T-2945: same shape as inception.md's block — the gate that reads it
     (audit_inception_recommendation, lib/task-audit.sh:117) is shared, so the
     shape is copied rather than reinvented.

     REQUIRED once this task reaches partial-complete: Agent ACs done, at least
     one `### Human` AC still unticked. `lib/review.sh:205-211` (T-2421) BLOCKS
     `fw task review` emission for build/refactor/test/decommission tasks in that
     state with no substantive block here — the operator would otherwise open
     /review/<id> to a blank Recommendation card and be asked to approve a form.

     Not required while every Human AC is ticked or the task has none: the gate
     only fires on the partial-complete transition. It is here from the start so
     you write it while you still have the evidence, not when the gate refuses.

     Format (the parser wants the `**Recommendation:**` line at the start of a
     line; a leading `-` or `*` bullet is also accepted):
     **Recommendation:** GO / NO-GO / DEFER
     **Rationale:** Why (cite evidence — what shipped, what was proven, what remains)
     **Evidence:**
     - Finding 1
     - Finding 2

     DEFER is for evidence gaps, not confidence gaps (CLAUDE.md §Presenting Work
     for Human Review). If the artefact is complete and you still don't want to
     commit, that is a calibration failure — recommend GO or NO-GO.
-->

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-09-16T18:45:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2967-restore-onedev-push-via-ring20-manager-c.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-7cea391b
- **Timestamp:** 2026-09-16T19:48:34Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-09-16T19:48:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
