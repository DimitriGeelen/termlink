# The voi_score gate (T-3174) — and the wiring you have to approve

## What it protects

An inception's `voi_score` (0..1) answers "how valuable is it to resolve this question at all",
and it ranks inceptions against each other and against ordinary work.

`.tasks/templates/inception.md` ships `voi_score: 0.5` pre-filled, and
`estimator.py::_score_inception_voi` maps **both** that default and an **absent** value to the
same internal `2`:

```python
voi = fm.get("voi_score")
if voi is None:  return 2, ["->2 (voi-absent-grandfathered)"]
score = int(round(voi_f * 5))     # round(0.5*5) == 2, banker's rounding
```

So "I judged this exactly middling" and "nobody ever looked at this" are indistinguishable
downstream. Measured 2026-09-26 across 245 inception tasks: **103 at the default, 141 absent, 1
with a considered value.** The field has discriminated once in the project's history, and the
other 244 land mid-rank *ahead* of work that was measured and scored low.

Peer project 832-Workflow-designer reported the same collision and the reason a softer fix will
not work: they put a warning comment directly above the field and **the figure did not move by
one.** A comment is not a gate.

## Why there is an override, and why it is not silent

The operator's instruction was explicit: a gate with no way through becomes friction, and
friction gets switched off. A switched-off gate protects nothing.

So `voi_score_waived: "<reason>"` in the task frontmatter clears it. Two deliberate constraints:

- **An empty reason does not waive.** `voi_score_waived: ""` and `voi_score_waived: true` both
  still block. A waiver without a reason is the unset field with extra steps.
- **Every waiver is logged** to `.context/checks/voi-waiver-ledger` with task id, reason and
  timestamp, append-only. A waiver is legitimate; a waiver nobody can count is not. This mirrors
  the framework's own Tier-2 model — situational authorization *with mandatory logging*.

## Why it fails OPEN

Unlike the static checks in this repo, which fail closed, this one exits 0 on an unreadable or
unparseable task file, a missing python3, or an unwritable ledger.

It guards a *ranking number*, not a safety property. Blocking the operator's work because this
script has a bug would be a worse outcome than the defect it prevents. That trade is the opposite
of the one `check-*.sh` makes, and it is made on purpose.

## Grandfathering

Only tasks created on or after `2026-09-26` are gated (`--cutoff`, or `FW_VOI_GATE_CUTOFF`).
244 of the 245 existing inceptions are in the offending state; gating them retroactively would
block a quarter of the backlog on day one and guarantee the gate got disabled.

## Running it

```bash
bash scripts/gate-inception-voi.sh --scan            # report every firing task
bash scripts/gate-inception-voi.sh --scan --json     # machine-readable
bash scripts/gate-inception-voi.sh --task <file>     # hook mode: non-zero == block
```

Exit 0 clear/waived/grandfathered/not-an-inception · 1 firing · 2 usage.
Fixtures: `bash tests/inception-voi-gate-fixtures.sh` (27 assertions, weighted to the override
cases, with a mutant that disables the reason requirement and must let an unreasoned waiver
through). It runs in CI as a guard-layer member.

## THE WIRING — not applied, needs your approval

Today the script **detects and reports**. It does not block, because blocking requires an entry
in `.claude/settings.json`, which is the root of every gate in this project. Editing that file as
a side-effect of building a feature is not something to do unasked — there is a
`check-settings-edit` hook watching it precisely because it is sensitive.

To make it blocking, add one entry under `hooks.PreToolUse`:

```json
{
  "matcher": "Write|Edit",
  "hooks": [
    { "type": "command",
      "command": "${CLAUDE_PROJECT_DIR}/scripts/gate-inception-voi.sh --task \"$CLAUDE_TOOL_FILE_PATH\"" }
  ]
}
```

Note what is different about this entry from every other one in that file: all the existing hooks
route through `${CLAUDE_PROJECT_DIR}/.agentic-framework/bin/fw hook <name>`, i.e. through vendored
code. Adding a hook *name* there would be a G-062 divergence deleted by the next re-vendor. This
one points at a script under `scripts/`, which is ours — so it survives a re-vendor, which is the
whole reason it is built this way.

**Before approving, know the cost:** it fires on Write/Edit against any new inception without a
considered value, including one you are part-way through drafting. The escape is one line in the
frontmatter, and it gets logged.

If you would rather not wire it at all, the scan mode still runs in CI on every push, so the
number stays visible — it just will not stop anyone.
