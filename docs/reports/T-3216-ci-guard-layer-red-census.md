# T-3216 — Guard layer red on GitHub CI: census of the 26 red members

Measured 2026-09-29 by T-3211 round 4. Surfacing only: nothing here is a decision.

## Consequence (why this matters)

- **Doc Lint: 0 green runs in the last 300** (`gh run list --workflow doc-lint.yml --limit 300`, 0 with `conclusion=success`).
- **v0.12.0 was never released.** `release.yml` run for tag v0.12.0 (2026-09-24T19:39Z): `Workspace test suite + guard layer: failure` → `build-linux`, `build-macos`, `release`: **skipped**. `gh release list` → latest is **v0.11.2 (2026-06-24)**. So `install.sh` and the Homebrew formula still ship v0.11.2, three months behind.
- The gate did its job (T-2686: a red suite blocks the build). What failed is that nothing *read* it: the mirror canary (T-1140/T-1696) checks that the tag reached GitHub, not that a release was published. The tag is there, so the canary is quiet. This is the G-069 shipped≠live class, one layer up.
- Context: T-3086 (`a2b4c3194`, 2026-09-24 20:50 local) ran the guard layer locally, found 7 FIRING, and wrote "NOT tagging v0.12.0 ... Tag is a sovereign call." The tag was pushed about 49 minutes later. Whether the operator knows the release did not publish has not been established.

## Method

1. CI reasons: `gh run view 36533229835 --log-failed` (Doc Lint, 2026-09-29 06:50Z). The runner prints 7 lines per member, so some failing assertions are hidden.
2. Local rc: each member run on the origin host with its `# guard-layer: source` args and `CI` unset.
3. CI reproduction: fresh `git clone --depth 1 file:///opt/termlink` (the `actions/checkout@v4` default; no workflow sets `fetch-depth`), then each locally-green member run there with `CI=true` and `PATH=/usr/bin:/bin` (no `termlink` binary). This reproduced 13 of the 18 CI-only failures.

## Census

