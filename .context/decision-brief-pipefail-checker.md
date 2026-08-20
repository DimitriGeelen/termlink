# Decision brief — which `check-verification-pipefail.sh` survives the merge

Two branches independently wrote a checker for the same defect (L-387: `cmd | grep -q PAT`
exits 141 when the pattern MATCHES under `pipefail`, so a P-011 verification line fails
precisely when it succeeds). Both are correct about the defect. One has to go.

Detected by axis C of `check-task-id-collisions.sh` — the case CLAUDE.md already records as the
worked example of duplicated work that a title-similarity heuristic misses entirely.

| | **ours** (T-2818, was T-2693) | **charter-review** (T-2775) |
|---|---|---|
| lines | 188 | 385 |
| detection | **wraps** `lib/reviewer/static_scan.py::detect_l387_sigpipe_risk` (6 refs) | **reimplements** the heuristic (0 refs) |
| acknowledgement | none — trends toward empty | `.context/checks/verification-pipefail-allowlist` |
| guard-layer marker | no | yes (`source --no-heartbeat`) |
| scoping | `--active-only` | `--tasks-dir` (repeatable), `--include-completed` |
| test seam | `--tasks-dir` / `--framework-root` | `--tasks-dir` / `--allowlist` |
| fail-closed | yes — exits 2 if the detector cannot be imported | yes |
| fixtures | 8 assertions | (theirs, not counted here) |

## The one difference that matters

**Ours delegates the heuristic; theirs owns it.**

Ours is 188 lines because it calls the framework's existing detector and confines itself to
scoping, exit codes and output. Its stated rationale: *"two copies of a subtle SIGPIPE
heuristic drift, and the copy that drifts is the one that quietly stops catching things."*

Theirs is 385 lines because it carries its own implementation, and buys real things with them:
a proper allowlist, the guard-layer marker so `run-guard-layer.sh` executes it, and finer
scoping.

## Recommendation: **take theirs, and port two things across**

I wrote ours and I still think its delegation argument is right in the abstract. Three
observations moved me:

1. **Theirs is already wired into the guard layer.** Main carries
   `### Running the guard layer — scripts/run-guard-layer.sh (T-2684)` and theirs has the
   `# guard-layer: source --no-heartbeat` marker; ours does not. Ours would need that added
   anyway, and a checker nobody runs is the PL-168 failure this repo names explicitly.

2. **The allowlist is load-bearing at this repo's scale.** The auditor's own finding was **150
   findings across 189 active task files**. Ours trends toward empty only if someone fixes all
   150; until then it is permanently red, and a permanently-red check is one people learn to
   skip — which is the argument ours makes *against* other checks.

3. **The delegation argument is weaker than it looks here.** `detect_l387_sigpipe_risk` lives
   in the **vendored** framework (`lib/reviewer/`), so "one copy" is only true until the next
   re-vendor changes it underneath us — the T-2812 problem. Depending on vendored internals is
   not obviously safer than owning 40 lines of regex we control.

**Port from ours if you take theirs:**

- the **fail-closed on import failure** — ours exits **2**, never 0, when the detector cannot
  load. Theirs is fail-closed too, but confirm it covers the *detector-missing* path
  specifically; "reports clean because it failed to load" is the disease this whole session has
  been about.
- the **`--active-only`** flag. Completed tasks are the best evidence of how widespread the
  shape is, which is why ours defaults to including them — but once you are *fixing*, scoping
  to `.tasks/active/` is what makes the list actionable.

**If you take ours instead**, add the guard-layer marker and an allowlist before it is useful,
which is most of the distance to theirs.

## Cost of the merge either way

Taking theirs removes `scripts/check-verification-pipefail.sh` and
`tests/verification-pipefail-check-fixtures.sh` (8 assertions) from this branch, and CLAUDE.md's
`### Verification-block pipefail auditor (T-2818, L-387 at repo scale)` section needs rewriting
to describe theirs. Taking ours means reconciling in the other direction.

Neither is hard. It is a judgement about which design you want to live with.
