# T-3341 — Re-vendor rehearsal: AEF 1.6.29 → bleeding-edge 1.7.424

**Operator ruling C (2026-10-04):** rehearse before re-vendoring. The live repository was not touched.
**Rehearsal tree:** a plain `git clone /opt/termlink` at 0e3ac03ce in the session scratchpad (never a worktree).
**Source:** AEF `bleeding-edge` b6af1a14 (VERSION 1.7.424), cloned from GitHub, unshallowed.
**Command:** `env FRAMEWORK_ROOT=<aef> PROJECT_ROOT=<clone> <aef>/bin/fw upgrade <clone> --no-self-vendor`
(the T-2099 fork-bomb guard; `ulimit -u 2000`, `timeout 900`). Full logs kept in the scratchpad.

## 0 Version history

| Version | Date | Change | Task |
|---|---|---|---|
| 0.1 | 2026-10-04 | Rehearsal run and findings | T-3341 |

## 1 Before running: side effects outside the target

1.1 The ten steps were read first (`lib/upgrade.sh`). Step 5 only reads `$HOME/.claude/settings.json` to report
duplicate hooks ("we don't auto-remove user state"). The cron step regenerates the crontab source inside the
target only; deployment needs a separate `fw cron install`. No hub, network or push calls. No global
`core.hooksPath`.
1.2 Step **4c "Shim migration + global install sync"** was not in the step list I read; in the run it SKIPPED
(`/root/.local/bin/fw exists but is not a symlink or shim`). Nothing outside the clone was written.

## 2 What happened

2.1 First run: **REFUSED, foreign-source.** Our pinned `version_sha` 67aeacc7 was absent from a 400-commit
shallow clone. After unshallowing, 67aeacc7 is an ancestor of bleeding-edge, and the refusal cleared. No bypass
was used.
2.2 Second run: **stopped at step 4b**: "23 of 3465 vendored file(s) are invisible to git in the target",
because `.gitignore:37` is `.agentic-framework/*`. The upgrade's own advice: re-include
`.agentic-framework/.context`, `.secret-scan-allowlist`, `.secret-scan-patterns`.
2.3 Third run (fresh clone, those three re-includes added): **completed, PARTIAL**, 25 changes, 1 failed step
(step 5: 3 hooks it ships but did not install: `check-paid-backend`, `check-worktree-governance-write`,
`stop-driver.sh`). Version 1.6.29 → 1.7.424; project id minted `pid-29ae57c30503160c`.

## 3 Findings

### 3.1 `CLAUDE.md`: far smaller loss than feared
Step 1 kept the whole project-specific region above `## Core Principle`. Only **7 lines** were dropped, all in
the governance region: the stale 120K/150K/170K budget ladder and one task-template line, which our own
project note already declares wrong. The 844-line loss recorded in T-2015 does not recur at this version.

### 3.2 Step 7b overwrites TermLink's own comms toolkit with older copies — the blocking finding
AEF propagates the doorbell+mail toolkit (`lib/templates/{skills,scripts}`) to consumers, and this project is
that toolkit's **origin**. Step 7b replaced 15 of our files with AEF's older copies (`.bak` kept), with no flag to
skip it:

| File | Live lines | After upgrade | Changed lines | Last local change |
|---|---|---|---|---|
| scripts/agent-send.sh | 739 | 242 | 659 | 2026-10-03 (T-3325 circuit stamping, T-3331 exit fix) |
| scripts/agent-chat-arc-recent.sh | 677 | 333 | 386 | 2026-08-16 |
| scripts/agent-listeners.sh | 387 | 232 | 239 | 2026-08-10 |
| scripts/be-reachable.sh | 509 | 380 | 157 | 2026-07-03 |
| scripts/recent-dm.sh | 383 | 300 | 109 | 2026-07-02 |
| scripts/agent-listeners-fleet.sh | 331 | 263 | 92 | 2026-06-24 |
| scripts/listener-heartbeat.sh | 253 | 181 | 72 | 2026-08-09 |
| scripts/chat-arc-broadcast.sh | 205 | 167 | 58 | 2026-08-26 |
| scripts/agent-respond.sh | 138 | 119 | 57 | 2026-07-12 |
| scripts/agent-conversation-status.sh | 175 | 175 | 6 | 2026-05-28 |
| .claude/commands/check-arc.md | 456 | 175 | 327 | 2026-10-01 |
| .claude/commands/agent-handoff.md | 264 | 167 | 163 | 2026-06-28 |
| .claude/commands/pulse.md | 270 | 209 | 61 | 2026-05-30 |
| .claude/commands/peers.md | 173 | 160 | 17 | 2026-06-09 |
| .claude/commands/be-reachable.md | 121 | 117 | 4 | 2026-06-09 |

