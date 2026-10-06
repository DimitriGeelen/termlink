# AEF upgrade v1.6.29 → v1.8.3 for 010-termlink: detailed analysis (2026-10-06)

Why now: peers' AEF senders cannot get RECEIVED from us (no `fw sidecar` verb in v1.6.29), and today's misrouting
(a fleet AEF pane and our fleet agent `framework-agent-systemd` taking wake-ups) made the operator the transport.

Method: (a) each of the 21 at-risk fixes checked against 1.8.3 by a subagent
(`docs/reports/T-3344-aef-upgrade-divergence-check.md`); (b) the **real** upgrade run end to end against a
disposable copy of this repo (git archive of HEAD + live settings.json + vendored tree, HOME redirected) from a
full clone of upstream. Nothing live was touched.

## 1. How the upgrade actually runs

1a. `fw upgrade` clones upstream **main** (`--depth=1`, `lib/upgrade.sh:1059`), not a tag, and hands off to that
    version's own `fw upgrade`. Today main == tag v1.8.3 (`542623a3c`); that can change by upgrade day.
1b. **As invoked normally it REFUSES**: the shallow clone cannot see our recorded `version_sha` 67aeacc73500, so it
    reports "foreign-source". Our sha IS an ancestor of v1.8.3 (checked on a full clone). Correct path: run from a
    full clone — `FRAMEWORK_ROOT=<full clone> PROJECT_ROOT=/opt/termlink <full clone>/bin/fw upgrade /opt/termlink`.
1c. `--dry-run` previews nothing (it returns before cloning). The copy-simulation is the only real preview.

## 2. What the upgrade did to the copy (completed rc=0, 22 changes, 2 skipped)

2a. **Step 4b refused first** until `.gitignore` re-includes 4 files and the designer corpus (our `.gitignore:37`
    `.agentic-framework/*` hides them): `.fw-vendor-stamp.json`, `.secret-scan-allowlist`, `.secret-scan-patterns`,
    `.upstream`, and `.agentic-framework/.context/designer/projects/` — while `.context/working/` stays ignored
    (holds `.fw-secret-key`; verified with `git check-ignore`).
2b. **CLAUDE.md** (step 1): 7 lines lost — exactly the stale 120K/150K/170K budget ladder our own rule says to
    ignore. Our project rules sit above `## Core Principle` and survive.
2c. **Hooks** (step 5): none removed, 9 added — check-human-ac-tick, check-paid-backend,
    check-worktree-governance-write (PreToolUse); **sidecar-autostart** (SessionStart startup+resume);
    **sidecar-receiver-ready**, stop-driver (Stop); **sidecar-inbox**, **sidecar-receiver-adapter** (UserPromptSubmit).
2d. **Step 7b overwrites 15 TermLink-origin toolkit files with OLDER, smaller template copies** (backups `.bak`):

| File | ours (lines, last change) | template |
|---|---|---|
| scripts/agent-send.sh | 739, 2026-10-03 | 242 |
| .claude/commands/check-arc.md | 456, 2026-10-01 | 175 |
| scripts/agent-chat-arc-recent.sh | 677, 2026-08-16 | 333 |
| scripts/agent-listeners.sh | 387, 2026-08-10 | 232 |
| scripts/be-reachable.sh | 509, 2026-07-03 | 380 |
| scripts/recent-dm.sh | 383, 2026-07-02 | 300 |
| .claude/commands/agent-handoff.md | 264, 2026-06-28 | 167 |
| scripts/listener-heartbeat.sh | 253, 2026-08-09 | 181 |
| (7 more, smaller deltas) | | |

    TermLink is the origin of this toolkit. Every one must be restored from git after the upgrade
    (`git checkout HEAD -- <15 paths>`), and AEF told that step 7b must not overwrite the origin project.
2e. Cron: adds `sidecar-sweep-5m` (needs `fw cron install`, run from /opt/termlink, never a worktree — T-2815).
2f. Seeds a one-time task "define project objectives"; mints a `project_id`; logs to `.context/audits/upgrades.yaml`.
2g. Post-upgrade `fw doctor`: OK on installation, hooks (36, portable), exec bits; WARN only for git identity
    (an artefact of the simulation's empty HOME).

## 3. Vendored fixes (21 at risk; detail in the divergence-check report)

3a. CARRIED 6 · PARTIAL 4 · NOT-CARRIED 10 · RESTRUCTURED 1. The register said 5 local-only; the real loss is larger
    because "filed-upstream" fixes were not all taken.
3b. NOT-CARRIED, applies cleanly: T-3336 (parallel embedding), T-3270, T-3269 (commit-pending helper paths),
    T-2687 (pickup fail-open, use be0fcfa68).
3c. NOT-CARRIED, manual merge: **T-3178 Tier-1 write gate** (1.8.3 still treats `2> file` / `&> file` as not a
    write — highest risk), T-3188 + T-3189 (cost estimator), T-3185 (bvp quadrants), T-2866 (doctor router check).
3d. PARTIAL: T-3184, T-3176 (bvp), T-2882 (handover markers), T-3316 (unknown checkpoint verb still prints a false
    HOOK CRASHED). RESTRUCTURED: T-3198 — do not re-apply; its fixture will exit 2 afterwards.
3e. Register hazard H1 is present in 1.8.3: the commit exemption blocks `fw git commit` but allows bare `git commit`.

## 4. The receiver: one consumer per inbox

4a. After the upgrade `sidecar-autostart` starts AEF's receiver for this project at the next session start.
4b. Our inbox `inbox:cacc73ea32b121dd/010-termlink` is today consumed/acked by **three** identities:
    our notify-sidecar (`6738c073…`, as claude-termlink), the host default (`d1993c2c…`), and the fleet agent
    `framework-agent-systemd` (`3bba15e6…`, cwd /opt/termlink, has the wake consumer — mail is pushed into ITS PTY).
    Adding AEF's receiver without retiring the others breaks the one-receiver rule (the failure class of today).
4c. Order therefore matters: set `FW_SIDECAR_AUTOSTART=0` for the upgrade, retire our notify-sidecar entry and the
    fleet agent's consumption of our inbox, THEN start AEF's receiver once and verify with a test send that reaches
    RECEIVED and HANDED_OVER into this session.
4d. Not fixed by the upgrade: an idle session is only reached on the next prompt (UserPromptSubmit injection);
    waking an idle session is AEF's open T-3891 / arc-011.

## 5. Proposed safe sequence (each step verified before the next)

5a. Commit everything outstanding; `cp CLAUDE.md CLAUDE.md.prevendor`; record the 15 toolkit paths.
5b. Add the `.gitignore` re-includes (2a); verify `.context/working/` still ignored.
5c. Full clone of upstream; confirm main == the intended tag and our sha is an ancestor.
5d. Run the upgrade with `FW_SIDECAR_AUTOSTART=0`.
5e. Restore the 15 toolkit files from git; diff CLAUDE.md vs `.prevendor` (expect only the 7 ladder lines).
5f. Re-apply NOT-CARRIED fixes (3b clean, 3c manual), each with its fixture; T-3178 first.
5g. Run the guard layer + `fw doctor` + `check-vendor-divergence.sh`; update `last_vendor_event` in the register.
5h. `fw cron install` from /opt/termlink.
5i. Receiver switchover (4c), verified by a test send; then restart this session with `claude-fw --termlink`
    (operator action, in runme).
5j. File upstream: step 7b overwriting the origin project; shallow-clone refusal in the default upgrade path;
    T-3178 still open; H1.
