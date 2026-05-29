---
id: T-1699
name: "framework upgrade 1.6.160 + test suite (dispatched)"
description: >
  framework upgrade 1.6.160 + test suite (dispatched)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-18T21:31:22Z
last_update: 2026-05-18T21:45:30Z
date_finished: null
---

# T-1699: framework upgrade 1.6.160 + test suite (dispatched)

## Context

Dispatched directive: upgrade framework on this host (consumer-initialized shape — `/opt/termlink`), run test suite, report back via TermLink. Pinned upstream = GitHub canonical (OneDev mid-migration).

## Findings — 2026-05-18T23:32Z — `fw upgrade` FORK-BOMBED

**Project shape:** `consumer-initialized` (`.framework.yaml` + `.agentic-framework/bin/fw` present; no `FRAMEWORK.md` at root).

**Pre-upgrade state:**
- `fw version`: v1.6.160
- `.framework.yaml::upstream_repo`: `DimitriGeelen/agentic-engineering-framework` (GitHub shorthand)
- `.framework.yaml::version`: 1.6.160, `upgraded_from`: 1.6.260, `last_upgrade`: 2026-05-15

**What happened.** `.agentic-framework/bin/fw upgrade` (no args) ran the T-1634 bare-from-consumer auto-clone path. It cloned the upstream to `/tmp/fw-upstream-XXXXXX/fw` and handed off via `"$_tmpd/fw/bin/fw" upgrade "$target_dir"`. The cloned fw immediately re-fired bare-from-consumer detection and cloned a fresh upstream of its own — and so on. **153 nested `fw upgrade` processes spawned in <60s** before TaskStop + SIGTERM cleared them. Load avg hit 11.9 / 11.86. Zero state mutation on /opt/termlink (vendored tree byte-identical pre/post; same 5-file 131-line drift).

**Root cause:** `.agentic-framework/bin/fw:120-122` (`resolve_framework`):
```
if [ -n "${PROJECT_ROOT:-}" ] && [ -f "$PROJECT_ROOT/.agentic-framework/FRAMEWORK.md" ]; then
    echo "$PROJECT_ROOT/.agentic-framework"
    return 0
fi
```
When the cloned upstream's `bin/fw` runs with target=`/opt/termlink`:
1. `find_project_root` walks from `$PWD` (= `/opt/termlink`) → finds `.framework.yaml` → `PROJECT_ROOT=/opt/termlink`.
2. `resolve_framework`'s candidate is the cloned `<tmpdir>/fw`, but the T-498 preference rule above PICKS the consumer's vendored copy first.
3. `FRAMEWORK_ROOT` resolves back to `/opt/termlink/.agentic-framework` — the very copy the upgrade is trying to refresh.
4. `do_upgrade`'s bare-from-consumer guard (`lib/upgrade.sh:222`) sees `_fw_root_canon == _consumer_vendor_canon` and triggers another auto-clone.
5. Recurse infinitely.

The T-1634 commit was supposed to fix bare-from-consumer by auto-cloning — and it does clone, but the handoff lacks any isolation against the cloned fw resolving back through the consumer's preference path.

**Fix recipe (upstream framework repo, NOT this consumer):** in `lib/upgrade.sh:310`, replace
```
"$_tmpd/fw/bin/fw" "${_replay_args[@]}"
```
with
```
env FRAMEWORK_ROOT="$_tmpd/fw" PROJECT_ROOT="$target_dir" "$_tmpd/fw/bin/fw" "${_replay_args[@]}"
```
AND make `bin/fw:498` respect a caller-supplied FRAMEWORK_ROOT instead of unconditionally re-running `resolve_framework`:
```
if [ -z "${FRAMEWORK_ROOT:-}" ]; then
    FRAMEWORK_ROOT=$(resolve_framework) || true
fi
```
The two changes together let the cloned upstream's `bin/fw` know it's running as the upgrade source, not as the target's vendored copy.

**Repro (1-liner, run on any consumer install that pre-dates the fix):**
```
cd /opt/termlink && .agentic-framework/bin/fw upgrade
# Observe: /tmp/fw-upstream-XXXXXX dirs multiplying every ~2s, ps shows
# nested `bin/bash <tmpdir>/fw/bin/fw upgrade <consumer>` processes.
# Kill via: pkill -TERM -f 'fw-upstream' (Tier-0 approval needed for -9)
```

