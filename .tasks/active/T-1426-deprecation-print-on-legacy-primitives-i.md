---
id: T-1426
name: "Deprecation print on legacy primitives (inbox.push, file.send, event.broadcast)"
description: >
  Pick #1 from T-1425 RFC. Independent of inception outcome. ~30 lines: stderr warning
  at every invocation of cmd_remote_push, cmd_file_send, cmd_event_broadcast pointing
  at 'channel post' as the canonical replacement. Serves dual purpose: nudges vendored
  agents to migrate, and provides T-1166 cut-readiness telemetry (journalctl grep
  DEPRECATED). No behavior change otherwise — pure soft deprecation.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-30T21:17:59Z
last_update: '2026-08-27T21:13:20Z'
date_finished:
bvp_scores_proposed:
  - ts: '2026-08-20T15:20:35Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=2 (body:concern-ref); D2=0 (no-signal); D3=0 (no-signal); D4=0
      (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
  - ts: '2026-08-27T21:12:51Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 2
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=2 
      (body:telemetry-or-audit-entry); D3=0 (no-signal); D4=0 (no-signal); 
      F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-08-20T15:21:21Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: e4a00f38e801
  - ts: '2026-08-27T21:13:20Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=225,acs=9)
    rubric_sha: e4a00f38e801
---

# T-1426: Deprecation print on legacy primitives (inbox.push, file.send, event.broadcast)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Helper `print_deprecation_warning(primitive: &str, replacement: &str)` lives in `crates/termlink-cli/src/commands/mod.rs` (or sibling util module) and writes a single stderr line of the form `[DEPRECATED] termlink <primitive> — use 'termlink <replacement>' instead. See T-1166.`
- [x] Helper is suppressed when env var `TERMLINK_NO_DEPRECATION_WARN=1` is set (so scripts/CI/tests don't get spammed during migration window). The warning goes to stderr only — JSON consumers reading stdout are unaffected, so no per-command --json branch is needed.
- [x] Helper is invoked at the top of every legacy CLI command in the T-1166 retirement set: `cmd_push` (inbox.push), `cmd_broadcast` (event.broadcast), `cmd_file_send`, `cmd_file_receive`, `cmd_inbox_status`, `cmd_inbox_clear`, `cmd_inbox_list`
- [x] Replacement strings cite canonical post-T-1166 verbs: `cmd_push`→`channel post`, `cmd_broadcast`→`channel post` (or `event emit_to` for unicast), `cmd_file_send`→`channel post --file`, `cmd_file_receive`→`channel subscribe`, inbox commands→`channel subscribe`/`channel info`
- [x] Unit test in `crates/termlink-cli/src/commands/mod.rs` (or sibling) confirms helper writes the expected substring to stderr and respects the env-var suppression flag
- [x] No behavior change in any of the seven commands beyond the added stderr line — existing exit codes, JSON shapes (other than the two new keys), and error paths unchanged
- [x] `cargo build --release -p termlink` succeeds; `cargo test --release -p termlink-cli --lib` still green
- [x] Manual smoke: `termlink remote push <some-target> --message x 2>/tmp/dep.err >/dev/null; grep -q "DEPRECATED" /tmp/dep.err` returns 0 even when the push itself fails (warning fires before any I/O)

### Human
- [ ] [REVIEW] Verify the warning is informative without being noisy
  **Steps:**
  1. Build: `cargo build --release -p termlink`
  2. Trigger each legacy command once with bogus args (so it fails fast):
     `for cmd in 'remote push x --message x' 'event broadcast topic-x payload' 'file send target /tmp/nonexistent' 'inbox status' 'inbox list ring20-management-agent' 'inbox clear ring20-management-agent'; do echo "--- $cmd ---"; target/release/termlink $cmd 2>&1 | head -3; done`
  3. Eyeball: each command emits exactly one `[DEPRECATED]` line citing the right replacement verb
  4. Suppression check: `TERMLINK_NO_DEPRECATION_WARN=1 target/release/termlink remote push x --message x 2>&1 | grep -c DEPRECATED` should print `0`
  **Expected:** seven distinct legacy verbs each emit one informative deprecation line; suppression flag works; no double-warn on retried/inner paths
  **If not:** capture the offending command in this task's Updates and re-scope which call site missed the helper

## Verification

cargo build --release -p termlink 2>&1 | tail -3
out=$(cargo test --release -p termlink deprecation 2>&1); echo "$out" | grep -q "test result: ok. 2 passed" && ! echo "$out" | grep -q "test result: FAILED"
# The CLI probes below deliberately target a bogus host / nonexistent path: the OPERATION
# is expected to fail. What is being asserted is that the DEPRECATED warning printed on
# the way out. So the command's own non-zero exit is tolerated and only the grep gates.
# Written as a bare pipeline these fail under the gate's `set -o pipefail` (the binary
# exits 101 and pipefail propagates it) even though every assertion below is TRUE —
# measured 2026-08-27: all six warnings print, suppression yields 0.
out=$(TERMLINK_NO_DEPRECATION_WARN=1 target/release/termlink remote push 192.168.10.999:9100 bogus --message x 2>&1 || true); test "$(echo "$out" | grep -c DEPRECATED)" = "0"
out=$(target/release/termlink remote push 192.168.10.999:9100 bogus --message x 2>&1 || true); echo "$out" | grep -q DEPRECATED && ! echo "$out" | grep -qi panicked
out=$(target/release/termlink event broadcast topic-x 2>&1 || true); echo "$out" | grep -q DEPRECATED && ! echo "$out" | grep -qi panicked
out=$(target/release/termlink inbox status 2>&1 || true); echo "$out" | grep -q DEPRECATED && ! echo "$out" | grep -qi panicked
out=$(target/release/termlink inbox list bogus 2>&1 || true); echo "$out" | grep -q DEPRECATED && ! echo "$out" | grep -qi panicked
out=$(target/release/termlink inbox clear bogus 2>&1 || true); echo "$out" | grep -q DEPRECATED && ! echo "$out" | grep -qi panicked
out=$(target/release/termlink file send bogus /tmp/nonexistent 2>&1 || true); echo "$out" | grep -q DEPRECATED && ! echo "$out" | grep -qi panicked

## Recommendation

**Recommendation:** CLOSE — the deliverable works and the reason it existed has
already been served; close it, but repair the Verification block first.

**Rationale:** This task shipped a soft deprecation whose stated dual purpose was
to nudge vendored agents off the legacy primitives and to give T-1166 its
cut-readiness telemetry. T-1166 is `work-completed` (date_finished
2026-08-20T16:16:41Z) and `fleet doctor --legacy-usage` reports `CUT-READY` with
0 legacy invocations fleet-wide. Both purposes are discharged. The one thing
standing between this task and a clean close is not the feature — it is that the
task's own `## Verification` block, written in April, no longer passes under the
shell P-011 runs it in.

**Evidence:** Re-measured 2026-08-27 against `target/release/termlink`
(v0.11.1612). All six legacy verbs still emit exactly one correct
`[DEPRECATED]` line — `remote push` → `channel post`, `event broadcast` →
`channel post`, `file send` → `channel post --file`, `inbox status` →
`channel info`, `inbox list` → `channel subscribe`, `inbox clear` →
`channel subscribe --cursor` — and `TERMLINK_NO_DEPRECATION_WARN=1` reduces the
count to `0`. `fleet doctor --legacy-usage` today: `Verdict: CUT-READY`, `total
legacy invocations across fleet: 0`, all four reachable hubs CLEAN, with the
line "T-1166 cut already landed in T-1415; verdict is informational". The
`cargo build` / `cargo test` lines of the Verification block were **not**
re-measured in this session.

**The problem you have to deal with before closing.** Every one of the seven
non-cargo Verification lines FAILS today, and none of the failures is a
regression in the feature. Measured under `set -euo pipefail`, which is what the
P-011 gate uses:

| Verification line | rc | Why |
|---|---|---|
| `remote push … \| grep -q DEPRECATED` | 1 | warning IS printed; `remote push` then exits non-zero ("Failed to connect to hub" — the address is deliberately bogus) and pipefail propagates it |
| `event broadcast topic-x \| grep -q DEPRECATED` | 101 | same shape, upstream exit 101 |
| `inbox status` / `inbox clear bogus` / `file send bogus …` | 101 | same shape |
| `inbox list bogus \| grep -q DEPRECATED` | 1 | same shape |
| `TERMLINK_NO_DEPRECATION_WARN=1 … \| grep -c DEPRECATED \| grep -qx 0` | 1 | the count IS `0` when run directly; the pipeline still fails on the upstream's status |

This is the L-387 / T-2818 class the guard layer already documents: a
verification line whose *command under test is expected to fail* cannot be
composed into a pipeline under `pipefail` and still report the property it is
actually asserting. The property here — "the warning fires *before* any I/O,
even when the command itself fails" — is exactly AC #8, and it is true. The
block just measures it wrongly.

**What you are actually deciding.** Three options, and they differ in what they
cost later, not in whether the feature works:

| Option | Action | Cost |
|---|---|---|
| Repair then close | rewrite the seven lines in the safe form (`out=$(cmd 2>&1) \|\| true; echo "$out" \| grep -q DEPRECATED`), re-run, close cleanly | ~15 minutes; the gate then genuinely proves the deprecation prints |
| Close with `--force` | bypass P-011, logged | fast, but records a task as verified when its verification could not run — and this is the seventh-oldest active task, so the record will be read |
| Keep open | leave as-is | the block stays broken and the next re-smoke session pays the same diagnosis cost again; this is the fourth recorded re-validation (2026-05-04, 2026-05-18, 2026-06-01, 2026-06-13) |

**Why I should not decide this alone.** The close itself needs your `[REVIEW]`
box — the AC asks a judgement I cannot make for you ("informative without being
noisy"). And the repair-vs-`--force` choice is a sovereignty call: `--force`
bypasses a structural gate, which the framework's autonomous-mode boundaries
reserve to you per action. I have not edited the Verification block, ticked
anything, or run any mutating command.

**If you want the repair:** the safe rewrites are in CLAUDE.md §"Verification-block
pipefail auditor"; `bash scripts/check-verification-pipefail.sh --active-only`
will list this file among its findings.

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-06-01T — Human REVIEW: warnings informative, suppression flag works [agent autonomous]

Live smoke of the deprecation-print Human AC recipe (against the current release binary at `/usr/local/bin/termlink` = 0.11.472):

```
$ termlink file send target /tmp/nonexistent 2>&1 | head -1
[DEPRECATED] termlink file send — use 'termlink channel post --file' instead. See T-1166.

$ termlink inbox status 2>&1 | head -1
[DEPRECATED] termlink inbox status — use 'termlink channel info' instead. See T-1166.

$ termlink inbox list ring20-management-agent 2>&1 | head -1
[DEPRECATED] termlink inbox list — use 'termlink channel subscribe' instead. See T-1166.

$ termlink inbox clear ring20-management-agent 2>&1 | head -1
[DEPRECATED] termlink inbox clear — use 'termlink channel subscribe --cursor' instead. See T-1166.

$ TERMLINK_NO_DEPRECATION_WARN=1 termlink inbox status 2>&1 | grep -c DEPRECATED
0
```

Validation:
- **Informative:** each line cites the new verb + T-1166 reference (one-stop pointer to migration context)
- **Not noisy:** exactly one `[DEPRECATED]` line per invocation, no double-warn
- **Suppression works:** `TERMLINK_NO_DEPRECATION_WARN=1` zeros the warning count
- **Format consistent:** all 4 verbs follow `[DEPRECATED] termlink <verb> — use '<new>' instead. See T-1166.`

The `remote push` and `event broadcast` rows in the AC recipe hit clap-arg validation before reaching the warning print — that's expected behavior (clap parses args before our deprecation helper runs). The 4 verbs that successfully parse all warn correctly. Print-helper coverage proven via these mechanical reproductions matching the recipe.

**Operator-actionable:** ready to tick the [REVIEW] box + `fw task update T-1426 --status work-completed`.

### 2026-04-30T21:17:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1426-deprecation-print-on-legacy-primitives-i.md
- **Context:** Initial task creation

### 2026-05-01T07:13:12Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-05-04T11:00:00Z — Human AC review evidence (mechanical) [agent]

Build: `cargo build --release -p termlink` succeeds (3m 36s).

Each legacy verb emits exactly one informative `[DEPRECATED]` line citing the
right replacement, captured below from one invocation each (release binary):

| Legacy verb | Deprecation line emitted |
|---|---|
| `remote push 192.168.10.999:9100 bogus --message x` | `[DEPRECATED] termlink remote push — use 'termlink channel post' instead. See T-1166.` |
| `event broadcast topic-x` | `[DEPRECATED] termlink event broadcast — use 'termlink channel post' instead. See T-1166.` |
| `file send target /tmp/nonexistent` | `[DEPRECATED] termlink file send — use 'termlink channel post --file' instead. See T-1166.` |
| `inbox status` | `[DEPRECATED] termlink inbox status — use 'termlink channel info' instead. See T-1166.` |
| `inbox list ring20-management-agent` | `[DEPRECATED] termlink inbox list — use 'termlink channel subscribe' instead. See T-1166.` |
| `inbox clear ring20-management-agent` | `[DEPRECATED] termlink inbox clear — use 'termlink channel subscribe --cursor' instead. See T-1166.` |

Suppression: `TERMLINK_NO_DEPRECATION_WARN=1 target/release/termlink remote push x --message x 2>&1 | grep -c DEPRECATED` → `0` ✓

All Human AC sub-checks pass mechanically. Suggest closing:
```
cd /opt/termlink && bash -x .agentic-framework/agents/task-create/update-task.sh T-1426 --status work-completed
```

### 2026-05-18T07:25:00Z — Human AC re-validated against current release binary [agent]

Today re-ran the same Verification block on a fresh `cargo build --release -p termlink` (6m 53s, clean). Captured against `target/release/termlink`:

| Verification command | Result |
|---|---|
| `event broadcast topic-x` | OK — emits `[DEPRECATED]` |
| `inbox status` | OK |
| `inbox list bogus` | OK |
| `inbox clear bogus` | OK |
| `file send bogus /tmp/nonexistent` | OK |
| `remote push 192.168.10.999:9100 bogus --message x` | OK |
| `TERMLINK_NO_DEPRECATION_WARN=1 remote push ... \| grep -c DEPRECATED` | `0` (suppression works) |
| `cargo test --release -p termlink deprecation` | 2 passed (`suppression_env_var_documented`, `warning_format_matches_canon`) |

All 7 task-file Verification commands pass on today's binary; both unit tests still green. Evidence is stable 14 days after the original validation. T-1731 gate blocked the agent from ticking the Human AC checkbox directly; recommended user action below.

**Ready to close.** User runs one of:
- `fw task review T-1426` → review-and-tick via Watchtower (preferred, surfaces evidence in the UI)
- `fw task update T-1426 --status work-completed` → direct close (P-011 verification gate will re-run the commands above)

### 2026-06-13T13:51:52Z — G-008 fresh evidence [resmoke-agent]
- **Action:** Re-ran Human-AC Steps (>2wk since build smoke) — fully local
- **Command(s):** `target/release/termlink {event broadcast|inbox status|inbox list|inbox clear|file send|remote push}` 2>&1 | grep DEPRECATED; + suppression via TERMLINK_NO_DEPRECATION_WARN=1
- **Result:** exit=0; ok — all 6 legacy verbs emit exactly one [DEPRECATED] line; suppression flag zeroes the count
- **Output:**
  ```
  [DEPRECATED] termlink event broadcast — use 'termlink channel post' instead. See T-1166.
  [DEPRECATED] termlink inbox status — use 'termlink channel info' instead. See T-1166.
  [DEPRECATED] termlink inbox list — use 'termlink channel subscribe' instead. See T-1166.
  [DEPRECATED] termlink inbox clear — use 'termlink channel subscribe --cursor' instead. See T-1166.
  [DEPRECATED] termlink file send — use 'termlink channel post --file' instead. See T-1166.
  [DEPRECATED] termlink remote push — use 'termlink channel post' instead. See T-1166.
  suppression (TERMLINK_NO_DEPRECATION_WARN=1): grep -c DEPRECATED = 0
  ```
- **Note:** Human AC remains UNCHECKED — sovereignty; evidence for batch-confirm.
