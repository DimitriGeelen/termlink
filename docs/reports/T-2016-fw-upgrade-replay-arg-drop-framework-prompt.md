# Framework-agent prompt — fix `fw upgrade` bare-from-consumer replay-arg drop

**Operator: copy everything inside the fenced block below into a fresh
`/opt/999-AEF` session. The prompt is self-contained.**

---

```
`fw upgrade` bare-from-consumer auto-clone handoff silently drops every
flag except `--force` and `--dedupe-user-hooks`. Observed today on
/opt/termlink: `--force-downgrade` was discarded mid-handoff, causing
the cloned upstream's bin/fw to re-fire the split-brain guard the
operator was specifically trying to bypass. Sibling to T-2099 (same
handoff site, same "parent intent not preserved across handoff"
pathology — different observable symptom).

Please prioritize. The workaround (manual env-bypass invocation) works
but is operator-unfriendly and undocumented.

## Symptom

Running `.agentic-framework/bin/fw upgrade --force-downgrade` (or any
other flag the framework supports beyond `--force` /
`--dedupe-user-hooks`) produces the same output as without the flag:

    [1/10] CLAUDE.md governance sections
    ...
    REFUSED  Consumer v1.6.160 is AHEAD of framework v1.6.7.
             ...
             To proceed anyway: re-run with --force-downgrade.

Operator types `--force-downgrade`, fw upgrade says "use --force-downgrade".
The flag never reaches the cloned upstream's bin/fw.

## Reproducer (file:// upstream)

    cd /tmp && rm -rf t-2016-{upstream,consumer}
    git clone --depth=1 /opt/999-AEF /tmp/t-2016-upstream
    mkdir /tmp/t-2016-consumer && cd /tmp/t-2016-consumer
    /opt/999-AEF/bin/fw init . --provider generic
    /opt/999-AEF/bin/fw vendor .
    cat >> .framework.yaml <<EOF
    upstream_repo: file:///tmp/t-2016-upstream
    version: 99.99.99  # force consumer-ahead-of-framework state
    EOF

    # Run with --force-downgrade — should bypass the split-brain refuse:
    .agentic-framework/bin/fw upgrade --force-downgrade 2>&1 | head -30

Expected (after fix): step 1 runs, no REFUSED line, upgrade completes.
Observed (current bug): REFUSED message fires with "To proceed anyway:
re-run with --force-downgrade" — even though --force-downgrade was on
the command line.

Cleanup: `rm -rf /tmp/t-2016-{upstream,consumer}`.

## Root cause (file:line)

`lib/upgrade.sh` at the bare-from-consumer auto-clone path (around
`lib/upgrade.sh:300-313` in current upstream, look for `_replay_args=`):

    local _replay_args=("upgrade" "$target_dir")
    [ "$force" = true ] && _replay_args+=("--force")
    [ "$dedupe_user_hooks" = true ] && _replay_args+=("--dedupe-user-hooks")
    # NOTE: do not replay --from-upstream — the upstream IS the source
    # now, the target is local-path-based from the upstream's PoV.

    ... env FRAMEWORK_ROOT=... PROJECT_ROOT=... "$_tmpd/fw/bin/fw" "${_replay_args[@]}"

The list whitelists exactly two flags. Anything else the operator
passed is silently dropped before the handoff. The whitelist pre-dates
`--force-downgrade`'s addition; it was never updated when that flag
was added to the upgrade verb. It will silently drop every future flag
too — `--dry-run` (if it lands), `--accept-clobber` (if T-2015 lands),
anything.

The design assumes the parent already parsed flags into named booleans
(`force`, `dedupe_user_hooks`) and the child only needs the named ones.
Any flag NOT promoted to a parent-side variable can't be replayed.

## Why nothing caught this

- `tests/e2e/upgrade-test.sh` covers the bare-from-consumer handoff
  shape (test #7 from T-2099) but tests cloning succeeds, not flag
  preservation.
- The handoff exec uses `env FRAMEWORK_ROOT=...` (T-2099 fix) so the
  identity of the framework is preserved — but the operator's intent
  (flags) is not.
- The framework has no lint or doc-comment that says "any new flag
  added to do_upgrade must also be added to the bare-from-consumer
  replay list."

## Recommended fix shape

### Primary — pass-through-all (RECOMMENDED)

Capture the original argv at `do_upgrade` entry, replay all of it
through the bare-from-consumer handoff:

    do_upgrade() {
        # Snapshot original args BEFORE flag-stripping logic:
        local _all_args=("$@")
        # ... existing arg-parse logic ...
    }

    # Later, when building _replay_args:
    # Strip --from-upstream (parent consumed it for the clone URL) but
    # keep everything else verbatim:
    local _replay_args=("upgrade")
    local _skip_next=0
    for _arg in "${_all_args[@]}"; do
        if [ "$_skip_next" = 1 ]; then _skip_next=0; continue; fi
        case "$_arg" in
            --from-upstream) _skip_next=1; continue ;;
            --from-upstream=*) continue ;;
            upgrade) continue ;;  # already prepended
        esac
        _replay_args+=("$_arg")
    done
    # Ensure target_dir is present (operator may have omitted it):
    [ "${_replay_args[*]}" = "${_replay_args[*]/$target_dir/}" ] && \
        _replay_args+=("$target_dir")

Pass-through-all means the bare-from-consumer code never needs updating
when a new flag is added — the flag flows through naturally.

### Secondary — explicit whitelist update

Add `--force-downgrade` (and any other current flag) to the existing
whitelist:

    [ "$force_downgrade" = true ] && _replay_args+=("--force-downgrade")

Cheaper than option 1 but creates ongoing drift risk — every future
flag needs a matching whitelist entry.

### Regression test

Add to `tests/e2e/upgrade-test.sh`:

- Seed consumer in a state where step 1 wants to refuse without
  `--force-downgrade` (consumer-ahead-of-framework via doctored
  `.framework.yaml` `version: 99.99.99`).
- Invoke `fw upgrade --force-downgrade` through the bare-from-consumer
  path.
- Assert upgrade COMPLETES (no REFUSED) — proves the flag survived
  handoff.

Or, more generally testable:

- Add a no-op `--test-replay-flag` to do_upgrade that just `echo`s
  "test-replay-flag-was-seen" if present.
- Run `fw upgrade --test-replay-flag` through bare-from-consumer.
- Assert the echo line appears in the output — proves arbitrary flag
  passthrough works.

### Doc

Add a comment at the `_replay_args=` site stating the invariant:

    # Any flag the operator passes to `fw upgrade` MUST survive the
    # bare-from-consumer handoff. The current whitelist approach is
    # known to drop flags silently when new ones are added — see
    # T-2016 forensics. Prefer pass-through-all via captured argv;
    # add explicit exclusions only for flags the parent has already
    # consumed (e.g. --from-upstream).

## Acceptance — when is this done?

1. The reproducer above runs to completion with no REFUSED line.
2. `tests/e2e/upgrade-test.sh` includes a flag-preservation regression
   and it passes.
3. Comment at the `_replay_args` site documents the invariant and
   warns against the whitelist approach.

## Consumer-side state (FYI for context)

- /opt/termlink T-2016 captures this RCA and the prompt artifact.
- Workaround in use: env-bypass invocation —
  `env FRAMEWORK_ROOT=/tmp/aef-fresh PROJECT_ROOT=/opt/termlink /tmp/aef-fresh/bin/fw upgrade /opt/termlink --force-downgrade`
- Sibling dispatch T-2015 (CLAUDE.md clobber on step 1 of same upgrade.sh).
- Sibling-of-siblings T-2014 / T-2099 (fork-bomb fix — primary bare-
  from-consumer handoff hardening already shipped, this builds on it).

## Out of scope for this fix

- The fork-bomb fix (T-2099 — already shipped).
- The CLAUDE.md clobber fix (T-2015 dispatch in flight).
- The global-shim refusal at upgrade.sh step 4c (operator-environment
  concern, not upstream).
- Any GitHub mirror behavior (G-058 unrelated).
```

---

## Notes for the operator

- Paste **only** the fenced block above into the framework-agent session at `/opt/999-AEF`.
- T-2015 and T-2016 prompts can be pasted together as a single session — they share the same upgrade.sh code area and the framework-agent will benefit from seeing the full bare-from-consumer surface as one block of work.
- Workaround documented in the body — operators can already use it today without waiting for the fix.
- When the fix lands and mirrors to GitHub, the next `fw upgrade --force-downgrade` (or any flagged invocation) should pass cleanly. At that point T-2016 closes via the verification block.
