---
id: T-2811
name: "Watchtower cannot start — 4 untracked web blueprints, and my drift checker calls web/ informational"
description: >
  `fw serve` dies at create_app with ModuleNotFoundError web.blueprints.bvp. Four web/*.py modules are untracked and absent here. T-2689's drift checker classifies untracked web/ as informational, never firing — but Flask registers blueprints at startup, so a missing one is fatal, not cosmetic. Recover the four and correct the classification.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [governance, framework-recoverability, watchtower, g-062]
components: [scripts/check-framework-tracking-drift.sh, tests/framework-tracking-drift-fixtures.sh]
related_tasks: [T-2689, T-2692, T-2806, T-2807, T-2705]
created: 2026-08-20
last_update: 2026-08-20T17:58:58Z
date_finished: 2026-08-20T17:58:58Z
---

# T-2811: Watchtower cannot start — four untracked web blueprints

## Context

The operator asked for Watchtower links. Watchtower does not start:

```
File ".agentic-framework/web/blueprints/__init__.py", line 40, in register_blueprints
    from web.blueprints.bvp import bp as bvp_bp
ModuleNotFoundError: No module named 'web.blueprints.bvp'
```

Same class as T-2806/T-2807: the file exists on the main checkout's disk, is untracked, and so
is absent from every worktree and clean clone. Measured against main: **114 of 118** `web/` +
`lib/` Python modules are present here; **4 are missing**, all web blueprints:

```
web/blueprints/bvp.py
web/blueprints/designer.py
web/blueprints/designer_api.py
web/designer_registry.py
```

### Not a duplicate of T-2705, and checked before acting

`worktree-charter-review-2026-0814` carries **T-2705**, "Vendored framework is internally
inconsistent — Watchtower cannot start (missing `lib/arc_membership.py`)". Same symptom, and
the collision checker (T-2800) flagged the overlap, so the branch was read before doing
anything.

It is a different instance. `lib/arc_membership.py` **is present here** — T-2807's recovery
covered `lib/` wholesale, so that half is already fixed on this branch. What remains is the
four `web/` files, which T-2807 explicitly scoped out. T-2705's chosen remedy is a re-vendor
via `fw update`; that is theirs and is not duplicated here.

### The part that is mine to fix: a wrong classification

T-2689's drift checker splits untracked framework files into two classes, and CLAUDE.md
records the split as:

> Fires on `bin/ lib/ policy/ agents/` (clean-clone-breaking); untracked `docs/`/`web/` is
> informational

**That is wrong for `web/blueprints/`.** Flask registers every blueprint at `create_app()`, so
a missing blueprint module is not a missing page — it is a hard `ModuleNotFoundError` before
the app binds a port. The reasoning behind "informational" was sound for `docs/` (a missing doc
is a gap, not a broken install) and got over-applied to `web/` by adjacency.

The evidence is that the checker reported this tree clean while `fw serve` was dead in it —
the same failure mode T-2692 was built to close, and it recurred one directory over.

## Approach

Recover the four modules the way T-2806/T-2807 did: enumerate against the checkout that has
them, move the bytes through a **TermLink session rooted there** (T-559), scan before
committing, then commit here where the narrowed `.gitignore` permits tracking.

Then correct the drift checker so `web/blueprints/` fires. Deliberately **not** all of `web/`:
templates and static assets genuinely are cosmetic-if-missing, and widening the firing class to
the whole tree would trade a false negative for a pile of false positives — the fatigue failure
this session keeps finding.

**Prove by outcome:** Watchtower must actually serve, not merely have its files present.

## Scope boundary

Recovers four modules and narrows one classification. Does **not** re-vendor (T-2705's remedy,
theirs to run). Does **not** widen firing to `docs/` or to `web/` templates and static files.
Does **not** patch any vendored web code — every byte committed is a byte that already runs on
the main checkout.

## Acceptance Criteria

### Agent
- [x] The missing modules are present here and **tracked** — scope grew from 4 to **39** on
      the evidence (4 blueprints, 17 templates, 11 fonts, 6 JS, 1 CSS)
- [x] They were secret-scanned before commit — 0 hard findings; 1 soft (a LAN IP already
      documented throughout this repo)
- [x] `check-framework-tracking-drift.sh` fires on untracked `web/**/*.py` **and**
      `web/templates/**`, and still does NOT fire on `docs/` or `web/static/**`
- [x] Its fixtures cover both new firing cases and the still-informational ones, plus an
      explicit guard that the static assets stay quiet after the widening — **13/13 pass**
- [x] **Proven by outcome:** Watchtower serves — `/` **200**, `/tasks` **200**,
      `/inception` **200**, and all nine operator links verified 200 individually
- [x] CLAUDE.md's T-2689 section is corrected, including why `web/static/**` stays
      informational — the boundary is as load-bearing as the firing rule

## Verification

# All four modules are tracked, not merely present.
test -n "$(git ls-files .agentic-framework/web/blueprints/bvp.py)"
test -n "$(git ls-files .agentic-framework/web/blueprints/designer.py)"
test -n "$(git ls-files .agentic-framework/web/blueprints/designer_api.py)"
test -n "$(git ls-files .agentic-framework/web/designer_registry.py)"
# The templates base.html includes are present — this is what took Watchtower from
# "will not start" to "starts and 500s on every route".
test -f .agentic-framework/web/templates/_pins.html
# Nothing under web/ is left untracked.
test -z "$(git status --porcelain .agentic-framework/web)"
# The drift checker still passes on both axes.
bash scripts/check-framework-tracking-drift.sh
# Fixtures pin the corrected classification.
bash tests/framework-tracking-drift-fixtures.sh

## Decisions

### 2026-08-20 — Fire on web MODULES and TEMPLATES; keep static assets informational

- **First chose:** blueprints only, reasoning that "a missing template or stylesheet is a
  degraded page".
- **Disproven within minutes.** Recovering the four blueprints took Watchtower from "will not
  start" to "starts and serves HTTP 500 on every route" — `TemplateNotFound: _pins.html`,
  because `base.html` includes it. A missing template is not a degraded page; it is no page.
- **Chose instead:** fire on `web/**/*.py` and `web/templates/**`; leave `web/static/**`
  informational.
- **Why the line sits there:** it is drawn from the two observed failure modes, not from
  taste. Blueprint missing → `ModuleNotFoundError` before the app binds a port. Template
  missing → 500 on every route extending `base.html`. Font or stylesheet missing → an ugly
  page that still works. Promoting all of `web/` would fire on 11 woff2 files, and a check
  that fires on cosmetics is one people learn to skip — which is exactly how the original
  over-broad "informational" call survived until it cost the whole app.
- **Recorded rather than quietly amended** because the first call was made from an armchair
  and the second from a stack trace, and that is the difference worth keeping.

### 2026-08-20 — Read the other branch before fixing an overlapping symptom

- **Chose:** Check T-2705 on `worktree-charter-review-2026-0814` first, and stay out of its half.
- **Why:** T-2800 exists because three agents independently implemented the same two fixes in
  one week. The collision checker flagged this overlap; ignoring its own output would be the
  clearest possible way to prove the tool useless. Their `lib/` half is already resolved here
  by T-2807, so the remaining work genuinely does not intersect.

## Reviewer Verdict (v1.5)

- **Scan ID:** R-5092f413
- **Timestamp:** 2026-08-20T17:59:00Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-20T17:58:58Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