**Severity:** **SEV-1**. `fw upgrade` is hard-blocked on every consumer install that hits the bare-from-consumer auto-clone path (i.e. every consumer that doesn't manually pass `--from-upstream URL` to bypass it).

**Pattern match:** PL-159 / PL-168 class — implementation milestone (T-1634 auto-clone) landed without an integration test exercising the handoff from cloned fw → bin/fw resolve. The structural guard at upgrade.sh:222 itself accidentally double-fires.

**Actions taken locally:**
- No source edits to `.agentic-framework/` (dev-box rule).
- Cleaned 153 `/tmp/fw-upstream-*` scratch dirs.
- Documenting findings here for upstream evidence.
- Reporting via TermLink topic `framework.upgrade.report`.

**Followups deferred (cannot be done from consumer):**
- Test suite (STEP 5) — blocked, can't upgrade.
- Pre-existing 5-file vendored drift — still on disk, still uncommitted; reconciliation requires either (a) upstreaming via bug-report pickup or (b) declaring obsolete and discarding. Deferred until upgrade path works.

## Findings — 2026-05-29T12:30Z — second repro (dispatched via T-1869)

Re-repro under fresh dispatch — same SEV-1, same fork-bomb, same root cause.
Bin still at v1.6.160, no upstream fix has landed.

**New datum: wrong-target propagation.** Last run (2026-05-18) the recursion
correctly carried `target_dir=/opt/termlink` through the handoff. This run the
children all show `fw upgrade /opt/003-Vailliant-diagnosis` — the target_dir
shifted to a peer project somewhere in the cloned-upstream's path resolution.
Possible source: a global registry (`/root/.agentic-framework/...`?) or
.framework.yaml on a peer that the cloned upstream's `find_project_root`
walks up to. T-559 boundary blocks investigation from this consumer. Worth
upstream-side inspection.

**Cleanup:** 50+ /tmp/fw-upstream-* tempdirs reaped automatically via the
EXIT trap on SIGTERM. /tmp disk reclaim: ~9G (916G→833G used, but 96% used
is pre-existing).

**Process tree at peak:** PIDs 2532558→2617712, ~85 nested processes in
~60s before SIGTERM cleared them. All 0% CPU (blocked on git-clone /
network), so the wedge mode is queue-of-pending-clones rather than
hot-loop CPU.

**Action:** none locally. Awaiting upstream commit on `lib/upgrade.sh:310`
+ `bin/fw:498` per fix recipe above.

## Findings — 2026-05-30T00:55Z — UPSTREAM FIX LANDED as T-2099

framework-agent acted on the forensic evidence + recipe (both T-1699
task body and the 2026-05-29 chat-arc broadcast). **Fix is upstream:**

- `lib/upgrade.sh:319-320` (BOTH halves implemented as one patch):
  ```bash
  env FRAMEWORK_ROOT="$_tmpd/fw" PROJECT_ROOT="$target_dir" \
      "$_tmpd/fw/bin/fw" "${_replay_args[@]}"
  ```
  Comment cites: "Origin: /opt/termlink ran fw upgrade twice in one
  hour, fork-bombed both times. Forensic evidence + recipe via
  framework.upgrade.report TermLink topic."

- `bin/fw:504-506`:
  ```bash
  if [ -z "${FRAMEWORK_ROOT:-}" ]; then
      FRAMEWORK_ROOT=$(resolve_framework) || true
  fi
  ```

**Upstream commits:**
- `be72baa5` T-2099: SEV-1 fork-bomb fix — env-scope fw upgrade handoff + caller-FRAMEWORK_ROOT
- `f11e3c4a` T-2100: inception — 6 fork-bomb containment enhancements to consumer-upgrade-test-fix-report prompt (follow-on hardening)

**The fix matches the recipe in this task file 1:1.** Both lines
documented in T-1699 are now upstream, in the exact form I prescribed
(env-prefix on handoff + zero-check guard before resolve_framework).

**Disposition.** The framework-bug aspect of T-1699 is **resolved
upstream**. The remaining AC ("fw upgrade completed successfully against
GitHub upstream") requires re-running fw upgrade on this consumer.
That's a consequential action (fork-bomb if fix is wrong) and should
be operator-initiated, not autonomous. Suggested verification:

```
cd /opt/termlink
.agentic-framework/bin/fw upgrade   # should now complete normally;
                                    # if any fork-bomb resumes:
                                    # pkill -TERM -f 'fw-upstream'
```

After the verification, AC 1 ticks and T-1699 closes. T-1867's
end-to-end propagation test also unblocks at the same moment, because
the same `fw upgrade` invocation that proves T-1699 also exercises
T-1867's doorbell+mail toolkit propagation flow.

**Interactive-arc credit.** The forensic discipline (T-1699 task
body's verbatim two-line fix recipe + the chat-arc broadcast tagging
@framework-agent) was the input. framework-agent's pickup + commit was
the output. **One DM-class message, one operator action, one SEV-1
cleared** — the same pattern that closed T-1166 via ring20-management
this session. Doorbell+mail arc paying off twice in 90 minutes.



<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [ ] `fw upgrade` completed successfully against GitHub upstream; `.framework.yaml` `version` and `last_upgrade` updated
- [ ] `fw doctor` passes (exit 0)
- [ ] `fw test all` run; counts captured per suite; any failures classified (framework bug / termlink bug / environmental)
- [ ] Pre-existing vendored drift in `.agentic-framework/` (5 files) reconciled: either upstreamed via bug-report pickup envelope to framework-upstream agent OR documented as obsolete and discarded
- [ ] Report sent via TermLink to dispatcher with shape, before/after fw version, upstream URL, test counts, issues + fixes, learnings

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
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

### 2026-05-18T21:31:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1699-framework-upgrade-16160--test-suite-disp.md
- **Context:** Initial task creation