| member | CI | local rc | clone rc | class | evidence |
|---|---|---|---|---|---|
| check-arc-claim-drift.sh | FAIL | 1 | – | TREE-finding | arc-003 prover exits non-zero; arc-004 unbound |
| check-audit-warning-acknowledgement.sh | FAIL | 1 | – | TREE-finding | local: "FIRING — 3 unexamined warning(s)" |
| check-go-propagation.sh | FAIL | 1 | – | TREE-finding | 9 GO inceptions firing (T-2971, T-3007, T-3011, T-3012 …) |
| check-pickup-deferred-freshness.sh | FAIL | 1 | – | TREE-finding | P-078 STRANDED, no breadcrumb |
| check-receiver-ack-lag.sh | ERROR | 1 | – | TREE-finding | fires locally; CI side ERROR is no-hub |
| check-unpaired-capture.sh | FAIL | 1 | – | TREE-finding | 1 active unpaired capture |
| runme-fixtures.sh | FAIL | 1 | – | TREE-finding | local 17/18: "summary counts skips" (CI: 4 fail) |
| voi-prompt.sh | FAIL | 1 | – | TREE-finding | T-3200 needs a voi_score decision |
| arc-slice-drift-fixtures.sh | FAIL | 0 | 1 | ENV-git-depth | "could not extract the historical arc from git" |
| run-record-parse-fixtures.sh | FAIL | 0 | 1 | ENV-git-depth | "could not extract the historical record" |
| check-vendor-divergence.sh | ERROR | 0 | 2 | ENV-git-depth | "git log failed for baseline 8c1cca561" |
| bvp-auto-confirm-fixtures.sh | FAIL | 0 | 1 | ENV-git-depth | "no ref in HEAD..HEAD~6 carries the pre-ruling gate" |
| bvp-no-signal-ranking-fixtures.sh | FAIL | 0 | 1 | ENV-git-depth | "no ref in HEAD..HEAD~15 predates the change" |
| bvp-target-blast-radius-fixtures.sh | FAIL | 0 | 1 | ENV-git-depth | "no revision in the last 25 shows the pre-change split" |
| handover-staleness-check-fixtures.sh | FAIL | 0 | 1 | ENV-git-depth | "case 12: real pre-fix corpus available" |
| handover-suggested-action-fixtures.sh | ERROR | 0 | 2 | ENV-git-depth | "could not read 35affce76:…/handover.sh" |
| safe-commands-checkpoint-fixtures.sh | FAIL | 0 | 1 | ENV-git-depth | "could not extract pre-fix safe-commands.sh from dc65976e2" |
| safe-commands-write-pattern-fixtures.sh | FAIL | 0 | 1 | ENV-git-depth | "no ref in HEAD..HEAD~5 carries the pre-fix redirect rule" |
| artifact-cli-fixtures.sh | ERROR | 0 | 2 | ENV-no-binary-or-hub | "no termlink binary found" |
| check-fleet-recipient-agreement.sh | ERROR | 0 | 2 | ENV-no-binary-or-hub | "could not read fleet status — no verdict" |
| check-installed-binary-drift.sh | ERROR | 0 | 0 | ENV-no-binary-or-hub | probes install paths; runner has none (not reproducible here, since this host has them) |
| cron-drift-firing-fixtures.sh | FAIL | 0 | 0 | ENV-host-state | case 9 read live `/etc/cron.d`. **Fixed by T-3008 (`3d0b93598`)**: clone rc 0 under `CI=true` |
| fabric-workflow-link.sh | ERROR | 0 | 2 | ENV-host-state | `.agentic-framework/.context/designer/projects` missing (not in a checkout) |
| l387-boundary-fixtures.sh | FAIL | 0 | 0 | ENV-host-state | runner ignores SIGPIPE: `echo: write error: Broken pipe`, rc 1 instead of 141 |
| hook-telemetry-race-fixtures.sh | FAIL | 0 | 0 | ENV-hardcoded-path | `FW_LIB` defaults to `/opt/termlink/.agentic-framework/lib/hook-telemetry.sh` (line 17) |
| planted-default-gate-fixtures.sh | FAIL | 0 | 0 | UNEXPLAINED | failing assertion hidden by CI's 7-line cap; not reproduced by the clone |

**Totals:** TREE-finding 8 · ENV-git-depth 10 · ENV-no-binary-or-hub 3 · ENV-host-state 3 (1 already fixed) · ENV-hardcoded-path 1 · UNEXPLAINED 1 = **26**.

So **18 of 26 are artifacts of the CI environment**, and 10 of those share one cause. The 8 TREE findings are real and fire on the origin host too. They are the same class T-3086 counted as "7 FIRING" on 09-24.

## Remediation options (not chosen)

1. **`fetch-depth: 0`** on the guard-layer checkouts in `doc-lint.yml` and `release.yml`'s `test` job. One config line per checkout, clears 10 members, costs a full-history clone per run.
2. **Make the 7 environment-dependent members CI-aware**, as T-3008 did: skip with a printed reason when `CI` is set and the prerequisite (binary, hub, designer dir, install paths) is absent. Fix `hook-telemetry-race`'s hard-coded default to `$(git rev-parse --show-toplevel)`. Give `l387-boundary` a SIGPIPE-disposition precondition (`trap - PIPE` in the subshell, or skip when SIGPIPE is ignored).
3. **The 8 TREE findings**: resolve each, or acknowledge it in its own allowlist with a cited reason. Several are sovereign-shaped (GO-propagation on human-decided inceptions, voi_score on T-3200).
4. **Release**: re-run `release.yml` for v0.12.0 once the job is green; or cut v0.12.x; or make the guard layer non-blocking for release. The last option reverses T-2686's decision and is the operator's call.
5. **Observability**: a check or canary that fires when the latest tag has no published GitHub release, or when Doc Lint has had no green run in N days. Either would have caught this before 300 red runs accumulated.
6. **UNEXPLAINED (planted-default-gate)**: raise the runner's per-member line cap, or run that member alone in CI, to see the hidden assertion.

Options 1 and 2 are mechanical and only restore the declared hermetic contract. Options 3–5 involve scope, release, or gate-policy decisions and belong to the operator.
