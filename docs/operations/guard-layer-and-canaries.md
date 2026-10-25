# Guard layer, canaries and static checks

What each TermLink canary and source-level static check catches, why it exists, its flags,
test seams and fixtures, and what to do when it fires.

**Moved out of CLAUDE.md by T-3246** (T-2989 GO step 3). CLAUDE.md is preloaded into every
session and an `@path` import is inlined at launch (T-3244 measured it), so these sections live
here as a plain file. CLAUDE.md keeps a one-line-per-guard index pointing here. The sections
below are verbatim from CLAUDE.md `## CI / Release Flow` as of the commit before T-3246.

### Mirror drift canary (T-1140 + T-1696, G-058 prevention)

The OneDev → GitHub mirror can fail silently (token expiry, OneDev job-runner
issue, GitHub auth change). G-058 documents a 16-day silent failure where
`v0.10.0` / `v0.11.0` / `v0.11.1` release tags all missed GitHub. A daily
cron runs `scripts/check-mirror-freshness.sh --quiet` (see
`.context/cron/release-mirror-canary.crontab`) and appends to
`.context/working/.release-mirror-canary.log`. The canary checks BOTH branch
HEAD drift AND tag drift (the most-recent local tag must exist on GitHub) —
the second check is what catches the failure mode where branches mirror but
tags don't. Empty log = healthy. Any entry = OneDev → GitHub mirror needs
operator restoration (T-1695-style task: inspect OneDev job log, rotate
`github-push-token` if expired, re-fire the mirror job). Ad-hoc check:
`bash scripts/check-mirror-freshness.sh` (exit 0 = synced, 1 = drift,
2 = network/tooling error).

**Root-cause diagnosis on drift (T-2052).** When drift is detected, the canary
now scans the `github_head..origin_head` commit range for any blob ≥100MB —
GitHub's per-file pre-receive limit (GH001). If found, the canary's drift
output surfaces the offending sha+path+size and a cleanup hint instead of just
"drift". This catches the G-058 ROOT CAUSE class (288MB fw-vec-index.db
committed 2026-05-25 silently rejected for 14 days), not just the symptom.
Empty `oversize_blobs` in `--json` output means drift has a different cause
(token, network, OneDev job-runner) — operator should still inspect the
OneDev job log per the T-1695 playbook. Pair with the active pre-commit
large-file gate (T-1845, 10 MiB BLOCK) which prevents new oversize blobs from
entering the history in the first place; `fw git install-hooks` activates it
if you see `secret-scan: scanner not found (skipping)` in commit output
(indicates the scanner scripts are non-executable — T-2052 install-time
chmod gap).

### Substrate preflight canary (T-2154 + T-2158 + T-2160)

The substrate has a symmetric deploy-time canary that catches PL-021
runtime_dir regressions, missing hubs.toml, and dead be-reachable state
files BEFORE clients hit hours-later auth-mismatch. Daily cron runs
`scripts/substrate-preflight.sh --quiet` (see
`.context/cron/substrate-preflight-canary.crontab`) and appends to
`.context/working/.substrate-preflight-canary.log`. Empty log = healthy.
Any entry = the host's substrate environment regressed (typically: a
reboot wiped /tmp, or systemd-tmpfiles override added post-install).
Each log entry is framed `=== <ts> ===\n<full preflight output>\n---`
for forensic clarity. Ad-hoc check: `/preflight` (skill, T-2158) or
`bash scripts/substrate-preflight.sh` (exit 0 PASS / 1 WARN / 2 FAIL).
Pair with the mirror-drift canary above — both follow the same
"empty-log = healthy" convention.

### Framework-pickup canary (T-2231, G-063 prevention)

