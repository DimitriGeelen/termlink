---
id: T-2929
name: "85 guard scripts carry no guard-layer marker and nothing runs them — classify
  the family and give the deploy-time bucket a runner"
description: >
  run-guard-layer.sh reports 85 scripts/check-*.sh and tests/*.sh as unclassified
  (no '# guard-layer:' marker), so it never runs them. T-2928 measured one of them,
  check-pickup-cron-lock.sh, returning exit 1 against the live host and correctly
  naming two unflocked pickup cron lines, while four duplicate task files were created
  and no surface reported anything. That is the T-2683 shape (static checks nothing
  ran) recurring inside the guard layer built to end it: existence mistaken for enforcement.
  Wiring that one script is the cheap answer and the wrong scope. It is one of 85;
  it reads /etc/cron.d so 'guard-layer: source' is wrong by the documented contract
  (no host state); and its sibling check-cron-install-drift.sh is deliberately ad-hoc-only
  to avoid a recursive canary-detecting-uninstalled-canaries. Classify first, wire
  second.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [scripts/check-guard-runner-coverage.sh, tests/guard-runner-coverage-fixtures.sh]
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
created: 2026-09-08T19:58:17Z
last_update: 2026-09-08T21:46:11Z
date_finished: 2026-09-08T21:46:11Z
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
bvp_scores_proposed:
  - ts: '2026-09-08T19:59:31Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 3
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=3 
      (body:component-discoverability); D4=2 (body:env-class-handled); 
      F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-08T21:28:14Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 3
      D4: 2
      F-RECALL: 1
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=3 
      (body:component-discoverability); D4=2 (body:env-class-handled); 
      F-RECALL=1 (body:episodic-only); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-08T21:28:27Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=255,acs=7)
    rubric_sha: e4a00f38e801
---

# T-2929: 85 guard scripts carry no guard-layer marker and nothing runs them — classify the family and give the deploy-time bucket a runner

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Measurement (session S-2026-0908-1948)

**"85 unclassified" is not "85 unrun", and the difference matters.** Cross-referencing
each of the 85 against every runner — installed `/etc/cron.d`, git-tracked
`.context/cron/`, `.github/workflows/`, and any calling script in `scripts/`,
`tests/`, `bin/`:

| class | n | meaning |
|---|---|---|
| COVERED | 36 | invoked by an installed crontab or CI — unclassified but not unrun |
| DORMANT | 46 | no runner anywhere |
| DORMANT (fixtures-only) | 2 | referenced only by their own fixture suite — `check-dashboard-deleted-root.sh`, `check-pickup-cron-lock.sh` |
| SHIPPED-DARK | 1 | `check-substrate-smoke-freshness.sh` — crontab committed (T-2696), not yet installed; already owned by that task's human AC |

So the honest headline is **48 dormant**, not 85. CI runs exactly two things
(`run-guard-layer.sh`, `cargo test --workspace`) — there is no glob over `tests/`
or `scripts/`, so nothing sweeps them up implicitly. There is no Makefile and no
`settings.json` hook referencing them.

**33 of the 48 were executed to see whether dormancy had hidden a real failure.**
Result: **31 green, 1 red, 1 hang.** No product regression was found, and that is
the finding — the dormant set is mostly healthy, so the cost of leaving it unrun is
latent risk rather than an accumulated backlog of breakage.

- `scripts/test-chat-arc-broadcast.sh` — rc 1 (4 pass / 2 fail). **Its own defect,
  not the product's.** T5/T6 capture with `2>&1` and feed the result to `jq`; the
  broadcast script writes a `skipping duplicate <hub>` diagnostic to **stderr**, so
  the merge corrupts the JSON the test then fails to parse. Re-run with stderr
  separated, the envelope is well-formed. Worth stating plainly because the
  tempting read — "a dormant test is red, therefore something regressed" — is wrong
  here, and reporting it that way would have manufactured a defect.
- `scripts/test-fleet-adoption-snapshot.sh` — rc 124, hangs at T6 under a 60 s
  timeout.

**The disposition cannot be decided by grep, and this task proved that on itself.**
The hermeticity screen used "no direct reference to a live hub" and got two of them
wrong: `test-chat-arc-broadcast.sh` reaches live hubs *through*
`scripts/chat-arc-broadcast.sh`, and running it **posted test broadcasts to three
live fleet hubs** (127.0.0.1, .121, .122) — a real side effect, triggered here, on
a script the screen had called safe. `test-fleet-adoption-snapshot.sh` hangs for
the same class of reason. That is precisely why the deliverable is a per-script
disposition table and not a bulk `# guard-layer: source` sweep: marking these
`source` would put live-fleet writes and a hang into every push and PR.

## Disposition (AC5 deliverable — session S-2026-0908-2240)

**The count moved from 48 to 57, and the tree did not change — the definition tightened.**
Last session's 48 counted a script as covered if *anything* referenced it. Two of those
references do not constitute coverage, and AC4 names both:

1. **A fixture reference is not coverage.** `tests/*fixtures*.sh` are guard-layer members,
   so a check they invoke looks "run on every push" — but it is run against a *synthetic
   fixture tree*, never against this repository. That is precisely how
   `check-pickup-cron-lock.sh` read as covered for a day while the defect it guards fired
   four times (T-2928). Fixture-only went 2 → 5.
2. **A dormant caller does not rescue its callee.** Verified concretely:
   `agent-send-idle-gate.sh`'s only caller is `scripts/lib-idle-gate.sh`, which is itself
   unmarked and unrun. Nine scripts moved out of COVERED on this rule alone.

So **48 was an undercount**, and the honest figure under the stricter definition is 57.

**A third class was found by running the check against the tree, and it is not dormant.**
`check-outbox.sh` backs the `/check-outbox` slash command. A human running it on demand is a
real invocation path — just not a scheduled one — so it is reported as OPERATOR-INVOKED and
does not fire. Without that class the check would have made a false claim about a script that
is deliberately operator-driven. `SHIPPED-DARK` is likewise reported and non-firing: it is the
T-2561/T-2682 install-drift class, already owned by `check-cron-install-drift.sh`, and firing
on it here would double-own the remediation.

**Basis for each row.** A script's own header is a far better signal than any grep, and 12 of
them state it outright ("hermetic", "no live hub, no live PTY"). Where a header declares
nothing and no live-hub or host-state signal appears, the row is marked **`guard-layer?`** —
provisional, needing a read before the marker is added. Twenty-seven rows are in that state
and are deliberately *not* presented as settled: this task's own measurement showed the
hermeticity screen was wrong twice, once posting real broadcasts to three live fleet hubs.

**Wiring is out of scope by AC5**, and the provisional column is the reason it must stay so.

| script | dir | disposition | basis |
|---|---|---|---|
| `agent-listeners-cv-read.sh` | tests/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `agent-listeners-identity-fp.sh` | tests/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `agent-send-idle-gate.sh` | tests/ | **guard-layer** | header self-declares hermetic (no live hub / no live PTY) |
| `check-addressed-posts.sh` | scripts/ | **deploy-time** | reaches a live hub or the network — must NOT join the push/PR layer |
| `check-approval-surfaces.sh` | scripts/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `check-dashboard-deleted-root.sh` | scripts/ | **deploy-time** | reads host state (/etc, /proc, systemctl, crontab) |
| `check-guard-runner-coverage.sh` | scripts/ | **deploy-time** | reads /etc/cron.d — in CI every cron-covered script would read dormant. Sibling of check-cron-install-drift.sh. |
| `check-pickup-cron-lock.sh` | scripts/ | **deploy-time** | reads host state (/etc, /proc, systemctl, crontab) |
| `fleet-capability-canary.sh` | tests/ | **deploy-time** | reaches a live hub or the network — must NOT join the push/PR layer |
| `mutate-2783.sh` | tests/ | **guard-layer** | mutation-proof harness; drives named tests, no host state |
| `relay-b1-doorbell-rail.sh` | tests/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `relay-b2-send-hops.sh` | tests/ | **guard-layer** | header self-declares hermetic (no live hub / no live PTY) |
| `relay-b3-hop-budget.sh` | tests/ | **guard-layer** | header self-declares hermetic (no live hub / no live PTY) |
| `relay-wake-confirm.sh` | tests/ | **guard-layer** | header self-declares hermetic (no live hub / no live PTY) |
| `stale-waker-code-canary.sh` | tests/ | **guard-layer** | header self-declares hermetic (no live hub / no live PTY) |
| `test-agent-chat-arc-recent.sh` | scripts/ | **deploy-time** | reaches a live hub or the network — must NOT join the push/PR layer |
| `test-agent-conversation-list.sh` | scripts/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `test-agent-conversation-selftest.sh` | scripts/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `test-agent-conversation-status.sh` | scripts/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `test-agent-listeners-fleet.sh` | scripts/ | **deploy-time** | reaches a live hub or the network — must NOT join the push/PR layer |
| `test-agent-listeners.sh` | scripts/ | **deploy-time** | reaches a live hub or the network — must NOT join the push/PR layer |
| `test-agent-send-auto-discover.sh` | scripts/ | **deploy-time** | reaches a live hub or the network — must NOT join the push/PR layer |
| `test-agent-send-orchestration.sh` | scripts/ | **deploy-time** | reaches a live hub or the network — must NOT join the push/PR layer |
| `test-agent-send-transport.sh` | scripts/ | **deploy-time** | reaches a live hub or the network — must NOT join the push/PR layer |
| `test-arc-live-probe.sh` | scripts/ | **deploy-time** | reaches a live hub or the network — must NOT join the push/PR layer |
| `test-be-reachable.sh` | scripts/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `test-charter-drift-freshness.sh` | scripts/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `test-charter-sentence-drift.sh` | scripts/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `test-chat-arc-broadcast.sh` | scripts/ | **deploy-time** | QUARANTINE — RED by its own `2>&1` capture (corrupts the JSON it feeds to jq) AND posts to LIVE hubs. Fix the test before wiring. |
| `test-check-fleet-binary-freshness.sh` | scripts/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `test-check-fleet-doorbell-mail-health.sh` | scripts/ | **deploy-time** | reads host state (/etc, /proc, systemctl, crontab) |
| `test-check-framework-pickup-freshness.sh` | scripts/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `test-check-unconfirmed-delivery-freshness.sh` | scripts/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `test-comms-selftest.sh` | scripts/ | **guard-layer** | header self-declares hermetic (no live hub / no live PTY) |
| `test-diagnose-unconsumed.sh` | scripts/ | **guard-layer** | header self-declares hermetic (no live hub / no live PTY) |
| `test-fleet-adoption-snapshot.sh` | scripts/ | **deploy-time** | QUARANTINE — hangs (rc 124 at T6 under a 60s bound). Diagnose before wiring. |
| `test-fleet-rearm-wakers.sh` | scripts/ | **guard-layer** | header self-declares hermetic (no live hub / no live PTY) |
| `test-journal-mirror.sh` | scripts/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `test-journal-reaper.sh` | scripts/ | **deploy-time** | reaches a live hub or the network — must NOT join the push/PR layer |
| `test-listener-heartbeat.sh` | scripts/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `test-mcp-desc-budget.sh` | scripts/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `test-notify-sidecar.sh` | scripts/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `test-pushwaker-ready-loop.sh` | scripts/ | **guard-layer** | header self-declares hermetic (no live hub / no live PTY) |
| `test-pushwaker-reap.sh` | scripts/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `test-sidecar-auto-confirm.sh` | scripts/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `test-substrate-preflight.sh` | scripts/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `test-tl-claude-cmd.sh` | scripts/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `test-watchtower-guard.sh` | scripts/ | **deploy-time** | reaches a live hub or the network — must NOT join the push/PR layer |
| `test_g052_inception_decision_gate.sh` | tests/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `test_g054_completion_smoke.sh` | tests/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `test_pl152_counter_arity_static.sh` | tests/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `test_t1619_metrics_trend_smoke.sh` | tests/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `test_t1628_task_verify_flags.sh` | tests/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `test_t1640_pgrep_self_match.sh` | tests/ | **deploy-time** | reads host state (/etc, /proc, systemctl, crontab) |
| `test_tl_dispatch_meta.sh` | tests/ | **guard-layer?** | no live-hub or host-state signal, but no self-declaration either — PROVISIONAL, read before adding the marker |
| `tl-claude-identity-binding.sh` | tests/ | **guard-layer** | header self-declares hermetic (no live hub / no live PTY) |
| `wake-confirm-reply-match.sh` | tests/ | **guard-layer** | header self-declares hermetic (no live hub / no live PTY) |

**Totals:** deploy-time = 18 · guard-layer = 12 · guard-layer? = 27 · **57 dormant**

**Two are quarantined, not merely dormant** — `test-chat-arc-broadcast.sh` is RED by its own
`2>&1` capture *and* posts to live hubs; `test-fleet-adoption-snapshot.sh` hangs. Adding the
marker to either would put a live-fleet write and a hang into every push and PR.

**The check flags itself.** `check-guard-runner-coverage.sh` appears in its own dormant list,
correctly: nothing runs it yet, and its own disposition is `deploy-time` because it reads
`/etc/cron.d`. In CI no termlink crontab is installed, so every cron-covered script would read
dormant and it would fire on every push. It is the sibling of `check-cron-install-drift.sh`,
unmarked for the identical reason.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] **The 85 are cross-referenced against every runner, and the truly-dormant subset is named.** For each unclassified script, record whether it is invoked by (a) an *installed* crontab under `/etc/cron.d`, (b) a git-tracked crontab under `.context/cron/` that is not yet installed, (c) a CI workflow, or (d) any other script. A script matched by none of these is dormant. The dormant count is stated as a number with the list, because "85 unclassified" is not the same claim as "85 unrun" and conflating them would overstate the finding.
- [x] **The measurement distinguishes shipped-but-dark from genuinely-unowned.** A check whose crontab exists in git but is absent from `/etc/cron.d` is a *different* defect (the T-2561/T-2682 install-drift class, already owned by `check-cron-install-drift.sh`) from a check no crontab has ever referenced. Both are reported, separately labelled, so remediation is not misrouted.
- [x] **A repeatable script produces the classification, not a one-off shell session.** `scripts/check-guard-runner-coverage.sh` emits the per-script disposition and exits 0 when every unclassified script has a runner, 1 when any is dormant, 2 on tooling error (fail-closed: an empty script inventory is an error, never a vacuous clean — the T-2747 zero-tools lesson). `--json` for scripting; test seams for the scripts dir, cron dirs and CI dir so fixtures need no host state.
- [x] **The check is load-bearing, proven by mutation.** A fixture suite (`tests/guard-runner-coverage-fixtures.sh`) pins the firing cases and the two false-positive guards: a script referenced only by its own fixture suite must still count as dormant, and a script referenced only from a task file or handover must not count as covered (that is how `check-pickup-cron-lock.sh` looked "referenced" while nothing ran it). At least one mutant is recorded that turns the check red.
- [x] **The dormant set is triaged into dispositions, and no wiring is done under this task.** Each dormant script gets exactly one of: `cron` (runtime canary — names the crontab it needs), `deploy-time` (reads host state; needs a preflight-tier runner), `guard-layer` (hermetic; just missing the marker), or `retired` (superseded/dead). The disposition table is the deliverable. Wiring is deliberately out of scope: `run-guard-layer.sh` requires no host state, so the deploy-time bucket cannot simply be marked `source`, and picking its runner is a decision this task informs rather than pre-empts.

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

bash tests/guard-runner-coverage-fixtures.sh > /tmp/.t2929-fx.txt 2>&1 && grep -q "0 failed" /tmp/.t2929-fx.txt
bash scripts/check-guard-runner-coverage.sh > /tmp/.t2929-run.txt 2>&1 || true
grep -q "FIRING" /tmp/.t2929-run.txt
bash scripts/check-guard-runner-coverage.sh --json > /tmp/.t2929.json 2>&1 || true
python3 -c "import json; d=json.load(open('/tmp/.t2929.json')); assert d['ok'] is False; s=d['summary']; assert s['dormant']+s['dormant_fixtures_only']==len(d['firing']); print('json ok')"
E=$(mktemp -d); mkdir -p "$E/scripts" "$E/tests"; set +e; GUARD_COVERAGE_SCRIPTS_DIR="$E/scripts" GUARD_COVERAGE_TESTS_DIR="$E/tests" bash scripts/check-guard-runner-coverage.sh >/dev/null 2>&1; rc=$?; set -e; rm -rf "$E"; test "$rc" = "2"
bash -n scripts/check-guard-runner-coverage.sh && bash -n tests/guard-runner-coverage-fixtures.sh
test -z "$(grep -m1 "^# guard-layer:" scripts/check-guard-runner-coverage.sh)"
test "$(grep -cE '\*\*(deploy-time|guard-layer)' .tasks/active/T-2929-85-guard-scripts-carry-no-guard-layer-ma.md)" -eq 57

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

### 2026-09-08T19:58:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2929-85-guard-scripts-carry-no-guard-layer-ma.md
- **Context:** Initial task creation

### 2026-09-08T19:59:31Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

## Reviewer Verdict (v1.5)

- **Scan ID:** R-4a3ed034
- **Timestamp:** 2026-09-08T21:46:26Z
- **Catalogue:** v1.3-seed
- **Overall:** FAIL
- **Needs Human:** yes
- **Findings:** 4

**Per-AC findings:**

- **AC#1 (Agent)** — **The 85 are cross-referenced against every runner, and the truly-dormant subset is named.** For each unclassified script, record whether it is invoked by (a) an *installed* crontab under `/etc/cron.d
  - **AC-verify-mismatch** (narrow, heuristic) — `path=etc/cron.d in: **The 85 are cross-referenced against every runner, and the truly-dormant subset is named.** For each unclassified script, record whether it is invoke`
- **AC#2 (Agent)** — **The measurement distinguishes shipped-but-dark from genuinely-unowned.** A check whose crontab exists in git but is absent from `/etc/cron.d` is a *different* defect (the T-2561/T-2682 install-drift
  - **AC-verify-mismatch** (narrow, heuristic) — `path=etc/cron.d in: **The measurement distinguishes shipped-but-dark from genuinely-unowned.** A check whose crontab exists in git but is absent from `/etc/cron.d` is a *`

**Verification-level findings:**

  1. **swallowed-errors** (severe, deterministic) @ Verification:line 61
     - evidence: `bash scripts/check-guard-runner-coverage.sh > /tmp/.t2929-run.txt 2>&1 || true`
  2. **swallowed-errors** (severe, deterministic) @ Verification:line 63
     - evidence: `bash scripts/check-guard-runner-coverage.sh --json > /tmp/.t2929.json 2>&1 || true`

- **Layer-1 escalations:** 1
  1. **destructive-action** (high) — Destructive operation in verification or AC
     - matched: `rm -rf`

### 2026-09-08T21:46:11Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
