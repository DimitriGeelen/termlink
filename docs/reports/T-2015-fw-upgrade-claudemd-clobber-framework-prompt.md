# Framework-agent prompt — `fw upgrade` destroys consumer work on four surfaces (PL-124 / G-055)

**Operator: copy everything inside the fenced block below into a fresh
`/opt/999-AEF` session. The prompt is self-contained.**

> **Revision history.** First written 2026-06-06 against a single observation:
> "today's run lost 18 lines of CLAUDE.md". That framing was wrong by two orders
> of magnitude and named the wrong surface as the only one. Rewritten 2026-08-25
> after two controlled runs on a clean tree. The CLAUDE.md half now has a fix
> that is **proven by re-running the operation that caused the loss**; the other
> three surfaces do not, and that is the actual ask below.

---

```
PL-124 / G-055 — `fw upgrade` silently deletes shipped consumer work.

CLAUDE.md is the surface this was originally filed against and it is the
SMALLEST one. Measured by git, twice, on a clean tree, on /opt/termlink:

    .claude/commands/     497 deletions across 6 files
    scripts/             1561 deletions across 9 files
    .agentic-framework/   676 files modified (a full re-vendor)
    CLAUDE.md              40 lines

Both runs produced IDENTICAL counts on the same files, so this is
deterministic, not a race or a one-off bad state.

## What the deletions contained

Not stale scratch. Deliverables of tasks marked work-completed:

  * check-arc.md lost the ENTIRE T-2402 Stage 6 wake protocol — drain all
    unread topics, post a receipt per topic, reply-or-explicitly-decline,
    the hop-budget relay-loop circuit breaker, ack on the exact rail that
    rang you. T-2402's own AC cited that text as its evidence. `fw upgrade`
    deleted the deliverable and left the task closed and the AC ticked.

  * agent-handoff.md lost T-2295's delivery-confirming path (the
    confirm-by-default vs fire-and-forget distinction).
  * peers.md lost T-2091's --filter-capability / --with-capabilities.
  * be-reachable.md lost the --capabilities advertisement.
  * pulse.md lost per-hub failure-data handling ("surface it inline,
    never silently swallow").

Every one is a shipped feature whose only user-facing documentation is the
skill file that was deleted.

## Why nobody notices

The CLAUDE.md step at least prints "N line(s) ... are absent from the new
CLAUDE.md" and names a few. The skill and script sync print only:

    UPDATED  scripts/agent-send.sh (backup: .bak)

...and move on. No diff, no count, no warning. The operator sees a green
UPDATED line while 466 lines of their own work are removed from that one
file.

The recovery window is ONE upgrade wide. `.bak` is the only witness for
anything uncommitted, and the next upgrade overwrites `.bak`. Two upgrades
and the original is gone.

## REQUEST 1 — make the skill/script sync warn like the CLAUDE.md step does

Diff before replacing. Print the deletion count and the first few removed
lines, exactly as step 1 already does for CLAUDE.md. This is the single
highest-value change and it is small: the comparison already happens, it
is just not reported.

## REQUEST 2 — give the skills a safe zone, or stop shipping over them

CLAUDE.md survives because lib/upgrade.sh:1139-1206 splits it positionally:

    project_header = everything BEFORE "## Core Principle"   -> PRESERVED
    governance     = template from "## Core Principle" on    -> REPLACED

That split is why a fix was possible downstream at all. The skills have no
equivalent: whole-file replace, no header/governance boundary. A consumer
carrying project-specific skill content has NO protection except git.

We fixed our CLAUDE.md side by relocating the at-risk content above the
split, and PROVED it by re-running the upgrade:

    before fix: 40 lines lost
    after fix :  5 lines lost, all 5 verified still present above the split

Landed at dd6f2caea. Any consumer can copy that. Nobody can copy it for
skills, because there is no line to move things above.

## REQUEST 3 — a task's deliverable can be un-delivered while its ACs stay ticked

T-2402 is the demonstration. Its deliverable was vendored-template text;
a routine maintenance command deleted it; the task record still says
work-completed with the AC checked; no surface anywhere reports the
difference. This is the same class as the verification-gate work: the
record says done and the artefact is gone.

Worth a check that closed tasks whose evidence is a file path still have
that evidence present.

## REQUEST 4 — `--check` should say what it is about to overwrite

List the vendored files that differ from upstream and are about to be
replaced, so a consumer can see the loss BEFORE applying rather than
reconstructing it from a divergence register afterwards.

## AND A CORRECTION TO SOMETHING WE FILED EARLIER — PLEASE DO NOT BUILD IT

We previously reported (framework:pickup offset 42) that `fw update --check`
"offers a 116-version downgrade and calls it an update" (v1.6.145 ->
v1.6.29), and asked you to refuse a lower target without --force-downgrade.

THAT REQUEST WAS WRONG AND WOULD BE HARMFUL. Retracted at offset 44.

v1.6.29 is NEWER than v1.6.145. Decided by content, since string order
cannot decide: the 1.6.29 tree contains T-3110..T-3129, tasks absent from
the 1.6.145 tree entirely and in flight in 999-AEF at the time of writing.
Had the guard been built, every consumer would have been refused the
current framework until they passed --force-downgrade to move FORWARD.

VERSION is a resetting counter. Our own divergence register records
1.6.260 -> 1.6.160 -> 1.6.7 -> 1.6.295 -> 1.6.145 across five vendor
events, and `fw --version` on a vendored copy says it outright: "Commit:
(none — vendored copy; VERSION file is the only identity)".

If you want an ordering guarantee, ship a monotonic vendor-time field —
commit date or an incrementing vendor serial — and compare that. Until
then "Update available" is the honest string precisely because the tool
cannot know the direction. Request 4 above is the decision-useful output.

## Second correction, same class

We also reported that a re-vendor "deleted three of our four recorded
divergences". At least one of those was not deleted, it was SUPERSEDED:
our inline _resolve_hook_path went to zero occurrences because upstream
replaced it with lib/hook_paths.py + lib/hook_portability.py (T-2468
generalizing T-2465/OBS-080, updated by T-2709), which re-derive the
project root from the per-call stdin cwd and are immune to the T-2446
daemon-poison class ours was not. Strictly better. Post-upgrade doctor
gained "27 hooks, all portable" and "23 hook(s) resolve from foreign CWD"
— the second being the exact worktree symptom our local patch existed for,
now covered upstream by a test.

Flagging it because the same reasoning error produced both wrong reports:
a marker going to zero has two causes that look identical to grep —
deleted, or superseded — and only reading the replacement separates them.
```

---

## Operator notes (NOT part of the prompt above)

**What is already fixed downstream and needs nothing from upstream:**

| surface | status | evidence |
|---|---|---|
| CLAUDE.md | **fixed here** | `dd6f2caea` — relocation above `## Core Principle`; re-run gave 40 → 0 changed |
| `.gitignore` allowlist | **fixed here** | `0bd97f38a` — `tools/`, `vendor/`, `status-transitions.yaml` re-included |
| `.claude/commands/` | **not fixed** | git-tracked, so restorable; no upstream protection |
| `scripts/` | **not fixed** | git-tracked, so restorable; no upstream protection |

**The one-command check any consumer can run before their next upgrade:**

```
for d in .agentic-framework/*/; do f=$(find "$d" -type f ! -name '*.pyc' | head -1); [ -n "$f" ] && git check-ignore -q "$f" && echo "DROPPED: $d"; done
```

If anything prints, git is silently discarding framework code that `bin/fw` executes.

**Before every upgrade from here on:** snapshot, upgrade, snapshot, diff. One clean
run proves nothing about the next — our first run did not re-vendor and our second did.