The `framework:pickup` hub topic receives bug-reports / feature-proposals /
RCAs filed by peer projects (ring20, CPN, etc.), but termlink has **no
automatic consumer** of that topic (G-063). G-063 surfaced when a
high-severity ring20 RCA (T-2229: cross-hub "federation" + heartbeat-freeze)
sat ~27h unprocessed because nothing surfaced it. A daily cron runs
`scripts/check-framework-pickup-freshness.sh --quiet` (see
`.context/cron/framework-pickup-canary.crontab`) and appends to
`.context/working/.framework-pickup-canary.log`. Empty log = healthy. Any
entry = there are filings on the topic newer than the last-acked offset.
Workflow on firing: triage the surfaced filings (file tasks / reply on the
peer's hub), then run `bash scripts/check-framework-pickup-freshness.sh --ack`
to bump the marker (`.context/working/.framework-pickup-canary.seen-offset`)
so they go quiet. Ad-hoc check: `bash scripts/check-framework-pickup-freshness.sh`
(exit 0 = nothing new, 1 = unprocessed filings, 2 = tooling error).
`/canaries` auto-discovers the log. Severity is shown as a best-effort
`[HIGH]` hint sniffed from the payload body — it is an annotation, NOT the
firing gate (the gate is "newer than last-acked"); gating on a parsed
severity field would be fragile given the free-form YAML payloads (T-2225
false-positive lesson).

**Own filings do not fire (T-2816).** The topic is bidirectional — termlink is a
prolific *poster* to it, not only a reader — so the canary attributes each filing
via `metadata.from_project` → `source_project` → `agent_id` and excludes anything
matching `FW_PICKUP_SELF_PROJECT` (default `010-termlink`) from the firing set.
Without this, filing a bug report upstream made the canary fire at us, and the
`--ack` needed to clear that echo *also* acked any genuine inbound filing that
landed in between — reintroducing the exact G-063 miss the canary exists to
prevent. Suppressed filings are always counted and reported (`N own filing(s)
… not counted`), so a quiet canary is never ambiguous between "nothing inbound"
and "the filter ate something". Attribution is a **constant**, deliberately not
`basename $PROJECT_ROOT` — a path-derived slug is wrong inside a git worktree
(that is the T-2815 defect filed upstream). **Unknown attribution still fires**:
we cannot prove an unattributed filing is ours, and a false fire is cheap while
a false silence is the whole point of G-063. Test seam
`FW_PICKUP_TEST_NDJSON=<file>` feeds canned `channel subscribe --json` NDJSON;
fixtures: `bash tests/pickup-canary-selffilter-fixtures.sh`. Pair with the mirror-drift and substrate-preflight
canaries above — all three follow the same "empty-log = healthy" convention.

### Frozen-husk canary (T-2239, G-019 prevention for T-2230/T-2235)

The T-2230 + T-2235 arc fixed the *symptom* of "frozen husk" sessions: a live
`termlink register` (or `register --self`) process that registers once and never
advances its `heartbeat_at`. Before that arc the framework was structurally
**blind** to the class — a live process could sit forever with a stale heartbeat
and nothing surfaced it (G-019: fix the symptom, then ask "why was the framework
blind?"). A daily cron runs `scripts/check-frozen-husk-freshness.sh --quiet` (see
`.context/cron/frozen-husk-canary.crontab`) and appends to
`.context/working/.frozen-husk-canary.log`. Empty log = healthy. Any entry =
one or more LIVE termlink processes under the local `runtime_dir` have a
heartbeat older than the threshold (default 600s ≈ 20 missed 30s beats).

The canary walks `$TERMLINK_RUNTIME_DIR/sessions/*.json`; a "frozen husk" is a
registration whose pid is alive **AND** confirmed to be a termlink process (via
`/proc/<pid>/cmdline` — guards against pid-recycle false positives) **AND** whose
`heartbeat_at` is stale beyond the threshold. Dead-pid and recycled-pid
registrations are counted as orphan cruft (informational, non-firing) — they are
a cleanup class, not the heartbeat bug.

**Two husk classes (T-2240).** Each husk is classified by its registered
`termlink_version` against the fix threshold (>= `0.11.1359`, tunable via
`FROZEN_HUSK_FIX_VERSION`):
- **REGRESSION** — binary HAS the fix yet the heartbeat froze anyway. The
  alarming case the canary exists to catch; file a bug task.
- **pre-fix** — binary predates the fix (e.g. `v0.9.0`) or version is unknown.
  A frozen heartbeat is EXPECTED; remediation is a binary upgrade (a known
  upgrade-backlog, not an incident).

The **daily cron runs with `--regressions-only`**, so the log accumulates ONLY
on genuine regressions — pre-fix husks (old binaries still in the field) do not
spam it, keeping "empty log = healthy" meaningful during a fleet upgrade. The
default (no flag) fires on ANY husk (back-compat with T-2239). Ad-hoc check:
`bash scripts/check-frozen-husk-freshness.sh` (exit 0 = healthy / no firing,
1 = firing husk(s), 2 = tooling error); add `--regressions-only` to fire only on
post-fix regressions, `--json` for scripting (carries `class` per husk +
`regression_count` / `prefix_count`), `--threshold-secs N` to tune staleness.
Operator action: a REGRESSION is a genuine T-2230/T-2235 regression (bug task);
a pre-fix husk wants a binary upgrade to >= `0.11.1359` + re-register, or
terminate + `termlink deregister <id>`. `/canaries` auto-discovers the log. Pair
with the mirror-drift, substrate-preflight, and framework-pickup canaries above —
all four follow the same "empty-log = healthy" convention.

### Topic-growth canary (T-2252, arc-002 R2 sweep-cron guard)

R2 (T-2245) bounds high-rate topics like `agent-presence` via `channel
set-retention latest-per-cv-key` + a periodic `channel sweep` — but the bus runs
**no background sweep thread** (T-1155: enforcement is explicit, never implicit),
so `sweep` depends on an operator **cron that may never fire**. If it doesn't, the
topic regrows silently — a T-1991 recurrence (the original silent agent-presence
bloat) with nothing to surface it. This is the same "framework relies on
out-of-band hygiene that may never run" class T-2251 closed for the audit log
(sibling to PL-168). A daily cron runs
`scripts/check-topic-growth-freshness.sh --quiet` (see
`.context/cron/topic-growth-canary.crontab`) and appends to
`.context/working/.topic-growth-canary.log`. Empty log = healthy.

The canary reads `termlink channel list --json` and FIRES (exit 1) when a
**watched** high-rate topic (default `agent-presence`, `agent-listeners-*`,
`agent-conv-*`, `dm:*` — tunable via `TERMLINK_GROWTH_WATCH_PATTERNS`) exceeds the
threshold (default 5000, `--threshold N`). Operator-durable topics
(`channel:learnings`, `policy-decisions`, `framework:pickup`, `broadcast:global`)
are **excluded** — they are intentionally `Forever` (mirrors the T-2057 audit §5 /
retention-reset runbook §1 exclusions). Each firing topic's `retention.kind`
selects the remediation: `forever` ⇒ retention was never set (run `set-retention
latest-per-cv-key` + `sweep` per `docs/operations/agent-presence-retention-reset.md`
§3); a **bounded** policy with a large count ⇒ the periodic `sweep` cron itself is
not firing (check it's installed). Ad-hoc check:
`bash scripts/check-topic-growth-freshness.sh` (exit 0 = healthy / no firing,
1 = a watched topic over threshold, 2 = tooling error / hub unreachable); add
`--json` for scripting, `--hub ADDR` to target a specific hub. Test hook
`TERMLINK_GROWTH_TEST_JSON=<file>` feeds canned `channel list` JSON for
hub-independent verification (PL-213). `/canaries` auto-discovers the log. Pair
with the mirror-drift, substrate-preflight, framework-pickup, and frozen-husk
canaries above — all five follow the same "empty-log = healthy" convention.

### Task-finalization canary (T-2290, G-066 prevention)

G-066: T-2203 (CTL-028) found **157 tasks** in `.tasks/completed/` whose
frontmatter `status` still said `started-work` with empty `date_finished` —
they were archived into `completed/` WITHOUT going through the finalize routine
(`fw task update --status work-completed`, which sets BOTH `status` and
`date_finished`). Several shared identical move-commit timestamps, the signature
of a bulk `git mv` / migration that skipped finalization. T-2203 repaired the
157 existing files, but the MECHANISM that lands tasks in `completed/` without
finalizing them is unaddressed and **will recur on the next bulk move** — and
the framework was blind to it (the duplicate-ID audit check passes regardless of
`status`). A daily cron runs `scripts/check-task-finalization-freshness.sh
--quiet` (see `.context/cron/task-finalization-canary.crontab`) and appends to
`.context/working/.task-finalization-canary.log`. Empty log = healthy.

The canary scans every `completed/*.md` and FIRES (exit 1) on any task whose
`status != work-completed` — the finalization-bypass class. A **softer**,
non-firing class is a task that IS `work-completed` but has empty
`date_finished` (the finalize routine half-ran, e.g. the PL-134
inception-auto-finalize path); these print as informational by default and fold
into the firing set with `--strict`. Ad-hoc check:
`bash scripts/check-task-finalization-freshness.sh` (exit 0 = healthy,
1 = a bypass detected, 2 = tooling error); add `--json` for scripting,
`--strict` to also fire on empty `date_finished`, `--tasks-dir P` to point at a
different `.tasks` root. Operator action on firing: repair each flagged task
(`fw task update <id> --status work-completed`), then root-cause the move path
that skipped finalize. `/canaries` auto-discovers the log. Pair with the
mirror-drift, substrate-preflight, framework-pickup, frozen-husk, and
topic-growth canaries above — all six follow the same "empty-log = healthy"
convention.

### Unconfirmed-delivery canary (T-2295, arc-003 reliable-comms V3b, G-063 prevention)

RC3b made delivery-confirmation observable: `channel post --await-ack` (T-2286)
writes a durable obligation row to `~/.termlink/awaiting_ack.sqlite`, and
`channel awaiting-ack` (T-2287) surfaces every send still waiting for a recipient
ack — INCLUDING rows retained after their retry loop was exhausted. Those
exhausted rows are the **"sent-but-never-confirmed"** class: the exact failure
G-063 named (`framework:pickup` at 36-sent / 0-received — a write-only sink nobody
noticed). Nothing surfaces them on its own. A daily cron runs
`scripts/check-unconfirmed-delivery-freshness.sh --quiet` (see
`.context/cron/unconfirmed-delivery-canary.crontab`) and appends to
`.context/working/.unconfirmed-delivery-canary.log`. Empty log = healthy.

The canary reads `channel awaiting-ack --json` and FIRES (exit 1) when any
awaiting-ack row has been outstanding longer than `--threshold-secs` (default 600
= 10 min: a send unacked this long is a stuck delivery). Each firing row names the
`dm_topic`, recipient, age, and attempt count. Ad-hoc check:
`bash scripts/check-unconfirmed-delivery-freshness.sh` (exit 0 = healthy,
1 = firing, 2 = tooling error); add `--json` for scripting, `--threshold-secs N`
to tune staleness, `--tracker-path P` to point at a non-default sqlite. Test hook
`TERMLINK_UNCONFIRMED_TEST_JSON=<file>` feeds canned `channel awaiting-ack --json`
for hub-independent verification. Operator action on firing: the recipient never
acked — confirm the peer is LIVE (`/peers --all`), re-send via `/agent-handoff`,
or drop the stale obligation if the thread is dead (delete the row from
`~/.termlink/awaiting_ack.sqlite`). `/canaries` auto-discovers the log. Pair with
the mirror-drift, substrate-preflight, framework-pickup, frozen-husk,
topic-growth, and task-finalization canaries above — all seven follow the same
"empty-log = healthy" convention.

### Fleet binary-freshness canary (T-2359, G-069 prevention)

G-069: fleet hubs ran stale/deleted-exe binaries for weeks with nothing firing —
.122 served a pre-arc-004 feature set for ~13 days while the push-transport arc
was recorded closed=shipped, and .107 itself was 26 hub-side commits stale.
`fleet doctor` prints per-hub `hub_version` + a `fleet_versions` histogram, but
nothing FIRES on it, and preflight Check 5 (T-2184) covers only the LOCAL hub.
A daily cron runs `scripts/check-fleet-binary-freshness.sh --quiet` (see
`.context/cron/fleet-binary-canary.crontab`) and appends to
`.context/working/.fleet-binary-canary.log`. Empty log = healthy.

The canary walks `fleet doctor --json` and FIRES (exit 1) when any **reachable**
hub serves a version below its **declared floor** in
`.context/cron/fleet-version-floors.conf` (`<hub-name> <min-version|->`; `-` =
exempt, optional `*` default row). It is deliberately NOT cross-hub skew
detection: patch numbers are commits-since-tag and are NOT comparable across
build lineages OR across tag epochs (e.g. ring20-dashboard reports 0.11.806 —
but that is a `git describe` tag-base artifact, NOT proof of a fork: v0.11.2 was
not tagged until 2026-06-24, so a stale ~June-6 mainline build counts from
v0.11.1 and reads 0.11.80x while post-v0.11.2 builds count from v0.11.2 and read
0.11.4xx. A larger patch number does NOT mean "newer" or "divergent lineage".
T-2377 found .121 is very likely a ~1050-commit-STALE build of our OWN mainline,
not the "own fork" this line previously asserted — confirmation needs a session
on .121 to read its git sha, which is ring20-manager scope). Rule of thumb:
never set a floor for a hub whose binary you genuinely don't build — but confirm
lineage first; do NOT infer "fork" from a larger patch number. Unknown `hub_version` on a reachable floored hub DOES fire (a
hub too old to report its version is the staleness class itself); unreachable
hubs are informational, never firing (PL-219 — `fleet doctor`/`fleet status`
already surface down hubs). **Bump the floor when hub-side rails ship** — that
is the operator's declaration that "shipped" must mean "capability-live"; the
canary then names lagging hubs daily until they restart onto the new binary.
Ad-hoc check: `bash scripts/check-fleet-binary-freshness.sh` (exit 0 = healthy,
1 = firing, 2 = tooling error); add `--json` for scripting, `--floors P` for an
alternate floors file. Test hook `TERMLINK_FLEET_FRESHNESS_TEST_JSON=<file>`
feeds canned `fleet doctor --json` for hub-independent verification (PL-213).
Operator action on firing: restart the named hub onto the upgraded binary
(systemd hosts: THROUGH the unit — stop any detached process, let systemd
start; see G-070/preflight Check 6), or adjust the floor/exemption if the
expectation changed. `/canaries` auto-discovers the log. Pair with the seven
canaries above — all eight follow the same "empty-log = healthy" convention.

### Waker-liveness canary (T-2387, G-069 shipped≠live guard for the comms rail)

arc-004 push-wake shipped and was live-proven (T-2388), but "0 wakers
fleet-wide" sat dark for weeks with nothing firing (T-2380 E4/F3) — the
operator only found out by asking "why is there still no response?". The
load-bearing signal is T-1834/T-2385's: **`metadata.pty_session` on the
agent-presence heartbeat == the waker-running signal**. A daily cron runs
`scripts/check-waker-liveness-freshness.sh --expect-armed --quiet` (see
`.context/cron/waker-liveness-canary.crontab`) and appends to
`.context/working/.waker-liveness-canary.log`. Empty log = healthy.

Three firing classes: **(a) LIVE-but-unwakeable** — a LIVE agent-presence
listener WITHOUT `pty_session` (peers can DM it durably but nothing rings its
PTY — T-2380 breakpoint #2); **(b) dead-waker** — a local
`~/.termlink/be-reachable*.state` whose `pushwaker_pid` is dead or recycled
(/proc cmdline guard, T-2239 pattern): the waker died silently after arming
(clean `be-reachable stop` removes the state file); **(c) rail-dark** (opt-in
`--expect-armed`, declared for .107 where agents ARE expected armed): ZERO
LIVE listeners carry `pty_session` at all — the literal G-069 "0 wakers"
observed state, which class (a) alone cannot see. Without the flag an empty
rail is quiet (hosts that legitimately run no agents). An unreachable hub is
informational, never firing (PL-219); class (b) still runs. Ad-hoc check:
`bash scripts/check-waker-liveness-freshness.sh [--expect-armed] [--json]
[--hub ADDR]` (exit 0 = healthy, 1 = firing, 2 = tooling error). Test hooks
`TERMLINK_WAKER_TEST_JSON=<file>` (canned `agent-listeners.sh --json`
envelope) + `TERMLINK_WAKER_STATE_DIR=<dir>` (state-file fixture dir) for
hub-independent verification (PL-213). Operator action on firing: relaunch the
named agent(s) through the T-2388 launcher — `bash scripts/tl-claude.sh start
--reachable --agent-id <name> -- --resume` (running headless claudes cannot be
retrofitted, PL-237 — arm at relaunch); dead wakers re-arm via
`bash scripts/be-reachable.sh start --agent-id <id> --pty-session <pty>`.
`/canaries` auto-discovers the log. Pair with the eight canaries above — all
nine follow the same "empty-log = healthy" convention.

### Stale-waker-code canary (T-2405, G-019 detection for T-2404)

T-2404 shipped the REMEDIATION (`scripts/fleet-rearm-wakers.sh`) for a push-waker
process still executing pre-current code after `scripts/be-reachable-pushwaker.sh`
was patched (e.g. a T-2402 idle-gating fix) — a running waker is a long-lived
detached process and keeps executing the OLD code until re-armed. But the framework
had no DETECTION: the drift was found only by a manual `/proc/<pid>` mtime compare.
G-019: fix the symptom, then close the blindness. A daily cron runs
`scripts/check-stale-waker-code-freshness.sh --quiet` (see
`.context/cron/stale-waker-code-canary.crontab`) and appends to
`.context/working/.stale-waker-code-canary.log`. Empty log = healthy.

The canary walks `$HOME/.termlink/be-reachable-<id>.state` (the same per-agent state
files T-2404 re-arms) and classifies each `pushwaker_pid`: **STALE** (pid alive AND
`/proc/<pid>` start-mtime < current waker-script mtime — the firing class), **current**
(alive and not older — healthy), **not-running** (dead/absent pid — informational
cleanup class, non-firing; that is T-2387's waker-liveness territory). It reuses
T-2404's exact staleness primitives (`code_mtime` / `proc_start_mtime` / `is_stale`)
so detection and remediation cannot drift apart. Three staleness canaries at distinct
layers: **T-2359** = stale HUB BINARY, **T-2387** = DEAD/unwakeable waker, **T-2405**
(this) = ALIVE waker on OLD CODE. Ad-hoc check:
`bash scripts/check-stale-waker-code-freshness.sh` (exit 0 = healthy, 1 = firing,
2 = tooling error); add `--json` for scripting. Test hooks `STALE_WAKER_STATE_DIR` +
`STALE_WAKER_PW_SCRIPT` feed a fixture state dir + controlled-mtime waker script for
host-independent verification. Operator action on firing: re-arm the named agent(s)
with `bash scripts/fleet-rearm-wakers.sh <agent>` (or `--all` for the whole fleet) —
a zero-outage waker-only respawn (heartbeat/presence never drops, T-2404). `/canaries`
auto-discovers the log. Pair with the nine canaries above — all ten follow the same
"empty-log = healthy" convention.

### Fleet capability-freshness canary (T-2415, G-084 prevention)

The fleet-binary canary (T-2359) checks each hub's version against a declared
**floor** — but a floor cannot express "this hub is UP, authenticating, and
version-floor-EXEMPT, yet structurally CANNOT carry the doorbell." G-084 surfaced
exactly that: ring20-dashboard (.121) passes `fleet doctor`, is floor-exempt (its
patch number is a `git describe` tag-epoch artifact, NOT comparable across
lineages — see the T-2359 note), and yet **rejects `channel.cv_keys`** (its binary
predates the cv_index primitive, T-2103), so it is silently excluded from
push-wake discovery with nothing firing. Version floors are provably blind to this
class; only a **capability probe** can see it.

> **Note (T-2467, 2026-07-28): the .121 example above is now HISTORICAL.** Fresh
> probes show ring20-dashboard was redeployed onto **0.11.588** (our own lineage,
> ~96 commits behind HEAD) and now **serves `channel.cv_keys`** — a live
> `ring20-dashboard-agent` presence entry is returned. So .121 specifically is
> doorbell-CAPABLE today; it is no longer the exclusion this paragraph describes.
> The canary and its rationale stand unchanged — the capability-incapable *class*
> is real (any hub predating cv_index still trips it); .121 is simply no longer a
> member. See `.context/cron/fleet-version-floors.conf` for its current status
> (exempt only for lack of an upgrade foothold, not incapability). A daily cron runs
`scripts/check-fleet-capability-freshness.sh --quiet` (see
`.context/cron/fleet-capability-canary.crontab`) and appends to
`.context/working/.fleet-capability-canary.log`. Empty log = healthy.

The canary walks `fleet doctor --json` for the hub list + reachability, then probes
`channel cv-keys agent-presence --hub <addr> --json` on every **reachable** hub — a
lineage-independent, tag-epoch-independent test of the doorbell's discovery
prerequisite. Per-hub classification: **capable** (exit 0 + parseable `.count` —
even `count:0` is healthy, an empty cv_index is not an error per T-2106),
**incapable** (RPC rejected: `-32001` / `-32601` / method-not-found — the firing
class), **inconclusive** (timeout / connection failure — fleet doctor owns
reachability, so these never fire), **unreachable** (no probe attempted —
informational per PL-219), **exempt** (`FLEET_CAP_EXEMPT=<hub>` — distinct from the
version-floor exemption; for hubs that legitimately are not doorbell participants).
The three fleet-fitness canaries now cover three orthogonal axes: **T-2359** = hub
binary below its version FLOOR, **T-2387** = DEAD/unwakeable waker, **T-2415**
(this) = hub structurally INCAPABLE of the doorbell RPC regardless of version.
Ad-hoc check: `bash scripts/check-fleet-capability-freshness.sh` (exit 0 = healthy,
1 = a reachable hub rejects the capability probe, 2 = tooling error); add `--json`
for scripting (carries per-hub `state` + `firing[]` with the failed `capability`),
`--no-heartbeat` to skip the heartbeat touch. Test seams (PL-213):
`TERMLINK_FLEET_CAP_DOCTOR_JSON=<file>` feeds a canned `fleet doctor --json`,
`TERMLINK_FLEET_CAP_PROBE_DIR=<dir>` feeds per-hub probe fixtures
(`<slug>.rc` / `<slug>.out`, slug = address with `:/.` → `_`) for hub-independent
verification. Operator action on firing: upgrade the named hub to a binary that
serves cv_index and restart THROUGH the systemd unit (G-070) — needs a foothold on
that host; or, if the hub is legitimately not a doorbell participant, set
`FLEET_CAP_EXEMPT=<hub-name>`. `/canaries` auto-discovers the log. Pair with the ten
canaries above — all eleven follow the same "empty-log = healthy" convention.

### Charter-drift canary (T-2483, G-019 breadth-accretion prevention)

The T-2468 purpose review found TermLink **over-built in breadth**. P4 (T-2471
delete + T-2478 deprecate) pruned 52 charter-untraceable social-analytics MCP
tools (reactions / emoji / stars / pins / typing / polls) — none of which trace
to the charter's four verbs (discover / exchange durable messages / claim work /
control terminal sessions). But **nothing prevented that breadth from
re-accreting**: the only correction was a human periodically asking for a hand
review, and that recurring manual review IS the symptom of a missing structural
check (G-019: fix why the framework was blind, not just the instance). A daily
cron runs `scripts/check-charter-drift-freshness.sh --quiet` (see
`.context/cron/charter-drift-canary.crontab`) and appends to
`.context/working/.charter-drift-canary.log`. Empty log = healthy.

The canary reads `termlink help --json` — a `{category: [tool, ...]}` map whose tool
objects carry `{name, deprecated}`, `deprecated` derived from the registry's own
`is_deprecated()`. It trusts the binary's own classification for both liveness and
category, and applies **two detectors** (T-2680): **name-pattern** (the original
social-analytics families — reactions / emoji / stars / pins / typing / polls, anchored
to avoid false-positives on core primitives: `_pin$` not `_ping`; `_star$` not `_start` /
`_starters`; `_poll_(start|vote|end|results)` not `event_poll`) and **category** (the
categories the binary itself names as analytics: `agent_rankings`, `agent_stats`,
`agent_thread_health`, `channel_engagement`, `*_poll`, `*engagement_metrics`). A LIVE
tool matching EITHER is off-charter.

**Do not read a healthy exit as a full-surface clean bill.** T-2483 originally shipped
the name detector alone and reported `{checked:214, live_off_charter:0}`, which read as
"every one of 214 tools traces to the charter". It did not: the regex knew only the six
families P4 had just deleted, so `termlink_agent_top_reacted` fired while
`termlink_agent_top_repliers` — a functionally identical leaderboard — passed clean, and
**28 live tools in explicitly-analytics categories were invisible**. Those 28 are exactly
what **T-2548** (`owner: human`) is incepting to subtract, so an open decision about ~30
off-charter tools coexisted with a daily canary calling that surface clean — Directive #2
violated in the guard layer, the costliest place, because a guard reporting green is why
nobody looks. T-2680 added the category detector and made every output path carry an
explicit scope disclaimer: the canary detects known off-charter **shapes**, it does not
audit every tool's charter traceability. Origin: T-2678 finding F2.

**Acknowledgement allowlist.** Detecting the 28 must not mean alarming daily on a decision
the human has not made. `.context/checks/charter-drift-allowlist` — **git-tracked on
purpose**, unlike the four static-check allowlists under gitignored `.context/working/`
(T-2681) — lists `<tool_name>  # <reason>` entries that are still **counted and reported**
as off-charter but do not fire. Removing a line re-fires that tool; a NEW off-charter tool
fires immediately because it is not listed. The allowlist is a ledger of an open question,
not a permanent exemption: on T-2548's resolution the entries are either deleted (tools
deprecated) or re-justified against a charter verb.

Ad-hoc check: `bash scripts/check-charter-drift-freshness.sh` (exit 0 = no unacknowledged
off-charter tool, 1 = drift, 2 = tooling error — fail-closed); add `--json` for scripting
(`firing[]` with `name`/`why`, plus `acknowledged_count`, `acknowledged[]`,
`off_charter_total`, `detectors[]`, `scope`), `--allowlist P` for an alternate ledger,
`--no-heartbeat` (meta-canary uses it). Test hooks `TERMLINK_CHARTER_DRIFT_TEST_JSON=<file>`
(canned catalog) + `CHARTER_DRIFT_ALLOWLIST=<file>` for binary-independent verification
(PL-213); fixture suite `bash tests/charter-drift-check-fixtures.sh` (17 assertions,
including the `top_repliers` regression that passed clean pre-T-2680). Current tree: 214
live tools scanned by both detectors, 28 off-charter, **all 28 acknowledged pending T-2548**,
0 unacknowledged. Operator action on firing: deprecate the named tool(s) (mirror the
`remote_inbox_*` T-1166 convention per `docs/operations/p4-surface-reduction.md`); or, if
one genuinely traces to a charter verb, rename it / adjust the detector; or, if it is a
known pending decision, acknowledge it in the allowlist with a cited reason. `/canaries`
auto-discovers the log. Pair with the eleven canaries above — all twelve follow the same
"empty-log = healthy" convention.

### Charter canonical-sentence drift canary (T-2484, G-019 charter-fork prevention)

P1 (T-2470) shipped `docs/CHARTER.md` as "the single owned statement of what
TermLink is". The canonical purpose sentence exists as **three copies** —
`docs/CHARTER.md` (bold, line-wrapped), `README.md` (plain), `docs/ARCHITECTURE.md`
(blockquote `> …`) — and markdown has no transclusion, so editing one silently
strands the other two. The charter's header even *claimed* "the docs follow by
reference" (mechanically false — they are copies). **Nothing verified the three
stayed identical**, so the charter — anchor of the entire T-2468 arc — was not
load-bearing for its own consistency claim (G-019: fix why the framework was blind).
A daily cron runs `scripts/check-charter-sentence-drift.sh --quiet` (see
`.context/cron/charter-sentence-drift-canary.crontab`) and appends to
`.context/working/.charter-sentence-drift-canary.log`. Empty log = healthy.

The canary extracts the canonical sentence from all three files, **normalizes** away
the per-file decorations (CHARTER's `**bold**` + line-wrapping, ARCHITECTURE's
blockquote prefix, whitespace/newline collapse), and FIRES (exit 1) if they diverge.
Extraction is by a unique anchor phrase truncated at the first `machines` (pure bash
param-expansion, no PCRE) — deliberately NOT a hardcoded full sentence (that would
be a 4th copy to drift) and robust to README's second `…machines.`-terminated
sentence (the greedy-match trap). A missing/unextractable file returns exit 2
(fail-closed — never a false "in-sync"). This is the **sibling of the P8
charter-drift canary** (T-2483): P8 guards the TOOL SURFACE against off-charter
breadth; this guards the CHARTER SENTENCE against its three copies forking —
together they make the charter load-bearing in both directions. Ad-hoc check:
`bash scripts/check-charter-sentence-drift.sh` (exit 0 = in-sync, 1 = drift,
2 = tooling); add `--no-heartbeat` (meta-canary uses it). Test hook
`CHARTER_SENTENCE_REPO_ROOT=<dir>` points the three-file read at a fixture repo for
host-independent verification (PL-213). Operator action on firing: `docs/CHARTER.md`
is authoritative (human-blessed) — bring `README.md` + `docs/ARCHITECTURE.md` back
into agreement. `/canaries` auto-discovers the log. Pair with the twelve canaries
above — all thirteen follow the same "empty-log = healthy" convention.

### Stuck-claims canary (T-2556, verb-3 "claim work" daily-detection gap)

The T-2468 charter-verb-completeness review found the substrate's four core verbs
each have an affirmative on-demand PROVER, but the daily-detection CANARY axis
covered only verbs 1 (discover) and 2 (exchange durable messages). **Verb 3 —
"claim work"** (`channel claim`/`renew`/`release` with lease-based ownership,
T-2019/T-2029/T-2042/T-2046) — had the RICHEST prover (`scripts/substrate-smoke.sh`
drives the full claim→renew→release lifecycle with ownership enforcement) yet ZERO
passive detection: a claim can sit expired, or a work-topic accumulate stuck claims,
for days with nothing firing (an idle worker's slot never reopens; a crashed
worker's lease never gets noticed). The substrate already ships the detector
primitive — `channel claims-summary --all --only-stuck --json` (T-2076) computes
stuckness via the T-2042 heuristic (a lease lapsed within the last 15min OR
`oldest_active_age_ms > 60_000`) and returns a truthful fleet-wide `stuck_count`.
**T-2709 narrowed the first arm**, which was `expired_count > 0` — a monotonic
latch. Expired claim rows are reaped lazily and only when the SAME
`(topic, offset)` is re-claimed, so on a topic nobody re-claims the row persists
for the life of the hub's SQLite and the "stuck" verdict never clears. Measured
here: 11 topics latched true, every one with `active_count: 0` (nothing held,
nothing that *could* be stuck), one carrying 81 expired rows. This canary sat on
that predicate, so it fired daily on permanent debris — the precise mechanism by
which a guard teaches its operator to stop reading it. The arm now keys on
`newest_expired_at_ms` so it self-clears; a hub predating the field reports
absent, which reads as "no recent expiry", never as stuck.
A daily cron runs `scripts/check-stuck-claims-freshness.sh --quiet` (see
`.context/cron/stuck-claims-canary.crontab`) and appends to
`.context/working/.stuck-claims-canary.log`. Empty log = healthy.

The canary reads the summary envelope and FIRES (exit 1) when `stuck_count > 0`,
naming each stuck topic with its `active`/`expired`/`oldest_active_age`. Per-topic
fetch errors (`ok:false` entries) are surfaced as a soft warning but do NOT
themselves fire (they could mask a stuck topic, so the operator is told — mirrors
the T-2076 "fetch errors always retained" note). Ad-hoc check:
`bash scripts/check-stuck-claims-freshness.sh` (exit 0 = healthy, 1 = firing,
2 = tooling error / hub unreachable); add `--json` for scripting, `--hub ADDR` to
target a specific hub. Test hook `TERMLINK_STUCK_CLAIMS_TEST_JSON=<file>` feeds
canned `claims-summary` JSON for hub-independent verification (PL-213). Operator
action on firing: inspect with `/claims <topic>`; if the holder is dead the lease
auto-expires, or reassign via `/claim-transfer`, or Tier-0
`termlink channel claim-force-release`. `/canaries` auto-discovers the log. Pair
with the thirteen canaries above — all fourteen follow the same "empty-log =
healthy" convention.

### Session-control canary (T-2557, verb-4 "control terminal sessions" daily-detection gap)

Sibling of T-2556, closing the last charter-verb canary cell. **Verb 4 — "control
terminal sessions"** (the founding verb: spawn a PTY-backed session, inject a
command, capture output) has an affirmative prover (`scripts/session-selftest.sh`,
T-2485: SPAWN a tmux-backed scratch session → EXEC `echo <sentinel>` verifying the
inject→run→capture round-trip → CLEANUP) but had ZERO passive daily detection —
spawn/exec could regress and only a manual run would catch it. The prover is
deterministic + local (uses `exec --json`, no live peer needed) and self-reaps, so
it is ideal canary substrate. A daily cron runs
`scripts/check-session-control-freshness.sh --quiet` (see
`.context/cron/session-control-canary.crontab`) and appends to
`.context/working/.session-control-canary.log`. Empty log = healthy.

The canary runs the prover and translates its verdict: selftest exit 0 (proven) →
canary exit 0 (healthy); selftest exit 1 (broken) → canary exit 1 (FIRE, naming the
`broken_stage`); selftest exit 2 (tooling — hub down / no tmux / dep missing) →
canary exit 2 (non-firing — that is a substrate/environment fault, `/preflight`
territory, NOT a verb-4 regression). This split keeps a firing log meaningful: it
fills ONLY when session control genuinely broke, never on a transient hub-down.
Ad-hoc check: `bash scripts/check-session-control-freshness.sh` (exit 0 = healthy,
1 = firing, 2 = tooling); add `--json` for scripting, `--hub ADDR` to target a
specific hub. Test hooks `TERMLINK_SESSION_CANARY_TEST_JSON=<file>` +
`TERMLINK_SESSION_CANARY_TEST_RC=<n>` feed a canned selftest verdict + exit code
for hub/tmux-independent verification (PL-213). Operator action on firing: reproduce
with `bash scripts/session-selftest.sh` (names the broken stage), then inspect
`termlink spawn` / `termlink exec <s> 'echo hi' --json`. `/canaries` auto-discovers
the log. Pair with the fourteen canaries above — all fifteen follow the same
"empty-log = healthy" convention.

### Dead-letter canary (T-2558, G-063 class — durable poison-drop sink)

T-2243 (R4) stopped *silent* poison-drops on the charter-core "exchange durable
messages" path: a `channel post` rejected `POISON_THRESHOLD=10` times (unknown
topic, bad signature, or a governance message rejected during a hub blip) is now
durably MOVED to the offline queue's `dead_letters` table instead of `DELETE`d —
converting silent data-loss into a durable, inspectable record. But that record is
surfaced ONLY if a human runs `/queue-status`: nothing reads it on a schedule (the
T-2083–2087 queue observability arc fires only on `Drained`/`Pending` transitions,
never on dead-letter growth). So a poison-dropped *guaranteed* message sits forever
with nothing firing — the "write-only sink nobody noticed" class (G-063) that the
unconfirmed-delivery canary (T-2295) closed for the await-ack path. This canary
closes it for the dead-letter path. A daily cron runs
`scripts/check-dead-letter-freshness.sh --quiet` (see
`.context/cron/dead-letter-canary.crontab`) and appends to
`.context/working/.dead-letter-canary.log`. Empty log = healthy.

The canary reads `channel queue-status --json` (a pure local SQLite read — works
even when the hub is down, which is exactly when the queue absorbs writes) and
FIRES (exit 1) when `dead_letters > 0`, naming each row's topic + reject reason +
attempt count. A never-created queue (`exists:false`, no `dead_letters` field) and
`dead_letters == 0` both read healthy. Ad-hoc check:
`bash scripts/check-dead-letter-freshness.sh` (exit 0 = healthy, 1 = firing,
2 = tooling error); add `--json` for scripting. Test hook
`TERMLINK_DEAD_LETTER_TEST_JSON=<file>` feeds canned `queue-status --json` for
hub-independent verification (PL-213). Operator action on firing: a guaranteed post
was rejected 10× and dead-lettered — fix the reject cause (unknown topic → create
it; bad signature → `fleet reauth`), re-send via `/agent-handoff`, or drop the row
from `~/.termlink/outbound.sqlite` if the thread is dead. `/canaries` auto-discovers
the log. Pair with the fifteen canaries above — all sixteen follow the same
"empty-log = healthy" convention.

### Forever-topic archival-growth canary (T-2562, charter non-goal #2 guard)

The charter's non-goal #2 is "**NOT a durable database / system of record**": topics
are retention-bounded append logs sized for coordination, and "durability" means
"survives a hub blip and replays", NOT "stored forever". That boundary had a
structural blind spot. Two existing guards do NOT cover the general archival case:
the create-time warn (`channel.rs` T-2058) is **warn-only** and only fires when the
topic NAME matches a high-rate / single-value-state pattern; the **topic-growth
canary** (T-2252) fires ONLY on watched high-rate patterns (`agent-presence`,
`agent-listeners-*`, `agent-conv-*`, `dm:*`) and **explicitly excludes** durable
topics. Net: an arbitrarily-named topic set to `Retention::Forever` and grown
unboundedly as archival storage matches NO watch pattern → invisible to both — the
"system of record" non-goal violated with nothing firing. A daily cron runs
`scripts/check-forever-archival-freshness.sh --quiet` (see
`.context/cron/forever-archival-canary.crontab`) and appends to
`.context/working/.forever-archival-canary.log`. Empty log = healthy.

The canary reads `termlink channel list --json` (the same plumbing T-2252 uses) and
FIRES (exit 1) on ANY topic whose `retention.kind == "forever"` and whose record
`count` exceeds an archival ceiling (default 50000, `--threshold N`), MINUS the
operator-durable allowlist (`channel:learnings`, `policy-decisions`,
`framework:pickup`, `broadcast:global` — intentionally Forever, mirrors the T-2252 /
T-2057 audit §5 exclusions; override via `TERMLINK_FOREVER_EXCLUDE_TOPICS`). The
ceiling is deliberately high: a small Forever topic is fine; a Forever topic with
tens of thousands of records is being used as a database. **Distinct axis from
T-2252** — T-2252 asks "is a high-rate topic un-swept?" (a hygiene-cron failure);
this asks "is a Forever topic being used as archival storage?" (a non-goal
violation) — same read, orthogonal firing gate. Ad-hoc check:
`bash scripts/check-forever-archival-freshness.sh` (exit 0 = healthy, 1 = firing,
2 = tooling error); add `--json` for scripting, `--hub ADDR` to target a specific
hub. Test hook `TERMLINK_FOREVER_TEST_JSON=<file>` feeds canned `channel list --json`
for hub-independent verification (PL-213). Operator action on firing: bound the topic
(`termlink channel set-retention <t> --retention messages --retention-value N &&
termlink channel sweep <t>`), or — if genuinely operator-durable — add it to
`TERMLINK_FOREVER_EXCLUDE_TOPICS`. `/canaries` auto-discovers the log. Pair with the
sixteen canaries above — all seventeen follow the same "empty-log = healthy"
convention.

### Substrate-smoke canary (T-2696, verb-3 end-to-end composition — the un-executed prover)

`scripts/substrate-smoke.sh` (T-2151) proves the substrate's canonical work-stealing
COMPOSITION in one command — create → post → claim → claim-transfer → worker-loop →
verify-clean, plus the four arc-demo regression gates (drain race, cooperative handoff,
lease expiry, hub-blip resilience) — and **nothing executed it on a schedule** (T-2696
measured: every `grep -rln` hit across `.context/cron/`, CI, and `/etc/cron.d` was a
comment or the prover's own file). The stuck-claims canary (T-2556) watches claim STATE;
this canary proves the claim COMPOSITION still round-trips. A daily cron runs
`scripts/check-substrate-smoke-freshness.sh --quiet` (see
`.context/cron/substrate-smoke-canary.crontab`) and appends to
`.context/working/.substrate-smoke-canary.log`. Empty log = healthy.

T-2557-pattern verdict translation — smoke exit 0 → healthy; 1 → **FIRE**, naming the
broken stage from `stages_failed[0]`; 2 → tooling (non-firing). **The load-bearing remap
(T-2694 F2/G3):** smoke's `create` stage on an UNREACHABLE hub is a `stage_fail` → smoke
exits **1**, i.e. the prover reports "substrate broken" on a quiet/hub-down host. The
canary therefore prechecks hub reachability itself (cheap `channel list` read, 15s bound)
BEFORE invoking smoke and exits 2 (non-firing, `/preflight` territory) when unreachable —
smoke's own exit-2 arm covers only usage/missing-dep and cannot make that distinction. A
firing thus carries a built-in claim: *the hub was reachable seconds earlier, so a stage
genuinely broke.* Smoke is cron-safe: self-reaps its `smoke:*` topic on every exit path
(T-2754), ~seconds of runtime, bounded retention. The COMMS sibling closed differently on
purpose: `comms-selftest`'s harness was already hermetic, so it joined the guard layer as
`tests/comms-selftest-fixtures.sh` (runs on every push/PR via T-2686 CI) instead of
becoming a 19th canary that proof-pings a live peer daily (T-2486 warned against exactly
that; it would half-overlap T-2387/T-2295). Ad-hoc check:
`bash scripts/check-substrate-smoke-freshness.sh` (exit 0 = healthy, 1 = firing,
2 = tooling); add `--json` for scripting, `--hub ADDR`, `--no-heartbeat`. Test seams
(PL-213): `TERMLINK_SMOKE_CANARY_TEST_HUB_RC` (canned precheck rc) +
`TERMLINK_SMOKE_CANARY_TEST_JSON`/`_RC` (canned smoke verdict); fixtures:
`bash tests/substrate-smoke-canary-fixtures.sh` (16 assertions, the hub-down→2 remap
pinned first). Operator action on firing: reproduce with `bash scripts/substrate-smoke.sh`
(names the stage), then `/claims --all` + `termlink channel claims-summary --all
--only-stuck`. `/canaries` auto-discovers the log. Pair with the seventeen canaries above —
all eighteen follow the same "empty-log = healthy" convention.

### Release-publication canary (T-3243, G-069 at the release layer)

`v0.12.0` was tagged and mirrored and **no GitHub Release was ever published** —
v0.11.2 stayed the newest release every installer served — while `install-check.yml`
sat red on main for ~3 months (947 runs, last green 2026-06-12) and `doc-lint.yml`
had no green run on main since 2026-08-16. Nothing fired on any of it (F19, T-3211).
The mirror canary proves a tag *reaches* GitHub; nothing proved it became a
*release*, or that the workflows gating a release still pass. A daily cron runs
`scripts/check-release-publication-freshness.sh --quiet` (see
`.context/cron/release-publication-canary.crontab`) and appends to
`.context/working/.release-publication-canary.log`. Empty log = healthy.

It FIRES (exit 1) when (a) the newest local `v*` tag has no **published** release
(`releases/tags/<tag>` HTTP 404, or a draft), or (b) a watched workflow (default
`install-check.yml doc-lint.yml`) has no successful run on `main` within
`--max-age-days` (default 7), or (c) **a WARN-tier guard-layer member has been red for
more than 14 days** (T-3260, `GUARD_WARN_ESCALATE_DAYS`; see *Severity tiers* below). For
(c) it first runs `run-guard-layer.sh --only-class warn --record-warn` to refresh the
first-red ledger, then reads it (`--no-warn-refresh` reads it as-is). It reads the REST API directly via `gh api`, **not**
`gh run list --status success` — measured 2026-09-29, the list form reported
install-check's last green as 2026-06-12 while the API returned a green run from
that same afternoon. **Fail-closed:** no `gh`, a non-404 API error (auth, network,
500), an unparseable response, or no `v*` tag exits 2 and lands in the `.stderr`
sink (ERRORING on `/canaries`). Needs `gh` authenticated for the cron user.
**Scope:** it does not check release assets (`check-release-artifact-drift.sh`), tag
mirroring (`check-mirror-freshness.sh`), or what a green run tested. Ad-hoc:
`bash scripts/check-release-publication-freshness.sh` (`--json`, `--quiet`,
`--no-heartbeat`, `--max-age-days N`, `--workflows "a.yml b.yml"`, `--repo O/N`).
Test seam (PL-213): `RELEASE_PUB_TEST_DIR=<dir>` (canned raw `gh api` responses +
`.rc`/`.err`) and `RELEASE_PUB_TEST_NOW=<epoch>`. Fixtures:
`bash tests/release-publication-canary-fixtures.sh` (30 assertions; a mutant muting
the release finding turns R1/R2/W4/F2 red; E1–E4 pin the 14-day escalation, not before
and past it, and mutant M2 muting it turns E1 red). Operator action on firing: publish the
release (re-run `release.yml` for the tag, or cut the next tag once CI is green); for
a stale workflow, open its latest run on main and fix it. Pair with the canaries
above — same "empty-log = healthy" convention.

### Arc-claim-drift canary (T-3288, G-019 for T-3066 — the check that could never run)

`check-arc-claim-drift.sh` (T-3066) re-runs the prover bound to each CLOSED arc's
claim: arc-003 "no silent loss" (`notify-rail-e2e.sh --experiment e4`) and arc-004
push-wake (`demo-ws-push.sh`). **Until T-3288 it ran only in the guard layer, i.e.
only in CI.** A CI runner has no hub and no identity key, so both provers exit 2
there and are `SKIP(CI)` by design (T-3238). The one host-side guard-layer run, the
release canary's, runs the WARN tier only, and this check is FAIL tier. So nothing
ever re-verified either claim automatically: green CI meant "skipped", not
"verified". A daily cron now runs it on this host, where both provers can run (4.7s
measured under cron's `env -i`). It is scheduled by
`.context/cron/arc-claim-drift-canary.crontab`, installed by `runme.sh` action 1, and
logs to `.context/working/.arc-claim-drift-canary.log`. Empty log = healthy.

**The crontab's PATH puts `/root/.cargo/bin` first, and that is load-bearing.**
`/usr/local/bin/termlink` and `~/.local/bin/termlink` were 0.12.13, which predates
`--no-create` (T-3286). E4 treats that as TOOLING, and on the host that fires as
CLAIM-FAILED, a false alarm. The heartbeat is written on EXIT; `--no-heartbeat` (the
guard-layer invocation) skips it, and the fixtures redirect it via
`ARC_CLAIM_HEARTBEAT_FILE`, so neither CI nor a test run can mask a dead cron (T17/T18
in `tests/arc-claim-drift-fixtures.sh`, both mutant-checked). Operator action on
firing: `bash scripts/check-arc-claim-drift.sh` names the arc and its prover. A
failing prover means the shipped claim is no longer true, so file a task against the
arc rather than allowlisting it.

### Session-leak canary (T-3296, T-3291 S4 — G-019 for the session lifecycle)

T-3291 found two leaks that had run for weeks with nothing firing: **~480 zombie
`termlink register --shell` sessions** (idle, detached tmux shells nobody touched since
creation, 208 older than a week, ~4 GB with their shells) and **10,144 orphaned
`<id>.sock.data` files**. Every register heartbeats every 30 s whether used or not, so
every liveness check saw the zombies as LIVE; and the hub sweep deleted only `.sock` +
`.json`, so nothing could find the data sockets again. The fixes: T-3293 (cleanup
removes all three files, the hub reaps orphans every sweep, register cleans up on
SIGTERM/SIGHUP), T-3294 (a `--shell` session ends when its shell ends), T-3295 (the hub
also sweeps the legacy `/tmp/termlink-<uid>` pool). A shell left idle at its prompt is
still a zombie, because the vendored dispatch never exits it. That part is upstream
(T-3291 S3a) and handled by the runme reap (S3b).

A daily cron runs `scripts/check-session-leak-freshness.sh --quiet` (see
`.context/cron/session-leak-canary.crontab`, installed by `runme.sh` action 1) and
appends to `.context/working/.session-leak-canary.log`. Empty log = healthy. It FIRES
when more than 25 zombies exist (`--threshold N`), or when any orphan data socket is
older than 1 h. The hub reaps orphans within 10 min, so one older than that means the
reap is not running.

**Detection is `scripts/lib/session-zombies.py`, the SAME detector the runme reap uses.**
What fires here is exactly what the reap would terminate (the T-2404/T-2405 pattern), so
its "never a zombie" classes are load-bearing:
- **busy:** the shell has a child;
- **attached:** a client is on the tmux session;
- **young:** under 24 h;
- **launcher_alive:** `claude-master-<pid>` whose `claude`/`claude-fw` launcher is alive;
- **non-shell:** `--self` endpoints;
- **unmanaged:** any register that is not a tmux pane.

The last one matters most. Systemd-supervised always-on agents (`framework-agent-systemd`,
`termlink-agent`, `email-archive`) wait at an idle shell by design; the first draft flagged
them, and a live run caught it before anything shipped.

Exit 0 healthy · 1 firing · 2 tooling (an unreadable process table is never clean);
`--json`, `--quiet`, `--no-heartbeat`, `--threshold N`. Seams: `SESSION_LEAK_PS_FILE`,
`SESSION_LEAK_TMUX_FILE`, `SESSION_LEAK_DIRS`, `SESSION_LEAK_HEARTBEAT_FILE`. Fixtures:
`bash tests/session-leak-canary-fixtures.sh` (16 assertions). Mutants remove the busy,
attached, launcher and tmux-pane guards, and each is caught. **Test-safety note from
T-3293:** a hub test that drove the real `supervisor::run` swept whatever
`TERMLINK_RUNTIME_DIR` pointed at on the machine running `cargo test`, and reaped about
10k real files. Tests now go through `supervisor::run_with` with a temp dir. Never write
a test that runs the real loop.

### Cron-install-drift check (T-2561, shipped≠live / G-069 for the canary layer)

A canary is only load-bearing if its crontab is actually installed to `/etc/cron.d`.
The canary crontabs live git-tracked under `.context/cron/*.crontab` as the source
of truth, but installation is a MANUAL operator step — so a freshly-committed canary
can sit **shipped but dark** (never fires) with nothing detecting it. The
meta-canary-aliveness check (T-1723) verifies the HEARTBEAT of canaries that ARE
running; it structurally cannot see one that was never scheduled (T-2556/57/58 were
committed dark and only a self-review caught it). `scripts/check-cron-install-drift.sh`
closes that blindness: for every git-tracked crontab it reads the crontab's OWN
`# Installed to: <path>` header (robust to naming exceptions like `agentic-audit` →
`/etc/cron.d/agentic-audit-termlink` — the path is self-declared, not derived from a
naming rule) and classifies into four states: **MISSING** (declared path absent →
FIRES, the G-069 class), **UNINSTALLED_JOBS** (present, but the installed file lacks
cron JOB lines that git declares → FIRES, T-2682), **DRIFT** (present, content
differs, but no job line is absent — comment churn, env tweaks, an extra
operator-added job → non-firing WARNING, fires under `--strict`), or **OK**
(byte-identical).

**Why UNINSTALLED_JOBS is its own class (T-2682).** T-2561 shipped with MISSING vs
DRIFT only, so every content difference read as one quiet "DRIFT (warning)" line. On
the origin host that hid two crontabs whose installed copies were missing their
**meta-canary job line entirely** — T-2175 (substrate-preflight) and T-2176
(fleet-doorbell-mail), the jobs that detect when the canary itself stops firing. Both
had never been scheduled, reported as cosmetic drift. A job that was never scheduled
is not a cosmetic difference; it is exactly the shipped-but-dark condition this check
exists to catch, so it now fires regardless of `--strict` and prints the specific
missing line(s) so the operator can install precisely what is absent. Direction
matters: git-declared work absent from the host fires; an EXTRA job the operator added
locally does not. Job lines are compared with comments, blanks, and `VAR=value` env
assignments stripped and whitespace collapsed, so a reformat alone never false-fires.
Found by the T-2678 charter guard-coverage review (gap G5).

This is a **deploy-time / preflight check, NOT itself a cron canary** — a
canary-to-detect-uninstalled-canaries would itself need installing (recursive). Run it
ad-hoc after committing a new canary. Exit codes: 0 = healthy, 1 = firing (missing /
uninstalled jobs / drift under `--strict`), 2 = tooling. `--json` for scripting
(adds `uninstalled_jobs_count` + `uninstalled_jobs[]` carrying
`<crontab>|<installed_path>|<missing line>`); a host without `/etc/cron.d` (macOS/dev)
is informational, never firing. Test hooks `CRON_DRIFT_SRC_DIR` +
`CRON_DRIFT_INSTALLED_DIR` feed fixture dirs (PL-213); fixture suite
`bash tests/cron-install-drift-fixtures.sh` (24 assertions). Operator action on
firing: install the named crontab with
`sudo cp .context/cron/<name>.crontab <declared-path>`.

### Unclamped caller-param → allocation-sink static check (T-2527, G-019 prevention for the T-2523/T-2526 class)

Twice in one window an adversarial hunter found a caller-supplied param reaching an
eager allocation / resource sink with no upper bound: `max_parallel` →
`tokio::sync::Semaphore::new(max_parallel)` (T-2523 — `permits > MAX_PERMITS` panics,
`0` hangs forever) and `count` → `Vec::with_capacity(count as usize)` (T-2526 — a
`u32::MAX` field pre-allocates ~100 GB, OOM-aborting the MCP server from one JSON
field). Each was a one-line omission, invisible until someone read that exact line.
The repo has a strong "clamp every numeric caller param" convention
(`clamp_max_parallel`, `max_depth.clamp(1,1024)`, `since_days.clamp(1,365)`, …) but
the convention is **by discipline, not enforced** — two instances in one window means
the mechanism recurs. G-019: fix the symptom (the two clamps), then fix why the
framework was blind. This check makes the convention load-bearing.

`scripts/check-alloc-sink-clamps.sh` is a grep/AST-lite scanner (NOT a runtime cron
canary — a **source-level static check**, sibling to the thirteen canaries above) over
the handler crates (`crates/termlink-mcp/src`, `crates/termlink-hub/src`,
`crates/termlink-session/src`). It flags each `Vec/String/HashMap/…::with_capacity(<x>)`,
`Semaphore::new(<x>)`, `vec![_; <x>]`, and `<expr>.repeat(<x>)` whose size arg `<x>`
is a **bare identifier or `p.<field>`** (optionally `<x> as usize`) and is NOT a
literal / not `.len()` of a materialized collection / not inline
`.clamp(`/`.min(`/`.take(`/`.saturating_`-bounded. Compound expressions (`m + 1`)
are OUT OF SCOPE (conservative — fewer false positives), and `//`-comment mentions
are skipped. **Binding-aware clear:** for a bare-ident site `foo(x)`, if the file
binds `let [mut] x = <…clamp…|…min(…|…take(…|validate_…>` the site is treated as
clamped-at-binding and cleared — this is why the T-2523 sites
(`let max_parallel = clamp_max_parallel(p.max_parallel);`) read clean, and why
reverting that clamp makes them fire again (the load-bearing property).

Output is a **REVIEW list, not a hard gate** — false positives are expected; the
value is surfacing NEW sinks for a human/agent to confirm-and-clamp. Confirmed-safe
sites (upstream guard, internally-derived count, library constructor param) are
acknowledged in `.context/checks/alloc-sink-allowlist`, one drift-stable signature
per line (`<relpath>::<sink>(<normalized-arg>)`, `as usize` casts stripped); the
check trends toward empty. The current tree is clean (98 sink calls scanned, 5
confirmed-safe sites allowlisted with cited reasons — including one, the data-plane
`codec.rs` frame reader, whose `payload_len` bound is enforced at the
`FrameHeader::decode()?` boundary via `MAX_PAYLOAD_SIZE`, one line above the
allocation, not by a `let`-clamp the grep can see). Exit codes: 0 = clean, 1 =
unacknowledged candidate(s), 2 = tooling error; `--json` for scripting; `--root`
(repeatable) + `--allowlist` for fixtures; `--quiet` / `--no-heartbeat` for cron.
Ad-hoc check: `bash scripts/check-alloc-sink-clamps.sh`. Fixtures (no live binary):
`bash tests/alloc-sink-check-fixtures.sh`. Operator action on firing: clamp the code
(`.clamp(1, N)` inline, or a `let x = clamp_*(...)` binding) OR — if confirmed
bounded-by-construction — add the site's signature to the allowlist with a cited
reason.

**Allowlists are git-tracked under `.context/checks/` (T-2681).** All four static
checks (alloc-sink, drain-sink, silent-exit, busy-spin) resolve their allowlist
**tracked-first**: `.context/checks/<name>-allowlist` when present, falling back to
the legacy `.context/working/.<name>-allowlist`, with an explicit env var /
`--allowlist` always winning over both. The legacy home was gitignored
(`.gitignore:80`), which made the entire guard layer **non-reproducible**: in a
fresh clone, a CI runner, or a git worktree the allowlists are absent and every
acknowledged site fires — alloc-sink `ok:false` (5), drain-sink `ok:false` (6),
busy-spin `ok:false` (4) — while this file documented all three trees as scanning
CLEAN. That was true only on the single machine holding the untracked copies. It
also meant the *cited reasons* — the whole justification for the acknowledgement
mechanism — lived on one disk outside version control, one `rm -rf` from silent
permanent loss. A guard whose reported health depends on unversioned local state
is a guard whose green is not evidence (same class as the T-2680 canary that
over-reported its scope). All 15 acknowledged sites were re-verified by reading
the code during the migration rather than copied forward on trust. Found by the
T-2678 charter guard-coverage review (finding F4).

### Unbounded peer-driven drain-sink static check (T-2531, G-019 prevention for the T-2518/2524/2525/2529 class)

The sibling of the T-2527 alloc-sink check, for the *accumulation* half of the
same class. Four fixes in one campaign each closed ONE instance of "peer-influenced
/ externally-driven bytes accumulated UNBOUNDED into the long-lived daemon's OWN
address space, risking OOM of the supervisor": `read_capped_line` (T-2518 bus line
reader), artifact download (T-2524), hub staging (T-2525), and `executor::execute`'s
`cmd.output()` child drain (T-2529). T-2527 made the *pre-allocation* clamp
convention load-bearing; the *drain* sinks (`.output()` full child-output capture,
`read_to_end`/`read_to_string` stream drains, full `collect::<Vec<u8>>()`) were still
unguarded — invisible until a human read that exact line. G-019: fix the four caps,
then fix why the framework was blind.

`scripts/check-drain-sink-caps.sh` is a grep/AST-lite **source-level static check**
(NOT a runtime cron canary — same tier as T-2527) over the daemon crates
(`crates/termlink-{mcp,hub,session,bus,protocol}/src`). It flags each
`<cmd>.output()`, `<reader>.read_to_end(...)`, `<reader>.read_to_string(...)`,
`<iter>.collect::<Vec<u8>>()`, and `<iter>.bytes().collect` — skipping `//`-comment
lines and clearing a `<reader>.take(N).read_to_*(...)` bounded read on the same line.
Confirmed-safe sites are acknowledged in `.context/checks/drain-sink-allowlist` by
a **drift-stable `<relpath>::<enclosing-fn>::<sink>` signature** (fn-name-based, so it
survives line moves; a fn RENAME re-fires the site — intended re-review on meaningful
change, mirroring T-2527's line-independence trade-off). The current tree is clean:
6 `.output()` sink calls scanned, all 6 allowlisted — every one runs the SELF binary
(`Command::new(&std::env::current_exe())`) on a fixed trusted subcommand (or a
`TERMLINK_SCRIPTS_DIR` `bash <script>`) with `kill_on_drop(true)` + `stdin(null)` +
`tokio::time::timeout`, so the captured output is bounded by the subcommand's own JSON
emission, not by peer bytes (INTERNAL by the T-2529 threat model). Zero `read_to_end`/
`read_to_string`/`collect::<Vec<u8>>` drains exist in scope, so the first one added
fires. **Load-bearing:** reintroducing a `.output()` into `execute_capped` (the
pre-T-2529 shape) makes the check fire on `executor.rs::execute_capped::output`;
restoring returns it to clean. Exit codes: 0 = clean, 1 = an unacknowledged drain
sink, 2 = tooling error; `--json` for scripting; `--root` (repeatable) + `--allowlist`
for fixtures; `--quiet` / `--no-heartbeat` for cron. Ad-hoc check:
`bash scripts/check-drain-sink-caps.sh`. Fixtures (no live binary):
`bash tests/drain-sink-check-fixtures.sh`. Operator action on firing: cap the code
(chunked read + size check like `executor::execute_capped`, or `reader.take(N)`) OR —
if confirmed a trusted-arg subprocess / trusted fixed source — add the site's
signature to the allowlist with a cited reason.

### Silent non-zero-exit static check (T-2666, G-019 prevention for the T-2663 class)

The third source-level static check (sibling of the T-2527 alloc-sink and T-2531 drain-sink
checks), for the *no-silent-failures* half of the charter's Directive #2. A CLI subcommand
that reaches `std::process::exit(<non-zero>)` on a user path while printing NOTHING leaves the
operator staring at a non-zero exit with zero explanation — indistinguishable from a crash.
T-2663 was exactly this: `termlink discover --first` on a no-match did a bare
`std::process::exit(1)` in TEXT mode (its JSON sibling branch called `json_error_exit`, which
prints; the text branch emitted nothing), found only by a human reading that one line. The
recurring shape is a copy-paste divergence — `if display.json { json_error_exit(...) } else
{ /* nothing */ } std::process::exit(1)` — and T-2663 fixed the `discover` instance but left
the SAME pattern un-migrated in `session.rs::cmd_list` and `remote.rs::cmd_remote_list_inner`
(both fixed in T-2667, surfaced by this check on its first run). The "name what happened
before a bare text-mode exit" convention was discipline-only; this makes it load-bearing.

`scripts/check-silent-exit.sh` is a grep/AST-lite **source-level static check** (NOT a runtime
cron canary) over the user-facing CLI crate (`crates/termlink-cli/src`; `--root` repeatable to
override). It flags each `std::process::exit(<non-zero INTEGER LITERAL>)` whose IMMEDIATELY-
preceding non-blank line is a lone `}` (a closed block, not an output/`flush()` line), where
that block carries a `json_error_exit` within a short window above and NO output macro
(`eprintln!`/`println!`/`eprint!`/`print!`) sits between the closing brace and the exit — the
precise json-gated-output / bare-text-exit signature. A precise rule, not a fuzzy preceding-
window: the ~40 LOUD exit sites in this tree print or `flush()` immediately above the exit (so
their preceding line is not a lone `}`) and are cleared automatically; a fuzzy window both
false-positived those and false-negatived the real defects (a success-branch `println!` in the
window masks the sibling failure-branch bare exit). **Exit-code-FORWARDING sites are out of
scope by construction** — `exit(code)` / `exit(exec_result.exit_code)` / `exit(exit_code as
i32)` forward a wrapped subcommand's own status (may be 0; the wrapped op already emitted
output), and the regex only matches a non-zero integer literal. Confirmed-loud sites (output
through a path the window can't see) are acknowledged in `.context/checks/silent-exit-allowlist`,
one drift-stable `<relpath>::<enclosing-fn>::silent-exit` signature per line (fn-name-based, so
it survives line moves; a fn RENAME re-fires — same trade-off as T-2527/T-2531). After the
T-2667 migration the current tree scans CLEAN (39 non-zero-literal exits scanned, 0 firing, 0
allowlisted). **Load-bearing:** reverting the `eprintln!` at any of the three fixed sites
(metadata.rs `cmd_discover`, session.rs `cmd_list`, remote.rs `cmd_remote_list_inner`) re-fires
the check on that site; restoring returns it to clean. Exit codes: 0 = clean, 1 = an
unacknowledged silent exit, 2 = tooling error; `--json` for scripting (`{ok, firing:[{file,
line,fn}], checked, candidates}`); `--quiet` / `--no-heartbeat` for cron; `--allowlist` +
`--root` for fixtures. Ad-hoc check: `bash scripts/check-silent-exit.sh`. Fixtures (no live
binary): `bash tests/silent-exit-check-fixtures.sh`. Operator action on firing: mirror the JSON
branch with an actionable stderr line before the exit (the T-2663 remediation: `eprintln!("No
matching sessions.");`) OR — if the site genuinely emits output the window cannot see — add its
signature to the allowlist with a cited reason.

### Long-poll busy-spin static check (T-2672, G-019 prevention for the T-2670/T-2671 class)

The fourth source-level static check (sibling of the T-2527 alloc-sink, T-2531 drain-sink,
and T-2666 silent-exit checks), for the *no-busy-spin* half of the charter's Directives #1
(antifragility) + #2 (no silent failures). A `loop {}` that re-dispatches a long-poll RPC
(`event.subscribe` / `event.collect` / `event.poll`) whose error arm re-iterates with NO
`tokio::time::sleep` busy-spins a CPU core the instant the hub goes dead/half-open: on a
LIVE hub the long-poll (server blocks until events or timeout) paces the loop, but a
dead/half-open hub errors near-instantly, so a bare warn-and-fall-through re-dispatches with
zero delay — and silently (the `tracing::warn!` above the re-iteration is gated out at the
default log level). T-2658/T-2636/T-2640 were runtime instances; T-2670 fixed two sites in
`agent.rs` and T-2671 fixed one in `tools.rs`; the blessed remediation is a 500ms
sleep-on-error before the next iteration — the convention across `events.rs:805/900/1349` and
`dispatch.rs` `COLLECT_ERR_BACKOFF`. But the convention was DISCIPLINE-ONLY: nothing detected
the next long-poll loop shipped without the backoff. On its very first run this check FOUND
three un-migrated sibling instances (`execution.rs cmd_request`, `file.rs cmd_file_receive`'s
`Ok(Err(e))` instant-error arm, `remote.rs cmd_remote_events`'s two arms — the last an
UNBOUNDED loop), all fixed in T-2673 — the same "hardened in one place, siblings not migrated"
divergence the sibling checks exist to catch.

`scripts/check-busy-spin.sh` is a grep/AST-lite **source-level static check** (NOT a runtime
cron canary) over the long-poll-adjacent crates (`crates/termlink-{cli,mcp}/src`; `--root`
repeatable to override). For each `loop {` it brace-matches the loop body (an awk brace
counter), keeps ONLY bodies that re-dispatch one of the three long-poll RPC method strings,
and FIRES on any such body with NO `tokio::time::sleep`. The anchor is narrow and low-FP by
construction: the ~74 analytics loops in `tools.rs` and the bounded recv/accept/iterator loops
never dispatch those strings, so they are excluded automatically; every convention-correct
long-poll loop carries the 500ms backoff, so it clears on the `sleep` presence. A long-poll
loop whose error path provably EXITS the loop (`return` / `bail!` — no re-dispatch, hence no
busy-spin) is SAFE but the grep cannot see that from the body, so it is acknowledged in
`.context/checks/busy-spin-allowlist` by a drift-stable `<relpath>::<enclosing-fn>::busy-spin`
signature (fn-name-based, survives line moves; a fn RENAME re-fires — same trade-off as the
sibling checks). The current tree scans CLEAN (14 long-poll loops scanned: 3 fixed in T-2673,
4 allowlisted as exit-on-error with cited reasons — `events.rs cmd_wait`, `tools.rs`
`termlink_request` / `termlink_wait` / `termlink_agent_ask` — the rest carry the backoff).
**Load-bearing:** reverting the T-2673 sleep on any of the three fixed sites re-fires the check
on that loop; restoring returns it to clean. Exit codes: 0 = clean, 1 = a long-poll loop with
no sleep-on-error, 2 = tooling error; `--json` for scripting (`{ok, firing:[{file,line,fn}],
checked, candidates}`); `--quiet` / `--no-heartbeat` for cron; `--allowlist` + `--root` for
fixtures. Ad-hoc check: `bash scripts/check-busy-spin.sh`. Fixtures (no live binary):
`bash tests/busy-spin-check-fixtures.sh`. Operator action on firing: add the 500ms
sleep-on-error before the next iteration (the T-2670/T-2671/T-2673 remediation) OR — if the
error path provably exits the loop — add the loop's signature to the allowlist with a cited
reason.

### Verification-block pipefail auditor (T-2818, L-387 at repo scale)

L-387: under `set -o pipefail`, `cmd | grep -q PATTERN` exits **141** when the pattern
MATCHES — `grep -q` exits on first match and closes the pipe, SIGPIPE kills the upstream,
and pipefail propagates the upstream's status. **The pipeline fails on success.** The P-011
gate runs every `## Verification` line under `set -euo pipefail`, so a command in that shape
blocks completion of a task whose verification actually passed.

The framework already ships the detector — `lib/reviewer/static_scan.py::detect_l387_sigpipe_risk`
— and fires it as a **non-blocking advisory at `started-work`, one task at a time** (T-2059).
Nothing ever asked the repo-wide question. The answer, first time it was asked: **150
findings across 189 active task files** (1142 across all 2442 including `completed/`).

Read the failure direction carefully, because it is what makes this urgent rather than
cosmetic. In isolation it looks like the *safe* direction — the gate blocks rather than waves
through. In aggregate it is not: a gate that blocks incorrectly 150 times teaches the
operator that P-011 failures are noise and that `--force` is the normal way past them, and a
verification gate people routinely force is a verification gate that no longer verifies.

`scripts/check-verification-pipefail.sh` is a **deploy-time / ad-hoc REVIEW list**, not a cron
canary. It **wraps** the framework's detector rather than reimplementing it — two copies of a
subtle SIGPIPE heuristic drift, and the copy that drifts is the one that quietly stops
catching things. Exit 0 = clean, 1 = findings, 2 = tooling. **Fail-closed:** if the detector
cannot be imported it exits **2, never 0** — a checker that reports clean because it failed to
load converts an unknown into a false assurance, which is worse than no checker. `--json` for
scripting, `--quiet` for cron-style use, `--active-only` to scope to `.tasks/active/`
(completed tasks are included by default because they are the best evidence of how widespread
the shape is), `--tasks-dir` / `--framework-root` for fixtures (PL-213). Ad-hoc:
`bash scripts/check-verification-pipefail.sh --active-only`. Fixtures:
`bash tests/verification-pipefail-check-fixtures.sh` (8 assertions; pins the wrapper's
scoping, exit codes and fail-closed contract — not the heuristic, which belongs upstream).

Safe rewrites, and the ones to use in every new `## Verification` block:

```
cmd > /tmp/.out 2>&1 && grep -q "PATTERN" /tmp/.out  # THE DEFAULT — no pipe at all
grep -q "PATTERN" <<< "$out"                          # herestring — no pipe either
test -z "$(git status --porcelain <path>)"            # for emptiness checks
```

**`echo "$out" | grep -q PAT` is the EXCEPTION, not the default, and it is only
conditionally safe (T-2743).** It is a pipe like any other: `echo` writes the whole
capture, so once `$out` exceeds the 65536-byte pipe capacity and `grep -q` matches
early, grep exits, echo takes SIGPIPE, and the line returns **141 — the exact L-387
failure the rewrite exists to avoid.** Measured both ways: on a 146,366-byte
Watchtower page (T-2743, 3/3 runs, deterministic) and again at the boundary
(T-2852 — clean at 32 KiB, 141 at 64 KiB, nondeterministic in the band between).
Rendered routes run 50–200 KB, so anything that curls a page is over the line.
The file-redirect form also preserves the PRODUCING command's exit code, which the
capture form discards — a 404 yields an empty capture that grep merely fails to
match. Prefer the redirect; reach for the capture only when you know the size.

This claim lives in four places and T-2743 corrected only one of them. The
authoritative copy is `.tasks/templates/default.md` § Pipefail/SIGPIPE, which states
the bound in full. The framework's own detector
(`lib/reviewer/static_scan.py::detect_l387_sigpipe_risk`) still exempts an `echo`
upstream **unconditionally**, so it cannot flag the shape it recommends — a
self-sealing blind spot, filed upstream per G-062 rather than patched here.

The check flagged **its own author** one commit after shipping — a `... | wc -l | grep -qx 0`
line in T-2819's Verification block took active findings 150 → 151. That is the intended
behaviour and the best available evidence it is load-bearing.

### Stranded auto-deferred pickup envelope check (T-2801, G-063 for the queue)

The pickup pipeline routes an inbound envelope to `.context/pickup/auto-deferred/` when it is
blocked on a local inception task (G-059). T-1425 added `pickup_write_breadcrumb()`, dropping
a `<envelope>.breadcrumb.yaml` naming the blocking task — explicitly so operators and
`fw pickup auto-deferred list` could trace *why*. T-2072 then made `fw pickup
promote-deferred` resolve the blocking task from that same file in order to promote the
envelope once the blocker ships.

**Both halves depend on the breadcrumb; neither checks that it exists.** With no breadcrumb
there is no blocking task to re-evaluate, so the envelope is not delayed — it is permanently
stranded. The only surface that shows it degrades silently: `fw pickup auto-deferred list`
prints `blocked-by=? reason=? at=?` and says nothing is wrong, while `fw pickup status` counts
it as one more deferred item, indistinguishable from one deferred yesterday for a good reason.

The cost, measured on the one envelope that was stuck: **P-043** (2026-06-08) is a careful
report of two framework bugs blocking every inception decision, with file:line references and
recommended fixes. It was never filed. Seventy-three days later **BUG 2 had been independently
re-discovered and re-fixed as T-2304** — with the fix P-043 already recommended — and **BUG 1
was still live** at `update-task.sh:791`. So the real cost of a stranded envelope is not a late
report; it is the same bug solved twice while its sibling stayed open. (BUG 1 is now filed at
`framework:pickup` offset 17, along with the breadcrumb hole itself, which is in vendored
`lib/pickup.sh` and therefore upstream's to close — G-062.)

`scripts/check-pickup-deferred-freshness.sh` is a **deploy-time / ad-hoc check, NOT a cron
canary** — same tier as `check-cron-install-drift.sh`. It pairs with the framework-pickup
canary (T-2231), which guards the inbound **rail**; this guards the **queue** behind it.
Classes: **STRANDED** (no breadcrumb — un-promotable by construction, FIRES), **STALE**
(breadcrumbed but deferred beyond `--threshold-days`, default 30 — either the blocker shipped
and promotion missed it, or it never will, FIRES), **deferred** (breadcrumbed and within the
window — healthy, the mechanism working). Both firing classes print the envelope's `summary:`,
because the entire failure mode is that nobody knows what is in the file.

**Age is read from the recorded timestamp, never file mtime** — the breadcrumb's
`deferred_at:`, else the envelope's `source.timestamp:`, and only then mtime with an explicit
"unreliable" label. A checkout, clone or worktree stamps every file with the checkout time, so
an mtime-based age reports every envelope as brand new and STALE silently stops working
(PL-213): P-043 is 73 days old and mtime called it 0.

**It detects and never drains.** Promotion belongs to `fw pickup promote-deferred`; discarding
is a human judgement about whether the work still matters. A checker that auto-drained would
turn a visible backlog into a silent one — the exact trade this exists to reverse. Exit codes:
0 healthy · 1 stranded/stale · 2 tooling (a missing pickup tree is never a false clean; an
absent `auto-deferred/` is healthy — nothing was ever deferred). `--json`, `--quiet`,
`--threshold-days N`, `--dir PATH`; test hooks `PICKUP_DEFERRED_DIR` /
`PICKUP_DEFERRED_THRESHOLD_DAYS`. Fixtures: `bash tests/pickup-deferred-freshness-fixtures.sh`
(18 assertions).

### Episodic-store readability check (T-2805, G-063 for the memory layer)

Episodic memory is one of the framework's three memory types and it is the only one whose
corruption nothing would report. Three layers are silent, and each one alone would have been
survivable:

- **Write.** `agents/context/lib/episodic.sh::mine_git_timeline` writes mined git subjects into
  a **double-quoted** YAML scalar escaping only `"`. Every other backslash arrives raw, and a
  double-quoted scalar accepts only a fixed escape set — so an ordinary commit subject
  containing a regex class, a glob or a version string emits `\-` / `\|` / `\*` / `\.` / `\[`
  and the file is unparseable from that line on. The `ScannerError` is raised once, to whoever
  was watching that terminal; the task still completes and the file is still written.
- **Audit.** The `episodic` section runs **hourly on cron** and checks only that the file
  **EXISTS**. It never opens it. A corrupt episodic is reported as *"All completed tasks have
  episodic summaries"* — which is worse than no check, because a check that names the artefact
  and inspects only its filename consumes the attention a real one would have got.
- **Read.** The one real consumer, `web/shared.py::get_episodic_tags`, ends its parse in
  `except yaml.YAMLError: continue`. No log line, no counter, no degraded mode. The task simply
  has no tags and the page renders 200.

Reported upstream by `832-Workflow-designer` (`framework:pickup` offset 18), who found the
generator bug and correctly identified the blindness as the more valuable half but did not have
the reader defect or the audit's existence-only check. Measured here the first time anyone
looked: **29 of 2259 files**, the oldest dead since March.

`scripts/check-episodic-parse.sh` is the reader that was missing — a **deploy-time / ad-hoc
check, NOT a cron canary**, same tier as `check-cron-install-drift.sh` (T-2561) and
`check-task-id-collisions.sh` (T-2800). It asserts **the real consumer's property**, not a
weaker one that is easier to pass: every episodic must `yaml.safe_load` into a **mapping** —
byte-identical to what `get_episodic_tags` requires, so a file this check calls unreadable *is*
being dropped by Watchtower today. Files that parse to a non-mapping are included for the same
reason: the reader's `isinstance(edata, dict)` guard discards them just as quietly.

It **classifies**, because the repair differs sharply per class and a flat list of 29 would not
tell an operator what to do. All classes fire — a legacy-format file is exactly as invisible to
the reader as a corrupt one, so calling it healthy would be false — but each carries its own
remediation: `CORRUPT-ESCAPE` (the generator bug; 4 here), `CORRUPT-OTHER` (3), `LEGACY-MULTIDOC`
(`---` frontmatter plus a body, `safe_load_all` reads it whole; 8), `LEGACY-MARKDOWN` (an older
generator's `summary:` line then markdown; 14), `NOT-A-MAPPING`. The two legacy classes have
**fully intact content** and want a format migration; only the 7 corrupt ones are damaged.
Classifying is what keeps 29 findings from becoming a wall people learn to force past — the
fatigue lesson T-2818 documented from the other direction.

**It detects and never repairs.** Regenerating a `CORRUPT-ESCAPE` file *before* the vendored
generator is fixed reproduces the identical bytes, so an auto-repair would report success while
changing nothing — converting a visible backlog into a silent one, the exact trade this exists
to reverse. Exit codes: 0 = all readable · 1 = one or more are not · 2 = tooling. **Fail-closed:**
missing `python3`, missing PyYAML, or an absent directory all exit **2**, never 0 — a checker
reporting clean because it failed to load is the same disease. `--json`, `--quiet`, `--dir PATH`;
test hook `EPISODIC_DIR`. Fixtures: `bash tests/episodic-parse-check-fixtures.sh` (31 assertions,
each class built from a real failure shape in this corpus; three mutants pinned — disabling
escape-recognition reddens 5, weakening `safe_load` to `safe_load_all` reddens 5, failing open on
an absent dir reddens 1).

The generator, the audit's existence-only check, and the reader's swallowed exception are all
**vendored** (G-062) — a local edit is erased on the next re-vendor — so they were filed back at
`framework:pickup` offset 20 rather than patched here.

### Local fixes to vendored code are deleted by the next re-vendor (T-2812, G-062)

`git log -- .agentic-framework/` carries 124 commits, of which **8 are wholesale vendor
events** — `fw upgrade`, `bootstrap-replace`, `fw update v1.6.7 → v1.6.295 (444 files)`. That
is roughly **one every two months**, and each replaces the tree: any local fix upstream does
not carry is deleted, silently, by a routine maintenance operation.

Found the hard way. **T-2813** (top HV/LC, BVP 99) was about to close as complete — 12/12
fixtures green against the real `lib/pickup.sh` — with its fix unfiled upstream and one
`fw update` from deletion, while a re-vendor was being proposed on another branch (T-2705).

**`.vendor-divergence.yaml` is the register** (convention borrowed from peer project
832-Workflow-designer, who cite theirs in filings). It records the declared
`last_vendor_event` baseline plus every local change since, in two classes:

- `divergences:` — a real local fix. `status:` is `local-only` (a re-vendor DELETES it),
  `filed-upstream` (reported, not yet confirmed carried — still at risk), or `landed-upstream`
  (confirmed; the local copy is redundant and a re-vendor is safe for it).
- `not_divergence:` — recovery of files untracked on our side, ports OF upstream fixes,
  committed artifacts, mode-only changes. A re-vendor costs nothing. **Recording these matters
  as much as the firing set** — without the distinction the register grows to include
  everything touching the vendored tree and becomes as unreadable as the commit log it replaces.

Currently registered: **T-2813** (pickup fail-open), **T-2304** (`update-task.sh` sys.path —
a defect that has already cost this project twice: reported in a pickup envelope that sat
stranded 73 days, then independently re-discovered and re-fixed), **T-2469** (budget-gate
wrap-up deadlock, PL-265). All three filed upstream at `framework:pickup` offset 27.

`scripts/check-vendor-divergence.sh` surfaces local changes since the baseline that nobody has
classified. **The baseline is DECLARED in the register, never detected** — the first
implementation grepped commit subjects for vendor-event keywords and picked one saying
"re-vendor *recommended*" as though it had happened, silently shortening the at-risk window
from 12 commits to 4. A heuristic that can quietly understate the answer is worse than a
constant someone must update deliberately. Exit 0 = all classified · 1 = unregistered change ·
2 = tooling. **Fail-closed**: a missing, baseline-less, or unparseable register exits 2, never
0. Fixtures: `bash tests/vendor-divergence-fixtures.sh` (10 assertions, weighted toward the
FIRING cases — a register-driven check is trivially green when everything is registered, and a
green check that cannot go red is not a check). **After any re-vendor**: update
`last_vendor_event`, re-check the modes flagged under `mode-only`, and re-verify anything still
marked `filed-upstream`.

**Run it before proposing or accepting a re-vendor.** That is the moment it exists for.

**A re-vendor also rewrites this file, and the boundary is positional (T-2015).** `fw upgrade`
step 1/10 takes everything from `## Core Principle` to **end of file** as framework-managed and
replaces it with the template; everything above survives. Nothing in CLAUDE.md marks that line.
Measured 2026-08-20: this file is 2493 lines, `## Core Principle` is at 1650, so **844 lines are
in the destroyed half — including the entire Quick Reference table** and its 29 project-specific
operator entry points. The framework does diff against `CLAUDE.md.bak` and print a lost-line
count, then proceeds: observation, not prevention, and the `.bak` is gone at the next commit
prune. It has fired three times in this lineage. Filed upstream at `framework:pickup` offset 29.
**The pre-re-vendor checklist lives at the top of `.vendor-divergence.yaml`** — the short version
is `cp CLAUDE.md CLAUDE.md.prevendor` first and `diff` after.

**Two `lib/upgrade.sh` defects were filed on 2026-08-20 (offsets 28 and 29), 76 days after both
were diagnosed.** Each had an acceptance criterion reading "framework-agent prompt artifact
written to `docs/reports/…` for operator copy-paste" — both ticked, neither ever pasted. If you
find yourself writing a report to a file so a human can relay it, post it to `framework:pickup`
instead; the rail has been there the whole time.

### `revisit_at` reminders only fire if the cron sets PROJECT_ROOT (T-2810, G-053)

`revisit_at` + `revisit_evidence_needed` (T-1451) are the framework's **only** structural
"come back to this later" mechanism, and the daily scan that reads them
(`agents/context/revisit-due-scan.sh`, wired by T-1452) **silently found nothing for months**
in this repo.

`revisit-due-scan.sh` resolves `PROJECT_ROOT` by walking up from its own location for
`.framework.yaml` **or `FRAMEWORK.md`** — and a vendored framework ships its own
`FRAMEWORK.md`. So the walk stops at `.agentic-framework/`, `TASKS_DIR` becomes
`.agentic-framework/.tasks/active` which does not exist, and the script `exit 0`s. That is
indistinguishable from "no revisits are due". The `cd` in the generated cron line does **not**
help: the walk starts at `${BASH_SOURCE[0]}`, not at cwd. Cost when finally checked: **T-1898
ripe 45 days, T-2250 ripe 26 days**, neither ever surfaced.

**The fix is in `.context/cron-registry.yaml`, and it is load-bearing.** That entry now sets
`PROJECT_ROOT="$(pwd)"` before the script — the script's own comment names the env var as the
intended override ("prefer env var (set by cron line)"), so this is the supported path, not a
workaround. Do not remove it, and set it the same way for any new job invoking a vendored
`agents/` script. The script itself is vendored (G-062) and was not patched; the marker
collision and the `exit 0` were filed upstream at `framework:pickup` offset 24, with the
`exit 0` flagged as the more serious of the two — "I could not look" and "I looked and found
nothing" must not share an exit code.

**Registry changes need installing from the main checkout**, not from a worktree:
`fw cron install` derives the `/etc/cron.d` filename from the checkout's basename (the T-2815
defect), so running it here would write `agentic-audit-<worktree-name>` instead of updating
`agentic-audit-termlink`. Fixtures: `bash tests/revisit-due-cron-fixtures.sh` (6 assertions;
the load-bearing one *reproduces* the defect against a fixture project carrying both markers —
without that leg the fix proves nothing).

### BVP estimator — safe to run, with one caveat that is about ruamel, not corruption (T-2809)

A June 2026 commit message (`444a7e9b3`) recorded, and never filed, that `fw bvp estimate all`
corrupted old-format task frontmatter into unparseable YAML across 106 files. **Checked
2026-08-20: it does not reproduce.** On scratch copies — never live task files, since the
reported defect destroys what it writes to — `estimate all` + `cost-all` across 25 anchor-less
old-format tasks wrote all 25 with 0 errors and 0 malformed frontmatter, and the no-`ruamel`
fallback branch (forced `_HAS_RUAMEL=False`) also produced valid YAML with the key present.
The vendored estimator had no git history to diff against before T-2807 committed it, so
"fixed upstream" is the inference that fits, not a proven bisect. Reported at
`framework:pickup` offset 23.

**This is recorded as an outcome, deliberately not as a warning.** Documenting a hazard that
no longer reproduces is cargo-culting: it taxes every future reader and cannot be falsified by
anyone who trusts it. Run the estimator normally.

**The one real caveat.** Without `ruamel` installed, the estimator rewrites frontmatter via
`yaml.safe_dump`, which **silently strips every comment** — and the task template documents its
own fields in comments (`# bvp_scores:`, `# revisit_at:`, the BVP block's semantics). Valid
YAML, exit 0, no warning. `ruamel` IS present on this host (0.19.1), so the good path is the
one that runs here; check `python3 -c "import ruamel.yaml"` before running the estimator on a
host you have not used it on. Exposure if it does fire: 1792 of 2460 task files (73%) lack the
`bvp_scores_proposed` anchor and an unscoped `estimate all` writes to all of them.

**Cost estimates need the estimator directly.** `fw bvp` surfaces `estimate` but **not** the
`cost-*` subcommands, so `fw bvp estimate all` populates BVP scores and leaves the COST axis
empty — and a quadrant is BVP-median × COST-median, so `--quadrant hv-lc` silently returns
"No tasks match" rather than saying the axis is missing. To get quadrants, run both:

```bash
.agentic-framework/bin/fw bvp estimate all --statuses started-work
PROJECT_ROOT="$PWD" FRAMEWORK_ROOT="$PWD/.agentic-framework" \
  python3 .agentic-framework/agents/termlink/bvp-estimator/estimator.py cost-all --statuses started-work
.agentic-framework/bin/fw bvp --quadrant hv-lc --include-proposed
```

Confirmed scores are **§ACD sovereignty-gated** (`fw bvp confirm --i-am-human`): an agent can
propose, only a human can confirm, and the `SOURCE` column keeps the two distinguishable.

### Cross-branch task-ID collision + duplicate-work check (T-2800)

Every worktree allocates task IDs by scanning its **own** `.tasks/` for the highest ID and
incrementing. Worktrees materialise only their own branch's files, so no branch can see any
other branch's allocations — and the framework creates those worktrees itself (bgIsolation),
so this is a framework-made race, not an operator oversight.

It was understood in March 2026. T-229 renumbered six colliding tasks by hand and recorded
the cause verbatim — *"task counter not safe for concurrent work"* — routing the fix upstream
as G-007. Nothing landed. By August 2026 twelve IDs collided across three branches, and
**three agents had independently implemented fixes for the same two defects** (the canary
stderr sink and the untracked static-check allowlists). The duplicate-ID check T-229 left
behind reads a single working tree and is structurally incapable of seeing any of it; it had
been passing cleanly throughout.

`scripts/check-task-id-collisions.sh` is a **deploy-time / ad-hoc check, NOT a cron canary** —
same tier as `check-cron-install-drift.sh`. Four axes:

- **Axis A — COLLIDING IDS.** An ID claimed by two or more branches beyond the merge base.
  **FIRES** (exit 1), printing each branch's filename so a cherry-pick (same ID, *same* task —
  not reported) is distinguishable from the real defect (same ID, different tasks).
- **Axis C — DUPLICATE NEW FILES.** The same path created independently on more than one
  branch with different content. **FIRES.** Precise where axis B is a heuristic: two branches
  both adding `scripts/check-verification-pipefail.sh` is duplicated work, full stop, and one
  implementation is about to be discarded. Identical blobs (shared history / cherry-pick) are
  excluded, as are per-session artifacts under `.context/handovers|episodic|audits|pickup`.
- **Axis B — NEAR-DUPLICATE TITLES.** Titles across *different* branches sharing rare terms.
  **WARNS, never fires.** It catches duplication *before* either branch has written the file —
  earlier than C, at the cost of being a judgement. A heuristic that blocks on a judgement
  gets switched off the first time it is wrong, so this one only advises.
- **Axis D — SAME-LINE FIX DUPLICATION** (T-2915, closing the T-2828 GO). Two sides MODIFIED
  the same pre-existing file since their mutual merge base, end blobs differ, and both
  DELETED at least one identical non-trivial line — you can only delete the same original
  line by touching the same pre-existing code. **WARNS, never fires** (axis-B rule). **Main
  IS a comparison side** (per-pair merge base) — axis C excludes BASE by construction, which
  is how T-2687(main)/T-2824(branch) independently fixed the identical `termlink_topics`
  defect 8 days apart with nothing firing. The signal was MEASURED on the real branch set
  (T-2915 spike): same-file fired on ~every pair (useless), same-ADDED-line matched
  generated boilerplate (noisy), same-DELETED-line hit 13/52 candidate files with zero
  boilerplate matches and caught the tools.rs ground truth. Blob-identical files are
  excluded first (carried/squash-merged work — removed a 133-file false-positive pair).
  `--no-fixes` skips it; `TASK_COLLISION_FIX_MIN_LEN` (default 11) tunes the
  deleted-line length floor.

A and C are facts and fire; B and D are hints and do not. They are genuinely complementary, and
the August incident proves it in both directions: **axis B** found the two branches that both
fixed the untracked-allowlist bug, and **axis C** found the two that both wrote
`check-verification-pipefail.sh` — whose task titles share no rare term, so B missed it
entirely. Axis C also surfaced four `.fabric/components/*.yaml` cards written twice.

Axis B scores on **shared rare words, not set overlap**, for two reasons found the hard way.
The filename slug is truncated at ~40 chars, which is exactly where the discriminating words
live (`blanket-contextworking-gitignore-makes-s` and `static-check-allowlists-live-under-a-git`
describe the same defect and share no token), so titles are read from the `name:` frontmatter.
And plain Jaccard is the wrong measure even on full titles — the real August pair scores
~0.19, and any threshold low enough to catch it drowns in noise. What identifies it is that
both say ALLOWLISTS and STATIC, terms rare across the corpus. Tunables:
`TASK_COLLISION_RARE_DF` (default 4 — a word in ≤4 titles counts as rare) and
`TASK_COLLISION_MIN_SHARED` (default 2 rare words to report a pair).

Exit codes: 0 clean · 1 colliding ID(s) or duplicate file(s) · 2 tooling (not a git repo / bad
base ref — never a false clean). `--json` carries all three axes with separate counts;
`--quiet` prints only when something fires; `--no-titles` skips axis B (and the summary then
says so rather than claiming titles are clean); `--base REF` sets the merge base (default
`main`). Test hooks: `TASK_COLLISION_BASE`, `TASK_COLLISION_BRANCHES`,
`TASK_COLLISION_SIMILARITY`. Fixtures: `bash tests/task-id-collision-fixtures.sh` (23
assertions, scratch repo with real branches; pins both false-positive guards — a cherry-picked
task and an identical file must never fire).

**Run it before starting work and before a merge.** On firing, renumber before merging (T-229
is the worked example) and pick new IDs above the highest claimed on ANY branch — note that
`fw task create` has **no `--id` flag**, so the rename is manual. This task hit that itself:
the allocator handed it T-2699, which charter-review already held, so it was created and then
renamed to T-2800 by hand. The check detects the collision; it cannot prevent it, because
allocation is vendored framework code — filed upstream per G-062 at `framework:pickup`
offsets 15–16.

### Vendored-framework recoverability check (T-2814 + T-2817)

`.gitignore:21` carries a blanket `.agentic-framework` rule labelled "Framework symlink
(machine-specific)". That label predates vendoring: the tree is now vendored with ~1565
files tracked, and ignore rules do not apply to already-tracked files — so everything
already in git keeps working and nothing looks broken, while every framework file added
*after* the rule landed is silently untrackable (`git add -A` skips it, `git status` never
mentions it). The result is invisible drift between the framework that RUNS and the
framework that is RECOVERABLE. Live casualties: `lib/bvp.sh` and the whole `policy/`
directory (`value-drivers.yaml`, `bvp-scoring-rubric.md`) are untracked, so the §ACD
sovereignty-gated BVP driver weights exist only on one host's disk, and `fw bvp` fails
outright in any clean clone.

`scripts/check-framework-tracking-drift.sh` is a **deploy-time / ad-hoc check, NOT a cron
canary** — the same tier as `check-cron-install-drift.sh` (T-2561). It runs **two
complementary axes**, because each is blind exactly where the other fires:

- **Axis A — UNTRACKED** (T-2814): a file is on disk but absent from `git ls-files`. Only
  visible in the checkout that CREATED the drift. Fires on `bin/ lib/ policy/ agents/`
  (clean-clone-breaking) **and, since T-2811, on `web/**/*.py` and `web/templates/**`**;
  untracked `docs/` and `web/static/**` are informational; `__pycache__/`, `*.pyc`, `*.pyo`,
  `.git/` are excluded.

  **The `web/` half was originally wrong, and the checker proved it the hard way (T-2811).**
  `web/` was lumped in with `docs/` as informational, on the reasoning that a missing page is
  a gap rather than a broken install. True for `docs/`; false for most of `web/`. Asked for
  Watchtower links, `fw serve` would not start — `ModuleNotFoundError: web.blueprints.bvp` —
  and the drift checker was reporting the same tree **clean**. Fixing that produced HTTP 500
  on every route: `TemplateNotFound: _pins.html`, because `base.html` includes it. **39
  untracked `web/` files** (4 blueprints, 17 templates, 11 fonts, 6 JS, 1 CSS). Flask
  registers blueprints in `create_app()`, so a missing module is a crash before the app binds
  a port; `base.html` includes partials, so one missing template is a 500 on every route that
  extends it. `web/static/**` stays informational for the symmetric reason — a missing font
  renders an ugly page, not a broken one, and firing on cosmetics is how a check earns being
  ignored.
- **Axis B — DANGLING** (T-2817): tracked framework code SOURCES or EXECUTES a
  `"$FRAMEWORK_ROOT/<path>"` that does not resolve. Only visible where the file is
  MISSING — a clean clone, a fresh deploy, or a git worktree (which materialises tracked
  files only).
- **Axis B — NOT-EXEC** (T-3145): the reference RESOLVES but cannot be executed. Distinct
  class, distinct errno (**126**, not 127) and distinct remediation (chmod, not recover the
  file), so it is counted and reported separately — telling an operator to go find a file
  that is sitting right there is how a check earns being ignored.

  **The verb list omitted `exec`, which is the only form where file mode is load-bearing.**
  `bash foo.sh` runs a mode-644 file perfectly well; `exec foo.sh` returns 126 Permission
  denied. The anchor matched `.` · `source` · `bash` · `sh` · `python3` · `python` and not
  `exec`, so all three exec-position references in the tree were outside the candidate set —
  including `exec "$FRAMEWORK_ROOT/lib/build.sh"` at `bin/fw:7534` against a file vendored at
  **100644**. `fw build`, advertised at `fw help` line 85, exits 126, and every existing test
  reported clean: axis A saw a tracked file, a mode-drift comparison saw the index agreeing
  with the disk (**both wrong** — index-vs-disk cannot see a value both sides got wrong), and
  axis B never looked. Measured 59 → 62 references after adding `exec`; the three new ones are
  exactly the exec sites, and one of them was dead. Filed upstream at `framework:pickup`
  offset 162; **not patched here** (`lib/build.sh` is vendored, G-062), which is why the local
  detector is the half that survives a re-vendor — same split as T-2859 and T-2833.

  Executability is required for `exec` position **only**. Testing `-x` on every verb would
  also "catch" `build.sh` and would be wrong: nearly every `. "$FRAMEWORK_ROOT/lib/*.sh"` in
  a real tree targets a non-executable library. That precision is pinned by a fixture, and
  by a mutant — stripping the `-x` test makes the check report the dead reference as clean,
  which is exactly what the real tree did for as long as the omission stood.

  Acknowledgements live in `.context/checks/framework-notexec-allowlist` (git-tracked per
  T-2681), `<path>  # reason`, and are **counted and reported on the clean path** so a green
  never conflates "clean" with "acknowledged". An entry is only appropriate when the finding
  is real and we are structurally barred from fixing it — anything fixable here gets fixed,
  not listed — and each entry must state what would let it be deleted. Currently one:
  `lib/build.sh`, pending the upstream fix. `--allowlist PATH` /
  `FW_TRACKING_NOTEXEC_ALLOWLIST` override; an ABSENT ledger acknowledges nothing rather
  than excusing everything (pinned by fixture).

  **Scope (T-2680):** this covers references in the listed VERB positions only. A bare
  `"$FRAMEWORK_ROOT/bin/foo" --args` invoked with no verb also needs the bit and is NOT
  covered — matching that shape is the broad anchor the T-2817 narrowing rejected as 44-of-47
  noise. Both output paths say so.

Axis A alone reports a worktree as clean while `fw bvp` is broken in it — that is how
T-2817 was found, the detector saying "no drift" seconds after the tool it protects failed
in the same directory. Axis B is the axis that matters for anyone who did not create the
drift.

**What DANGLING actually costs (T-2806).** The known price was "`fw bvp` fails in a clean
clone". It is worse than that: `lib/evolution_log.sh:52` sources `lib/arc_membership.sh`
**unguarded**, and `update-task.sh::check_evolution_log` guards only the outer file — so with
`arc_membership.sh` absent, **`fw task update --status work-completed` exits 1 AFTER passing
every gate**. The run prints all ACs checked and all verification commands PASS, then dies and
leaves the task `started-work` in `active/`. Scoped to `workflow_type: build`, which is why
inception and refactor tasks finalize fine and the failure looks intermittent. Measured when
first checked: **24 active build tasks fully ticked and stuck**, including every task two
consecutive sessions had completed. Note this produces the same end state as the G-066
finalization-bypass class (work done, register disagrees) by a different route, so the T-2290
canary — which gates on `status != work-completed` in `completed/` — structurally cannot see
it: these files never leave `active/`.

**Axis B's count is a LOWER BOUND, and should be read as one.** Recovering the two reported
files exposed a third, `agents/termlink/bvp-estimator/estimator.py`, which was invisible while
its parent was missing — dependencies chain, so expect to converge over a couple of rounds
rather than one. More importantly, axis B can only see `source`/`exec`-position references in
TRACKED code: a hook that `settings.json` invokes by path is a real dependency that nothing
static can detect. When T-2806 measured the whole subtree the true figure was **113 untracked
files out of 326** on disk under `lib/ bin/ policy/ agents/` — 35% of the framework's
executable surface, 19 of them hook scripts — against the 2 axis B had named.

**Three of four pre-commit guards were failing open in every worktree (T-2807).** Claude Code
hooks are fine — `.claude/settings.json` invokes them as an absolute path into the MAIN
checkout's `bin/fw`, so `check-active-task`, `check-tier0` and the rest all run. **Git hooks are
not.** `.git/hooks/pre-commit` is shared across worktrees but resolves `FRAMEWORK_ROOT` to
`$PROJECT_ROOT/.agentic-framework` — the worktree's own copy — and gates each scanner on `-f`.
Measured against this branch's HEAD before recovery: `secret-scan.sh` was tracked and **did**
run, but `large-file-scan.sh` (T-1845, the G-058 gate), `master-guard.sh` (blocks commits to
master) and `dup-task-scan.sh` were all untracked, therefore absent, therefore silently skipped.
The fail-open is deliberate (T-2061, working around the T-2052 chmod gap) and is right for a
missing optional tool; it is wrong when the file is missing because it was never committed, and
the hook cannot tell those apart. T-2807 recovered the remaining 92 files, taking `lib/ bin/
policy/ agents/` to fully tracked. **Recovering more code can surface new false positives in
axis B** — the 92 files included help text carrying `. "$FRAMEWORK_ROOT/lib/<name>.py"`, a
documentation placeholder that the source-position anchor matches correctly and which is still
not a real reference; angle-bracket placeholders now join `$`-interpolations in the
skipped-and-counted set, for the same reason (cannot be resolved statically, and one permanent
false positive makes a check nobody reads).

**The deadlock, and how T-2806 broke it.** The checkout that HAS the files cannot `git add`
them (the blanket rule is still in force there; `git add` on an ignored path exits 1 with its
message on **stderr** — T-2803 reported false success for a while by reading stdout only). The
checkout with the corrected rule does not HAVE the files, because they were never committed and
there is nothing to pull. Neither finishes alone. The break is to move the bytes across — via a
session rooted in the other checkout, so T-559's project boundary is respected and that
checkout's own governance runs in its own process — then commit where the corrected rule
applies. **Scan before committing:** framework files from another disk can carry machine-local
paths or credentials, and a commit is permanent. T-2806's scan flagged a bare 64-hex in
`policy/designer-pin.yaml` which on inspection is a `sha256:` reproducibility pin, not a secret
— a true pattern match and a false risk, which is exactly the judgement the scan exists to
surface rather than resolve on its own. **Axis B's anchor is deliberately narrow**: matching every `"$FRAMEWORK_ROOT/..."`
occurrence yielded 47 hits of which ~44 were noise (bare `$VAR` interpolations,
`path/to/script.sh` usage examples, and the framework's own `tests/`/`tools/`/`.git/` which
a vendored copy legitimately omits); restricting to source-or-execute position
(`.`/`source`/`bash`/`sh`/`python3`/`python`) and dropping comment lines took it to 2 real
hits with zero false positives. Same precision-over-recall trade as the sibling static
checks above. Paths still carrying a `$` in the tail are skipped and counted, never guessed.

Exit codes: 0 = neither axis fires, 1 = untracked load-bearing file OR dangling reference,
2 = tooling (not a git repo / framework dir absent). `--json` carries both axes separately
(`firing[]`/`firing_count` for A, `dangling[]`/`dangling_count`/`refs_checked`/
`refs_skipped_dynamic` for B); `--quiet` drops the remediation block; `--root DIR` points
the scan at a fixture tree (PL-213). Ad-hoc: `bash scripts/check-framework-tracking-drift.sh`.
Fixtures (no real framework): `bash tests/framework-tracking-drift-fixtures.sh` (axis A) +
`bash tests/framework-dangling-ref-fixtures.sh` (axis B). Operator action differs per axis:
**UNTRACKED** — narrow the `.gitignore` rule and `git add -f` the load-bearing subtrees,
reviewing for anything machine-local or secret-bearing; **DANGLING** — you cannot fix it in
the checkout that fires, because the files were never committed and there is nothing to
pull; fix it in the checkout that still HAS them (run the check there, it reports them as
UNTRACKED), commit, then update.
### Platform-lock static check (T-2693, Directive #4 Portability)

The fifth source-level static check (sibling of alloc-sink, drain-sink, silent-exit,
busy-spin), for the Constitutional Directive no review had ever examined: **#4
Portability — "no provider/language/environment lock-in"**.

**Why it exists.** README §Platform Support asserts `Yes` for macOS five times (core
binary, PTY operations, Terminal.app spawn, tmux spawn, TCP hub) and names Homebrew
the *recommended* macOS install; `release.yml` cross-builds two Darwin targets and
publishes them. Yet T-2690 found `whoami`'s PID-ancestor fallback reading
`/proc/<pid>/stat` with `.ok()?` on **both** the CLI and MCP surfaces. There is no
`/proc` on macOS, so the chain collapsed to `[self]` and `whoami` reported "ambiguous
— here are all candidates": not a crash but a **plausible wrong answer**, recommending
an action that could never help. Every existing guard was structurally blind to it —
alloc/drain/silent-exit/busy-spin all ask about resource safety; none asks "does this
run off Linux?".

**What it flags** in the product crates: `/proc/` and `/sys/` path literals, and
`Command::new("<tool>")` for `ss` · `systemctl` · `journalctl` · `ufw` · `setsid` ·
`nproc` · `lsb_release`. **What it deliberately does not flag:** cross-platform
subprocesses (`git`, `sh`, `bash`, `ssh`, `tmux`, `pgrep`), `osascript` (macOS-only
*on purpose* — the Terminal.app spawn backend), and comment lines. The check is about
*undeclared lock-in*, not about using a subprocess.

**The allowlist rule is stricter than its siblings.** A grep cannot distinguish "reads
/proc and silently returns the wrong answer" from "reads /proc behind a runtime probe
that names the limitation" from "unreachable off Linux because the enclosing block
already required a Linux-only tool". So each entry in
`.context/checks/platform-lock-allowlist` must state **how the non-Linux path
behaves**. "It's fine" is not a reason — the acknowledgement *is* the portability
documentation for macOS, kept next to the signature so it stays in sync with the code.
What is NOT allowlisted is the interesting part: the whoami defect was **fixed**
(T-2691), not acknowledged. An entry is for a site that behaves correctly off Linux,
never for one that fails quietly.

Current tree (measured 2026-09-30): 10 sites scanned, all 10 acknowledged with cited
degradation reasons — the `/proc/` reads guarded by `procfs_available()`, the probes
themselves, and the tests pinning the probe on both platform families. (An earlier "8
sites" figure here had gone stale.) Exit 0 clean / 1 unacknowledged / 2 tooling;
`--json`; `--root` + `--allowlist` for fixtures. Ad-hoc: `bash scripts/check-platform-lock.sh`.
Fixtures: `bash tests/platform-lock-check-fixtures.sh` (26 assertions). **It earned its
keep on first run:** it flagged T-2691's own new test, which asserted `/proc` exists and
would have failed on the macOS CI runner T-2692 was adding in the same session.

**Quoted bare roots are matched too (T-3268).** The scan originally matched only the
trailing-slash forms `/proc/` and `/sys/`, so `procfs_available_at("/proc")` was
invisible — and exactly that line, asserted unconditionally in an MCP test, was the
v0.12.3 macOS red (T-3266) while this check scanned clean. `"/proc"` / `"/sys"` quoted
literals now fire as the distinct primitives `proc-root` / `sys-root` (distinct so the
existing `proc-path` signatures stay valid); fixture 10 and a mutant pin it.

**Companion — macOS CI (T-2692).** `release.yml` gains a `test-macos` job running the
same `cargo test --workspace` as the Linux job. It shipped **non-blocking**, because gating
releases on an unmeasured suite would have violated T-2686's own AC. **It now blocks (T-3271,
2026-09-30).** Its first measured run was red on v0.12.3 (the unconditional `/proc` assertion,
fixed in T-3266), and its first green run was v0.12.4 (run 36719281724). `continue-on-error` is
gone, and both build jobs `needs: [test, test-macos]`, so a red macOS suite now produces no
binaries rather than shipping a macOS break behind a green release.

### Error-code emission check (T-2699, Directive #2 — a refusal is a claim)

The sixth source-level static check. `check-error-code-docs.sh` (T-2213..T-2217)
verifies that every doc-cited `SYMBOL(-320NN)` pairing matches `control.rs` — that
guards the **docs** against the **code**. Nothing guarded either against **reality**:
whether the error can be emitted at all.

**Why it exists.** T-2698 treated the error taxonomy as what it is — a set of
assertions that the system will refuse you under condition X — and found three of 23
codes with zero emission sites: `SESSION_BUSY` (-32002), `MESSAGE_EXPIRED` (-32004),
`PROTOCOL_VERSION_TOO_OLD` (-32011). All three are listed in the T-005 protocol design
as ordinary refusals, so a client written from that table would infer protections that
do not exist: that a busy session refuses a second command, that an over-TTL message is
rejected rather than delivered late, and that the hub turns away peers on too old a
protocol.

`PROTOCOL_VERSION_TOO_OLD` is the shape that matters. `control.rs` ships
`check_protocol_version()` which builds the structured error, plus a passing test
`check_protocol_version_rejects_when_declared_is_older` — and **zero callers**. Every
conventional coverage signal reads green because the *builder* is covered. **Coverage
of a builder says nothing about whether the builder is called.** That is the T-2683
pattern — a guard nothing executes — in compiled Rust rather than shell. Wiring it is
**T-2700** (`owner: human`): it begins rejecting live peers, and this fleet has hosts
~1000 commits stale (T-2377).

**It also found a live wire collision.** `artifact.rs` emitted the bare literal
`-32004` at three sites for "artifact not found" — one wire code carrying two
incompatible meanings, demanding opposite client responses. Invisible to the doc lint,
because a hand-written number cites no symbol. Fixed by `ARTIFACT_NOT_FOUND` (-32024),
pinned by `artifact_not_found_does_not_reuse_message_expired`. The collision had also
*masked* `MESSAGE_EXPIRED`'s deadness — the value appeared on the wire, just never
meaning what the taxonomy said.

**What counts as an emission:** a reference to `error_code::NAME`, **or** the bare
numeric literal, anywhere in the product crates **excluding the defining file** — so
neither a constant nor a builder sitting beside it counts as its own use. Comments are
stripped first: prose *about* a code is not a use of it (the check's own first run
cleared -32011 because a doc comment mentions it).

Genuinely-reserved codes go in `.context/checks/error-code-emission-allowlist`
(git-tracked, T-2681) with a reason stating why it is not emitted **and what would
change that** — "reserved" alone is not a reason, because unstated reservation is how
three of these read as ordinary refusals for months. Current tree: 24 scanned, 3
declared-reserved, 0 unacknowledged. Ad-hoc:
`bash scripts/check-error-code-emission.sh` (0 clean / 1 unacknowledged / 2 tooling;
`--json`; `--def-file` / `--root` / `--allowlist` for fixtures). Fixtures:
`bash tests/error-code-emission-fixtures.sh` (18 assertions).

### Version-derivation static check (T-2746, G-019 prevention for the T-1458 / T-2744 class)

The seventh source-level static check. A crate that reads `env!("CARGO_PKG_VERSION")`
gets its own `Cargo.toml` version unless a `build.rs` overrides it with the git-derived
one. When that override is missing the crate does not fail — it reports a **plausible
wrong version, indefinitely**, which is the Directive #2 shape (a wrong answer, not an
error).

**Why it exists: this happened twice, three months apart, in two different crates.**
T-1458 (2026-05-03) found `termlink-hub` returning `"0.9.0"` from `hub.version` for every
fleet hub regardless of binary freshness. T-2744 (2026-08-15) found `termlink-session`
stamping `"0.9.0"` into every session's metadata while the binary reporting it was at
`0.11.720` — for the entire life of the field.

The first instance produced **PL-148**, which is accurate, specific, and even names the
detection lever ("if a fleet-wide version histogram shows uniform '0.9.0', or matches the
workspace Cargo.toml literal exactly, suspect a missing build.rs"). It did not prevent the
second. A learning that precise failing to prevent a recurrence is the argument for a
structural check rather than more documentation.

**PL-148 also names why the obvious test cannot catch it:**
`assert_eq!(reported_version, env!("CARGO_PKG_VERSION"))` passes whether `env!` resolves to
`0.9.0` or `0.11.1359`, because both sides are the same compile-time constant. The bug is
invisible from inside the crate that has it — which is what makes it a job for a check
reading the build configuration from outside, not for a unit test.

`scripts/check-version-derivation.sh` decides two things independently per crate:
**(1) does it READ the version** (any non-comment `env!`/`option_env!("CARGO_PKG_VERSION")`
in its sources, `build.rs` excluded — a build script reading its own Cargo.toml value is the
mechanism, not a surface); **(2) does it DERIVE the version** (a `build.rs` containing a
non-comment `cargo:rustc-env=CARGO_PKG_VERSION`). Reads-but-does-not-derive fires. A crate
that does neither is never examined — the check does **not** say "every crate must have a
build.rs", and `termlink-bus` / `termlink-protocol` / `termlink-test-utils` are live proof
it discriminates.

**Scope — it detects a MISSING derivation, not a WRONG one.** It reads whether the emit line
is present; it does not evaluate what the build script computes. The T-2744 regression test
(`recorded_version_is_the_build_version_not_the_cargo_toml_constant`) covers that other half
from inside the crate, comparing against a separately-exported copy of the Cargo.toml value
rather than a literal — which is how it escapes the PL-148 tautology. The two are
complements; neither alone is sufficient.

A **test-only** read is deliberately treated as a read: that is precisely the tautological
assertion PL-148 warns about, so a crate whose only use is that one is a crate whose version
cannot be verified from inside itself. Allowlist:
`.context/checks/version-derivation-allowlist` (git-tracked per T-2681), `<crate>  # <reason>`;
entries are counted and reported but do not fire, and the reason must say why the Cargo.toml
version is the *right* answer for that crate. Currently empty on purpose — every crate here
that reads the version also derives it.

Current tree: 4 crates scanned (cli / mcp / hub / session), 0 firing, 0 allowlisted. Exit
0 clean / 1 firing / 2 tooling; `--json`, `--quiet`, `--root`, `--allowlist`. Ad-hoc:
`bash scripts/check-version-derivation.sh`. Fixtures:
`bash tests/version-derivation-check-fixtures.sh` (16 assertions). **Load-bearing:** removing
the emit from `crates/termlink-session/build.rs` fires it with `never emits …`; deleting the
file fires it with `has no build.rs` — the two failure shapes report distinct reasons.

### MCP/CLI parity census check (T-2747, herdr rank 13 — coverage, not execution)

The eighth source-level static check. `crates/termlink-mcp/tests/parity.rs` asserts that an
MCP tool and its CLI verb produce the same structured output. **It covers 24 tools. There
are 260.** The other 236 are not passing — they are UNEXAMINED, and nothing distinguished
those two states.

That is the T-2680 lesson one layer up. The charter-drift canary once reported
`{checked:214, live_off_charter:0}`, which read as "all 214 trace to the charter" when it
had only ever looked for six known families. A suite covering 9.2% of a surface and saying
nothing about the rest has the same shape: its green is a statement about the 24 and gets
read as a statement about the 260. The guard layer had no member that knew the two surfaces
are supposed to correspond (`ls scripts/ | grep -i parit` → nothing).

**Distinct from T-2686.** `parity_topics` WAS covered and DID catch T-2624 — what failed
from 2026-08-12 is that *nothing ran it*. T-2686 closed the **execution** gap; this closes
the **coverage** gap. Different bugs.

**What it does not do:** it does not prevent drift and does not verify that any two
implementations agree. It answers one question — is every MCP tool either ASSERTED by a
parity case or ACKNOWLEDGED with a cited reason? It converts *unexamined* into
*acknowledged*. The value is the **ratchet**: all 236 uncovered tools are enumerated in
`.context/checks/mcp-parity-census-allowlist` (git-tracked per T-2681, a ledger of an open
question in the T-2483/T-2548 tradition, worked down by **T-2748**), so a NEW MCP tool added
tomorrow is in neither set and fires on the next run. Today's gap is frozen and visible;
tomorrow's requires a decision. Both output paths — including the clean one — state the
census (`260 tools: 24 asserted, 236 acknowledged, 9.2%`) plus the scope disclaimer, so a
green can never be misread as full coverage.

**Counting note.** Three counts of "`*_mcp` parallel helpers" were circulating — 83 (T-1904),
68 (`parity.rs` header as of T-2683/T-2689), 94 (herdr worker 3) — none asserted correct.
Measured: **79** distinct `fn *_mcp` names. They disagree because the unit is ill-defined:
`fn to_json_mcp` alone occurs 26 times as a small helper redefined inside separate functions,
and counting it as 26 parallel implementations is as defensible as counting it as one. The
check therefore counts **tools** (`name = "termlink_…"` inside `#[tool(…)]`) — a unit with
exactly one meaning — not helpers.

Comment lines are stripped on both sides: `tools.rs` contains a doc comment quoting
`name = "termlink_help"` that would inflate the total, and `parity.rs` discusses tool names
in prose that would mark them covered without asserting anything. Exit 0 clean / 1 firing /
2 tooling (an empty tool surface is a tooling error, never a clean census). `--json`,
`--tools-file`, `--parity-file`, `--allowlist`. Ad-hoc:
`bash scripts/check-mcp-parity-census.sh`. Fixtures:
`bash tests/mcp-parity-census-fixtures.sh` (26 assertions). **Load-bearing:** deleting an
allowlist line re-fires that tool; renaming a covered tool's reference in `parity.rs` moves
it from covered to unexamined (24→23, 9.2%→8.8%) and fires it.

### Release-artifact drift check (T-2751, the install path nothing guarded)

The ninth source-level static check. The release artifact-name list exists as **three
hand-maintained copies** — `install.sh:70-81` (case arms choosing what to download),
`.github/workflows/release.yml` (the publish block), `homebrew/Formula/termlink.rb` (url
lines) — and nothing verified they agree. Same shape as T-2484 (the charter sentence as
three copies with no transclusion), applied to release artifacts instead of prose.

**Why this one is sharper than ordinary drift.** `install.sh` is the **first** option in
README Quick Start, advertised as "no toolchain required" — the path a new user takes. It
has zero CI coverage. `install-check.yml` guards the **third** option (`cargo install
--git`) and its header still calls that "the documented installation path", which is stale.
So the guarded path is the least-used one, while the unguarded one is piped into `sh` on a
stranger's machine, where a rename surfaces as `die "failed to download"` on **their** host
rather than as a red build on ours. Same class as T-2683 (static checks nothing ran) and
T-2686 (`parity_topics` failing undetected since 2026-08-12).

**install.sh ↔ release.yml is checked bidirectionally**, because the two directions fail
differently and only one of them is loud: *offered-but-unpublished* (install.sh selects a
name release.yml does not ship → the user gets a 404) versus *published-but-unreachable*
(release.yml ships a name install.sh never selects → we build and host a target the primary
installer cannot deliver, and **nobody sees an error** — the Directive #2 shape). They are
reported separately.

**The formula is checked as a SUBSET, not an equality.** Every artifact it references must
be published; a published artifact it omits does not fire — the gnu `linux-x86_64` variant
is deliberately excluded in favour of the musl static one (T-1135), so equality would fire
daily on a decision already made. Only `url "…releases/download/…"` lines are parsed, never
any `termlink-*` token: the formula also contains `Dir["termlink-*"].first`, a runtime glob
that is not an artifact name.

**Scope — it compares NAMES and nothing else.** It does not verify an asset is
downloadable, that `checksums.txt` carries a line for it, or that the binary runs. A
release publishing all five names as empty files passes. Both output paths state this, so a
green is never misread as "the install path works" (T-2680). There is deliberately **no
allowlist**: unlike the sibling checks, every mismatch here is a real defect with a real fix,
so an allowlist could only ever silence a genuine break. An empty extraction from either
side is exit 2, never a clean census — "0 agrees with 0" is vacuously true and would report
green over a parse that silently stopped matching (the T-2747 zero-tools lesson).

Current tree: 5 published, 5 offered, 4 in formula — clean. Exit 0 clean / 1 firing /
2 tooling. `--json`, `--quiet`, `--install-sh`/`--release-yml`/`--formula` for fixtures.
Ad-hoc: `bash scripts/check-release-artifact-drift.sh`. Fixtures:
`bash tests/release-artifact-drift-fixtures.sh` (34 assertions, including a PL-219 control
that the real tree scans clean). **Load-bearing:** renaming one case arm in `install.sh`
fires it twice — once per direction, correctly attributed — and restoring returns it to clean.

**Origin note.** This came out of herdr backlog **rank 20**, which proposed *building* a
`curl | sh` user-level installer. That premise was false — `install.sh` has existed since
T-1134 with checksum verification, multi-arch detection, a no-sudo fallback and a PATH
warning. Rank 20 is closed as ALREADY-IMPLEMENTED. The gap was never the installer; it was
that nothing guarded it.

### Verification-block misfile check (T-2831, G-019 prevention for the P-011 vacuous pass)

The tenth source-level static check. P-011 extracts commands from the `## Verification`
section **only**. A block that lands under a neighbouring heading is silent in BOTH
directions: the task file renders identically, and the gate reports nothing when it finds
nothing — **"no commands to run" and "all commands passed" are the same output.**

**Why it exists.** T-2830 completed reporting *"Acceptance criteria: 8/8 checked"* having
executed **zero** verification commands. Its six commands — the ones proving a 39-conflict
merge was sound — sat under `## Evolution`, because the edit that added them anchored on the
wrong section's closing `-->`. The gate found an empty Verification section and passed
**vacuously**. It would have passed identically had the merge been garbage.

That is the exact defect class this repo's guard layer exists to catch, committed *in the
guard layer*, one commit after being written up: **a check that asserts a property adjacent
to the one it claims.** The tell holds — *if the check would still pass while the subsystem
is entirely broken, it is the wrong check.*

**The evidence was never missing; only the gate was.** All six commands were run by hand and
passed, so T-2830's claims are true. What was false is the implication that a gate proved
them. T-2830 was therefore **repaired, not reopened or `--force`d** — forcing a completion to
make a gate look like it ran would be the same dishonesty inverted.

`scripts/check-verification-misfile.sh` scans every task file and fires on a command line in
any section other than `## Verification`. The anchor is deliberately narrow, in the sibling
checks' precision-over-recall tradition: a line fires only if it is outside HTML comments and
fenced blocks, at column 0, not a markdown marker, its **leading token is command-shaped**
(vocabulary MEASURED from the corpus's real Verification blocks — 2559 files, 141 distinct
leading tokens), it is a **complete statement** (carries a shell operator/flag **or** a
path-like argument), and it **opens a block** (preceded by a blank, a `#` comment, a heading,
or another command).

The last two rules are what make it usable. The leading-token rule **alone** yields 49
candidates across the corpus and **every one is wrapped prose** beginning with `timeout`,
`test`, `git` or `fw`. Adding them takes that to zero without losing the defect. A check that
false-positives permanently is a check nobody reads.

**Scope — do not read a green as a full bill of health.** It detects commands in the WRONG
SECTION. It does **not** verify that a task's verification is adequate, that its commands
test the ACs, or that a task has any verification at all: **a task with an empty
`## Verification` passes this check and still gates on nothing.** Both output paths say so.

Allowlist: `.context/checks/verification-misfile-allowlist` (git-tracked per T-2681), one
`<relpath>::<Section>` per line with a cited reason — the heading text **without** its `##`,
since a `#` in the signature is eaten by the trailing-comment rule (a bug the fixtures caught
in this script's own first draft). Entries are counted and reported but do not fire. Currently
empty on purpose: the one real instance was repaired, and an entry here is for a section
correct as written, never for commands that should have executed.

Current tree: 2559 task files scanned, 0 misfiled blocks, 0 acknowledged. Exit 0 clean / 1
firing / 2 tooling — **fail-closed**: a missing tasks dir, an unreadable allowlist, absent
`python3`, or a corpus with zero task files all exit 2, never a vacuous clean. `--json`,
`--quiet`, `--tasks-dir`, `--allowlist`. Ad-hoc:
`bash scripts/check-verification-misfile.sh`. Fixtures:
`bash tests/verification-misfile-check-fixtures.sh` (29 assertions, weighted toward the
firing cases and the false-positive guards). **Load-bearing:** pointed at T-2830 as it stood
before the repair — extracted straight from git, not a synthetic mutant — it fires on all six
commands; against the repaired tree it is clean.


### Stranded-finalized task check (T-2833, the finalize latch the T-2290 canary cannot see)

The eleventh source-level static check. `update-task.sh` writes `status: work-completed`
**unconditionally** at line ~1681 and then runs the finalize block — stamp `date_finished`,
`git mv` to `completed/`, clear focus, generate episodic — **221 lines later**, behind this
guard at line ~1902:

```bash
if [ -n "$NEW_STATUS" ] && [ "$NEW_STATUS" = "work-completed" ] \
   && [ "$OLD_STATUS" != "work-completed" ]; then
```

The two writes are not atomic, and **the guard reads the value the first write already
committed to disk**. So if a run writes the status and then dies before reaching the finalize
block, every later invocation sees `OLD_STATUS` is already `work-completed`, the condition is
false, and the whole block is skipped. Not an error — an `if` that does not match. It is a
**latch**: once entered, the normal command can never leave it, and `date_finished` becomes
unreachable by any code path.

**Why it is not cosmetic: it deadlocks commits.** P-002 refuses a commit while focus is on a
completed task; the T-1730 focus-drift gate refuses one while focus is on any *other* task. The
only exits are a logged Tier-2 bypass or attributing the commit to an unrelated task — which
commits `2dabb2da5` and `9798e715a` had to do, and said so in their messages.

**A second, partial recovery masks it incompletely.** Re-running the command *does* reach a
different branch — the partial-complete re-check at line ~1395 — which moves the file to
`completed/`. That branch never stamps `date_finished`. So the recovery fixes the location and
not the timestamp, landing the task in exactly the T-2290 **soft, non-firing** class
(`work-completed` with empty `date_finished`), which is informational by default and only fires
under `--strict`. Both observed instances (T-2831, T-2832) finalized that way.

**The T-2290 canary is structurally blind to the pre-move state.** It scans `.tasks/completed/`
only, so a task that never leaves `active/` is outside its corpus by construction. Same end
state as the G-066 finalization-bypass class (work done, register disagrees) reached from the
opposite direction.

**The discriminator is the whole precision of the check, and the first draft got it wrong.**
`work-completed` sitting in `active/` is **not** by itself a defect: T-193 partial-complete is
exactly that state by design — agent ACs pass, human ACs remain, the task stays in `active/`
with `owner: human`. The first draft fired on all of them: **58 findings**, every one
legitimate. A check that is permanently red is a check nobody reads — the fatigue lesson T-2818
documented from the other direction. What separates the defect is `date_finished`: the finalize
block stamps it *before* it branches on `PARTIAL_COMPLETE`, so a partial-complete task always
carries a date, while the latched task can never acquire one. The detector keys on that field
rather than on `owner: human` deliberately — owner is a convention a future task could vary,
the date is produced by the code path itself. Partial-complete tasks are **counted and
reported**, never fired on, so a green is not ambiguous between "none" and "the filter ate them".

**Scope — read a green narrowly (T-2680).** It answers one question: is any file in
`.tasks/active/` declaring `status: work-completed` with no `date_finished`? It does **not**
audit whether a task's work is done, whether its ACs are honest, or how tasks in `completed/`
finalized — that last is T-2290's corpus.

Allowlist: `.context/checks/stranded-finalized-allowlist` (git-tracked per T-2681), one path per
line with a cited reason; repo-relative, absolute, or bare filename all match. Entries are
counted and reported but do not fire. Currently empty on purpose — the two real instances were
**repaired, not acknowledged**.

Exit 0 clean / 1 stranded / 2 tooling — **fail-closed**: a missing tasks dir, an unreadable
allowlist, absent `python3`, or a corpus with zero task files all exit 2, never a vacuous clean.
`--json`, `--quiet`, `--tasks-dir`, `--allowlist`, `--no-heartbeat`. Ad-hoc:
`bash scripts/check-stranded-finalized-tasks.sh`. Fixtures:
`bash tests/stranded-finalized-check-fixtures.sh` (29 assertions, weighted toward the firing
cases and the false-positive guards; case 2 pins the 58-finding regression). **Load-bearing:**
pointed at T-2831 and T-2832 as they stood at commit `389488a65` — extracted straight from git,
not a synthetic mutant — it fires on both; against the repaired tree it is clean.

**The defect is vendored (G-062).** `update-task.sh` lives under `.agentic-framework/`, so it was
**not patched here** — a local fix is erased by the next re-vendor. It is filed upstream; this
check is the local detection that survives one.

### Human-AC Steps heading check (T-2859, the approval page dropping the operator's command)

The rendered review page is the approval surface — it is what a human reads before stamping
an action. Its Human-AC block has three fields, and the renderer parses only two of them
symmetrically. `.agentic-framework/web/blueprints/tasks.py` (~line 419) captures the
remainder of the `**Expected:**` and `**If not:**` marker lines into the field's content;
the `**Steps:**` branch sets `current_content = []` and `continue`s, discarding it.

Two silent losses follow. **Class 1** — a heading like `**Steps (copy-paste):**` fails the
exact `startswith('**Steps:**')` test, so `current_field` never becomes `steps` and the
whole block is dropped. **Class 2** — a canonical heading with content on the same line
loses that content, and where the entire list was on that line the page renders **no Steps
section at all**.

**The failure presents as a complete page.** Nothing errors, the route returns 200, and
Expected and If-not still render — so there is no visual cue that a field is missing.
Measured across `.tasks/active/`: **8 of 129** headings affected (2 class 1, 6 class 2). The
worst, T-2522, is a human *decision* task whose three options were absent from the page
asking for the decision. Another, T-1696, had already been reported to the operator as a
verified, stamp-ready approval — its page carried a Rationale, Evidence, Expected and
If-not, and no command. This lands at the sovereignty boundary: the human approves an action
whose command the page never showed them, which is Directive #2 failing where it costs most.

`scripts/check-human-ac-steps-heading.sh` fires on both classes. **Scope — read a green
narrowly (T-2680):** it checks the heading FORM. It does not verify that a task *has* steps,
that the steps are correct, or that anything else on the page renders. Comment regions are
blanked before scanning, so the template's own examples never fire. Acknowledgements go in
`.context/checks/steps-heading-allowlist` (git-tracked per T-2681) as `<file>::<line>` with a
cited reason; entries are counted and reported but do not fire. Currently absent on purpose —
all 8 instances were **repaired, not acknowledged**.

Exit 0 clean / 1 firing / 2 tooling — **fail-closed**: a missing tasks dir, absent `python3`,
an unreadable allowlist, or a corpus with zero task files all exit 2, never a vacuous clean.
`--json`, `--quiet`, `--tasks-dir`, `--allowlist`. Ad-hoc:
`bash scripts/check-human-ac-steps-heading.sh`. Fixtures:
`bash tests/human-ac-steps-heading-fixtures.sh` (15 assertions, weighted to the firing cases
and the false-positive guards). **Load-bearing:** it found both classes on its first run,
including the six nobody had looked for.

**The renderer is vendored (G-062)** and was **not** patched here — filed upstream at
`framework:pickup` offset 74 with the three-line symmetric fix. This check is the local
detection that survives a re-vendor.

### Running the guard layer — `scripts/run-guard-layer.sh` (T-2684)

**One command runs every source-level guard.** Before T-2684 there was none, and
nothing automatic ran them at all.

```bash
bash scripts/run-guard-layer.sh            # all static checks + fixture suites (minutes, not
                                            # seconds — measured ~16min contended-host wall time,
                                            # see docs/reports/T-3090-guard-layer-timing.md; a
                                            # 3-member long pole accounts for over half of it)
bash scripts/run-guard-layer.sh --list     # what the layer consists of
bash scripts/run-guard-layer.sh --tests    # ...plus `cargo test --workspace` (minutes)
bash scripts/run-guard-layer.sh --json     # {ok, members[], summary} for scripting
```

TermLink's guards split in two, and only one half was ever automated. The **runtime**
guards — the 17 cron canaries above — are wired to cron and heartbeat daily. The
**source-level** guards, which protect the code that changes on every commit, were
executed by nothing: T-2683 grepped the whole tree and found the four static checks
referenced only by their own fixture suites and by `.context/episodic/*`, the records
of the tasks that created them. `release.yml` ran `cargo build --release` and never
`cargo test`; `doc-lint.yml` ran 2 of 28 check scripts; the pre-push audit runs the
`structure` section only. That is precisely the disease the static checks exist to
cure — `check-alloc-sink-clamps.sh`'s own header says "the convention is by
discipline, not enforced" — reproduced one level up, where nothing was watching.

**Membership is declared, not guessed.** A static check joins the layer by carrying a
marker in its own header:

```
# guard-layer: source [extra args...]
```

`source` means "safe to run anywhere: no live hub, no network, no host state". The
optional trailing args are how the check wants to be invoked here — chiefly
`--no-heartbeat`, so a check run from the runner cannot refresh its cron heartbeat and
mask a dead cron from the T-1723 meta-canary. Fixture suites (`tests/*fixtures*.sh`)
are members by naming convention; they are hermetic by construction. A
`scripts/check-*.sh` with **no** marker is reported as unclassified rather than
silently ignored — a forgotten marker is itself the shipped-but-dark condition. Most
unmarked checks are legitimately runtime canaries and belong to cron.

**Verdicts preserve the exit-code contract the layer already uses:** `PASS` (rc 0) ·
`FAIL` (rc 1 — a guard fired, a real finding) · `ERROR` (rc 2 or an unexpected status
— the guard *could not run*) · `SKIP`. Keeping ERROR distinct from PASS is the point:
a check that never looked must never read as a clean bill. Roll-up: any FAIL → exit 1;
else any ERROR → exit 2; else 0 — findings dominate tooling errors, mirroring
`fleet verify`'s "drift dominates". A member that hangs is bounded by
`GUARD_LAYER_TIMEOUT` (default 300s) and counts as ERROR, never PASS. The **release gate runs
with 600s per member** (T-3281, SQ-21). The slowest member took 226s and timed out once on the
contended .107 host, while GitHub runners took 6m24s for the whole layer. So 600s is insurance
against a flake holding a release, and push CI keeps 300s.
**Per-member limit (T-3571).** A member may declare `# guard-layer-timeout: <1..1800>  # <reason>`
in its header. It only ever raises the limit (the larger of the declaration and
`GUARD_LAYER_TIMEOUT` applies), and a value with no reason or out of range is reported MALFORMED
and ignored. `tests/runme-fixtures.sh` declares 900s: 137 full runme runs measured 183-303s here
under load, so it sat on the 300s line as ERROR for a timing reason, not a finding.

**Wired into CI by T-2686.** `doc-lint.yml` gains a `guard-layer` job (runs on every
push and PR — no Rust build, but **~16 minutes, not seconds**: measured 945s wall on
this host under its normal load of ~450 concurrent agent processes, T-3090. The
operator ruled that the contended figure IS the answer, because this host is never
quiet — busy is its steady state, so a quiet-host number would be a promise the
machine never keeps. Budget CI accordingly; a single member,
`check-verification-heading-shadow.sh`, was ~29% of the total. **T-3282 reduced it from 252 s to
~2.8 s (~90×) on this host.** It now extracts every task's Verification block in one Python
process that imports the framework's own `comment_strip.py`, instead of starting `python3`
~3000 times. Equivalence to `extract_verification_block` is proven over the whole corpus
(`tests/verification-block-batch-proof.sh`: 2989 files, 0 mismatches). Every run also
cross-checks the firing files plus a random sample of 25 against the real extractor, and exits
2 on any divergence. `HEADING_SHADOW_MODE=per-file` keeps the reference path. The old "seconds" claim mattered because it taught
operators to kill a slow-but-healthy run, and a check people stop running protects
nothing), and `release.yml` gains a `test` job running
`cargo test --workspace` **plus** the guard layer, which both build jobs now `needs:`
— so a red suite blocks the build and no binary is produced at all. That gate found
its first real defect immediately: `parity_topics` had been failing since 2026-08-12
(T-2624 added four partial-inventory fields to the CLI's `topics --json` and never to
the MCP tool — fixed in T-2687) with nothing to surface it.

Test seams: `GUARD_LAYER_SCRIPTS_DIR`, `GUARD_LAYER_TESTS_DIR`, `GUARD_LAYER_TIMEOUT`.
Fixtures: `bash tests/guard-layer-runner-fixtures.sh` (27 assertions).

**Severity tiers (T-3260, operator-approved; supersedes T-3258's two classes).** The
tier is the first word after `source` on the marker line; no word means **FAIL**.

```
# guard-layer: source [args]                  FAIL — red fails push CI AND the release gate
# guard-layer: source warn [args]  # <reason> WARN — never fails either gate
# guard-layer: source info [args]  # <reason> INFO — never red; output printed only
```

A red **WARN** member (FAIL *or* ERROR — a WARN guard that could not run is reported as
WARN, never as a pass) is printed in its own section with its reason, emitted as a GitHub
`::warning::` annotation (automatic under `GITHUB_ACTIONS`, or `--annotate`), appended to
`$GITHUB_STEP_SUMMARY`, and counted in `--json` (`warn_fired`/`warn_errored`;
`advisory_fired`/`advisory_errored` kept as aliases). `--gate all|release` is still
accepted, and both now gate the same way — only FAIL-tier reds set the exit code, so
Doc Lint stays green on a WARN red. `advisory` (the T-3258 word) is an alias for `warn`.
A `warn`/`info` marker with no reason is refused — the member runs as FAIL and prints
`MALFORMED` — and `scripts/check-guard-severity-markers.sh` fires on it (`NO-REASON`), on
a severity word among the args (`MISPLACED`), and on any demotion of a fixture suite
(`ALWAYS-FAIL`). `cargo test` and `tests/*fixtures*.sh` are always FAIL.

**The 14-day clock.** A WARN that is red for more than 14 days **escalates**: the
release-publication canary (above) fires on it. The clock is
`.context/checks/guard-warn-first-red` — git-tracked, one `<member>  <UTC ISO>` line per
WARN member currently red. It is written **only** by a run given `--record-warn`, which
adds a member the first time it is seen red, **keeps** an existing date, and drops a
member the first time it is seen green; the release canary does that refresh daily on this
host. Every other run — CI included — only **reads** it, so a fresh CI checkout cannot
reset the clock: it has no write path. A red WARN member with no ledger entry is printed
as "clock not started", never silently as new. **Limits, stated plainly:** (1) the clock
starts when the host first *records* a red, not when the member first went red — members
already red before T-3260 (check-receiver-ack-lag, seeded 2026-09-29) started at seeding;
(2) the durable copy is the host's working tree until someone commits the ledger — a
`git checkout`/re-clone of the host before that commit resets it, and a CI checkout sees
only the committed dates; (3) escalation needs the release canary's cron installed (SQ-9).
Do not hand-edit a date to silence escalation — fix the member or reclassify it.

**Escalation is an ACTION, not a line (T-3267 — the operator's binding condition).** On the
host path the release canary also runs `scripts/warn-escalation-file.sh`, which **files a
task**: one per WARN member red > 14 days (owner agent, horizon now; the member's current
output, first-red date and tier reason in Context; AC "member is green or reclassified by the
operator"), and one **umbrella** task the moment more than 5 WARN members are red at once
(`GUARD_WARN_MAX_RED`; the canary fires on that too). De-dup is a body marker
(`<!-- warn-escalation: member=<m> -->` / `umbrella`) against every open `.tasks/active/`
file, whatever its status. **Re-file rule:** closing the task while the member is still red
restarts the window at the close date — it files again only when red > 14 days past
max(first-red, `date_finished`). It files and never fixes; it refuses under `$CI`; a
FAIL-tier or unknown-tier member is never filed on (the runner's `--list --json`, which now
carries `reason` + `cmd`, is the tier source, and unreadable means rc 2). **Filed tasks and the
ledger are left UNCOMMITTED** — each filing fires as "filed T-XXXX … UNCOMMITTED" — and are
committed by the next interactive session; an unattended cron commit would race live sessions'
index (T-3231). That choice is SQ-22, **ruled option C** by the operator (2026-09-30). A filing
failure fires rather than exiting 2. `--no-file-tasks` skips filing; `--dry-run` on the filer
prints `WOULD` lines. Fixtures: `bash tests/warn-escalation-file-fixtures.sh` (34 assertions,
7 mutants — de-dup, threshold, `>` vs `>=`, re-file window, CI guard, canary wiring,
accumulation line).

**The pending-commit manifest (T-3269, SQ-22 option C).** "The next session commits them" is
structural, not a hope. Every unattended writer records what it wrote —
`scripts/commit-pending.sh add <T-ID> <reason> <path>…` → `.context/working/pending-commit.list`
(gitignored, host-local) — and the filer and the canary's ledger refresh both do. The next
session commits **by name**: `scripts/commit-pending.sh commit` runs `git commit -- <exactly
those paths>`, one commit per recorded task id, so anything else staged stays staged (the
T-3090/T-3231 sweep, pinned by fixture). Only `.tasks/` and `.context/` paths are ever accepted
(anything else is refused loudly and kept for a human); a vanished file is reported and dropped;
HEAD is verified to carry each path before its entry is removed; a second run is a no-op. It is
wired as a standing first step in `/resume` (Step 1 lists entries **by name** — a filed task
drowns in this host's ~20-file dirty baseline if it is only counted — and Step 3 commits them),
in the procAsFit round prompt, and in `handover --commit` before its own pathspec commit (a
vendored patch, registered in `.vendor-divergence.yaml`). Commits go under the **writer's**
task id, not the session's focus; the T-1730 focus-drift hook reads only the literal Bash
command, so it does not see the helper's inner commit — no bypass is used, git's own hooks still
run, and the attribution rule stands in for the drift check. Fixtures:
`bash tests/commit-pending-fixtures.sh` (17 assertions, 4 mutants — sweep, scope, prune,
missing file).

Current markings (operator rulings in `docs/reports/T-3258-guard-classification-draft.md`
§ Operator rulings): **INFO** invocation-usage, check-receiver-ack-lag (2026-10-18, T-3441);
**WARN** voi-prompt, check-go-propagation,
check-human-ac-escalation, check-human-ac-steps-heading, check-handover-staleness,
check-pickup-deferred-freshness, check-stranded-finalized-tasks, check-task-id-collisions,
check-arc-slice-drift, check-installed-binary-drift,
check-audit-warning-acknowledgement, check-budget-ladder-drift, check-episodic-parse,
check-vendor-divergence, fabric-workflow-link; everything else FAIL (Part 3 of the draft
is still being decided). Fixtures: `bash tests/guard-layer-severity-fixtures.sh`
(32 assertions, 6 mutants) plus the escalation cases in the release-publication fixtures.

### Canary log hygiene — split the streams (T-2685)

Every canary implements `exit 0 healthy / exit 1 FIRING / exit 2 tooling error`, and
T-2557 states why the exit-2 class exists: *"This split keeps a firing log meaningful:
it fills ONLY when session control genuinely broke, never on a transient hub-down."*

**The crontab used to throw that split away.** All 30 job lines across all 24 crontabs
were written `>> <findings>.log 2>&1`, merging stderr into the findings log — so a
check that *could not run* wrote into the channel whose entire documented meaning is
"the watched thing is broken". T-2683 found a live instance:
`.release-mirror-canary.log` held `error: origin HEAD empty` while the script itself
exited 0 with "GitHub mirror: synced", which per this file directs an operator to
rotate a GitHub token for a fault that did not exist.

The compounding harm is worse than one false positive: **"empty log = healthy" is a
one-bit channel.** Once a tooling error dirties the log, a subsequent genuine finding
appends to an already-non-empty file and changes nothing an operator can see — the
canary is not merely noisy, it is *deaf* until someone truncates it by hand.

The correct idiom, now used everywhere:

```
… bash scripts/check-<x>.sh --quiet >> .context/working/.<x>-canary.log 2>> .context/working/.<x>-canary.log.stderr
```

Nothing is lost — the diagnostic stream is preserved, just not conflated with the
signal. The `.stderr` suffix deliberately does not match `/canaries`' `.*-canary.log`
discovery glob (`canary-status.sh:97`). `2>/dev/null` is **also** wrong: it trades a
false positive for a silent failure, Directive #2 violated inside the monitoring
layer. `scripts/check-canary-log-hygiene.sh` enforces this, carries the
`# guard-layer: source` marker, and fires on either mistake. Ad-hoc:
`bash scripts/check-canary-log-hygiene.sh` (0 clean / 1 firing / 2 tooling; `--json`;
seam `CANARY_HYGIENE_SRC_DIR`). Fixtures: `bash tests/canary-log-hygiene-fixtures.sh`
(19 assertions). See `docs/operations/substrate-cron-recipes.md` § "The redirect
idiom".

### The stderr sink had no reader — ERRORING (T-2842) + completion heartbeats (T-2843)

Splitting the streams (T-2685 above) was correct and incomplete. It gave each canary a
clean firing log **and a diagnostic sink that nothing ever opened.** Three locally
reasonable decisions composed into a blind spot:

- every canary touched its `.heartbeat` **unconditionally at startup**, before doing any
  work, so freshness proved "cron fired", never "the canary succeeded";
- on a tooling error a canary writes its diagnostic to **stderr and nothing to stdout**;
- the crontabs route stderr to `<log>.stderr`, keeping `.log` purely a firing log.

Compose them and a canary erroring **every single day** shows an empty `.log` (HEALTHY on
`/canaries`), a fresh heartbeat (ALIVE to the T-1723 meta-canary), and its diagnostic in a
file no surface reads. That is the G-063 write-only-sink class landing on the detection
layer itself — the layer whose whole job is to notice.

**`ERRORING`** is the class that reads it. A canary whose `<log>.stderr` has content
written inside the staleness window is ERRORING; it counts toward `PROBLEMS` (exit 1),
carries `stderr_bytes` in `--json`, and renders the stderr line itself rather than the
firing log. It **outranks FIRING deliberately**: a canary that could not complete its run
cannot be trusted to have found *or missed* anything, so canary integrity is the more
urgent signal. The window is bounded by the same staleness threshold, so a long-resolved
transient error does not pin the verb red forever (T-2818). Truncating the sink clears it.

**The heartbeat now proves completion, not scheduling.** All 31 scripts that write one
defer the write to an `EXIT` trap. A SIGKILLed or hung canary leaves the heartbeat
untouched and surfaces as STALE; an exit-2 tooling error still writes it, because that run
did complete. **`trap ... EXIT` REPLACES any previous EXIT trap** — four scripts
(`check-fleet-doorbell-mail-health`, `check-forever-archival-freshness`,
`check-topic-growth-freshness`, `substrate-preflight`) install their own tmpfile cleanup
later in the file, so they use a self-guarding helper armed early and **chained** onto that
cleanup. A naive second `trap` there would have silently killed those four heartbeats — a
worse failure than the one being fixed. Copy the chained shape, not the plain one, into any
canary that owns a cleanup trap.

Two NOT_SCHEDULED predicates were written independently; only `is_cron_scheduled` (anchored
on `.<name>.log`, **fail-open** on an absent cron dir) survives. The rejected variant grepped
a bare name and failed closed, which would mass-downgrade real STALE canaries to "not a
problem" exactly when the check could no longer tell.

Fixtures: `bash tests/canary-status-fixtures.sh` (16 assertions; Case 2 is the load-bearing
empty-log/fresh-heartbeat state, Case 14 is a mutant that disables the ERRORING branch and
must reproduce the pre-fix HEALTHY/0) and `bash tests/canary-heartbeat-fixtures.sh`
(6 assertions, driving a REAL canary — deliberately one of the four chained ones — with Case
3 SIGKILLing it, since EXIT traps do not run on SIGKILL).
