# T-3420: AEF ed683536a field test (010-termlink, 2026-10-12)

AEF asked consumers to field-test bleeding-edge ed683536a before v1.8.11 reaches master (conversation
`field-test-ed683536a`). The operator ruled A (take it now). Host .107, consumer `/opt/termlink`, previous vendor
1.8.9 (cc2e08a02).

## Procedure

1. Pre-re-vendor checklist (`.vendor-divergence.yaml`): divergence check clean (0 unregistered); `CLAUDE.md` copied
   aside; `fw doctor` saved (0 failures, 8 warnings).
2. Full upstream clone at ed683536a (VERSION 1.8.10; v1.8.9 is an ancestor). Rehearsal on a throwaway clone of this
   repo: dry run (21 changes, 2 manual-review skips, 9 PRESERVE, customised templates KEPT), then a real run (rc 0).
3. Real run from inside the clone: dry run (20 changes), then `fw upgrade /opt/termlink` (rc 0, "doctor PASS").
   No `--force`, no `--allow-delete-locals`.
4. After: `CLAUDE.md` byte-identical to the copy; every index-100755 file executable; 15 `.upstream` copies
   unchanged upstream since 1.8.9 removed; the task-template servicing-loop note (T-4024) merged into our
   customised template.
5. Two of our 9 preserved files had changed upstream since 1.8.9 and were kept local by the upgrade: 
   `agents/context/lib/safe-commands.sh` (T-3814) and `web/blueprints/discovery.py` (T-4061). Both 3-way merged
   (base v1.8.9, ours, theirs ed683536a): 0 conflicts; `bash -n` / `py_compile` and our safe-commands fixtures pass.
   **Without this merge, items 3 and 13 would have stayed broken here while the upgrade reported success.**
6. Commits 0570db4e2 (re-vendor), the register update, and the merge commit. Watchtower restarted onto the new code.

## fw doctor before / after

| | Before (1.8.9) | After (ed683536a) |
|---|---|---|
| Failures | 0 | 0 |
| Warnings | 8 | 10 |
| Changed | Watchtower smoke 4/53 failed | Watchtower smoke 1/53 failed (`/cron` timed out) |
| New | | "Installed claude-fw (fixed copy) drifted from repo source", twice: `/root/.local/bin/claude-fw` and `/usr/bin/claude-fw` |

The new warning is **accurate**, not a false positive: both installed copies are byte-identical to the 1.8.9
`bin/claude-fw` (69,580 bytes), not the router, and the repo copy changed in this upgrade. Not refreshed (host
change, left to the operator).

## The 19 items

