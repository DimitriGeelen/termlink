---
id: T-2805
name: "Episodic memory has no reader: 29 of 2259 files do not parse, and nothing would ever notice"
description: >
  Build a classifying parse check over .context/episodic/*.yaml. Inbound pickup filing (offset 18, 832-Workflow-designer) reports a generator escaping defect and notes the more valuable half is the blindness: nothing ever reads the store. Measured here: 29/2259 unparseable, four distinct corruption classes, oldest dead since March.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [governance, episodic-memory, g-019, g-063, directive-2]
components: []
related_tasks: [T-2800, T-2801, T-2561, T-2818, T-2231]
created: 2026-08-20T14:45:29Z
last_update: 2026-08-20T15:03:01Z
date_finished: 2026-08-20T15:03:01Z
---

# T-2805: Episodic memory has no reader

## Context

An inbound filing on `framework:pickup` (offset 18, from `832-Workflow-designer`) reports a
defect in the vendored episodic generator and — more usefully — names the half it did not
fix: *"an unread store cannot report its own corruption."*

Checked here for the first time. **29 of 2259 episodics do not parse.**

The generator bug is real and reproduces independently. `agents/context/lib/episodic.sh`,
`mine_git_timeline`, writes mined git subjects into a **double-quoted** YAML scalar while
escaping only `"`. Every other backslash arrives raw, and a double-quoted YAML scalar accepts
only a fixed escape set. The filing cited `\-` and `\|`; this repo adds three more:

```
T-1146   found unknown escape character '|'
T-1626   found unknown escape character '*'
T-1644   found unknown escape character '.'
T-836    found unknown escape character '['
```

Ordinary commit subjects — a regex class, an alternation, a glob, a filename — corrupt the
record permanently.

### Why it survived: three layers, each silent

**Write.** The `ScannerError` is raised once, to whoever was watching that terminal. The task
still completes and the file is still written.

**Audit.** The `episodic` section runs **hourly on cron** and checks only that the file
**exists** — it never opens it. A corrupt episodic passes as *"All completed tasks have
episodic summaries."*

**Read.** The one real consumer, `web/shared.py::get_episodic_tags`, ends its parse in:

```python
except yaml.YAMLError:
    continue
```

No log line, no counter, no degraded mode. The task simply has no tags and the page renders
fine. This is a Directive #2 violation in the plainest form the codebase has: a silent
failure, in the reader, of the store the reader exists to read.

So corruption is invisible at write time, at audit time, and at read time. The oldest
casualty here dates to March.

### The store holds more than one defect

Sorting the 29 by failure mode shows the filing described one class of four:

| class | n | what it is |
|---|---|---|
| `LEGACY-MARKDOWN` | 14 | a `summary:` line then a **markdown** body — an older generator |
| `LEGACY-MULTIDOC` | 8 | `---` frontmatter plus a body; `safe_load_all` reads it whole |
| `CORRUPT-ESCAPE` | 4 | the reported generator bug |
| `CORRUPT-OTHER` | 3 | an unquoted scalar opening with a backtick; two broken `decisions` blocks |

The two legacy classes have **fully intact content** and want a format migration. The seven
corrupt ones are damaged. That distinction decides the repair, so the check reports it.

## Approach

Ship the reader that was missing: `scripts/check-episodic-parse.sh`, a deploy-time / ad-hoc
check at the same tier as `check-cron-install-drift.sh` (T-2561), `check-task-id-collisions.sh`
(T-2800) and `check-pickup-deferred-freshness.sh` (T-2801).

**Assert the property the real consumer needs**, not a weaker one that is easier to pass:
every episodic must `yaml.safe_load` into a **mapping**. That is exactly what
`get_episodic_tags` does, so any file this check calls unreadable *is* being dropped by
Watchtower today. Files that parse to a non-mapping are included for the same reason — the
reader's `isinstance(edata, dict)` guard discards them just as quietly.

**All classes fire; classification steers the repair.** Suppressing the 22 legacy files as
"not really broken" would be false: they are exactly as invisible to the reader as the
corrupt ones. But a flat list of 29 would not tell an operator that 22 are a mechanical
migration and 7 are damage, so each class carries its own remediation line.

**Detect; never repair.** The check does not rewrite episodics. Regenerating a
`CORRUPT-ESCAPE` file *before* the vendored generator is fixed reproduces the same bytes, and
an auto-repair that silently re-emitted them would convert a visible backlog into an
invisible one — the exact trade this exists to reverse.

## Scope boundary

Delivers detection plus the upstream report. Does **not** fix the generator, the audit's
existence-only check, or `get_episodic_tags`'s swallowed exception — all three are vendored
(G-062), and a local edit is erased on the next re-vendor. Does **not** repair the 29 files;
that is a separate deliverable and the corrupt ones are blocked on the generator fix.

## Acceptance Criteria

### Agent
- [x] `scripts/check-episodic-parse.sh` exists, is executable, and carries the
      `# guard-layer: source` marker (T-2802) so it runs after merge
- [x] Asserts the real consumer's property: `safe_load` to a mapping, matching
      `get_episodic_tags`
- [x] Classifies each unreadable file as `CORRUPT-ESCAPE` / `CORRUPT-OTHER` /
      `LEGACY-MULTIDOC` / `LEGACY-MARKDOWN` / `NOT-A-MAPPING`, each with its own remediation
- [x] Fails **closed**: missing python3, missing PyYAML, or an absent directory exit **2**,
      never 0
- [x] Exit codes: 0 = all readable, 1 = one or more unreadable, 2 = tooling
- [x] `--json` carries `scanned`, `unreadable_count`, per-class counts, and `findings[]`
- [x] `--dir` / `EPISODIC_DIR` retarget the scan at a fixture tree (PL-213)
- [x] Run against this repo it reports the 29, split 4 / 3 / 8 / 14 across the classes
- [x] A fixture suite covers each class, both fail-closed paths, and a clean tree
- [x] The upstream filing is posted to `framework:pickup` with the reader defect
      (`shared.py` swallowed exception) and the audit's existence-only check named —
      the two the reporting project had not found
- [x] CLAUDE.md documents the check alongside its sibling deploy-time checks

## Verification

# The check exists, is executable, and declares guard-layer membership.
# No pipe into grep -q: under `set -o pipefail` that exits 141 when the pattern
# MATCHES, because grep closes the pipe and SIGPIPE kills sed (L-387). A
# temp file has no such failure direction, and mktemp keeps parallel runs apart.
test -x scripts/check-episodic-parse.sh
f=$(mktemp); sed -n '2p' scripts/check-episodic-parse.sh > "$f"; grep -q 'guard-layer: source' "$f"; rc=$?; rm -f "$f"; test $rc -eq 0
# Fixtures pass — every class, both fail-closed paths, and a clean tree.
bash tests/episodic-parse-check-fixtures.sh
# Fail-closed on an absent directory: exit 2, never 0.
bash scripts/check-episodic-parse.sh --dir /nonexistent-episodic-dir >/dev/null 2>&1; test $? -eq 2

## Decisions

### 2026-08-20 — Assert the consumer's property, not a friendlier one

- **Chose:** `safe_load` into a mapping — byte-identical to what `get_episodic_tags` does.
- **Why:** A check may only claim a file is readable if the code that reads it can read it.
  Accepting `safe_load_all` would pass 8 files that Watchtower drops today, which is a
  checker reporting on its own convenience rather than on the system.

### 2026-08-20 — Fire on the legacy classes too, but name them separately

- **Chose:** All 29 fire; the class carries the remediation.
- **Why:** Calling 22 files healthy because their content is intact would be false — they are
  invisible to the reader either way. But firing on 29 undifferentiated files invites exactly
  the fatigue the pipefail auditor (T-2818) documented, where a gate that blocks incorrectly
  teaches people to force past it. Classifying keeps the count honest and the response cheap.

### 2026-08-20 — Detect, never repair

- **Chose:** No auto-regeneration.
- **Why:** Regenerating a `CORRUPT-ESCAPE` file before the vendored generator is fixed
  reproduces the identical bytes, so an auto-repair would report success while changing
  nothing — turning a visible backlog into a silent one.

### 2026-08-20 — Report the two defects upstream that the filing had not reached

- **Chose:** File back on `framework:pickup` rather than only consuming the report.
- **Why:** The reporting project found the generator bug and correctly identified the
  blindness as the more valuable half, but did not have the reader defect
  (`except yaml.YAMLError: continue`) or the audit's existence-only check. Those are the
  mechanism. Sending them back is what makes the topic bidirectional rather than a sink —
  the G-063 failure the framework-pickup canary exists to prevent.

## Reviewer Verdict (v1.5)

- **Scan ID:** R-aef1a505
- **Timestamp:** 2026-08-20T15:03:04Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** yes
- **Findings:** none

- **Layer-1 escalations:** 1
  1. **destructive-action** (high) — Destructive operation in verification or AC
     - matched: `rm -f`

### 2026-08-20T15:03:01Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