`to_circuit` (T-3325) appears in the live `agent-send.sh` and not in the upgraded one. Also created:
`scripts/agent-identity.sh`. Mitigation in a real re-vendor: restore these 15 files from git immediately after
step 7b, and file upstream that the toolkit's origin project must be excluded (or the templates refreshed from it).

### 3.3 `resume.md` is replaced by the template
The upgraded `.claude/commands/resume.md` lacks both local steps: the session-start alerts (T-3327, 3 mentions
live, 0 after) and the pending-commit step (T-3269, 3 live, 0 after). `tests/commit-pending-fixtures.sh` fails W1
(resume.md not wired) and W3 (handover.sh not wired). Mitigation: restore `resume.md`; re-apply the T-3269
handover hook (a registered divergence).

### 3.4 New hooks: a second receiver on our inbox
Step 5 adds `check-human-ac-tick` and four sidecar hooks (`sidecar-autostart` twice, `sidecar-inbox`,
`sidecar-receiver-adapter`, `sidecar-receiver-ready`): AEF's receiver would start in our sessions. TermLink's own
`notify-sidecar.sh` already reads this project's inbox, so two receivers would share one inbox and one read
marker: 055's N3 failure (one reader's ack hid ~10 messages for a day). This needs an explicit choice (OD-9:
whose receiver) before or at the re-vendor, not after.

### 3.5 The 26 registered divergences
3.5.1 Line test (do our added lines still exist in the upgraded file?): 2 carried, 1 partial, 21 "lost", 2 not
measurable. This **overstates** loss: upstream often fixed the same defect with different code.
3.5.2 Behaviour checks, where a test exists:

| Divergence | Behaviour after upgrade | Evidence |
|---|---|---|
| T-3337 reader never rebuilds | **carried** (upstream T-3786) | reader fixture 3/3 reader cases pass |
| T-3337 discovery page | **carried**, fixed at the caller ("never build from a page view") | discovery.py:350-354; our fixture's text check is too strict (false FAIL) |
| T-3336 reindex import path + exit 2 | **carried** | upgraded `bin/fw` sets `PYTHONPATH=$FRAMEWORK_ROOT`, exits 2 on unimportable |
| T-3336 `fw ask` sys.path | **carried** | upgraded `lib/ask.py:26-34` puts the framework root on sys.path |
| T-3336 parallel embedding (`FW_EMBED_WORKERS`) | **lost** (performance only, +53%) | no ThreadPoolExecutor in upgraded embeddings.py |
| T-3269 handover pending-commit wiring | **lost** | commit-pending fixture W3 |
| T-3327 resume alerts | **lost** | 3.3 |
| vector-index freshness canary | works | fixture 8/8 |
| divergence register check | works | fixture 10/10 |
| warn escalation filer | works | fixture 34/34 |
| framework tracking / dangling refs | works | fixtures 13/13, 20/20 |

3.5.3 The other 16 entries (bvp estimator and `lib/bvp.sh` work T-3176..T-3189, safe-commands T-3178/T-3187,
update-task T-3198, handover T-2882, check-active-task T-3270, pickup T-2687, budget-gate T-3127/T-2469,
`bin/fw` T-2866, checkpoint T-3316, audit T-2721, T-2304) have no behaviour test here; their exact lines are
absent, so each must be re-verified against 1.7.424 or re-applied. Several were filed upstream and may be fixed
differently.

### 3.6 Known hazards (`.vendor-divergence.yaml` DETECT lines)

