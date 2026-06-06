# Framework-agent prompt — fix CLAUDE.md governance-section clobber on `fw upgrade` (PL-124 / G-055)

**Operator: copy everything inside the fenced block below into a fresh
`/opt/999-AEF` session. The prompt is self-contained.**

---

```
PL-124 / G-055 — `fw upgrade` step 1 ("CLAUDE.md governance sections")
silently destroys project-specific operating guidance every run. PL-124
was registered against T-1447; PL-022 (T-1069) documents the broader
clobber pattern. G-055 codifies the gap. Today's run on /opt/termlink
lost 18 lines including the entire conversation-arc skill table (10
operator-facing slash commands). Operator restored via `cp CLAUDE.md.bak
CLAUDE.md`. The framework knew it was destructive (printed a warning
listing the lost lines) but did not block. Please prioritize.

This is a sibling to T-2099 in the bare-from-consumer cluster — same
root pathology ("framework overrides project intent during handoff/
refresh"), different code path.

## Symptom

Running `fw upgrade` (or `--force-downgrade`, etc.) executes step 1 of 10:

    [1/10] CLAUDE.md governance sections
      UPDATED  Governance sections refreshed from framework template. Backup: CLAUDE.md.bak
      !  N line(s) in CLAUDE.md.bak are absent from the new CLAUDE.md.
          These may be project-specific inline customizations the template
          merge cannot preserve. First lines:
            <... actual project-specific lines ...>
          Review and re-apply if needed, then remove CLAUDE.md.bak to clear.
          Background: G-055 / PL-124

The framework correctly identifies it lost N project-specific lines but
takes no action — no block, no auto-restore, no opt-out flag. The agent/
operator must remember to manually restore from `.bak`, OR the lines
are lost permanently on the next commit.

## Reproducer (file:// upstream — no network)

    cd /tmp && rm -rf t-2015-{upstream,consumer}
    git clone --depth=1 /opt/999-AEF /tmp/t-2015-upstream
    mkdir /tmp/t-2015-consumer && cd /tmp/t-2015-consumer
    /opt/999-AEF/bin/fw init . --provider generic
    /opt/999-AEF/bin/fw vendor .
    cat >> .framework.yaml <<EOF
    upstream_repo: file:///tmp/t-2015-upstream
    EOF
    # Add a project-specific line outside any framework-managed block:
    cat >> CLAUDE.md <<EOF

    ## Project-Specific Override
    TEST_MARKER_PROJECT_LINE_PRESERVED — must survive fw upgrade
    EOF
    git -c user.email=t@t -c user.name=t add -A && \
        git -c user.email=t@t -c user.name=t commit -q -m "seed"

    # Run upgrade:
    .agentic-framework/bin/fw upgrade 2>&1 | head -30

    # Verify:
    grep -q "TEST_MARKER_PROJECT_LINE_PRESERVED" CLAUDE.md && echo "PASS: preserved" || echo "FAIL: clobbered"

Expected (after fix): "PASS: preserved".
Observed (current bug): "FAIL: clobbered" + warning message listing the
lost lines from `.bak`.

Cleanup: `rm -rf /tmp/t-2015-{upstream,consumer}`.

## Root cause shape

`lib/upgrade.sh` step 1 ("CLAUDE.md governance sections") replaces the
whole CLAUDE.md governance-section text with the framework's template
content. The implementation is whole-section overwrite, not block-aware
merge. The framework template has no `<!-- fw-upgrade:section-start -->`
/ `<!-- fw-upgrade:section-end -->` markers; consumer CLAUDE.md files
have no opt-out flag per section. So the only way to retain project-
specific governance is to never let `fw upgrade` run that step — which
conflicts with the framework's goal of getting consumers onto the
latest governance rules.

The framework KNOWS this is happening (it emits the warning) but
chooses to proceed rather than block.

## Recommended fix shape

### Primary — sentinel markers (RECOMMENDED)

Wrap framework-managed content in CLAUDE.md template with HTML-comment
sentinels:

    <!-- fw-upgrade:governance-start -->
    [framework-managed governance text]
    <!-- fw-upgrade:governance-end -->

`lib/upgrade.sh` step 1 only rewrites text BETWEEN the markers. Project-
specific lines OUTSIDE the marker block are preserved automatically.

Operators who want to extend the governance section can:
- Add lines AFTER `<!-- fw-upgrade:governance-end -->` (preserved across upgrade)
- Add lines AFTER `<!-- fw-upgrade:authority-end -->` (or whichever sub-block)

Lines they DO want auto-managed go INSIDE the markers (rare).

This is a non-breaking change for consumers that have never customized
CLAUDE.md — their content sits inside the markers and continues to be
auto-refreshed. Consumers that have customized just need a one-time
migration: move their custom lines outside the markers (or add the
markers around the framework block manually).

### Secondary — per-section opt-out

`.framework.yaml` declares which step-1 sub-blocks are managed:

    governance_sections:
      authority_model:     managed   # default
      task_system:         managed
      enforcement_tiers:   managed
      project_specific:    custom    # opt-out — never touched by step 1

Step 1 only refreshes blocks marked `managed`. Operators with deep
customization opt their relevant blocks out wholesale.

### Tertiary — block unless `--accept-clobber`

Make the warning a refusal. Today's print becomes:

    REFUSED  Step 1 would lose N line(s) of project-specific content.
             To proceed anyway: re-run with --accept-clobber.

This is the cheapest fix but moves the problem from "silently
destructive" to "noisy but still destructive when bypassed".

### Regression test

Add to `tests/e2e/upgrade-test.sh`:

- Seed a consumer with a marker line outside any framework block
- Run `fw upgrade`
- Assert the marker line still exists in CLAUDE.md
- Assert .bak is created (so manual restore is still possible)

The test should be PURE file:// (no network).

### Doc

Add a comment at the step 1 site in `lib/upgrade.sh` stating the
invariant: "Project-specific content outside the sentinel markers MUST
survive an upgrade. Inside-marker content may be regenerated. Operators
who want to extend governance text put it outside the markers."

## Acceptance — when is this done?

1. The reproducer above runs to completion and prints "PASS: preserved".
2. `tests/e2e/upgrade-test.sh` includes the marker-preservation
   regression and it passes.
3. PL-124 is updated to point at the fix commit. G-055 is closed.
4. Optional but recommended: existing template CLAUDE.md is migrated
   to use markers, with a short upgrade-guide section for operators.

## Consumer-side state (FYI for context)

- /opt/termlink T-2015 captures this RCA and the prompt artifact.
- /opt/termlink CLAUDE.md was restored from .bak after today's clobber.
- T-1447 / T-1069 / T-1063 (PL-018 / PL-022 / PL-123 / PL-124) are
  prior-art on the broader "fw upgrade clobbers project state" pattern.
- T-2014 / T-2099 are the sibling fork-bomb arc — same family of bug
  (bare-from-consumer handoff doesn't preserve parent intent).

## Out of scope for this fix

- The fork-bomb fix (T-2099 — already shipped).
- The `--force-downgrade` flag drop during bare-from-consumer replay
  (separate TermLink T-2016 dispatch — same handoff code area).
- Any GitHub mirror behavior (G-058 unrelated).
- Watchtower UI changes (not impacted).
```

---

## Notes for the operator

- Paste **only** the fenced block above into the framework-agent session at `/opt/999-AEF`.
- The reproducer uses `file://` upstream — runs on any host without GitHub traffic.
- If `tests/e2e/upgrade-test.sh` already covers something similar, point the agent at the existing structure to amend rather than duplicate.
- Sibling dispatch in this session: T-2016 (CLAUDE.md/framework-agent prompt for the `--force-downgrade` drop bug). They share the bare-from-consumer code area but address different observable symptoms.
- When the fix lands and mirrors to GitHub, the next `fw upgrade` should leave project-specific lines untouched. At that point T-2015 closes via the verification block and PL-124 is updated.
