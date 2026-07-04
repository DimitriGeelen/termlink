# CLAUDE.md

Claude Code integration for the Agentic Engineering Framework.
For the provider-neutral framework guide, see `FRAMEWORK.md`.

This file is auto-loaded by Claude Code. It contains the full operating guide
plus Claude Code-specific integration notes.

## Project Overview

**Project:** 010-termlink

<!-- Add your project description, tech stack, and conventions below -->

## Tech Stack and Conventions

<!-- Define your project's tech stack, coding standards, and conventions here -->

## CI / Release Flow

**NEVER push to GitHub.** Only push to OneDev (`git push origin`). The rest is automated:

```
git push origin main --tags          # OneDev (only manual step)
        ↓
.onedev-buildspec.yml                # auto-mirrors all branches + tags to GitHub
        ↓
.github/workflows/release.yml       # GitHub Actions triggers on v* tags
        ↓
GitHub Releases                      # macOS + Linux binaries + checksums published
        ↓
Homebrew formula                     # brew install works (downloads from GitHub Releases)
```

- **OneDev** is the source of truth. Push here only.
- **GitHub** is a read-only mirror. OneDev's `PushRepository` buildspec job handles sync automatically using `github-push-token`.
- **Releases** happen when you tag (`git tag v0.X.0`) and push to OneDev. GitHub Actions builds the binaries.
- **Versioning** is git-derived via `build.rs`: tag = exact version, N commits after tag = `major.minor.N`.

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
false-positive lesson). Pair with the mirror-drift and substrate-preflight
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
build lineages (ring20-dashboard serves 0.11.806 from its own fork — numerically
"newest" while lacking our commits) — never set a floor for a hub whose binary
you don't build. Unknown `hub_version` on a reachable floored hub DOES fire (a
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

## Project-Specific Rules

### Hub Auth Rotation Protocol

TermLink hubs use a persistent 32-byte HMAC secret (`hub.secret`) and a
persistent TLS cert (`hub.cert.pem` / `hub.key.pem`) under their `runtime_dir`
(T-933 / T-945 / T-1028 / T-1031). Under normal operation the hub preserves
these across restarts and clients never need to re-pin. Rotation still
happens in three scenarios: first-time deploy of persist-if-present onto a
hub that previously regenerated on restart, a systemd restart landing in a
different runtime_dir, or an intentional operator regeneration. In all three
cases the client's cached secret becomes stale and **`termlink fleet doctor`
reports `Token validation failed: invalid signature`**. See
`docs/reports/T-1051-termlink-auth-reliability-inception.md` for the full
root-cause analysis and Option D decision.