| # | Item | Result | Evidence |
|---|---|---|---|
| 1 | T-4081 decisions not reverted | **works** (scan) / not exercised (live) | `watchtower_decision_revert.py`: "0 Watchtower decision(s) no longer at HEAD (last 60 days)". No Watchtower inception decision was made during the test (operator action) |
| 2 | T-4070 Human-AC tick provenance | not exercised | needs an operator tick in Watchtower |
| 3 | T-4061 query text not kept | **works**, after the merge in step 5 | Browser search on `/search`: form posts (with `_csrf_token`); 20 results; address bar `/search` with no query; query absent from `watchtower.log`. A direct `GET /search?q=<unique>` also left 0 traces in the log. Before the merge, our preserved `discovery.py` was GET-only |
| 4 | T-4062 secret key mode | **works** | `.fw-secret-key` `-rw-------` after two restarts; doctor "Watchtower secret key is owner-only (mode 600)" |
| 5 | T-4052 task from /approvals stays | **works** | Playwright: /approvals, click `/tasks/T-2487`, wait 32 s, still on `/tasks/T-2487` |
| 6 | T-4053/T-4045 approvals split, board column | **works** (page) | /approvals headings "READY FOR YOU" and "WAITING ON THE AGENT"; `/tasks` board has "Awaiting review". `fw review-queue` match not compared item by item |
| 7 | T-4043/44/46/47 pages | **works, one gap** | `/file/docs/reports/arc011-ph1-acceptance.md`: one h1, title once. Inception page (T-3385): 5 criteria checkboxes, none disabled, Verification card present. **Gap:** doc paths under `docs/design/` render as plain code with the tooltip "outside the file viewer's allowlist (VIEWABLE_DIR_PREFIXES in web/shared.py)"; in this project the design docs live there, so task-page links to them never work |
| 8 | T-4041 FW_HOST / no localhost:3000 | **works** (links) / not exercised (FW_HOST) | no `localhost:3000` in `/`, `/approvals`, `/tasks`, `/search` or a task page; we do not bind to one address |
| 9 | T-4042 MESSAGING_TOOLKIT decided once | **works** (decision) / not exercised (deleted files) | `.framework.yaml` gained `MESSAGING_TOOLKIT: 1` (toolkit present); no toolkit file was deleted to test re-creation |
| 10 | T-4055 version string | **works** | `.agentic-framework/VERSION` and `.framework.yaml`: `1.8.9+227.ged683536a` |
| 11 | T-4051/T-4056 doctor router false positive | not exercised | this host has fixed copies, not the router (see doctor table); the drift warning is correct here |
| 12 | T-4067 dispatch without --task-type | **works** | `fw termlink dispatch --name t3420-probe --task T-3420 --prompt ...` (no `--task-type`): worker spawned ("Task-type: build"), answered `PROBE-OK` |
| 13 | T-3814 commit with no focused task | see the close-out note below | |
| 14 | T-4058 unit failure output kept | not exercised | no unit-suite run |
| 15 | T-4059 no HTTP 400 on retried deliveries | not exercised | no retried delivery; our one send to AEF went through first time ("delivered: RECEIVED") |
| 16 | T-4069 claude-fw started before the upgrade | **partly** | `fw upgrade` printed nothing about the running wrapper. Doctor: "Running claude-fw wrapper(s) (1) newer than their script" (OK), because the wrapper runs the installed copy, which the upgrade did not change; the separate drift warning names the installed copies. So the stale wrapper is visible only through the drift warning |
| 17 | T-3886 bare fw upgrade not refused | not exercised | we upgraded from inside the clone (our procedure); no behind-upstream consumer to try from |
| 18 | T-4040 greenfield T-002 editable | **works** | fresh `fw init --provider claude` in a scratch dir; T-002 is `workflow_type: inception`; `fw hook check-inception-schema` on an Edit of T-002 → rc 0 |
| 19 | T-4050 opencode review seat | not exercised | no `.opencode/` here |

## New findings (not on the list)

1. **A preserved file silently blocks an upstream fix, and the upgrade reports success.** `.fwvendor-preserve.yaml`
   kept our `discovery.py` and `safe-commands.sh`; upstream's T-4061 and T-3814 changes to those files were
   dropped, with only a PRESERVE line saying "kept local over upstream". Nothing says "upstream changed this file
   since your base; merge needed". Here that hid exactly two field-test items (3, 13). Suggest: when a preserved
   file changed upstream between the pinned version and the target, print a MERGE-NEEDED line with the base and
   target versions (or write the 3-way merge result beside it as `.merge`).
2. **`fw watchtower current` (and the audit's "Watchtower currency" PASS) can never fire in a consumer.**
   `bin/watchtower.sh:do_current` checks `$PROJECT_ROOT/web`, which does not exist in a consumer; the code is under
   `$FRAMEWORK_ROOT/web`. Before the restart it reported the pre-upgrade process as current although the upgrade
   had just changed `web/`. Same class as OBS-097 (PROJECT_ROOT resolution of framework-owned assets).
3. **`docs/design/` is not viewable** (item 7 gap). A per-project addition to `VIEWABLE_DIR_PREFIXES` would fix it.