| Hazard | Result in rehearsal |
|---|---|
| H1 commit exemption | **present**: `git commit` ALLOW, `fw git commit` and `.agentic-framework/bin/fw git commit` BLOCK (`check-active-task.sh:559-569`) |
| Mode drift (100755 not executable) | 0 |
| Dead exec targets | none |
| `.fw-secret-key` under vendored `.context` | absent; vendored `.context` holds only designer BPMN files. The `.gitignore` re-include must be narrowed to `.agentic-framework/.context/designer/` so a later-generated key can never be committed |
| `upstream_repo` wrapped | ok |

### 3.7 Guard layer: baseline vs rehearsal
3.7.1 Same runner, same host, two fresh clones: **baseline** (no upgrade, 78d59ec82) and **rehearsal** (upgraded).

| Run | Passed | Failed | Errored | Total |
|---|---|---|---|---|
| Baseline | 183 | 6 | 2 | 191 |
| Rehearsal | 154 | 31 | 6 | 191 |

3.7.2 **30 members turn red because of the upgrade** (red in rehearsal, not in baseline; 7 red in both; 1 red
only in baseline). After restoring the 15 toolkit files and `resume.md` in the rehearsal tree, **7 clear**
(the toolkit's own fixtures: listeners liveness, idle gate, chat-arc-recent, relay hops, wake-confirm,
tl-claude identity binding, and one more). **22 stay red:**
3.7.2.a lost local framework fixes (registered divergences): `budget-ladder-drift`, `check-budget-ladder-drift`,
`budget-status-session-key`, `checkpoint-verb`, `bvp-auto-confirm`, `bvp-derived-blast-radius`,
`bvp-no-signal-ranking`, `bvp-target-blast-radius`, `commit-pending`, `focus-drift-helper-exception`,
`handover-suggested-action`, `hook-telemetry-race` (timed out), `pickup-failopen`,
`safe-commands-write-pattern`, `reviewer-sovereignty`;
3.7.2.b upstream template, ignore and cron changes: `task-template-idioms` (fixture and check),
`verification-heading-shadow`, `gitignore-framework-scope`, `check-framework-tracking-drift`,
`cron-drift-firing` (the new `sidecar-sweep-5m` job);
3.7.2.c one false alarm: `vector-index-reader-no-rebuild` (text check too strict; behaviour carried, 3.5.2).

## 4 Recommendation

4.1 **NO-GO for a straight re-vendor.** It would turn 30 guard members red, overwrite 15 of TermLink's own
comms-toolkit files with older copies, replace `/resume`, start a second receiver on our inbox, and land the H1
commit-gate regression. Nothing here is unknown any more; it is a defined list of work.
4.2 **GO for a planned re-vendor as its own task**, in one session, with this sequence:
4.2.1 Before: decide OD-9 (whose receiver runs here) so the four AEF sidecar hooks are either kept and
`notify-sidecar.sh` retired for this inbox, or removed; `cp CLAUDE.md CLAUDE.md.prevendor`.
4.2.2 `.gitignore`: re-include `.agentic-framework/.context/designer/`, `.secret-scan-allowlist`,
`.secret-scan-patterns` only (never all of `.agentic-framework/.context`).
4.2.3 Run the upgrade from the bleeding-edge clone with `FRAMEWORK_ROOT`/`PROJECT_ROOT` set (as rehearsed).
4.2.4 Immediately restore the 15 toolkit files and `resume.md` from git.
4.2.5 Work the 22 reds: for each 3.7.2.a item decide re-apply (still needed) or update the fixture (upstream
fixed it differently), using the divergence entry; adapt 3.7.2.b fixtures to the new templates; fix our 3.7.2.c
fixture.
4.2.6 Patch H1 locally (register it) or keep `git commit`-only spelling until upstream fixes it.
4.2.7 Done when the guard layer equals the baseline (or better), `fw doctor` is clean, and
`.vendor-divergence.yaml` has a new `last_vendor_event`.
4.3 **Upstream filings** (to AEF): step 7b overwrites the toolkit's origin project with older copies, no skip
flag; H1 still present at 1.7.424; step 5 ships 3 hooks it does not install; step 4c is missing from the step
listing.
4.4 Estimated effort: one focused session for 4.2.1-4.2.4, one to two for 4.2.5.