**Special case — volatile runtime_dir (T-1290 / T-1294).** A degenerate
sub-case of scenario 2: if the hub is started without `TERMLINK_RUNTIME_DIR`
set (legacy default `/tmp/termlink-0`) on a host where `/tmp` contents do
not survive boot, every container or system reboot wipes the entire
runtime_dir. Persist-if-present cannot help — there is nothing to find.
Symptom: BOTH the TLS fingerprint AND the HMAC secret rotate simultaneously
after a reboot (cert-only rotation does not happen here; persist applies
to both equally). When you see PL-021 ("hub rotates BOTH secret and TLS
cert") fire, suspect this.

**Two distinct mechanisms produce volatile /tmp** — check BOTH:

1. **/tmp mounted as tmpfs** — kernel reclaims contents on shutdown.
   Detect: `mount | grep ' /tmp '` shows a `tmpfs on /tmp` line.
2. **/tmp on regular disk but wiped by systemd-tmpfiles** — a
   `D /tmp 1777 root root -` rule in `/usr/lib/tmpfiles.d/tmp.conf` (or
   any override under `/etc/tmpfiles.d/`) makes `systemd-tmpfiles --boot`
   delete /tmp contents on every boot. Mount table looks innocent (no
   tmpfs) but the directory is still volatile. T-1294 confirmed this on
   ring20-management (.122).

**Diagnostic.** Either of these positive means volatile /tmp:

```
ls -la /tmp/termlink-0/ /var/lib/termlink/
mount | grep -E ' /tmp |termlink'
cat /usr/lib/tmpfiles.d/tmp.conf /etc/tmpfiles.d/tmp.conf 2>/dev/null
```

If files in `/tmp/termlink-0/` all share the boot-time mtime (compare
against `systemctl show -p ActiveEnterTimestamp init.scope`), you're
seeing post-boot regeneration regardless of mechanism.

**Fix.** Move runtime_dir off /tmp permanently. Insertion point depends
on how the hub is launched:

- **systemd-launched hub:** install/repair the systemd unit per
  `docs/operations/termlink-hub-runtime-migration.md` (T-935) so
  `Environment=TERMLINK_RUNTIME_DIR=/var/lib/termlink` is set.
- **watchdog-launched hub** (e.g. ring20-watchdog.sh on the ring20 CTs):
  `export TERMLINK_RUNTIME_DIR=/var/lib/termlink` near the top of the
  watchdog script, before any `termlink hub start` invocation. Also
  update any hardcoded `/tmp/termlink-0/{hub.sock,hub.secret}` references
  in the same script. T-1294 documented this for ring20-management.

Pre-seed the new path with the existing secret/cert (`cp -a /tmp/termlink-0/.
/var/lib/termlink/`) so persist-if-present preserves rather than regenerates;
remove the stale `hub.sock` / `hub.pid` to free TCP bind. Restart hub once,
all clients re-pin once. The next reboot must NOT trigger rotation — that's
the persistence ground-truth.

**Symptom recognition.** Any of the following means rotation happened and
the client needs healing:

- `fleet doctor` hint: `Secret mismatch — hub was likely restarted with a new secret`
- `fleet doctor` hint: `TOFU VIOLATION` / `fingerprint changed`
- Auto-registered `PL-XXX` learning in `.context/project/learnings.yaml` with
  `date_observed=` and `hub_fingerprint=` fields (T-1052).
- After ≥3 consecutive auth-mismatches spanning >24h, an auto-registered
  `G-XXX` concern in `.context/project/concerns.yaml` with
  `type: gap, severity: high, status: watching` (T-1053).

**Detection — primitive verbs (T-1656/57/58/59/60/61) + unified (T-1663/1666) + continuous (T-1667) + event hook (T-1669) + history (T-1671).**
Read-only, no-auth, no-`KnownHubStore`-mutation diagnostics. Use these to
confirm a rotation happened and identify which hub before reaching for the
heal paths below.

| Verb | Reads | One-line purpose |
|---|---|---|
| `termlink hub export-secret` | live local `<runtime_dir>/hub.secret` | Authoritative secret-share source (G-011 R3); never the IP-keyed cache. |
| `termlink hub fingerprint` | live local `<runtime_dir>/hub.cert.pem` | TLS fingerprint of the local hub for peers to pin. |
| `termlink hub probe <addr>` | remote leaf cert via TLS handshake | Pre-pin diagnostic — confirm a hub is up and capture its current fingerprint. |
| `termlink tofu verify <addr>` | wire vs `~/.termlink/known_hubs` | Per-host drift check. Exit 0=match, 1=drift, 2=no-pin, 3=probe-fail. |
| `termlink fleet verify` | all profiles in `~/.termlink/hubs.toml` | Fleet rollup. Drift dominates. `--exit-on-drift-only` for cron alerting on rotations only. |
| `termlink_fleet_verify` (MCP) | same as `fleet verify` | Agent-callable companion. Returns `{verdict, profiles[], actions[]}` JSON with heal hints when drift detected. |
| `termlink_hub_probe` (MCP) | same as `hub probe` | Agent-callable single-host TLS-probe — returns `{ok, fingerprint, error}`. T-1663. |
| `termlink_tofu_verify` (MCP) | same as `tofu verify` | Agent-callable single-host pin-check — returns `{status, wire, pinned, actions[]}`. T-1663. |
| `termlink fleet doctor --include-pin-check` | auth (per-hub) + TLS (per-hub) | **Unified single-shot:** runs the existing fleet doctor sweep AND probes each profile's TLS cert in parallel. One command answers "auth-mismatch OR cert-drift OR both?" without two commands. T-1666. |
| `termlink fleet doctor --watch <secs>` | same as above, looped | **Continuous monitor:** re-runs the unified diagnostic every N seconds (5..=3600), emits only per-hub state changes after a baseline. Cron-replacement; SIGINT exits cleanly. T-1667. |
| `termlink fleet doctor --watch <secs> --notify <cmd>` | same; fires hook on change | **Event hook:** operator-pluggable shell command invoked on per-hub state change. Fire-and-forget — hanging scripts don't block the loop; cmd-not-found doesn't kill the watch. Env vars passed: `TERMLINK_WATCH_HUB`, `TERMLINK_WATCH_CHANGE_KIND` (`transition`/`new`/`removed`), `TERMLINK_WATCH_{OLD,NEW}_{CONN,PIN,LEGACY}`, `TERMLINK_WATCH_TS` (RFC3339 detection time — for log correlation, prefer over the script's own `date`; T-1676). T-1669. |
| `termlink fleet doctor --watch <secs> --auto-heal` | same; heals on rotation transitions | **Built-in auto-heal (continuous):** fires when EITHER (a) `new_pin=drift` (cert rotation, needs `--include-pin-check`) OR (b) `new_conn=auth-mismatch` (secret-only rotation, T-1681 — closes PL-162 gap), AND the profile has declared `bootstrap_from` (T-1291). Spawns `fleet reauth <hub> --bootstrap-from auto` fire-and-forget. Both gates share the R2 declared-anchor check; one heal per change cycle (PL-021's "BOTH rotate" case dedups). T-1680 + T-1681 (gate) + T-1682 (parser fix making the auth-mismatch path actually fire). |
| `termlink fleet doctor --auto-heal` (no `--watch`) | same; heals on current state | **Built-in auto-heal (one-shot, T-1683):** page-respond mode — drops the `--watch` requirement so operators can fix a known rotation without spinning up a watch loop. Runs the existing fleet-doctor sweep, then classifies each hub's current state and fires the same heal for any profile in `pin=drift` (with `--include-pin-check`) or `conn=auth-mismatch` AND declared `bootstrap_from`. Without `--include-pin-check` only the auth-mismatch path can fire — surfaced via stderr info hint. |
| `termlink fleet doctor --auto-heal --dry-run` | same; prints intended fires | **Preview (T-1684):** classifies and reports what `--auto-heal` would do without spawning any heal subprocesses. Per affected hub: `[DRY-RUN] would fire: termlink fleet reauth <name> --bootstrap-from auto` to stderr. Same skip-no-anchor lines as live mode. Header reads "Auto-heal: would fire N (dry-run, T-1684)". Works with both single-shot and `--watch`. Clap requires `--auto-heal`. Use when wiring automation or debugging the bootstrap_from gate. |
| `termlink fleet reauth --all-drifted` | parallel TLS probe of every profile | **Bulk heal:** one-shot companion to `fleet reauth <profile>`. Probes all profiles, classifies drift, and for every drifted profile with declared `bootstrap_from` runs the heal. Per-profile failures don't abort the loop. Exit 0 = no drift or all healed; exit 1 = any skip or fail. Operator UX win for fleet-wide rotation events. T-1679. |
| `termlink fleet history [--since DAYS] [--hub NAME] [--json] [--include-heals]` | `~/.termlink/rotation.log` + (T-1686) `~/.termlink/heal.log` | **Retrospective history:** read-only diagnostic answering "is this hub's drift the 1st or Nth time?". `--watch` appends one NDJSON line per state change (ts/hub/kind/old/new). `fleet history` filters & summarizes. Default 7-day window, clamped 1..=365. Empty log prints a hint. T-1671. With `--include-heals` (T-1686): also pulls heal events from heal.log (T-1685), interleaved chronologically, rendered with `HEAL/<mode>` kind marker plus trigger+action fields. Summary footer splits rotation/heal counts per hub. |
| `termlink_fleet_history` (MCP) | same as `fleet history` | **Agent-callable retrospective (T-1687):** MCP parity for `fleet history`. Params: `since_days` (default 7, 1..=365), `hub` (optional name filter), `include_heals` (default false). Returns one JSON blob `{ok, entries[], summary}` with entries pre-sorted chronologically when `include_heals=true` and each tagged `event_type: "rotation"\|"heal"`. Read-only, no auth, no network — pure scan of `~/.termlink/{rotation,heal}.log`. Use when an agent investigating a flap needs to answer "have we seen this drift before?" without shelling out. |
| `termlink fleet bootstrap-check [<profile>\|--all] [--json]` | declared `bootstrap_from` channels (no write) | **Anchor preflight (T-1688):** validates that each declared `bootstrap_from` (T-1291) actually returns a parseable 64-hex secret BEFORE an `--auto-heal` ever fires. Runs the same `fetch_bootstrap_secret` + `normalize_and_validate_secret_hex` path as the live heal, but stops short of writing the secret_file. Per-profile status taxonomy: `ok` / `no-anchor` / `fetch-fail` / `invalid-format`. Exit codes: 0 = nothing broken, 1 = any fetch-fail / invalid-format, 2 = `--all` and no profile declares an anchor at all. Use after declaring a new anchor or after rotating SSH keys — catches "ssh: permission denied" / "file not found" / "secret got truncated" at declaration time instead of under pressure during a rotation event. |
| `termlink fleet history --analyze [--since DAYS] [--hub NAME] [--json]` | same as `fleet history` (rotation.log) | **PL-021 flap detector (T-1690):** classifies each hub's rotation history into one of `clean` / `cert-only` / `secret-only` / `single-double-rotation` / `pl021-candidate`. A "double rotation" is a single log row carrying both `new_pin=drift` (was-not-drift) AND `new_conn=auth-mismatch` (was-not-auth-mismatch). ≥2 double rotations in the window flags a PL-021 candidate (recurring volatile runtime_dir). Candidate output embeds the volatile-/tmp diagnostic verbatim (`ls -la /tmp/termlink-0/...`, `mount`, `tmpfiles.d` check) so the operator has a copy-pasteable next step. Exit code 2 when any candidate detected (cron/CI alerting hook), 0 otherwise. Read-only; never auths, never writes. |
| `termlink_fleet_bootstrap_check` (MCP) | same as `fleet bootstrap-check` | **Agent-callable anchor preflight (T-1689):** MCP parity for `fleet bootstrap-check`. Params: `profile` (string, mutex with `all`), `all` (bool, default false, mutex with `profile`), `timeout_secs` (default 10, clamped 1..=120). Implementation subprocesses the resolved `termlink` binary with `fleet bootstrap-check --json` under `tokio::time::timeout` + `kill_on_drop=true` + null stdin — so a hanging interactive `ssh:` anchor can't wedge the MCP server. Returns the CLI's `{verdict, profiles}` shape decorated with `ok` and `exit_code`. Timeout returns `{ok: false, verdict: "timeout", error: "..."}`. Use during agent flap-investigation flows to answer "would the configured heal actually fire?" without shelling out. |

**Auto-heal recipe — built-in (T-1680/T-1683, preferred):**

Continuous (watch loop, fires on transitions):

```bash
termlink fleet doctor --watch 30 --include-pin-check --auto-heal
```

One-shot (page-respond, fires on current state — T-1683):

```bash
termlink fleet doctor --include-pin-check --auto-heal
```

Both forms gate on declared `bootstrap_from` in `hubs.toml` (T-1291) and
fire `fleet reauth <hub> --bootstrap-from auto` fire-and-forget per
affected hub. Profiles without declared `bootstrap_from` are skipped
with a one-line stderr hint (R2 — no implicit anchors). The continuous
form is right when a hub is flapping or you want hands-off detection;
the one-shot form is right when fleet doctor already told you what's
broken and you just want it fixed.

Preview before wiring automation (T-1684): append `--dry-run` to either
form. Same classification, same per-hub output, but each fire site emits
`[DRY-RUN] would fire: termlink fleet reauth ... --bootstrap-from auto`
to stderr instead of spawning a subprocess. Use it to validate the
declared anchors and the bootstrap_from gate before turning a watch
loop loose unattended.

Audit trail (T-1685): every auto-heal decision — live fire, dry-run
preview, or skip-for-missing-anchor — appends one NDJSON line to
`~/.termlink/heal.log`. Schema: `{ts, hub, mode, trigger, action,
bootstrap_from}`. Symmetric to T-1671's `rotation.log` (state transitions)
but specifically for the operator-actionable response. Read with
`jq -c 'select(.hub=="ring20-management")' ~/.termlink/heal.log` or
similar. Write failures emit to stderr but never block the heal.

**Auto-heal recipe — shell-script (T-1669 + T-1291, pre-T-1680):**

Use this form when you want custom logic (e.g. Slack post, page on-call,
specific routing) rather than a straight reauth:

```bash
# /usr/local/bin/termlink-autoheal.sh
#!/bin/sh
[ "$TERMLINK_WATCH_NEW_PIN" = "drift" ] || exit 0  # only act on cert drift
exec termlink fleet reauth "$TERMLINK_WATCH_HUB" --bootstrap-from auto

# Then run the watch with notify wired to it:
termlink fleet doctor --watch 30 --include-pin-check --notify /usr/local/bin/termlink-autoheal.sh
```

The script gates on `NEW_PIN=drift` (cert rotation) and delegates to the
declared `bootstrap_from` per profile (T-1291). Termlink ships detection +
event; operator ships response policy. R2 still applies — the
`bootstrap_from` channel must be out-of-band.

**Coverage scope (PL-162).** TLS-probe verbs (`hub probe`, `tofu verify`,
`fleet verify`) detect **CERT rotation** only. **Secret-only rotation**
(cert unchanged, HMAC secret regenerated — e.g. from a partial
persist-if-present landing where `hub.cert.pem` survived but `hub.secret`
did not) is invisible to TLS probes and surfaces via `fleet doctor`'s
auth-mismatch state. PL-021's "both rotate" case is detectable by either —
prefer `fleet verify` because it succeeds with no profile auth needed.
Operator rule: if `fleet verify` reports `match` but `fleet doctor` still
flags auth-mismatch, the rotation was secret-only — heal via
`fleet reauth <profile> --bootstrap-from <source>` directly without
clearing the TOFU pin.

**Coverage of `--auto-heal` (T-1681 + T-1682 + T-1683).** `--auto-heal`
covers **both** rotation types in **both** modes:

- Continuous (with `--watch`): heal fires on per-cycle transitions —
  cert-drift when `new_pin=drift` (needs `--include-pin-check`), secret-only
  when `new_conn=auth-mismatch`. T-1681 introduced the OR-gate; T-1682
  fixed the dead gate (the auth-mismatch class is computed internally
  but never appears in JSON `status`, so the watch parser now bridges
  via `derive_watch_conn` — auth-mismatch error message → `auth-mismatch`
  class in watch's in-memory state).
- One-shot (no `--watch`, T-1683): heal fires on current state at end of
  the single sweep — same gate, same fire-and-forget heal subprocess.
  Page-respond pattern: fleet doctor flagged it, fix it now.

Both modes gate on declared `bootstrap_from` (R2) and skip profiles
without an anchor with a stderr hint. The continuous mode dedups
PL-021's "both rotate" case at one heal per cycle; the one-shot mode
fires per affected hub.

**Retrospective check (T-1671).** After confirming a rotation just
happened, the next question is usually "first time or Nth?" — a hub
flapping repeatedly points at PL-021 (volatile /tmp or partial
persist-if-present), not a one-off operator action. Run:

```
termlink fleet history --hub <name> --since 30
```

This reads `~/.termlink/rotation.log` (populated by any prior
`fleet doctor --watch` session) and prints a chronological list +
per-hub event count. Empty log means no prior watch session captured
this hub — start one before the next rotation if recurrence diagnosis
matters.

**Heal path — Tier-1 (print the incantation).** For visibility and ad-hoc
triage:

```
termlink fleet reauth <profile>
```

Reads `~/.termlink/hubs.toml`, does NOT contact the hub, does NOT write. Prints
the exact copy-pasteable SSH-read → local-file-write → chmod 600 → verify
sequence. Safe to run anywhere. Implementation: T-1054,
`crates/termlink-cli/src/commands/remote.rs::render_fleet_reauth_plan`.

**Heal path — Tier-2 (autoheal via explicit trust anchor).** When you're
confident in the source:

```
termlink fleet reauth <profile> --bootstrap-from file:/path/to/new-secret.hex
termlink fleet reauth <profile> --bootstrap-from ssh:<host>
```

Fetches the new secret via the named out-of-band channel, validates 64-char
hex, backs up the existing `secret_file` to `.hex.bak`, atomically writes the
new file at chmod 600, and prints a 12-char fingerprint preview. Refuses
profiles that use inline `secret = ...` (migration hint provided).
Implementation: T-1055, same file, `cmd_fleet_reauth_bootstrap`.

**Heal path — Tier-2 declarative (T-1291).** Once an anchor is known
correct, declare it on the profile and use `auto`:

```
# One-time declaration (or pass --bootstrap-from to `profile add`):
[hubs.ring20-management]
address        = "192.168.10.122:9100"
secret_file    = "~/.termlink/secrets/ring20-management.hex"
bootstrap_from = "ssh:192.168.10.122"

# Per-incident heal:
termlink fleet reauth ring20-management --bootstrap-from auto
```

`auto` resolves to the declared `bootstrap_from` and delegates to the
T-1055 fetch path. Operator types one flag instead of remembering the
exact OOB incantation per hub. Missing declaration emits a two-option
hint (declare it, or pass an explicit source). Same R2 rule applies —
the declared channel must not depend on the auth being healed.

**R2 — out-of-band trust anchor rule.** The `--bootstrap-from` source MUST
NOT itself depend on the termlink auth being healed (chicken-and-egg).
`file:` and `ssh:` are safe by construction. `command:<shell>` was
deliberately excluded from T-1055 and requires a later task with explicit
security review before adoption. The operator picks the anchor per incident;
there is no default.

**R3 — read-live, not cache, for own-hub secret (G-011).** When sharing your
local hub's secret with a peer (e.g. during heal-after-rotation handoff),
read directly from the authoritative `<runtime_dir>/hub.secret` file, NOT
from the IP-keyed convenience cache at `~/.termlink/secrets/<hub-ip>.hex`.
The cache is written once at heal time and is NOT invalidated when the hub
regenerates — peers handed a stale cached value will see auth-mismatch
symptoms but the giving end appears clean, masking the true source of the
drift. For self-hub access (where `<runtime_dir>/hub.secret` is locally
readable), point profiles' `secret_file = ...` directly at the live file.
Reserve IP-keyed caches for remote hubs that the operator has explicitly
chosen to cache. Mirror-image of T-1051/T-933 (which address receiving-end
drift); this rule covers giving-end drift. Source incident: 2026-04-20
peer-share where `~/.termlink/secrets/192.168.10.121.hex` had been stale for
~1 day after a hub restart.

**R1 — memory-drift detection via `hub_fingerprint`.** Every auto-registered
learning from T-1052 carries `hub_fingerprint=sha256:<hex>` captured from the
client's `KnownHubStore` at observation time. A future agent that finds a
matching-hub learning should compare that fingerprint against the current
pinned fingerprint (`termlink_session::tofu::KnownHubStore::default_store().get(address)`).
If they differ, the learning predates a rotation and must be treated as
potentially stale — do not act on its conclusions without re-verifying
against current state. This addresses the failure mode observed in the
T-1051 peer review where a prior learning on ring20-dashboard claimed a hub
lived at `.122` after it had already moved back to `.109`.

**Related tasks.** T-1051 (inception, GO on Option D) → T-1052 (learning
auto-register, R1) → T-1053 (concern auto-register, G-019) → T-1054 (Tier-1
heal printer) → T-1055 (Tier-2 `--bootstrap-from`, R2) → T-1056 (rmcp pin,
unblocks consumer installs of the heal CLI) → T-1057 (build.rs version
freshness, so operators can confirm they're running the version that has
these commands) → T-1291 (declarative `bootstrap_from` per profile +
`--bootstrap-from auto`, lowers the floor on every heal). T-1058 added
this documentation.

### Channel Topic Semantics — Per-Hub State (G-060 / T-1791 / T-1792)

TermLink hubs maintain **independent** channel-topic storage. A topic named
`agent-chat-arc` on hub A and `agent-chat-arc` on hub B are unrelated state.
There is no inter-hub federation primitive. Cross-hub visibility requires
explicit, client-driven cross-posting via `termlink channel post --hub <addr>`
(or the `termlink_remote_call` MCP tool for arbitrary peer RPC). A non-zero count delta between
hubs for a shared-name topic is **expected**, not a bug — it just means
different posters used different hubs. See
[`docs/operations/channel-topic-semantics.md`](docs/operations/channel-topic-semantics.md)
for the full diagnostic recipe and implications for T-1166 retirement.

## Core Principle

**Nothing gets done without a task.** This is enforced structurally by the framework, not by agent discipline.

## Four Constitutional Directives (Priority Order)

All architectural decisions must trace back to these directives:

1. **Antifragility** — System strengthens under stress; failures are learning events
2. **Reliability** — Predictable, observable, auditable execution; no silent failures
3. **Usability** — Joy to use/extend/debug; sensible defaults; actionable errors
4. **Portability** — No provider/language/environment lock-in; prefer standards (MCP, LSP, OpenAPI)

## Authority Model

```
Human    →  SOVEREIGNTY  →  Can override anything, is accountable
Framework →  AUTHORITY   →  Enforces rules, checks gates, logs everything
Agent    →  INITIATIVE   →  Can propose, request, suggest — never decides
```

## Instruction Precedence

When multiple instruction sources conflict (CLAUDE.md, plugins, skills, user messages), this resolution order applies:

1. **Framework rules (this file)** — Core Principle, Authority Model, Enforcement Tiers, and Task System rules take absolute precedence. No plugin or skill can override "Nothing gets done without a task."
2. **User instructions** — Direct human instructions can override framework rules via Tier 2 (situational authorization with logging).
3. **Skills/plugins** — Apply AFTER framework gates are satisfied. A skill that says "invoke before any response" means: after verifying an active task exists. Skills enhance workflows; they do not replace framework governance.

**The practical rule:** Before following ANY skill or plugin workflow, first ensure a task exists and focus is set. If a skill's instructions conflict with creating a task first, the task wins.

**Why this matters:** Third-party plugins are not aware of project-specific governance. They will issue instructions like "implement now" or "code first, test first" without checking for task context. The agent must apply framework rules as a pre-filter before deferring to skill workflows.

## Task System

### File Structure

```
.tasks/
  active/      # In-progress tasks (e.g., T-042-add-oauth.md)
  completed/   # Finished tasks
  templates/   # Task templates by workflow type
```

### Task File Format

Tasks are Markdown with YAML frontmatter. Use `default.md` as template.

**Required frontmatter fields:**
- `id`, `name`, `description`, `status`, `workflow_type`, `horizon`, `owner`, `created`, `last_update`

### Horizon (Priority Scheduling)

The `horizon` field controls when a task should be considered for work:

| Value | Meaning | Handover behavior |
|-------|---------|-------------------|
| `now` | Ready to work on (default) | Appears first in Work in Progress, eligible for Suggested First Action |
| `next` | Ready after current work | Appears in Work in Progress, eligible for Suggested First Action |
| `later` | Parked/backlog — not yet | Appears last in Work in Progress, excluded from Suggested First Action |

**Rules:**
- Default horizon is `now` (tasks created via `fw work-on` or `fw task create`)
- Use `--horizon later` for tasks captured for future reference
- Use `fw task update T-XXX --horizon now` to promote a backlog task
- The handover agent sorts tasks by horizon and instructs the enricher to skip `later` tasks in suggestions

**Body sections:**
- Context (brief, link to design docs for substantial tasks)
- Acceptance Criteria (checkboxes — completion gate P-010)
- Verification (shell commands — verification gate P-011, see below)
- Decisions (only when choosing between alternatives; most tasks have none)
- Updates (auto-populated by git mining at completion; manual entries optional)

### Verification Gate (P-011)

The `## Verification` section contains shell commands that **must pass** before `work-completed` is allowed. This is a structural gate — the framework runs the commands mechanically, not the agent self-assessing.

**How it works:**
1. Agent writes verification commands in `## Verification` while working (knows what to check)
2. On `fw task update T-XXX --status work-completed`, update-task.sh extracts and runs each command
3. If any command exits non-zero → completion is **blocked** (same as unchecked AC)
4. `--force` bypasses the gate (with warning, logged)
5. Tasks without `## Verification` pass through (backward compatible)

**What to verify:**
- YAML/JSON files parse correctly: `python3 -c "import yaml; yaml.safe_load(open('file'))"`
- Web pages load: `curl -sf "$(cat .context/working/watchtower.url 2>/dev/null || echo http://localhost:$(bin/fw config get PORT 2>/dev/null || echo 3000))/page"` — never hard-code `:3000`; the triple file `.context/working/watchtower.{pid,port,url}` is the source of truth for Watchtower's port
- Commands succeed: `fw doctor`
- Output contains expected content: `grep -q "expected" output.txt`

**Rules:**
- Lines starting with `#` are comments (skipped)
- Empty lines are ignored
- Each non-comment line is executed as a shell command
- First 5 lines of failure output are shown for debugging

### Task Lifecycle

```
Captured → Started Work ↔ Issues → Work Completed
```

### Workflow Types

| Type | Purpose | Typical Agent |
|------|---------|---------------|
| Specification | Define what to build | Specification Agent |
| Design | Determine how to build | Design Agent |
| Build | Create implementation | Coder Agent |
| Test | Verify correctness | Test Agent |
| Refactor | Improve existing code | Coder Agent |
| Decommission | Remove obsolete code | Deployment Agent |
| Inception | Explore problem, validate assumptions, go/no-go | Human / Any Agent |

## Task Sizing Rules

- **One task = one deliverable.** If a task has multiple independent spikes or deliverables, decompose it.
- **One bug = one task.** Never compound multiple independent bugs into a single ticket. Each bug has its own root cause, fix, and regression test. Compounding destroys causality traceability and dilutes episodic memory.
- **One inception = one question.** An inception task should explore one problem and produce one go/no-go decision. "Umbrella inceptions" that bundle independent explorations create all-or-nothing decisions and coarse progress tracking.
- **Target: fits in one session.** If a task's time-box exceeds 4 hours or requires 3+ sessions, it should be split.
- **Decomposition signal:** 3+ spikes in an exploration plan, or 3+ independent problem domains, means the task is too big.

## Enforcement Tiers

| Tier | Description | Bypass | Implementation |
|------|-------------|--------|----------------|
| 0 | Consequential actions (force push, hard reset, rm -rf /, DROP TABLE) | Human approval via `fw tier0 approve` | PreToolUse hook on Bash (`check-tier0.sh`) |
| 1 | All standard operations (default) | Create task or escalate to Tier 2 | PreToolUse hook on Write/Edit (`check-active-task.sh`) |
| 2 | Human situational authorization | Single-use, mandatory logging | Partial (git --no-verify + bypass log) |
| 3 | Pre-approved categories (health checks, status queries, git-status) | Configured | Spec only |

## Working with Tasks

When starting work (**BEFORE reading code, editing files, or invoking skills**):
1. Check for existing task or create new one following `zzz-default.md` template
2. Set status to `started-work`
3. Set focus: `fw context focus T-XXX`
4. THEN proceed with implementation (skills, code changes, etc.)
5. Record decisions in Decisions section ONLY when choosing between alternatives
6. Updates section is auto-populated at completion — manual entries optional

When encountering errors or unexpected behavior (**NEVER silently work around them**):
1. **STOP and investigate** — do not switch to an alternative path without understanding WHY the error occurred
2. Report the error and your investigation findings to the user
3. If the error is in framework tooling: fix it (this is higher priority than the current task)
4. If the error is environmental: document it and inform the user
5. Only after investigation may you proceed with an alternative approach
6. If the error seems minor but you cannot explain it: that is a signal, not noise — investigate anyway

When encountering task-level issues:
1. Set status to `issues`
2. Log error reference and healing loop suggestions
3. Record resolution when fixed for pattern learning

When discovering structural flaws (bugs in framework tooling, spec-reality gaps):
1. **Register first, fix second.** Add the flaw to `gaps.yaml` BEFORE or alongside the fix
2. Gaps persist in the register (visible in Watchtower, checked by audit); completed tasks archive and become invisible
3. Each independent bug gets its own task (see Task Sizing Rules: "One bug = one task")

When completing:
1. Verify all acceptance criteria met
2. If source files were changed: run `fw fabric blast-radius HEAD` to understand downstream impact
3. Record any design choices in the task's `## Decisions` section (auto-captured to context fabric on completion)
4. Set status to `work-completed`
5. Framework auto-generates episodic summary and captures decisions for future reference

## Context Integration

Tasks feed three memory types:
- **Working Memory** — Active task status and pending actions
- **Project Memory** — Patterns across all tasks (failure modes, effective approaches)
- **Episodic Memory** — Completed task histories for future reference

## Error Escalation Ladder

Graduated response from tactical to structural:
1. **A** — Don't repeat the same failure
2. **B** — Improve technique
3. **C** — Improve tooling
4. **D** — Change ways of working

### Proactive Level D: Operational Reflection

Not all improvement comes from failures. When you notice a practice repeating ad-hoc across 3+ tasks, consider codifying it:

1. **Mine** episodic memory for evidence of the pattern (how often, what worked, what broke)
2. **Assess** codification value — use inception go/no-go criteria
3. **Codify** if warranted: protocol in CLAUDE.md, templates in agents/, guidelines
4. **Record** as learning + decision + workflow pattern

**Trigger:** An organic question about "how we do X" + 3+ instances in episodic memory.

**Canonical example:** T-097 analyzed sub-agent dispatching across 96 tasks → discovered the real problem (result management, not agent specialization) → produced dispatch protocol (T-098) and prompt templates (T-099). The framework used its own episodic memory as the evidence base for an architectural decision.

## fw CLI (Primary Interface)

The `fw` command is the single entry point for all framework operations. It resolves paths, sets environment variables, and routes to agents.

```bash
fw help              # Show all commands
fw version           # Show version and paths
fw doctor            # Check framework health
fw audit             # Run compliance audit
fw context init      # Initialize session
fw git commit -m "T-XXX: description"
fw handover --commit # Generate and commit handover
fw task create --name "Fix bug" --type build --owner human
```

**Path resolution:** `fw` finds the framework via `bin/fw`'s location (inside framework repo) or via `.framework.yaml` in the project root (shared tooling mode).

## Agents

The framework includes agents for common operations. Each agent has a bash script (mechanical) and AGENT.md (intelligence/guidance). All agents can be invoked directly or via `fw`.

### Task Creation Agent

**Location:** `agents/task-create/`

**When to use:** Before starting any new work, create a task.

```bash
# Interactive mode
./agents/task-create/create-task.sh

# With arguments
./agents/task-create/create-task.sh --name "Fix bug" --type build --owner human --start
```

### Task Update (with auto-triggers)

**Location:** `agents/task-create/update-task.sh`

**When to use:** To change task status. Auto-triggers healing diagnosis on `issues`, and finalizes tasks on `work-completed`.

```bash
# Change status (auto-triggers healing if issues)
fw task update T-015 --status issues --reason "API timeout"

# Complete a task (auto: date_finished, move to completed/, generate episodic)
fw task update T-015 --status work-completed

# Change owner
fw task update T-015 --owner human
```

### Audit Agent

**Location:** `agents/audit/`

**When to use:** Periodically check framework compliance. Run after completing work or when suspecting drift.

```bash
./agents/audit/audit.sh
```

**Exit codes:** 0=pass, 1=warnings, 2=failures

### Session Capture Agent

**Location:** `agents/session-capture/`

**When to use:** MANDATORY before ending any session or switching context.

Review the checklist in `agents/session-capture/AGENT.md` and ensure:
- All discussed work has tasks
- All decisions are recorded
- All learnings are captured as practices
- All open questions are tracked

### Git Agent

**Location:** `agents/git/`

**When to use:** For all git operations that involve code changes. Enforces task traceability (P-002).

```bash
# Commit with task reference (required)
./agents/git/git.sh commit -m "T-003: Add bypass log"

# Task-aware status
./agents/git/git.sh status

# Install enforcement hooks (run once per repo)
./agents/git/git.sh install-hooks

# Log a bypass (when --no-verify was used)
./agents/git/git.sh log-bypass --commit abc123 --reason "Emergency hotfix"

# View task-filtered history
./agents/git/git.sh log --task T-003
./agents/git/git.sh log --traceability
```

### Handover Agent

**Location:** `agents/handover/`

**When to use:** MANDATORY at end of every session.

```bash
# Create handover (manual commit)
./agents/handover/handover.sh

# Create handover and auto-commit via git agent
./agents/handover/handover.sh --commit
```

Creates a forward-looking context document in `.context/handovers/` to enable the next session to continue seamlessly.

### Context Agent

**Location:** `agents/context/`

**When to use:** To manage the Context Fabric (persistent memory system).

```bash
# Initialize session (start of session)
./agents/context/context.sh init

# Show context state
./agents/context/context.sh status

# Set/show current focus
./agents/context/context.sh focus T-005
./agents/context/context.sh focus

# Record a learning
./agents/context/context.sh add-learning "Always validate inputs" --task T-014 --source P-001

# Record a pattern (failure/success/workflow)
./agents/context/context.sh add-pattern failure "API timeout" --task T-015 --mitigation "Add retry"

# Record a decision
./agents/context/context.sh add-decision "Use YAML" --task T-005 --rationale "Human readable"

# Generate episodic summary for completed task
./agents/context/context.sh generate-episodic T-014
```

Manages three memory types:
- **Working Memory** — Session state, current focus, priorities
- **Project Memory** — Patterns, decisions, learnings
- **Episodic Memory** — Condensed task histories

### Healing Agent

**Location:** `agents/healing/`

**When to use:** When a task encounters issues (status = `issues`). Implements the antifragile healing loop.

```bash
# Diagnose task issues and get recovery suggestions
./agents/healing/healing.sh diagnose T-015

# After fixing, record the resolution (adds pattern + learning)
./agents/healing/healing.sh resolve T-015 --mitigation "Added retry logic"

# Show all known failure patterns
./agents/healing/healing.sh patterns

# Check all tasks with issues
./agents/healing/healing.sh suggest
```

The healing loop:
1. **Classify** — Identifies failure type (code, dependency, environment, design, external)
2. **Lookup** — Searches for similar patterns in patterns.yaml
3. **Suggest** — Recommends recovery using Error Escalation Ladder
4. **Log** — Records resolution as pattern for future learning

### Resume Agent

**Location:** `agents/resume/`

**When to use:** After context compaction, returning from breaks, or when feeling lost about current state.

```bash
# Full state synthesis (use after compaction)
./agents/resume/resume.sh status

# Fix stale working memory
./agents/resume/resume.sh sync

# One-line summary
./agents/resume/resume.sh quick
```

Synthesizes current state from:
- **Handover** — "Where We Are" and suggested action
- **Working Memory** — Session, focus, may be stale
- **Git State** — Uncommitted changes, recent commits
- **Tasks** — Active tasks with status

## Component Fabric

The Component Fabric (`.fabric/`) is a structural topology map of every significant file in the framework. It enables impact analysis, dependency tracking, and onboarding.

### When to Use

- **Before modifying a file:** `fw fabric deps <path>` — see what depends on it and what it depends on
- **Before committing:** `fw fabric blast-radius` — see downstream impact of your changes
- **After creating new files:** `fw fabric register <path>` — create a component card
- **Periodic health check:** `fw fabric drift` — detect unregistered, orphaned, or stale components

### Key Commands

| Command | Purpose |
|---------|---------|
| `fw fabric overview` | Compact subsystem summary |
| `fw fabric deps <path>` | Show dependencies for a file |
| `fw fabric impact <path>` | Full transitive downstream chain |
| `fw fabric blast-radius [ref]` | Downstream impact of a commit |
| `fw fabric search <keyword>` | Search by tags, name, purpose |
| `fw fabric drift` | Detect unregistered/orphaned/stale |
| `fw fabric register <path>` | Create component card for a file |

### Component Cards

Each component has a YAML card in `.fabric/components/` with: id, name, type, subsystem, location, purpose, interfaces, depends_on, depended_by. Cards are the source of truth for structural relationships.

## Context Budget Management (P-009)

**Context is a finite, non-renewable resource within a session.** Treat it like a battery gauge.

### Commit Cadence Rule
- **Commit after every meaningful unit of work** (not just at session end)
- A "meaningful unit" = completing a subtask, finishing a file, or making a decision
- Each commit is a checkpoint: if context runs out, work up to the last commit is safe
- Target: at least one commit every 15-20 minutes of active work

### Handover Timing Rule
- **Generate handover AFTER work is done, not before**
- Never generate a skeleton handover "to fill in later" — the session may not survive to fill it
- When generating handover: fill in ALL [TODO] sections immediately in the same operation
- For mid-session checkpoints: `fw handover --checkpoint`

### Agent Output Discipline
- When using Task/Agent tools, request concise output (summaries, not raw data)
- See **Sub-Agent Dispatch Protocol** below for detailed rules on managing sub-agent results
- Prefer `fw resume quick` over `fw resume status` for routine checks
- Prefer `git log --oneline -5` over `git log -5`

### Work Proposal Rule
- **Before proposing the next unit of work, check context budget** (`checkpoint.sh status`)
- Below 60% (120K tokens): proceed normally
- 60-75% (120K-150K): propose only small, bounded tasks; commit first
- Above 75% (150K+): propose only wrap-up actions (commit, learnings, handover)
- Above 85% (170K+): handover immediately, no new work
- **This applies especially in autonomous mode** — without a human to catch the mistake, proposing work that can't complete in remaining context risks losing all uncommitted work

### Automated Monitoring (Claude Code)
- **Primary enforcement:** A PreToolUse hook runs `budget-gate.sh` which reads **actual token usage** from the session JSONL transcript and **blocks** Write/Edit/Bash at critical level (exit code 2)
- **Fallback:** A PostToolUse hook runs `checkpoint.sh` for warnings and auto-handover (T-136)
- Escalation ladder: **120K** ok→warn (note), **150K** warn→urgent (warning), **170K** urgent→critical (**BLOCK**)
- At critical, allowed: git commit/add, fw handover/task, reading files, Write/Edit to `.context/` `.tasks/` `.claude/` (wrap-up paths). Blocked: Write/Edit to source files, general Bash
- Status cached in `.context/working/.budget-status` (JSON: level, tokens, timestamp)
- Check current usage: `./agents/context/checkpoint.sh status`
- If no transcript is available, fails open (PostToolUse fallback handles it)

### Critical Protocol
- If you see a SESSION WRAPPING UP block: the session is wrapping up. Only wrap-up work is allowed.
- **Allowed:** git commit/add, fw handover, fw task update, Write/Edit to .context/.tasks/.claude/, reading files
- **Blocked:** Write/Edit to source files, general Bash commands
- Wrap up calmly — task files already have all essential state from continuous capture

## Sub-Agent Dispatch Protocol

When using Claude Code's Task tool to dispatch sub-agents (Explore, Plan, Code, etc.), follow these rules to manage context budget.

### Result Management Rules

**Content generators** (enrichment, file creation, report writing):
- Sub-agent MUST write output to disk (Write tool), NOT return full content
- Return only: file path + one-line summary
- This prevents context explosion from agents returning full file contents

**Investigators/researchers** (codebase exploration, root cause analysis):
- Return structured summaries with findings, NOT raw file contents
- Format: numbered findings with file:line references
- Keep return under 2K tokens per agent

**Auditors/reviewers** (compliance checks, code review):
- Write detailed report to file if >1K tokens
- Return summary + file path to orchestrator
- Include pass/warn/fail counts in summary

### Dispatch Guidelines

| Factor | Rule |
|--------|------|
| Max parallel agents | **5** |
| Token headroom | Leave **40K tokens** free for result ingestion before dispatching |
| When parallel | Tasks are independent, no shared files, no sequential dependency |
| When sequential | Tasks depend on prior results, or editing same files |
| Background agents | Use `run_in_background: true` for agents >2K tokens expected output |

### Prompt Template Structure

When dispatching sub-agents, include in the prompt:

1. **Scope**: Exactly what to investigate/produce (one clear deliverable)
2. **Framework context**: Relevant framework structure (task format, episodic template, etc.)
3. **Output format**: How to return results (write to file vs. return summary)
4. **Constraints**: Don't modify files outside scope, don't return raw data
5. **Token hint**: "Keep your response concise — the orchestrator has limited context budget"

### Result Ledger (`fw bus`)

The result ledger formalizes the "write to disk, return path + summary" convention into a protocol with typed YAML envelopes and automatic size gating. Use it for sub-agent dispatch:

```bash
# Sub-agent posts result (instead of returning full content)
fw bus post --task T-XXX --agent explore --summary "Found 3 issues" --result "inline data"
fw bus post --task T-XXX --agent code --summary "Wrote file" --blob /path/to/output

# Orchestrator reads manifest (5 lines instead of 25KB)
fw bus manifest T-XXX

# Orchestrator reads specific result if needed
fw bus read T-XXX R-001

# Cleanup after task completion
fw bus clear T-XXX
```

**Size gating:** Payloads < 2KB are inline. Payloads >= 2KB are auto-moved to `.context/bus/blobs/` and referenced.

### Dispatch Patterns (from project history)

**Parallel Investigation** (T-059, T-061, T-086): 3-5 Explore agents scan different aspects. Each returns structured findings. Orchestrator synthesizes.

**Parallel Audit** (T-072): 3 agents review different artifact categories. Each returns pass/warn/fail summary. Combined into report.

**Parallel Enrichment** (T-073): N agents each produce one file. MUST write to disk, return only path+summary. Cap at 5 parallel. Use `fw bus post` for formal tracking.

**Sequential TDD** (T-058): Fresh agent per implementation task with review between.

## Agent Behavioral Rules

These rules govern agent behavior during work. They are structural expectations, not suggestions.

### Choice Presentation
Always present choices as a **numbered or lettered list** so the user can reply with just the identifier (e.g., "1" or "b"). Never present options as prose paragraphs.

### Autonomous Mode Boundaries
When the human says "proceed as you see fit", "go ahead", "do what you think is best", or similar broad directives, this delegates **initiative** (choosing what to work on), NOT **authority** (approving, completing, or bypassing). Specifically:

**Delegated (agent may do autonomously):**
- Choose which task to work on next
- Choose implementation approach within a task
- Run verification, tests, audits
- Commit completed work and report back

**NOT delegated (requires explicit human approval per action):**
- Completing human-owned tasks (`owner: human`)
- Using `--force` to bypass any gate (sovereignty, AC, verification)
- Changing task ownership away from human
- Destructive actions (Tier 0)
- Any action the sovereignty gate or structural enforcement blocks

**The rule:** If a structural gate blocks you, that gate exists precisely for moments like this. A broad directive does not override structural enforcement. Stop and ask.

### Pickup Message Handling (G-020, T-469)
Pickup messages from other sessions are **PROPOSALS, not build instructions.** A detailed spec with file lists and implementation steps is a suggestion, not authorization.

Before acting on a pickup message:
1. **Assess scope** — if it describes >3 new files, a new subsystem, a new CLI route, or a new Watchtower page, create an **inception** task (not build)
2. **Write real ACs** before editing any source file — the build readiness gate (G-020) will block tasks with placeholder ACs
3. **Never treat detailed specs as authorization to skip scoping** — the more detailed a pickup message is, the more likely it needs inception, not less

### Human Task Completion Rule (T-372, T-373)
Human ACs represent real verification steps. Unvalidated deliverables carry downstream risk. A clean task list is not progress — validated deliverables are progress.

**You MAY suggest closing a human-owned task IF you provide evidence that the Human ACs are already satisfied:**
- Cite specific evidence (file exists, endpoint responds, output matches expected, config is in place)
- Explain why no further human action is needed

**You MUST NOT suggest closing without evidence:**
- No "batch-close stale tasks" — each task needs individual evidence
- No "just use `--force`" — that skips the verification the AC exists to perform
- No treating Human ACs as administrative overhead — they catch real problems

**Use `fw task verify`** to see what Human ACs are unchecked before suggesting anything.

**The test:** "Can I cite specific evidence that this task's Human ACs are satisfied?" If yes, suggest closing with that evidence. If no, either help the human execute the verification steps, or move on.

### Commit Cadence and Check-In
After **every commit**, briefly report what was done and ask if the user wants to continue. Do not chain multiple commits without user interaction.

**Structural enforcement (T-139):** The `budget-gate.sh` PreToolUse hook reads actual token usage from the session transcript and **blocks** Write/Edit/Bash tool calls when context reaches critical level (>=150K tokens, ~75%). At critical, only git commit, fw handover, and read operations are allowed. The hook writes `.context/working/.budget-status` with current level (ok/warn/urgent/critical) for fast caching. PostToolUse `checkpoint.sh` remains as fallback for warnings and auto-handover.

### Copy-Pasteable Commands (T-609)
When giving the human a command to run (Tier 0 approvals, inception decisions, verification steps, Human AC instructions), the command MUST be:

1. **Single-line, copy-pasteable** — works when pasted into any terminal, from any directory
2. **Prefixed with `cd`** — always include `cd /path/to/project &&` so directory context is explicit
3. **Use the vendored path, not global `fw`** — in this consumer project the executable lives at `.agentic-framework/bin/fw`, never `bin/fw` (that only exists inside the framework repo itself). The global `fw` may resolve to a different install. Correct: cd into the project and call `.agentic-framework/bin/fw <cmd>`.
4. **No bare multi-line** — if multiple commands are needed, chain with `&&` on one line

### Inception Discipline
When the active task has `workflow_type: inception`:
1. **State the phase** — Say "This is an inception/exploration task" before doing any work
2. **Present the filled template** for review before executing any spikes or prototypes
3. **Do not write build artifacts** (production code, full apps) before `fw inception decide T-XXX go`
4. **The commit-msg hook enforces this** — after 2 exploration commits, further commits are blocked until a decision is recorded
5. After a GO decision, **create separate build tasks** for implementation — do not continue building under the inception task ID
6. **Research artifact first (C-001)** — When starting inception work, create `docs/reports/T-XXX-*.md` BEFORE conducting research. Update the file incrementally as dialogue produces findings. Commit after each dialogue segment. The thinking trail IS the artifact — conversations are ephemeral, files are permanent.
7. **Dialogue log (C-001 extension)** — For phases involving human dialogue, include a `## Dialogue Log` section in the research artifact. Record: questions the human posed, answers given, course corrections, and the outcome/decision that resulted.
8. **DEFER outcomes set `revisit_at` (T-1451, G-053)** — When `fw inception decide T-XXX defer` is the outcome, also set `revisit_at: YYYY-MM-DD` in the task frontmatter to the date when the decision should be reconsidered. Pair with `revisit_evidence_needed: <one-line>` to specify what evidence makes the revisit actionable. The daily G-053 cron (T-1452) will surface ripe revisits in the handover banner — without this field, the deferral has no structural reminder.

### Web App Startup
When building a web application:
1. **Check port availability** before starting (`ss -tlnp | grep :PORT`)
2. **Start the app** and report the URL to the user
3. **Report access options** — localhost, LAN IP (for other devices), internet (if applicable)
4. Never leave a built web app unstarted without informing the user

### Constraint Discovery
For tasks involving hardware APIs (microphone, camera, GPS, Bluetooth):
1. **Research platform constraints first** before building (e.g., getUserMedia requires HTTPS or localhost)
2. **List constraints in the exploration plan** before writing code
3. **Test the API access path** in a minimal spike before building the full app

### Agent/Human AC Split (T-193)
Tasks may have `### Agent` and `### Human` sections under `## Acceptance Criteria`:
- **Agent ACs:** Criteria the agent can verify (code, tests, commands). P-010 gates on these.
- **Human ACs:** Criteria requiring human verification (UI behavior, subjective quality). Not blocking.
- **NEVER check a `### Human` AC.** Only the human may verify and check these boxes.
- When agent ACs pass but human ACs remain unchecked, the task enters **partial-complete**: stays in `active/` with `owner: human`.
- The human finalizes by checking their ACs and running `fw task update T-XXX --status work-completed`.

### Human AC Format Requirements (T-325)
When writing `### Human` acceptance criteria, each criterion MUST include:
- **Steps:** block with numbered, copy-pasteable instructions (no placeholders the human must figure out)
- **Expected:** what success looks like (exact text, status code, or observable outcome)
- **If not:** diagnostic steps or fallback action

Optionally prefix the criterion with a confidence marker:
- `[RUBBER-STAMP]` — mechanical action, no judgment needed (publish, deploy, click)
- `[REVIEW]` — genuine human judgment required (tone, UX, architecture decisions)

**Prerequisite awareness (T-358):** Steps must start from the human's actual environment, not the agent's dev context. If the feature requires deployment, upgrade, or setup before testing, include those steps first.

If a human AC cannot be made specific (e.g., "code quality is acceptable"), replace it with a measurable proxy or remove it. Vague ACs that nobody acts on are worse than no AC.

### Verification Before Completion
Before setting any task to `work-completed`:
1. Run all commands in the task's `## Verification` section
2. Check every `### Agent` acceptance criterion checkbox (or all ACs if no split headers)
3. If tests exist for the changed code, run them
4. Report results to user with pass/fail evidence
5. Do NOT call `fw task update --status work-completed` until all pass
6. The verification gate (P-011) enforces this structurally — this rule makes you check BEFORE hitting the gate

### Hypothesis-Driven Debugging
When encountering errors or unexpected behavior:
1. **State the symptom** in one sentence
2. **Form one hypothesis** for the root cause
3. **Design one test** to prove or disprove it (a command, a log check, a code read)
4. Run the test and report the result
5. If disproved, form the next hypothesis — max **3 hypotheses** before escalating to user
6. Never shotgun-debug (trying random fixes without understanding the cause)
7. After resolution, record the pattern: `fw healing resolve T-XXX --mitigation "what fixed it"`

### Bug-Fix Learning Checkpoint
When fixing a bug discovered through real-world usage (user testing, production incident, cross-platform failure):
1. **Classify the bug** — Is this a new failure class, or a repeat of a known pattern?
2. **Check learnings.yaml** — Does a learning already exist for this class?
3. If new class: `fw context add-learning "description" --task T-XXX --source P-001`
4. If systemic (same class hit 2+ times): register in `concerns.yaml`, consider tooling fix (Level C/D)

**Trigger:** Any fix cycle addressing a bug found by someone other than the agent (user report, CI failure, production monitoring, cross-platform testing).

**Not triggered by:** Fixes for bugs found during development (pre-commit). Those are normal development, not field discoveries.

**The test:** "If another agent encounters this same class of bug in 6 months, would a learning entry help them fix it faster?" If yes, capture it now.

### Post-Fix Root Cause Escalation (G-019)
After fixing any problem discovered by the human (not found during development):
1. **Fix the symptom** — make it work (Level A/B/C)
2. **Ask: "Why did the framework allow this?"** — not "why did the code break" but "what structural omission let this go undetected?"
3. **If the framework was blind for >7 days:** register a gap in `concerns.yaml` — even if it's a single incident, sustained blindness reveals a systemic flaw
4. **Do not close the gap until prevention exists** — mitigation (cleaned up the mess) is not prevention (can't happen again). Ask: "Did I fix the symptom, or did I fix the reason the framework couldn't detect it?"

**Trigger:** Human corrects the agent's escalation level, or agent discovers a problem that existed undetected for >7 days.

## Plan Mode Prohibition

**NEVER use the built-in `EnterPlanMode` tool.** It bypasses all framework governance:
- No task gate — planning starts without a task
- No session init — Session Start Protocol is skipped entirely
- No research artifacts — plan files go to `.claude/plans/` (untracked, ephemeral)
- Its system prompt says "This supercedes any other instructions" — overriding CLAUDE.md
- Post-plan execution skips commit cadence, task updates, and check-ins

**Use `/plan` instead** — the framework's governance-aware planning skill that:
- Requires an active task (verified in Step 1)
- Writes to `docs/plans/` (tracked, committed)
- Respects instruction precedence

If you need to explore before planning, use the Explore agent or `/explore` skill.
If you need to plan implementation, create a task first, then use `/plan`.

## Session Start Protocol

**Before beginning any work:**
1. Initialize context: `fw context init`
2. Read `.context/handovers/LATEST.md` to understand current state
3. Review the "Suggested First Action" section
4. Set focus: `fw context focus T-XXX`
5. Run `fw metrics` to see project status
6. If handover feedback section exists, fill it in
7. *(Optional, recommended for any non-trivial session)* Opt into agent-presence so peers can reach you: `/be-reachable` (T-1841). Idempotent — safe to run unconditionally; stops cleanly on `/be-reachable stop` before session end. Skip on throw-away sessions (<2 min) or hosts that should not appear on the fleet.

**Before ANY implementation (even if a skill says "start now"):**
1. Verify a task exists for the work: `fw work-on "name" --type build` or `fw work-on T-XXX`
2. Confirm focus is set in `.context/working/focus.yaml`
3. THEN proceed with implementation

This gate is non-negotiable. The PreToolUse hook will block Write/Edit without an active task. Use `/start-work` if unsure.

**Manual compaction (`/compact`):**
- Auto-compaction is disabled by design (D-027 — compaction destroys working memory)
- `/compact` is available for manual use when context is high and you want a clean slate
- The PreCompact hook automatically generates a handover before compaction
- The SessionStart:compact hook reinjects structured context into the fresh session
- After compaction, follow the recovery steps below

**After context compaction (mid-session recovery):**
1. Run resume: `fw resume status`
2. Sync working memory: `fw resume sync`
3. Continue from recommendations

## Quick Reference

| Action | fw command | Direct |
|--------|-----------|--------|
| **Start work** | **`fw work-on "name" --type build`** | Creates task + sets focus + starts work |
| Resume task | `fw work-on T-XXX` | Sets focus + status to started-work |
| Create task | `fw task create` | `./agents/task-create/create-task.sh` |
| Create with tags | `fw task create --tags "ui,api"` | `create-task.sh --tags "..."` |
| Update task | `fw task update T-XXX --status ...` | `./agents/task-create/update-task.sh T-XXX ...` |
| Add tags | `fw task update T-XXX --add-tag "ui"` | `update-task.sh T-XXX --add-tag "..."` |
| Set horizon | `fw task update T-XXX --horizon later` | `update-task.sh T-XXX --horizon later` |
| Commit changes | `fw git commit -m "T-XXX: ..."` | `./agents/git/git.sh commit -m "T-XXX: ..."` |
| Task-aware status | `fw git status` | `./agents/git/git.sh status` |
| Install git hooks | `fw git install-hooks` | `./agents/git/git.sh install-hooks` |
| Run audit | `fw audit` | `./agents/audit/audit.sh` |
| Show gaps | `fw gaps` | _(fw only)_ |
| Health check | `fw doctor` | _(fw only)_ |
| View metrics | `fw metrics` | `./metrics.sh` |
| Predict effort | `fw metrics predict --type build` | _(fw only)_ |
| Promotion candidates | `fw promote suggest` | _(fw only)_ |
| Promote learning | `fw promote L-XXX --name "..." --directive D1` | _(fw only)_ |
| Graduation status | `fw promote status` | _(fw only)_ |
| Initialize session | `fw context init` | `./agents/context/context.sh init` |
| Set focus | `fw context focus T-XXX` | `./agents/context/context.sh focus T-XXX` |
| Context status | `fw context status` | `./agents/context/context.sh status` |
| Add learning | `fw context add-learning "..."` | `./agents/context/context.sh add-learning "..."` |
| Diagnose issue | `fw healing diagnose T-XXX` | `./agents/healing/healing.sh diagnose T-XXX` |
| Resolve issue | `fw healing resolve T-XXX` | `./agents/healing/healing.sh resolve T-XXX` |
| Show patterns | `fw healing patterns` | `./agents/healing/healing.sh patterns` |
| Resume state | `fw resume status` | `./agents/resume/resume.sh status` |
| Sync working memory | `fw resume sync` | `./agents/resume/resume.sh sync` |
| Session capture | Review `agents/session-capture/AGENT.md` checklist | |
| Post bus result | `fw bus post --task T-XXX --agent TYPE --summary "..."` | |
| Read bus results | `fw bus read T-XXX [R-NNN]` | |
| Bus manifest | `fw bus manifest [T-XXX]` | |
| Clear bus channel | `fw bus clear T-XXX` | |
| Generate handover | `fw handover` | `./agents/handover/handover.sh` |
| Handover + commit | `fw handover --commit` | `./agents/handover/handover.sh --commit` |
| Read last handover | `cat .context/handovers/LATEST.md` | |
| **Start inception** | **`fw inception start "name"`** | Creates inception task + sets focus |
| Inception status | `fw inception status` | Lists active inception tasks |
| Inception decide | `fw inception decide T-XXX go` | Records go/no-go with rationale |
| Add assumption | `fw assumption add "..." --task T-XXX` | Register assumption |
| Validate assumption | `fw assumption validate A-XXX --evidence "..."` | Mark validated |
| List assumptions | `fw assumption list` | Show all by status |
| Tier 0 approve | `fw tier0 approve` | Approve a blocked destructive command |
| Tier 0 status | `fw tier0 status` | Show Tier 0 enforcement status |
| Fabric overview | `fw fabric overview` | `./agents/fabric/fabric.sh overview` |
| Fabric deps | `fw fabric deps <path>` | `./agents/fabric/fabric.sh deps <path>` |
| Fabric impact | `fw fabric impact <path>` | `./agents/fabric/fabric.sh impact <path>` |
| Blast radius | `fw fabric blast-radius [ref]` | `./agents/fabric/fabric.sh blast-radius [ref]` |
| Fabric drift | `fw fabric drift` | `./agents/fabric/fabric.sh drift` |
| Register component | `fw fabric register <path>` | `./agents/fabric/fabric.sh register <path>` |
| **Find idle agents (DISPATCH)** | **`termlink agent find-idle [--role R] [--capability C] [--limit N] [--json\|--watch SECS]`** | T-2045/T-2020 substrate primitive #2 — hub-derived `LIVE(agent-presence) \ DISTINCT(claimer)`. Pure read, local-hub-only. Returns `{agent_id, last_heartbeat_ms, role, capabilities}` per idle agent. MCP parity: `termlink_agent_find_idle`. Producers advertise via `metadata.capabilities` (csv) on heartbeat — wire it with `TERMLINK_CAPABILITIES=…` env or `--capabilities` on `/be-reachable` and `scripts/listener-heartbeat.sh`. Pair with `channel.claim` (T-2019) for the assign step. See `docs/operations/agent-find-idle.md`. T-2078 added Slice 1 of the substrate-find-idle observability arc: `--watch <secs>` continuous monitor (clamped [5, 3600], mirror of `claims-summary --watch`). Clears screen + re-renders the idle table each tick + groundwork diff helper `diff_idle_sets` producing `IdleChangeEvent` (`New` / `Removed` kinds — idle is binary, re-heartbeat is not a state change). Mutex with `--json` (NDJSON-on-cleared-screen would be unparseable). Fetch errors during a tick print "fetch error (will retry next tick)" and the loop continues. T-2079 added Slice 2 — `--notify <CMD>` event hook fired fire-and-forget per `IdleChangeEvent` (mirror of T-2072 claims `--notify` / T-1669 fleet doctor `--notify`). Baseline tick skipped. Per-event env vars: `TERMLINK_IDLE_AGENT_ID`, `TERMLINK_IDLE_CHANGE_KIND` (`new`/`removed`), `TERMLINK_IDLE_TS` (RFC3339), `TERMLINK_IDLE_ROLE` (`"-"` if absent), `TERMLINK_IDLE_CAPABILITIES` (csv, empty if none), `TERMLINK_IDLE_LAST_HEARTBEAT_MS`. Hanging scripts do NOT block the loop; command-not-found does NOT kill the watch (spawn failure → stderr line + continue). Requires `--watch` (events only exist across ticks). Pure helper `fire_idle_notify_env` extracted + unit-tested for both `new` and `removed` event kinds. Common gate: `[ "$TERMLINK_IDLE_CHANGE_KIND" = "new" ] || exit 0` then dispatch work to the freshly-idle agent. Recipe: `termlink agent find-idle --role claude-code --watch 30 --notify /usr/local/bin/dispatch-on-idle.sh` — orchestrator's event-driven "assign work when a worker frees up" loop. T-2080 added Slice 3 — `--log <PATH>` append-only NDJSON audit trail (mirror of T-2073 claims `--log` / T-2066 governor `--log`). Schema: `{ts, agent_id, kind, role, capabilities, last_heartbeat_ms}` — one flat jq-friendly line per `new`/`removed` event (absent role serializes as JSON null, empty capabilities as `[]`). Best-effort writes (parent dir auto-created; disk-full / permission errors print one-line stderr warning + continue, watch never crashes). Symmetric with `--notify` — when both flags are set, each event lands in both surfaces from the same per-tick event source. Forensic retrospective via `jq -c 'select(.agent_id=="claude-alpha" and .kind=="removed")' ~/.termlink/find-idle.log` — answers "when did this worker go busy?" without keeping the watch terminal attached. Recipe (real-time dispatch + forensic trail): `termlink agent find-idle --role claude-code --watch 30 --notify /usr/local/bin/dispatch-on-idle.sh --log ~/.termlink/find-idle.log`. T-2081 added Slice 4 — `termlink agent find-idle-history [--since DAYS] [--agent-id ID] [--log PATH] [--json]` retrospective verb that walks the audit log (default `~/.termlink/find-idle.log`), filters by window + agent_id, renders one human-format line per matching entry + per-agent aggregate footer (`<agent_id>  N new  N removed`). Default 7-day window, clamped 1..=365. Mirror of T-2074 `channel claims-history` / T-2068 `fleet governor-history`. Pure read; no auth; no network. Answers "did claude-alpha go busy in the last hour?" / "is this worker flapping?" without keeping the watch terminal still attached. Pure helpers `parse_find_idle_log` + `aggregate_find_idle_entries` extracted; malformed lines skip + count in summary. Missing log → hint pointing back at `agent find-idle --watch --log`. JSON envelope shape: `{ok, entries[], summary{total, per_agent:{<id>:{new, removed}}, since_days, agent_id_filter, malformed_lines_skipped, log_path}}`. Idle is binary (no `transition` kind — re-heartbeat is not a state change, see T-2078 design note), so aggregate counts new/removed only. Recipe: `termlink agent find-idle-history --since 1 --agent-id claude-alpha` after a flap to see the new/removed timeline. T-2082 added Slice 5 — `termlink_agent_find_idle_history` MCP parity — same params (`since_days` default 7 clamped 1..=365, `agent_id` exact-match, `log_path` override), same `{ok, entries, summary{total, per_agent:{<id>:{new_events, removed_events}}, since_days, agent_id_filter, malformed_lines_skipped, log_path}}` shape. Pure helpers `parse_find_idle_log_mcp` + `aggregate_find_idle_entries_mcp` duplicated into termlink-mcp/src/tools.rs (T-2069 convention — no cross-crate sharing for these tiny pure helpers). Read-only file scan; no auth; no network; no log mutation. Missing log returns `{ok:true, entries:[], hint:"no find-idle history yet — run \`agent find-idle --watch --log <path>\` to start capturing"}`. Use when an agent investigating worker flap needs to answer "is this worker flapping?" / "when did claude-alpha go busy?" without shelling out. MCP `--watch` parity (T-2071 analog) + filter slices follow. T-2109 closed the substrate primitives #2 + #9 cross-reference: `handle_agent_find_idle` now reads `cv_index().current_values("agent-presence")` and, when non-empty, drives discovery through new `Bus::find_idle_agents_from_hint` — O(N_agents) single-offset reads (via new `Bus::envelope_at` primitive) instead of O(N_heartbeats) walk. For a 5-agent fleet running 24h × 30s heartbeats: 14,400 envelopes walked → 5 targeted reads (~3000× cheaper). Same role/capability/LIVE-window filters + claimer-anti-join + freshest-first sort + limit semantics — drop-in fast path. Empty cv_index (cold start, no producers wired post-T-2107) falls back to the walk path; producers that opt out via `--no-cv-key` are invisible to the fast path (documented trade-off — opt-out is for tests / migration). 13 new bus unit tests cover both primitives end-to-end. |
| **Transfer claim ownership (ASSIGN)** | **`termlink channel claim-transfer --claim-id C --to-owner W --by B [--reason ...] [--json]`** | T-2046/T-2021 substrate primitive #3 — atomic cooperative ownership transfer of an existing claim. The orchestrator-to-worker handoff verb that eliminates the release-then-claim race. `by` MUST equal current `claimer` (returns CLAIM_NOT_OWNED -32017 otherwise); lease timestamps survive the transfer. MCP parity: `termlink_channel_claim_transfer`. Distinct from `channel claim-force-release` (operator-Tier-0 ownership bypass). End-to-end assign recipe (find-idle → claim → post DM → transfer → release) in `docs/operations/substrate-claim-primitive.md` § "Hand a unit to a specific worker without a race window". |
| **Stuck-claim event hook (CLAIM-OBSERVABILITY)** | **`termlink channel claims-summary [<TOPIC>\|--all] --watch <secs> --notify <CMD> [--log <PATH>]`** | T-2072 substrate primitive #1 observability arc — operator-pluggable shell command fired fire-and-forget on per-topic stuck-state transitions (and `new`/`removed` topics under `--all`). Mirror of T-2065's `fleet governor-status --watch --notify`. Baseline tick fires no events. Per-event env: `TERMLINK_CLAIM_TOPIC`, `TERMLINK_CLAIM_CHANGE_KIND` (`transition`/`new`/`removed`), `TERMLINK_CLAIM_TS` (RFC3339), `TERMLINK_CLAIM_HUB`, `TERMLINK_CLAIM_OLD_STUCK` / `TERMLINK_CLAIM_NEW_STUCK` (`true`/`false`/`n/a`), `TERMLINK_CLAIM_ACTIVE_COUNT`, `TERMLINK_CLAIM_EXPIRED_COUNT`, `TERMLINK_CLAIM_OLDEST_AGE_MS` (or `n/a`). Hanging scripts do NOT block the loop; command-not-found does NOT kill the watch. Common gate: `[ "$TERMLINK_CLAIM_NEW_STUCK" = "true" ] || exit 0` then page/Slack. Recipe: `termlink channel claims-summary --all --watch 30 --notify /usr/local/bin/page-on-stuck.sh`. Composes with `--all` for fleet-wide stuck-detection. Underlying heuristic for "stuck" (T-2042): `expired_count > 0` OR `oldest_active_age_ms > 60_000`. Clap requires `--watch`. T-2073 added the audit-trail companion: `--log <PATH>` appends append-only NDJSON lines (`{ts, topic, kind, hub, old_stuck, new_stuck, active_count, expired_count, oldest_age_ms}`) — one per change event. Mirror of T-2066's `fleet governor-status --watch --log`. Parent dir auto-created. Disk-full / permission errors print one-line stderr warning + continue (watch never crashes). Symmetric with `--notify` — when both flags are set, each event lands in both surfaces. Forensic retrospective: `jq -c 'select(.topic=="work-queue" and .new_stuck==true)' ~/.termlink/claims.log`. Recipe (real-time page + forensic trail): `termlink channel claims-summary --all --watch 30 --notify /usr/local/bin/page-on-stuck.sh --log ~/.termlink/claims.log`. T-2074 added the read-side companion: `termlink channel claims-history [--since DAYS] [--topic NAME] [--log PATH] [--json]` walks the audit log (default `~/.termlink/claims.log`), filters by window + topic, renders one human-format line per matching entry + per-topic aggregate footer (`<topic>  N transition(s)  N new  N removed`). Default 7-day window, clamped 1..=365. Mirror of T-2068 `fleet governor-history` (which reads `governor.log`). Pure read; no auth; no network. Answers "has this topic been stuck repeatedly?" without needing the watch terminal still attached. Pure helpers `parse_claims_log` + `aggregate_claims_entries` extracted; malformed lines skip + count in summary. Missing log → hint pointing back at `claims-summary --watch --log`. T-2075 added the MCP parity — `termlink_channel_claims_history` — same params (`since_days` default 7 clamped 1..=365, `topic` exact-match, `log_path` override), same `{ok, entries, summary{total, per_topic:{<topic>:{transitions, new_events, removed_events}}, since_days, topic_filter, malformed_lines_skipped, log_path}}` envelope. Pure helpers `parse_claims_log` + `aggregate_claims_entries` duplicated into termlink-mcp/src/tools.rs (T-2069 convention — no cross-crate sharing for these tiny pure helpers). Read-only file scan; no auth; no network; no log mutation. Use when an agent investigating stuck claims needs to answer "is this topic flapping?" without shelling out. T-2076 added the operator-actionable subset filter: `termlink channel claims-summary --all --only-stuck` drops non-stuck topics from the output. Pure presentation-level filter — the fleet-wide footer keeps truthful `topic_count` + `stuck_count` totals plus a new `shown` count of rendered rows. Healthy fleet path under `--only-stuck` prints `All topics healthy (0/N stuck)` — affirmative confirmation, not silent success (mirror of T-2070's `fleet governor-status --only-pressured`). Fetch errors always retained regardless of filter (they could mask a stuck topic). JSON envelope gains `shown` + `only_stuck` fields. Clap requires `--all` (single-topic mode has nothing to filter). Recipe: `termlink channel claims-summary --all --only-stuck` — operator's "show me what needs attention" verb on a fleet with hundreds of topics. T-2077 added the MCP parity — `termlink_channel_claims_summary_all` now accepts an `only_stuck` param (default false, backward compatible). When true, the same predicate filters non-stuck `ok:true` entries from `topics[]` at the JSON layer; fetch errors (`ok:false`) are always retained (they could mask a stuck topic); the `summary` keeps fleet-wide `topic_count` + `stuck_count` totals and gains matching `shown` + `only_stuck` fields. Agents investigating fleet claim health get the same "show me what needs attention" affordance as the operator at the CLI without filtering client-side. Mirror of T-2071's MCP shape for the governor arc. |
| **Hub governor status (BACKPRESSURE)** | **`hub.governor_status` JSON-RPC / `termlink_hub_governor_status` MCP / `termlink hub status --governor` CLI / `termlink fleet governor-status [--watch N]` CLI / `termlink fleet governor-history [--since DAYS] [--hub NAME] [--json]` CLI / `termlink_fleet_governor_history` MCP** | T-2048/T-2028 Track B+C+D+E substrate primitive #10 — read connection-cap + per-sender rate-limit counters from the running hub. Returns `{connections_active, connections_max, capacity_hits_total, rate_buckets_active, rate_hits_total, max_rate_per_sec}` plus T-2049 dedupe fields `{dedupe_entries_active, dedupe_hits_total, dedupe_ttl_ms}`. Pure Observe-scope read. `capacity_hits_total > 0` means a connection was refused with `HUB_AT_CAPACITY` (-32019); `rate_hits_total > 0` means an RPC was refused with `RATE_LIMITED` (-32008); `dedupe_hits_total > 0` means a spoke retry was absorbed before double-applying. LOUD-refuse per IW-3: both refusals carry structured `data.retry_after_ms` and (for rate-limit) `data.sender`. Operators tune via `TERMLINK_MAX_CONNECTIONS` (default 256) + `TERMLINK_RATE_LIMIT_PER_SEC` (default 1000) + `TERMLINK_DEDUPE_TTL_MS` (default 300_000) + `TERMLINK_DEDUPE_CAPACITY` (default 10_000) env vars at hub start. T-2060 (Track C) added the `--governor` CLI surface so an operator at a console can read the counters inline alongside lifecycle without shelling out to MCP. T-2062 (Track D) added the fleet-wide aggregation — `fleet governor-status` walks `~/.termlink/hubs.toml`, probes each hub under a per-hub `--timeout` bound (default 8s), renders per-hub blocks + roll-up totals (`hubs_at_capacity`, `hubs_rate_limited`) so a multi-hub operator can answer "which hub is wedged?" in one command. T-2064 (Track E) added the continuous-monitor — `fleet governor-status --watch <secs>` polls every N seconds (clamped [5, 3600]), emits a baseline cycle then change-only output (per-hub `cap_hits=X→Y(+delta)`, `rate_hits=X→Y(+delta)`, `dedupe_hits=X→Y(+delta)`, plus loud `UNREACHABLE` / `REACHABLE again` transition lines); silent cycles render a single "no changes" footer; SIGINT exits cleanly. Pattern parity with `fleet doctor --watch` (T-1667). Leave it running in a terminal and the next time a hub starts refusing connections you'll see it loud. T-2065 (Track F) added the `--notify <CMD>` event hook — operator-pluggable shell command fired fire-and-forget on every per-hub change event (skipped on baseline). Per-event env: `TERMLINK_GOV_HUB`, `TERMLINK_GOV_CHANGE_KIND` (`transition`/`new`/`removed`), `TERMLINK_GOV_TS`, plus before/after for `REACH`/`CONN_ACTIVE`/`CAP_HITS`/`RATE_HITS`/`DEDUPE_HITS` and `+DELTA` for the three counters. Mirror of T-1669 (fleet doctor's `--notify`). Common gate: `[ "$TERMLINK_GOV_CAP_HITS_DELTA" -gt 0 ] || exit 0` then page/Slack. Recipe: `termlink fleet governor-status --watch 30 --notify /usr/local/bin/page-on-cap.sh`. T-2066 (Track G) added the `--log <PATH>` append-only NDJSON audit trail — one flat jq-friendly line per transition/new/removed event with numeric counters + nullable dedupe fields + explicit `cap_hits_delta`/`rate_hits_delta`/`dedupe_hits_delta`. Mirror of T-1671's `~/.termlink/rotation.log`. Best-effort writes (parent dir auto-created, disk-full/permission errors log to stderr but never crash the watch). Forensic retrospective via `jq -c 'select(.hub=="ring20-management" and .cap_hits_delta>0)' ~/.termlink/governor.log` — or use T-2068's native `fleet governor-history` verb (below) which renders the same lines as the watch-loop change events plus per-hub aggregate footers.

T-2068 (closure) added the read-side companion — `fleet governor-history --since DAYS --hub NAME --log PATH --json` walks `~/.termlink/governor.log` (or any `--log` override path matching the watch loop's destination), filters by window + hub, renders one human-format line per matching entry (`<ts>  <hub>  <kind>  conn=A→B cap=X→Y(+d) rate=X→Y(+d) dedupe=…`), and prints a per-hub aggregate footer summing `cap_hits_delta`, `rate_hits_delta`, `dedupe_hits_delta`. Mirror of T-1671's `fleet history` (which reads `rotation.log`). Pure read; no auth; no network. Answers "has this hub been backpressured recently?" / "how many rate-limit hits across the fleet this week?" without keeping a watch terminal open. Malformed lines skip with stderr warning + count in summary. Missing log → hint pointing back at `fleet governor-status --watch --log`. Closes the §6 #10 substrate-governor arc end-to-end (RPC → CLI single → CLI fleet → watch → notify → log → history). T-2069 added the MCP parity — `termlink_fleet_governor_history` — same params (`since_days` default 7, clamped 1..=365; `hub` filter; `log_path` override), same `{ok, entries, summary{total, per_hub:{<hub>:{events, cap_hits_total, rate_hits_total, dedupe_hits_total, cv_overflow_hits_total (T-2119)}}, since_days, hub_filter, malformed_lines_skipped, log_path}}` shape. Read-only file scan; pure helpers `parse_governor_log` + `aggregate_governor_entries` extracted and unit-tested for malformed-line skip + hub/cutoff filter + delta aggregation. Use when an agent investigating fleet backpressure needs to answer "is this hub the one being refused?" without shelling out. T-2070 added `--only-pressured` to `fleet governor-status` — operator filter that shows only hubs needing attention (unreachable, at-capacity, capacity_hits_total > 0, OR rate_hits_total > 0). Pure presentation-level filter; the summary footer still carries fleet-wide totals so the operator sees both "1/5 pressured" and the raw counts at a glance. Mirror of `fleet verify --exit-on-drift-only` (T-1661). Healthy fleet path: prints `All hubs healthy (0/N pressured)` instead of an empty block — affirmative confirmation, not silent success. T-2071 added the MCP parity — `termlink_fleet_governor_status` now accepts an `only_pressured` param (default false, backward compatible). When true, the same predicate filters `hubs[]` at the JSON layer; the `summary` block keeps fleet-wide totals and gains matching `shown` + `only_pressured` fields. Agents investigating fleet backpressure get the same affordance as the operator at the CLI without iterating the array client-side. Recipe: `termlink fleet governor-status --watch 30 --log ~/.termlink/governor.log --notify /usr/local/bin/page-on-cap.sh` (combine for both real-time paging + forensic trail). Missing fields (older hubs that pre-date T-2048) render as `n/a`. The "T-1991 found-in-production-not-predicted" failure mode now surfaces here BEFORE it wedges the substrate. See `docs/operations/substrate-governor.md` + `docs/operations/substrate-post-idempotency.md`. T-2110 closed the substrate primitives #9 + #10 cross-reference: the response now carries four additional cv_index telemetry fields — `cv_index_entries_active`, `cv_index_topics_active`, `cv_index_overflow_total`, `cv_index_cap_per_topic`. Pure additive (existing fields unchanged). Surfaced in `hub status --governor`, `fleet governor-status` (per-hub + fleet rollup `total_cv_index_entries_active` / `total_cv_index_overflow_total`), and both MCP tools. A non-zero `cv_index_overflow_total` is the smoking gun for "some topic has saturated its per-topic cap and new cv-tagged posts are being silently un-indexed" — usually a producer mis-emitting cv_key (e.g. timestamp instead of stable id). T-2118 closed the deferred predicate gap: `governor_hub_is_pressured` (CLI) and `mcp_governor_hub_is_pressured` (MCP) both now fire when `cv_index_overflow_total > 0` — overflow is binary, ANY non-zero value is operator-actionable (producer fix), no tuning threshold needed. `--only-pressured` / `only_pressured` now surfaces cv_index overflow alongside cap_hits / rate_hits / at-capacity / unreachable. Pre-T-2110 hubs missing the `cv_index_overflow_total` field default to "not pressured" for backward compatibility. T-2119 closed the rest of the cv_index observability loop at the watch/notify/log/history layer: `WatchGovernorState` extended to a 7-tuple with `cv_overflow: Option<i64>` (None for pre-T-2110 hubs); `render_governor_watch_change_line` adds the `cv_overflow=A→B(+delta)` segment (or `n/a` sentinel); `build_governor_notify_env` emits `TERMLINK_GOV_OLD_CV_OVERFLOW` / `_NEW_CV_OVERFLOW` / `_CV_OVERFLOW_DELTA` env vars (empty string when either side missing — `[ -z "$VAR" ]` gate); `build_governor_log_entry` writes `old_cv_overflow` / `new_cv_overflow` / `cv_overflow_delta` JSON fields (null when missing, NOT 0 — jq filters distinguish "no field" from "0 hits"); `render_governor_history_line` shows the cv_overflow segment; `GovernorHubAgg` + `GovernorHubAggMcp` add `cv_overflow_hits` summing `cv_overflow_delta` over window; CLI footer adds `cv_overflow=+N` column; both CLI `--json` and MCP `per_hub` envelopes ship `cv_overflow_hits_total`. Common gate: `[ "$TERMLINK_GOV_CV_OVERFLOW_DELTA" -gt 0 ] || exit 0` then page-the-producer-team. Recipe (page-on-cv-overflow + audit trail): `termlink fleet governor-status --watch 30 --notify /usr/local/bin/page-on-cv-overflow.sh --log ~/.termlink/governor.log`. |
| **Post idempotency (EXACTLY-ONCE)** | **`termlink channel post … --client-msg-id <id>` / hub-side LRU dedupe** | T-2049/T-2023 Gap A substrate primitive #5 — closes the spoke-retry-double-apply window. CLI mints a random 128-bit hex id by default; persisted with the offline-queue row so a flush-replay reuses it. Hub keeps a short-TTL (5min) LRU keyed by `(sender_id, client_msg_id)`; a duplicate post returns the cached `{offset, ts, deduped: true}` envelope without re-appending. Backward compatible — old clients omit the field and behave exactly as before. Sender_id is the T-1427 verified identity fingerprint so the cache namespace is attacker-proof. Watch `dedupe_hits_total` via `hub.governor_status` for retry-absorption telemetry. See `docs/operations/substrate-post-idempotency.md`. |
| **Offline queue (RESILIENCE)** | **`~/.termlink/outbound.sqlite` / `TERMLINK_OUTBOUND_CAP=1000`** | T-2051/T-2018 §6 #5 — operator recipe for the durable FIFO that absorbs hub blips. CLI's `channel post` either delivers directly or enqueues (`PostOutcome::Queued`) — never silent-drops. Background flush task drains every 5s once the hub returns; T-2049 dedupe makes replay safe (exactly-once across blips). R3 loud-fail when full: `QueueError::QueueFull{cap}`. Poison-pill detection at `POISON_THRESHOLD=10` consecutive hub-rejects → drop with `tracing::warn!` + `dropped_poison` counter. Operator surface: `sqlite3 outbound.sqlite` for inspection, `termlink channel post smoke:drain ...` to force a drain pass. Backoff audit + the one open gap (jitter, T-2055) in `docs/reports/T-2050-offline-queue-backoff-audit.md`. End-to-end recipe in `docs/operations/substrate-offline-queue-recipe.md`. T-2083 opened the substrate primitive #5 RESILIENCE observability arc with Slice 1 — `termlink channel queue-status --watch <secs>` continuous queue-depth monitor (clamped [1, 300], mirror of T-2078 / T-2041). Clears screen + re-renders the header (`queue=<path> pending=N oldest_age=Mms watch interval=Ss`) each tick + groundwork diff helper `diff_queue_states` producing `QueueChangeEvent` with `Drained` / `Pending` kinds (binary state — depth changes within still-pending are NOT events, only the `0↔N+` state flip is). Mutex with `--json`. Fetch errors during a tick print `# queue-status watch: read error: <e> (will retry)` to stderr and the loop continues. Pure helpers `QueueSnapshot`, `read_queue_snapshot`, `diff_queue_states` extracted + unit-tested for baseline-no-event + drained↔pending transitions + steady-state-no-event. Recipe: `termlink channel queue-status --watch 5` to watch a live blip. T-2084 added Slice 2 — `--notify <CMD>` event hook fired fire-and-forget per `QueueChangeEvent` (mirror of T-2079 find-idle `--notify` / T-2065 governor `--notify`). Baseline tick skipped. Per-event env vars: `TERMLINK_QUEUE_CHANGE_KIND` (`drained`/`pending`), `TERMLINK_QUEUE_TS` (RFC3339), `TERMLINK_QUEUE_OLD_PENDING`, `TERMLINK_QUEUE_NEW_PENDING`, `TERMLINK_QUEUE_OLDEST_AGE_MS` (numeric or `n/a` when queue drained), `TERMLINK_QUEUE_PATH`. Hanging scripts do NOT block the loop; command-not-found does NOT kill the watch. Requires `--watch`. Pure helper `fire_queue_notify_env` extracted + unit-tested for both `drained` and `pending` event kinds and schema-stability (always 6 vars). Common gate: `[ "$TERMLINK_QUEUE_CHANGE_KIND" = "pending" ] || exit 0` then page Slack/PagerDuty. Recipe (page-on-blip): `termlink channel queue-status --watch 5 --notify /usr/local/bin/page-on-queue-pending.sh` — operator gets paged the second the hub blip causes work to start queuing. T-2085 added Slice 3 — `--log <PATH>` append-only NDJSON audit trail (mirror of T-2080 find-idle `--log` / T-2066 governor `--log`). Schema: `{ts, kind, old_pending, new_pending, oldest_age_ms, queue_path}` — exactly 6 flat jq-friendly fields per `drained↔pending` event (`oldest_age_ms` serializes as JSON `null` on `drained`, NOT the string `"n/a"` — that convention is `--notify`-only). Best-effort writes (parent dir auto-created; disk-full / permission errors print one-line stderr warning + watch never crashes). Symmetric with `--notify` — when both flags are set, each event lands in both surfaces from the same per-tick event source. Requires `--watch` (events only exist across ticks). Forensic retrospective via `jq -c 'select(.kind=="pending")' ~/.termlink/queue.log` — answers "how often does the queue actually back up?" without keeping a watch terminal still attached. Pure helpers `render_queue_log_line` + `append_queue_log_line` extracted + unit-tested for both `pending`/`drained` schemas, JSON-null oldest_age on drained, exact 6-field cardinality lock, parent-dir auto-create, and multi-append correctness. Recipe (real-time page + forensic trail): `termlink channel queue-status --watch 5 --notify /usr/local/bin/page-on-queue-pending.sh --log ~/.termlink/queue.log`. T-2086 added Slice 4 — `termlink channel queue-history [--since DAYS] [--kind pending\|drained] [--log PATH] [--json]` retrospective verb that walks the audit log (default `~/.termlink/queue.log`), filters by window + kind, renders one human-format line per matching entry (`<ts>  <kind>  pending=A→B  oldest_age=Cms\|n/a  queue=<path>`) + per-kind aggregate footer (`pending=N  drained=M`). Default 7-day window, clamped 1..=365. Mirror of T-2074 `channel claims-history` / T-2081 `agent find-idle-history`. Pure read; no auth; no network. Answers "has the queue been backing up?" / "how often does this host blip?" without keeping the watch terminal attached. Pure helpers `parse_queue_log` + `aggregate_queue_entries` + `render_queue_history_line` extracted; malformed lines skip + count in summary; aggregate drops unknown kinds (schema-drift defense). Missing log → hint pointing back at `queue-status --watch --log`. JSON envelope shape: `{ok, entries[], summary{total, pending_events, drained_events, since_days, kind_filter, malformed_lines_skipped, log_path}}`. Queue state is binary (no `transition` kind — see T-2083 design note), so the summary tracks pending/drained event counts only. Recipe: `termlink channel queue-history --since 7 --kind pending` to count how many times the queue has hit backpressure in the last week. T-2087 added Slice 5 — `termlink_channel_queue_history` MCP parity (closure slice). Same params (`since_days` default 7 clamped 1..=365, `kind` exact-match `pending`/`drained`, `log_path` override), same `{ok, entries, summary{total, pending_events, drained_events, since_days, kind_filter, malformed_lines_skipped, log_path}}` shape as the CLI `--json` modulo the optional `hint` field on missing-log. Pure helpers `parse_queue_log_mcp` + `aggregate_queue_entries_mcp` duplicated into termlink-mcp/src/tools.rs (T-2069 convention — no cross-crate sharing for these tiny pure helpers). Read-only file scan; no auth; no network; no log mutation. Use when an agent investigating fleet RESILIENCE needs to answer "how often has this host been queuing?" without shelling out. **CLOSES the substrate primitive #5 RESILIENCE observability arc end-to-end** (write surface: post-or-queue + flush + dedupe + idempotency; read surface: RPC → CLI status → CLI watch → notify → log → history-CLI → history-MCP). Symmetric with substrate primitive #2 DISPATCH (find-idle, T-2078..T-2082 closed earlier in the same session). |
| **Cross-host handoff (SEND)** | **`/agent-handoff <target> <task-id> "<msg>"`** | Skill wrapping `termlink agent contact` — see `.claude/commands/agent-handoff.md` |
| **Ad-hoc reply (SEND, targeted)** | **`/reply <peer-short> "<text>"`** | SEND/RECEIVE-symmetric companion to `/agent-handoff` — `/agent-handoff` opens a thread, `/reply` answers one that already exists (T-1880). Auto-resolves self-fp via PL-195 chain, topic via substring match against `dm:*` names containing both self-fp AND peer-substring (multi-match refuses with candidate list), and `conversation_id` from the topic's latest-cid envelope. Pass `--ensure-cid` to mint `reply-<utc-iso>` when topic has no cid yet. Delegates to `scripts/agent-respond.sh` for the actual receipt+reply transport. Targeted-one-thread complement to `/check-arc respond` (which iterates ALL unread). See `.claude/commands/reply.md` |
| **Pending DM inbox (RECEIVE)** | **`/check-arc`** | Surfaces unread `dm:<self>:<peer>` topics + agent-chat-arc broadcasts — see `.claude/commands/check-arc.md` |
| **Outbound unread (SENT-AUDIT)** | **`/check-outbox [--fleet]`** | OUTBOUND complement of `/check-arc` (T-1891). Walks `dm:<self>:<peer>` topics, computes `outbound_unread = count-1 - max(peer_receipt.up_to)`, surfaces topics where peers haven't acked your posts. Answers "whose mailbox am I filling without them reading?". Read-only. Local-default; `--fleet` walks hubs.toml with TLS-fp dedup (T-1889 sibling). Single-self-fp-resolve avoids per-hub fallback timeout cascade (PL-195/T-1693 shared-host case). Pair with `/agent-handoff` (nudge) or `/peers --all` (is peer LIVE?). Wraps `scripts/check-outbox.sh`. See `.claude/commands/check-outbox.md` |
| **DM history per peer (READ)** | **`/recent-dm <peer> [N [HOURS]]`** | Per-peer conversation history — the asymmetric companion to `/recent-chat` (T-1862). Walks dm:* topics on every hub, substring-matches `<peer>`, dedups federated copies, renders ts/topic/sender/preview. Read-only; defaults to last 20 posts in 24h window. Use after `/check-arc` shows unread DMs and you need thread context before replying. Wraps `scripts/recent-dm.sh`, which parameterizes T-1849's chat-arc-recent engine via `--topic <T>`. See `.claude/commands/recent-dm.md` |
| **Be reachable (PRESENCE)** | **`/be-reachable [start\|stop\|status]`** | Opt this session into agent-presence so peers can `--to <agent_id>` reach you; idempotent lifecycle wrapping `listener-heartbeat.sh` — see `.claude/commands/be-reachable.md` (T-1841) |
| **Peers (LIST)** | **`/peers [--all]`** | List LIVE listeners across every hub — the "who's around to DM?" verb completing the conversation-arc skill set (T-1859). Wraps `scripts/agent-listeners-fleet.sh` with LIVE-default filter; appends per-peer `/agent-handoff` hints; empty-fleet path points at `/be-reachable` + `/broadcast-chat`. Read-only; pair with `/recent-chat` for context before initiating. See `.claude/commands/peers.md`. T-2091 added the capabilities surface — `metadata.capabilities` (advertised by `/be-reachable --capabilities` or `TERMLINK_CAPABILITIES=…` env, also wired into `scripts/listener-heartbeat.sh`) is now ALWAYS included in the per-listener JSON envelope (`{capabilities: "csv,list"}`) — backward compatible (empty string when listener has not advertised any). Text mode opt-in via `--with-capabilities` adds a CAPABILITIES column (default-off to preserve the legacy row width). New `--filter-capability CAP` filter selects only listeners advertising CAP — exact csv-token equality (no substring match — so `--filter-capability deploy` does NOT match `auto-deploy`). Forwards through `scripts/agent-listeners-fleet.sh` to per-hub probes BEFORE the merge, so fleet-wide filtered queries stay payload-cheap. Operator UX win: answers "who can do X?" from the presence rail without shelling out to `termlink agent find-idle`. Reuses the same csv-token split + index pattern as `--filter-listen-topic` for predicate symmetry. |
| **Find idle (DISPATCH)** | **`/find-idle [--role R] [--capability C ...] [--limit N] [--json]`** | DISPATCH counterpart to `/peers` — answers "who's free RIGHT NOW to take work?" by wrapping `termlink agent find-idle` (substrate primitive #2, T-2020/T-2045) at the skill layer (T-2092). LIVE listeners on `agent-presence` MINUS active claims (T-2019 anti-join), pure read, local-hub-only by ADR §6 #2 design. Empty-result path is loud (not silent zero): surfaces "all LIVE listeners busy" / "no LIVE listeners" / "capability filter too narrow" / "no presence on this hub" diagnostic ladder + a `/be-reachable --capabilities …` suggestion so the operator can become idle themselves if nobody else is. Human mode appends per-agent `/agent-handoff <agent_id> <focus-task> "..."` hints. `--json` envelope `{ok, idle: [{agent_id, last_heartbeat_ms, role, capabilities}, ...]}` — pure passthrough from the underlying verb. AND-subset capability semantics (repeat `--capability` for multi-cap match). Pairs: `/peers` for full presence picture, `/agent-handoff` for the handoff after pick, `channel.claim` for exclusive reservation, `/be-reachable --capabilities` for self-advertise. The long-running orchestrator pattern (`--watch` event-loop dispatch) stays at the CLI tier (T-2078..T-2082) because watch loops sit awkwardly inside slash-commands. See `.claude/commands/find-idle.md` + `docs/operations/agent-find-idle.md` master recipe. |
| **Claims (CLAIM-READ)** | **`/claims <topic>\|--all [--only-stuck] [--json]`** | CLAIM-READ daily verb (T-2093) — answers "what's claimed on this topic right now?" / "are any claims stuck?" by wrapping `termlink channel claims-summary` (substrate primitive #1, T-2019/T-2042) at the skill layer. Sibling to `/find-idle`: `/find-idle` answers "who's free?", `/claims` answers "what's already in flight?" — together they form the orchestrator's situational-awareness pair for substrate work-stealing. Local-hub-only by ADR §6 #1 design. `--only-stuck` (T-2076) is a presentation-level filter — JSON envelope still carries fleet-wide `topic_count` + `stuck_count` so the operator sees both "1/N stuck" and the raw subset. "Stuck" heuristic (T-2042): `expired_count > 0` OR `oldest_active_age_ms > 60_000`. Empty-result path is loud (not silent zero): per-mode diagnostic hint (single-topic vs `--all` vs `--all --only-stuck` healthy-fleet path). Human mode appends stuck-claim recovery ladder (`claim-force-release` for Tier-0 bypass, `claim-transfer` for cooperative handoff per T-2046, `claims-summary --watch --notify` for continuous alerting per T-2072). Watch/notify/log/history forms stay at the CLI tier (T-2072..T-2075) because long-running loops sit awkwardly inside slash-commands. See `.claude/commands/claims.md`. |
| **Queue status (RESILIENCE-READ)** | **`/queue-status [--json]`** | RESILIENCE-READ daily verb (T-2094) — answers "is my queue draining or backing up right now?" by wrapping `termlink channel queue-status` (substrate primitive #5, T-2051) at the skill layer. The fast-check after suspecting a host blip. Pure local SQLite read of `~/.termlink/outbound.sqlite` (the durable FIFO that absorbs hub blips per T-2051) — no network, no auth, no state mutation. Empty-result path is loud per-state: `pending=0` (steady-state, "leave a watch running if you suspect a blip"), `pending>0 oldest_age small` (actively draining, wait), `pending>0 oldest_age large` (host blipped, points at `fleet doctor` + `hub status --governor` per T-2048 substrate #10), queue path missing (lazy-init, normal until first absorption). Human mode appends watch/notify/log/history pointer ladder (T-2083..T-2087). Operational reading rules: `pending=0` is steady; growing past `TERMLINK_OUTBOUND_CAP` (default 1000) triggers R3 `QueueFull` loud-fail per T-2051. Sibling to `/find-idle` / `/claims` — three substrate-read daily verbs covering DISPATCH / CLAIM / RESILIENCE. See `.claude/commands/queue-status.md` + `docs/operations/substrate-offline-queue-recipe.md` master recipe. |
| **Governor (BACKPRESSURE-READ)** | **`/governor [--only-pressured] [--json]`** | BACKPRESSURE-READ daily verb (T-2095) — answers "is the hub being rate-limited or at-capacity right now?" by wrapping `termlink fleet governor-status` (substrate primitive #10, T-2048/T-2060/T-2062/T-2070) at the skill layer. **Completes the substrate-read daily-verb quad** alongside `/find-idle` (#2), `/claims` (#1), `/queue-status` (#5). Pure Observe-scope read of `hub.governor_status` JSON-RPC per hub in `hubs.toml` — no auth side-effects, no state mutation. Per-hub block + fleet-wide footer (`hubs_at_capacity` / `hubs_rate_limited` totals). `--only-pressured` (T-2070) filters to hubs needing attention (unreachable, at-capacity, capacity_hits > 0, rate_hits > 0); healthy-fleet path under filter prints `All hubs healthy (0/N pressured)` — affirmative confirmation. Operational reading: `capacity_hits_total > 0` → connection refused (tune `TERMLINK_MAX_CONNECTIONS`); `rate_hits_total > 0` → RPC refused (tune `TERMLINK_RATE_LIMIT_PER_SEC`); `dedupe_hits_total > 0` → spoke retries absorbed (this is **good**, exactly-once working per T-2049). Empty-result loud per-state: steady-state-zero (hint), all-unreachable (points at `fleet doctor` + `fleet verify`). Watch/notify/log/history forms stay at CLI tier (T-2064..T-2069) — same design rationale as siblings. See `.claude/commands/governor.md` + `docs/operations/substrate-governor.md` master recipe. |
| **CV-keys (BROADCAST-WITH-REPLAY INSPECTION)** | **`/cv-keys <topic> [--hub <addr>] [--json]`** | BROADCAST-WITH-REPLAY inspection daily verb (T-2121) — answers "which cv_keys are currently advertising on this topic, and at what offsets?" by wrapping `termlink channel cv-keys` (substrate primitive #9 inspection, T-2106) at the skill layer. **Natural follow-up to `/governor` when `cv_overflow > 0` fires:** /governor detects the overflow, /cv-keys identifies which keys are on the saturating topic, the operator fixes the producer that is mis-emitting `metadata.cv_key`. Pure Observe-scope read of `channel.cv_keys` JSON-RPC — no auth side-effects, no state mutation. Local-hub-default by ADR §6 #9 design (cv_index is per-hub state per G-060 — no fleet-wide form would aggregate meaningfully). Empty-result path is loud per-state: emits the verb's `no cv_keys recorded on topic` line, then appends a diagnostic ladder distinguishing healthy "broadcast-only topic" from misconfigured-producer cases. Cross-references the cv_overflow observability arc end-to-end (T-2110 telemetry → T-2118 predicate → T-2119 watch/notify/log/history → T-2120 docs → /cv-keys diagnosis). Common pattern: `/governor --only-pressured` → identify pressured hub → `/cv-keys <suspect-topic> --hub <addr>` → check `count` vs `TERMLINK_CV_INDEX_CAP_PER_TOPIC` (default 1000); near-cap = producer bug. Highest-value default invocation: `/cv-keys agent-presence` (one entry per LIVE agent per T-2107 listener-heartbeat wiring). See `.claude/commands/cv-keys.md` + `docs/operations/substrate-broadcast-with-replay.md` master recipe. |
| **Broadcast-with-replay (BROADCAST-READ)** | **`termlink channel cv-keys <TOPIC> [--json]` / `termlink channel subscribe <TOPIC> --include-current-value` / `termlink_channel_cv_keys` MCP / `termlink_channel_subscribe.include_current_value` MCP** | T-2027/T-2089 substrate primitive #9 — late-joiners read current-state-per-cv_key in O(K) instead of replaying the full event log. T-2103 added the hub-side `cv_index: HashMap<topic, HashMap<cv_key, offset>>` recorded on every post carrying `metadata.cv_key` (last-write-wins, per-topic cap 1000 via `TERMLINK_CV_INDEX_CAP_PER_TOPIC`). T-2104 wired `channel.subscribe` to accept `include_current_value: bool` and respond with `current_values: [{cv_key, offset, msg}, ...]` inline before the regular stream. T-2105 surfaced `--include-current-value` at the CLI + MCP tiers (snapshot is one-shot — sent on first hub call only). T-2106 added the read-only inspection verb `channel.cv_keys` / `termlink channel cv-keys <TOPIC>` / `termlink_channel_cv_keys` answering "what cv_keys are advertising on this topic and at what offsets?" without forcing the operator to subscribe. T-2107 wired `metadata.cv_key=$agent_id` into `listener-heartbeat.sh` as the highest-value producer — every `/be-reachable` heartbeat now populates cv_index, so `channel cv-keys agent-presence` returns one entry per agent (vs O(N_heartbeats) walk). Discovery cost: 5-agent fleet × 30s × 24h drops from ~14,400 envelopes to 5 entries per query. `--no-cv-key` opt-out for tests/migration. Empty cv_index is NOT an error (healthy state). cap-overflow: post stays atomic, only annotation drops (loud-refuse via internal counter). cv_index is in-memory only — process-local to the hub, cleared on restart, repopulated within one heartbeat cycle. See `docs/operations/substrate-broadcast-with-replay.md`. |
| **Preflight (DEPLOY-TIME)** | **`/preflight [--json]`** | Deploy-time substrate correctness verb (T-2158) — wraps `scripts/substrate-preflight.sh` (T-2154) at the skill layer. Answers "before I trust this substrate, is the environment actually set up right?" — the load-bearing precondition under every runtime-read verb. Six checks: (1) `TERMLINK_RUNTIME_DIR` NOT on /tmp [HIGH — PL-021 prevention; detects BOTH tmpfs mount AND systemd-tmpfiles D-rule wipe], (2) `~/.termlink/hubs.toml` present + has `[hubs.*]` sections [MEDIUM — every heal path / fleet verb depends on it], (3) `~/.termlink/be-reachable.state` PID alive [MEDIUM — "I forgot to /be-reachable again after reboot" footgun], (4) **`termlink --version` >= project root `VERSION`** [MEDIUM — T-2181: catches stale-binary footgun where catalog promises flags like `--only-stuck` (T-2076) or subcommands like `fleet governor-status` (T-2062) that an older binary refuses with `unknown flag` / `unrecognized subcommand`. WARN, not FAIL — substrate still works for primitives the binary has. Skipped silently when run outside the project tree (no VERSION file). Remediation: `cargo build --release && install -m 755 target/release/termlink ~/.cargo/bin/`], (5) **local hub serves T-2139 field** [MEDIUM — T-2184: symmetric companion to Check 4. Probes running hub via `termlink hub status --governor --json` and tests for `rate_buckets_evicted_total` field presence. Absence ⇒ pre-T-2139 hub binary (typically: operator ran `cargo install` but never restarted hub — `/proc/<pid>/exe` shows `...(deleted)`, in-memory binary keeps serving old envelopes). WARN, not FAIL — substrate still works for primitives the hub binary has. Skipped when hub is down (different failure mode — Check 1 territory). Remediation: restart hub to pick up new binary; verify runtime_dir persists secret/cert per Check 1 first. Origin: PL-209 spent ~30min chasing "missing telemetry" that was actually a missing restart], (6) **systemd unit health + detached-ghost detection** [MEDIUM — T-2358, G-070 prevention: WARNs when termlink-hub.service is crash-looping/failed, when the pidfile PID is alive but differs from the unit MainPID (detached ghost serving outside supervision), when a hub runs with the unit inactive (no crash-restart / reboot-survival), or when NRestarts > `TERMLINK_PREFLIGHT_NRESTARTS_MAX` (default 5 — flap residue persists until acknowledged via `systemctl reset-failed termlink-hub`). Origin: G-070, unit crash-looped 2178 times "Hub is already running" while a detached hub held the pidfile and every other surface stayed green. Skips silently on non-systemd / watchdog-launched hosts]. Exit codes: 0 PASS, 1 WARN, 2 FAIL. Read-only, no network, no auth, no state mutation — safe anywhere. **Distinct from `/substrate` (runtime digest), `/self-test` (framework E2E), `fw doctor` (framework health)** — four verbs answer four distinct operational questions; conflating them is how a hub silently regenerates its secret every reboot for 14 days before anyone notices (PL-021 / G-058 class). Contextual Step-5 next-step hints per failed check (runtime_dir → `docs/operations/termlink-hub-runtime-migration.md` + CLAUDE.md §"Special case — volatile runtime_dir"; hubs.toml → `termlink remote profile add`; be-reachable → `/be-reachable start`). PASS path nudges operator to runtime layer: `/substrate` + `/peers --all`. Cold-start sequence (first 5 minutes on a new host): `/preflight` → `/be-reachable start` → `/substrate` → `/peers --all`. See `.claude/commands/preflight.md` + `docs/operations/substrate-getting-started.md`. |
| **Substrate digest (SUBSTRATE-PULSE)** | **`/substrate [--json]`** | Substrate cold-start digest (T-2096) — answers "is the substrate healthy and what's it doing right now?" by composing the four substrate-read daily verbs (`/find-idle` + `/claims --all --only-stuck` + `/queue-status` + `/governor --only-pressured`) in parallel into one unified four-section view. **Pattern parity with T-1860 `/pulse`** (which composes peers + recent-chat for the conversation arc) — same design, different domain. Read-only by composition, no auth side-effects, no `AskUserQuestion`. Parallel-by-default — total latency = max(four reads), not sum-of-four. Graceful degradation: a failed sub-query renders as one stderr line, not a hard stop (per-section `ok:false` in JSON mode, not silent drops). Substrate-healthy path is affirmative: "substrate steady-state: dispatch=0 idle, 0 stuck claims, queue drained, 0 hubs pressured" + pointer at `/peers --all` and `/pulse` if you expected busier. Contextual Step-5 hints tuned to which sub-section flagged (queue-pending → `/queue-status`, hub-pressured → `/governor`, stuck-claims → `/claims --all --only-stuck` + recovery options). Cold-start pairing: `/pulse` (conversation) + `/substrate` (substrate) = two-keystroke full operational picture across both domains. See `.claude/commands/substrate.md`. |
| **Canaries (CRON-TIER VISIBILITY)** | **`/canaries [--json] [--quiet] [--max-age-hours N]`** | Cron-tier protection visibility verb (T-2172) — wraps `scripts/canary-status.sh` at the skill layer. Answers "are my cron canaries firing AND clean?" by auto-discovering every `.context/working/.*-canary.log` (no hard-coded list — new canaries appear the first time their log is written) and pairing each with its `.heartbeat` companion. Per-canary classification: `HEALTHY` (log empty AND heartbeat fresh), `FIRING` (log has entries newer than heartbeat — cron is finding real problems), `STALE` (heartbeat older than threshold, default 48h — cron may have stopped firing), `NO_HEARTBEAT` (log present but no `.heartbeat` companion — classified by log content alone). Exit codes: 0 = all healthy, 1 = any FIRING/STALE (operator action required), 2 = tooling error. **Signal-bearing line surfacing (T-2180):** FIRING entries render the most-recent log line matching `fail|drift|stale|warn|error|behind` (case-insensitive, last 50 lines) rather than naive `tail -n 1`. For multi-line canary entries (typical fleet-doorbell-mail shape: `=== <ts> ===\\n<verdict line>\\n---`), `tail -n 1` returns the trailing `verdict=pass` separator/footer and makes a FIRING canary look healthy at a glance; the signal-bearing heuristic extracts the actionable mid-log line (e.g. `↳ laptop-141@192.168.10.141:9100: verdict=setup-fail elapsed=8004ms`) instead. Falls back to first non-separator line when no signal keyword matches. Closes the silent-misread failure mode where operators saw `verdict=pass` in the /canaries output and assumed nothing was wrong. `--quiet` renders only problems (cron-friendly, mirror of `check-canary-aliveness.sh` convention); `--json` emits `{ok, summary, canaries[]}` jq-friendly envelope; `--max-age-hours N` tunes the stale threshold (T-1723 meta-canary convention) for non-daily cadences. **Closes PL-168** (canary scripts without an operator-facing trigger are dormant tooling) for the cron-tier layer — the substrate-arc safety set wired N canaries (T-2160 substrate-preflight, T-1696 release-mirror, T-1723 meta-canary-aliveness, fleet-doorbell-mail, ...) but operators had no canonical verb to read them. Substrate-arc framing: completes the safety set visibility tier (CLI/T-2154 preflight → skill/T-2158 → smoke/T-2170 → cron/T-2160 → THIS). **Cold-start three-verb sequence:** `/preflight` (deploy-time) → `/substrate` (runtime) → `/canaries` (cron-tier protection) — three orthogonal questions, three orthogonal answers; pair them at session start when picking up a host. Read-only by contract; never heals. See `.claude/commands/canaries.md` + `docs/operations/substrate-cron-recipes.md` § "Checking that the canaries are firing". |
| **AEF integration master recipe (T-2018 §9 DOC CLOSURE)** | **`docs/operations/substrate-orchestrator-recipe.md`** | Master integration walkthrough (T-2124) — end-to-end work-stealing pattern combining every shipped substrate primitive: find-idle (#2) + claim (#1) + claim-transfer (#3) + renew (#1) + release (#1) + outbound-queue (#5) + post-idempotency (#5) + cv_index (#9) + governor (#10) + substrate-status (#11). The doc an AEF integration developer reads first when wiring a parallel-worker orchestrator on top of TermLink. Contains: mental model (orchestrator + N workers + shared substrate), the contract (which RPCs the AEF layer depends on, read + write surfaces), canonical orchestrator pattern (5-step shell walkthrough — `find-idle → claim → claim-transfer → contact` with race-correctness rationale per step), canonical worker pattern (heartbeat → poll DM → verify claim ownership → background-renew loop → release ack vs ack=false), failure-modes table (10 symptoms × diagnosis × recovery covering CLAIM_CONFLICT / CLAIM_NOT_OWNED / CLAIM_NOT_FOUND / RATE_LIMITED / HUB_AT_CAPACITY / queue-buffering / cv_overflow / etc.), observability hooks (which read-side verb answers which operational question), cross-hub limits (G-060 — one orchestrator per hub by substrate design, workers belong to one hub), AEF integration checklist (10 wiring items operators ratchet through), and a worked "5-unit queue across 2 workers" walkthrough. Cross-referenced from every per-primitive ops doc (substrate-claim-primitive / substrate-broadcast-with-replay / substrate-offline-queue-recipe / substrate-governor / substrate-post-idempotency / agent-find-idle) and from ADR §9 "Hard dependencies" paragraph. Closes the T-2018 §9 collaboration-seam consumer-facing doc gap that was fragmented across two per-primitive sections (substrate-claim-primitive.md § "Hand a unit to a specific worker" + agent-find-idle.md § "minimum viable orchestrator loop"). T-2125 added the "Recommended retention settings" section — per-topic-pattern table mapping `agent-presence` / `agent-chat-arc` / `agent-listeners-*` / `agent-conv-*` / `dm:*` / work-topics / audit-topics to recommended `Retention` (e.g. `Messages(1000)` for `agent-presence`, `Forever` for framework audit logs) with rationale tied to T-1991 (production agent-presence bloat) and the T-2058 hub-side `is_high_rate_pattern` loud-warn that operators don't always see. Concrete `termlink channel create --retention messages --retention-value N` examples per pattern. The operator-facing complement to T-2058: the hub warns at create time; the recipe doc tells the operator what to set BEFORE deploying agents. Code-level follow-up (auto-pick high-rate retention in CLI's `ensure_topic` helper) logged separately. |
| **Claim (CLAIM-WRITE)** | **`/claim <topic> <offset> [--claimer <id>] [--ttl-ms N] [--json]`** | CLAIM-WRITE daily verb (T-2097, T-2100-fixed) — reserves a specific offset on a topic by wrapping `termlink channel claim` (substrate primitive #1, T-2029/T-2032). WRITE-side counterpart to T-2093 `/claims` — together complete the substrate #1 daily-verb surface across both directions. **Substrate claim model is offset-based.** A claim is exclusive ownership of (topic, offset) for `ttl_ms` (default 30000ms, hub-clamped to 1h max). There is NO `unit-id` to auto-mint — the operator must pick an offset (typically from `termlink channel subscribe <topic>` or `/claims <topic>` to avoid contested ones). **This skill WRITES state.** Auto-resolves `--claimer` from `$TERMLINK_AGENT_ID` env then `~/.termlink/be-reachable.state` (T-1857 sender-resolution chain, T-1841 identity source). Refuses with hint when claimer unresolved — never invents a claimer (claim accountability invariant). **Loud refusal taxonomy** for known error classes: CLAIM_CONFLICT → suggest `/claims <topic>` to see who; AUTH_FAIL → suggest `termlink fleet reauth` + `fleet doctor`; RATE_LIMITED (-32008) → suggest `/governor`; HUB_AT_CAPACITY (-32019) → `/governor`; unknown errors pass through verbatim. Success render shows topic + offset + claim_id + auto-resolved claimer + claimed_until + next-step hints (`/release` for done, `/release --retry` for return-for-retry, `/claim-transfer` for handoff). Don't auto-retry transient errors — surface and let operator decide. Natural orchestrator chain: `/find-idle --capability X` → `/claim <topic> <offset>` → `/claim-transfer <claim-id> <worker>` (T-2099 cooperative handoff). See `.claude/commands/claim.md`. |
| **Renew (CLAIM-EXTEND)** | **`/renew <claim-id> [--by-ms N] [--claimer <id>] [--json]`** | CLAIM-EXTEND daily verb (T-2101) — extends an active claim's lease by wrapping `termlink channel renew` (substrate primitive #1 lifecycle, T-2030/T-2032). **Closes the substrate #1 lifecycle surface end-to-end** at the skill tier: claim → renew → release / claim-transfer. The substrate's pressure-relief verb — claims default to 30s lease (substrate is biased toward short work units). For longer work, renew before lapse OR the slot reopens to another worker mid-task (substrate-correctness footgun for any non-trivial unit). **This skill WRITES state.** Operator-friendly `--by-ms` alias for the underlying awkward `--additional-ttl-ms` flag (skill UX is the friction-reduction layer); operator never sees the awkward CLI name. Default extension: +30000ms (matches CLI default). Auto-resolves `--claimer` via T-1857 chain (env → `~/.termlink/be-reachable.state` → refuse). **Loud refusal taxonomy:** CLAIM_NOT_FOUND → `/claims <topic>` + `claims-history` (T-2074); CLAIM_NOT_OWNED → see actual holder via `/claims`; **CLAIM_NOT_FOUND → the failure mode renew exists to prevent. Slot has reopened — must re-claim (`/claim <topic> <offset>`) under a new claim_id, not retry renew. Next time, renew SOONER (well before claimed_until)**; AUTH_FAIL → `fleet reauth` + `fleet doctor`; RATE_LIMITED → `/governor` + WARNING that lease ticks down while waiting (may need to re-claim). Success render shows claim_id + offset + added_ms + new_claimed_until + auto-resolved claimer + "plan next renew BEFORE that" hint. **Not in scope:** auto-renew daemon (operator wires watch loop themselves), absolute-timestamp renewal (CLI is additive only), bulk renew. Authored from `termlink channel renew --help` per PL-206 (T-2100 fix-up established the rule). Natural long-work pattern: `/claim <topic> <offset>` → `/renew <claim-id> --by-ms N` (repeat as needed) → `/release <claim-id>`. See `.claude/commands/renew.md`. |
| **Release (CLAIM-RELEASE)** | **`/release <claim-id> [--retry] [--claimer <id>] [--json]`** | CLAIM-RELEASE daily verb (T-2098, T-2100-fixed) — releases a claim by id by wrapping `termlink channel release` (substrate primitive #1, T-2029/T-2032). RELEASE-side counterpart to T-2097 `/claim` — together complete the substrate #1 WRITE-side lifecycle (acquire → complete/retry) at the skill tier. **This skill WRITES state.** **Critical substrate semantic — `--ack` controls cursor advancement.** The underlying CLI release verb has an `--ack` flag: WITH `--ack`, the claimer's persisted cursor advances past the offset (work completed). WITHOUT `--ack`, the slot reopens for the next worker with no cursor advance (work returned for retry). This skill encodes the "done" heuristic by ALWAYS adding `--ack` unless the operator passes `--retry`, which negates it. The asymmetric default ("done by default, retry by opt-in") matches the >90% pattern AND prevents silent retries (a substrate-correctness footgun if the default were reversed). Auto-resolves `--claimer` identity via T-1857 sender-resolution chain (env → `~/.termlink/be-reachable.state` → refuse). Refuses with hint when claimer unresolved — never invents a claimer (the hub would refuse via CLAIM_NOT_OWNED anyway; refusing client-side surfaces the failure mode loudly). **Loud refusal taxonomy:** CLAIM_NOT_FOUND → `/claims <topic>` + `claims-history` (T-2074); CLAIM_NOT_OWNED → `/claims <topic>` to see actual holder, `/claim-transfer` (T-2099) cooperative path, or `claim-force-release` (Tier-0 last resort); AUTH_FAIL → `fleet reauth` + `fleet doctor`; RATE_LIMITED → `/governor`; unknown errors pass through verbatim. Success render shows claim_id + topic + offset + ack=true/false + auto-resolved claimer (observable) + released_at + a one-line summary of "work marked completed" vs "slot reopened — next worker gets this offset". Don't auto-retry transient errors — surface and let operator decide. **Not in scope:** bulk release (`--all`/`--topic`), force release (Tier-0 only), auto-release on session end (leases expire naturally per T-2042). Natural lifecycle: `/claim <topic> <offset>` → do work → `/release <claim-id>` (done) or `/release <claim-id> --retry` (return-for-retry). See `.claude/commands/release.md`. |
| **Claim-transfer (HANDOFF)** | **`/claim-transfer <claim-id> <to-owner> [--by <id>] [--reason "..."] [--json]`** | COOPERATIVE-HANDOFF daily verb (T-2099) — atomically hands a claim's ownership to a different agent by wrapping `termlink channel claim-transfer` (substrate primitive #3, T-2046). **Closes the substrate #1 daily-verb surface end-to-end** alongside `/claims` (read), `/claim` (acquire), `/release` (release). **This skill WRITES state.** The orchestrator-to-worker handoff verb — atomic at the hub layer (T-2046 guarantee: lease moves from `--by` to `--to-owner` with zero gap), eliminating the release-then-claim race window. Positional UX: skill takes `<claim-id> <to-owner>` positionally instead of the CLI's three long `--flag VALUE` pairs. Auto-resolves `--by` identity via T-1857 chain (env → `~/.termlink/be-reachable.state` → refuse). Refuses with hint when identity unresolved — never invents an owner (hub would refuse via CLAIM_NOT_OWNED -32017 anyway; refusing client-side surfaces the failure mode loudly). **Loud refusal taxonomy** for known error classes: CLAIM_NOT_FOUND → `/claims <topic>` + `claims-history` (T-2074); CLAIM_NOT_OWNED → `/claims <topic>` to see actual holder + cooperative `claim-transfer` from holder OR Tier-0 `claim-force-release` (last resort, logs override); AUTH_FAIL → `fleet reauth` + `fleet doctor`; RATE_LIMITED → `/governor`; unknown errors pass through verbatim. **Distinction from `claim-force-release`:** claim-transfer is COOPERATIVE — current holder volunteers the handoff (preserves claim accountability in audit log). claim-force-release is Tier-0 OPERATOR BYPASS — overrides ownership without holder cooperation. The two verbs are intentionally distinct; conflating them undermines accountability. **Not in scope:** force transfer (use Tier-0 force-release + re-claim), bulk transfer (each hop should be deliberate), transfer to self (pointless). Natural orchestrator chain: `/find-idle --capability X` → `/claim <topic> <T-XXX>` → `/claim-transfer <claim-id> <worker>` → worker does work + `/release`. See `.claude/commands/claim-transfer.md`. |
| **Recent chat (CONTEXT)** | **`/recent-chat [N [HOURS]]`** | Fleet-wide recent agent-chat-arc posts — the "what's been said?" verb (T-1849/T-1851). Read-only; pairs with SEND/RECEIVE/PRESENCE skills. See `.claude/commands/recent-chat.md` |
| **Broadcast (BROADCAST)** | **`/broadcast-chat <text>`** | Fan a chat-arc post to every hub in the fleet — the "tell everyone" verb (T-1856/T-1857). Wraps `scripts/chat-arc-broadcast.sh`: walks hubs.toml, per-hub `timeout 8` (PL-189), auto-attributes via `metadata.agent_id` from `/be-reachable` identity (PL-191). G-060 mitigation. WRITES state — pair with `/recent-chat` for context first. See `.claude/commands/broadcast-chat.md` |
| **Pulse (DIGEST)** | **`/pulse [N [HOURS]]`** | Single-shot conversation arc digest — runs `/peers` + `/recent-chat` in parallel and renders a two-section view (T-1860). The "what's happening on the rail right now?" cold-start verb. Read-only; degrades gracefully if one side fails; cold-rail path points at `/be-reachable` + `/broadcast-chat`. See `.claude/commands/pulse.md` |
| **Conversation index (THREADS)** | **`/conversations <topic>`** | List active doorbell+mail threads (distinct `metadata.conversation_id` values) on a topic — the orchestrator-view companion for supervising N concurrent threads (T-1864). Wraps `scripts/agent-conversation-list.sh` (T-1827); read-only. Use on `dm:*` topics to see distinct cids (task threads); chat-arc returns 0 cids by design (broadcast-style). See `.claude/commands/conversations.md` |
| **Auto-restart** | **`claude-fw [args...]`** | Wrapper: runs claude, auto-restarts on handover signal |

## Auto-Restart (T-179)

When context budget hits critical, `checkpoint.sh` auto-generates a handover and writes `.context/working/.restart-requested`. If the user started their session via `claude-fw` (instead of `claude`), the wrapper detects this signal on exit and auto-restarts with `claude -c`. The `SessionStart:resume` hook then injects handover context into the fresh session.

**Flow:** Budget critical → auto-handover → signal file → claude exits → wrapper detects → `sleep 3` → `claude -c` → context injected → `/resume` ready.

**Safety:** 5-minute TTL on signal files, max 5 consecutive restarts, 3-second cancel window, opt-out via `--no-restart`.

## Session End Protocol

**Before ending any session:**
1. Run session capture checklist (`agents/session-capture/AGENT.md`)
2. Create tasks for all uncaptured work
3. Update practices with learnings
4. Generate handover: `fw handover`
5. Fill in the [TODO] sections in the handover document
6. Commit all changes with task references
7. Run `fw metrics` to verify state
8. If you ran `/be-reachable` at session start, stop it: `/be-reachable stop` (the heartbeat is detached via `nohup setsid` so it will otherwise outlive the session)

**Do not end a session without generating a handover.**
