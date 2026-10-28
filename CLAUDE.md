# CLAUDE.md

Claude Code integration for the Agentic Engineering Framework.
For the provider-neutral framework guide, see `.agentic-framework/FRAMEWORK.md`
(vendored — this project uses the vendored path convention throughout, never
a bare `FRAMEWORK.md` at project root, which does not exist here).

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

**Per-guard documentation is in `docs/operations/guard-layer-and-canaries.md`** (moved by T-3246;
plain path on purpose, an `@` import would be inlined at launch, T-3244). Open it before you change,
silence, allowlist or schedule a canary or static check, when one fires or errors, when you edit
`scripts/run-guard-layer.sh` or a `.context/cron/*.crontab`, and before any re-vendor of the framework.

Rules from those sections that apply to everyday work:

1. In a `## Verification` block, redirect to a file and grep the file (`cmd > .context/working/.out 2>&1 && grep -q PAT .context/working/.out`); never `cmd | grep -q PAT` (exits 141 under pipefail when it matches, L-387).
2. Before proposing or accepting a framework re-vendor, run `bash scripts/check-vendor-divergence.sh` and follow the checklist at the top of `.vendor-divergence.yaml`.
3. Any cron job that runs a vendored `agents/` script must set `PROJECT_ROOT="$(pwd)"` (T-2810); do not remove it from `.context/cron-registry.yaml`.
4. Canary crontabs send stderr to `<log>.stderr`, never into the findings log and never to `/dev/null` (T-2685).
5. Never write a test that drives the real hub `supervisor::run` loop; use `supervisor::run_with` and a temp dir (T-3293 reaped ~10k real files).
6. Exit codes everywhere: 0 healthy, 1 firing, 2 tooling (could not look); a tooling error is never reported as clean.

Index (one line per section in the doc; the script named is where to start):

- Mirror drift canary (T-1140 + T-1696, G-058 prevention) — `scripts/check-mirror-freshness.sh`
- Substrate preflight canary (T-2154 + T-2158 + T-2160) — `scripts/substrate-preflight.sh`
- Framework-pickup canary (T-2231, G-063 prevention) — `scripts/check-framework-pickup-freshness.sh`
- Frozen-husk canary (T-2239, G-019 prevention for T-2230/T-2235) — `scripts/check-frozen-husk-freshness.sh`
- Topic-growth canary (T-2252, arc-002 R2 sweep-cron guard) — `scripts/check-topic-growth-freshness.sh`
- Task-finalization canary (T-2290, G-066 prevention) — `scripts/check-task-finalization-freshness.sh`
- Unconfirmed-delivery canary (T-2295, arc-003 reliable-comms V3b, G-063 prevention) — `scripts/check-unconfirmed-delivery-freshness.sh`
- Fleet binary-freshness canary (T-2359, G-069 prevention) — `scripts/check-fleet-binary-freshness.sh`
- Waker-liveness canary (T-2387, G-069 shipped≠live guard for the comms rail) — `scripts/check-waker-liveness-freshness.sh`
- Stale-waker-code canary (T-2405, G-019 detection for T-2404) — `scripts/check-stale-waker-code-freshness.sh`
- Fleet capability-freshness canary (T-2415, G-084 prevention) — `scripts/check-fleet-capability-freshness.sh`
- Charter-drift canary (T-2483, G-019 breadth-accretion prevention) — `scripts/check-charter-drift-freshness.sh`
- Charter canonical-sentence drift canary (T-2484, G-019 charter-fork prevention) — `scripts/check-charter-sentence-drift.sh`
- Stuck-claims canary (T-2556, verb-3 "claim work" daily-detection gap) — `scripts/check-stuck-claims-freshness.sh`
- Session-control canary (T-2557, verb-4 "control terminal sessions" daily-detection gap) — `scripts/check-session-control-freshness.sh`
- Dead-letter canary (T-2558, G-063 class — durable poison-drop sink) — `scripts/check-dead-letter-freshness.sh`
- Forever-topic archival-growth canary (T-2562, charter non-goal #2 guard) — `scripts/check-forever-archival-freshness.sh`
- Substrate-smoke canary (T-2696, verb-3 end-to-end composition — the un-executed prover) — `scripts/substrate-smoke.sh`
- Release-publication canary (T-3243, G-069 at the release layer) — `scripts/check-release-publication-freshness.sh`
- Arc-claim-drift canary (T-3288, G-019 for T-3066 — the check that could never run) — `scripts/check-arc-claim-drift.sh`
- Session-leak canary (T-3296, T-3291 S4 — G-019 for the session lifecycle) — `scripts/check-session-leak-freshness.sh`
- Cron-install-drift check (T-2561, shipped≠live / G-069 for the canary layer) — `scripts/check-cron-install-drift.sh`
- Unclamped caller-param → allocation-sink static check (T-2527, G-019 prevention for the T-2523/T-2526 class) — `scripts/check-alloc-sink-clamps.sh`
- Unbounded peer-driven drain-sink static check (T-2531, G-019 prevention for the T-2518/2524/2525/2529 class) — `scripts/check-drain-sink-caps.sh`
- Silent non-zero-exit static check (T-2666, G-019 prevention for the T-2663 class) — `scripts/check-silent-exit.sh`
- Long-poll busy-spin static check (T-2672, G-019 prevention for the T-2670/T-2671 class) — `scripts/check-busy-spin.sh`
- Verification-block pipefail auditor (T-2818, L-387 at repo scale) — `scripts/check-verification-pipefail.sh`
- Stranded auto-deferred pickup envelope check (T-2801, G-063 for the queue) — `scripts/check-pickup-deferred-freshness.sh`
- Episodic-store readability check (T-2805, G-063 for the memory layer) — `scripts/check-episodic-parse.sh`
- Local fixes to vendored code are deleted by the next re-vendor (T-2812, G-062) — `scripts/check-vendor-divergence.sh`
- `revisit_at` reminders only fire if the cron sets PROJECT_ROOT (T-2810, G-053)
- BVP estimator — safe to run, with one caveat that is about ruamel, not corruption (T-2809)
- Cross-branch task-ID collision + duplicate-work check (T-2800) — `scripts/check-task-id-collisions.sh`
- Vendored-framework recoverability check (T-2814 + T-2817) — `scripts/check-framework-tracking-drift.sh`
- Platform-lock static check (T-2693, Directive #4 Portability) — `scripts/check-platform-lock.sh`
- Error-code emission check (T-2699, Directive #2 — a refusal is a claim) — `scripts/check-error-code-emission.sh`
- Version-derivation static check (T-2746, G-019 prevention for the T-1458 / T-2744 class) — `scripts/check-version-derivation.sh`
- MCP/CLI parity census check (T-2747, herdr rank 13 — coverage, not execution) — `scripts/check-mcp-parity-census.sh`
- Release-artifact drift check (T-2751, the install path nothing guarded) — `scripts/check-release-artifact-drift.sh`
- Verification-block misfile check (T-2831, G-019 prevention for the P-011 vacuous pass) — `scripts/check-verification-misfile.sh`
- Stranded-finalized task check (T-2833, the finalize latch the T-2290 canary cannot see) — `scripts/check-stranded-finalized-tasks.sh`
- Human-AC Steps heading check (T-2859, the approval page dropping the operator's command) — `scripts/check-human-ac-steps-heading.sh`
- Running the guard layer — `scripts/run-guard-layer.sh` (T-2684): severity tiers, the 14-day WARN clock, the pending-commit manifest
- Canary log hygiene — split the streams (T-2685) — `scripts/check-canary-log-hygiene.sh`
- The stderr sink had no reader — ERRORING (T-2842) + completion heartbeats (T-2843) — `scripts/canary-status.sh`

## Project-Specific Rules

### TermLink Substrate + Skills Reference (clobber-safe — T-2015)

**Why this lives here and not in the governance sections below.**
`fw upgrade` rebuilds CLAUDE.md as *(everything above `## Core Principle`)* +
*(the framework template from `## Core Principle` onward)*. Anything we add
**inside** the governance region is therefore destroyed on every upgrade —
`lib/upgrade.sh` says so directly: the wholesale replace "cannot preserve
project-specific INLINE additions inside governance sections". It warns, prints
a few of the lost lines, and proceeds.

On 2026-08-25 an upgrade silently dropped 40 lines this way, including this
entire table. This section is above the split, so it survives verbatim.
**Add project-specific rules HERE, never below `## Core Principle`.**

**The per-capability reference (substrate primitives and the skills that wrap them) is in `docs/operations/termlink-skills-reference.md`** (moved by T-3245; plain path on purpose, an `@` import would be inlined at launch, T-3244). Open it when you need a verb's flags, env vars, JSON shape or the task that built it, or when choosing between skills such as `/find-idle`, `/claims`, `/queue-status`, `/governor`, `/cv-keys`, `/preflight`, `/substrate`, `/canaries`, `/agent-handoff`, `/reply`, `/check-arc`, `/check-outbox`, `/recent-dm`, `/be-reachable`, `/peers`, `/claim`, `/renew`, `/release`, `/claim-transfer`, `/recent-chat`, `/broadcast-chat`, `/pulse`, `/conversations`. Each skill's own definition is `.claude/commands/<name>.md`.

#### Session rules the governance template does not carry

These are restated here because the template's own numbered lists renumber
and overwrite them. Where the template and this section disagree about a
**consumer project**, this section wins.

- **Every command the operator must run ships in ONE runnable script, named by its full path (operator standing instruction, 2026-09-30, T-3272).** Never hand the operator copy-paste command lines, `for` loops or `&&` chains, not even a single line. Add the action to `/opt/termlink/runme.sh` (T-3052) and tell the operator to run exactly `/opt/termlink/runme.sh`, always printed as that absolute path. The operator should not need flags; `--dry-run` is only an optional preview. Each action must be **idempotent** (it checks and skips when already done), must **verify itself on disk** after running rather than trusting an exit code, and must count as FAILED (exit 1) when that verification fails. An action is removed once it is permanently done. Human-authority items run in a default run only when the decision is **already made and recorded** (e.g. the T-3211 SQ-3 closures); an undecided item stays listed-not-executed. Any new action gets a hermetic case in `tests/runme-fixtures.sh` through the `RUNME_*` seams, and never touches real host state or real task files. This also overrides the template's "Copy-Pasteable Commands (T-609)" rule below for this project. **Every run is logged, and the agent reads the log (T-3273).** The script writes its complete stdout and stderr to `/opt/termlink/.context/working/runme-logs/runme-<UTC>-<pid>.log`, and `latest.log` points at the newest one. The log opens with a header (time, args, uid, git HEAD) and ends with `=== runme.sh finished rc=<N> ===`. After the operator runs it, the agent reads `/opt/termlink/.context/working/runme-logs/latest.log` itself; it must not rely on the operator's report. It confirms every action shows `OK` or `skip`. For any `FAILED` line it diagnoses the cause, fixes the script or the underlying problem, and asks the operator to run `/opt/termlink/runme.sh` again. The script is not done until the log shows `rc=0`. **"running" is the signal (operator instruction, T-3275).** When the operator types `running` (or equivalent), the script has been started. The agent immediately reads `latest.log` and keeps monitoring it until the `finished rc=` line appears, then reports each action's outcome, remediates any FAILED line, and asks for a re-run. It does not wait to be asked "did it work?". **Arm the watch when handing the script over, not when told it is running (T-3326; filed by 832-Workflow-designer at `framework:pickup` offset 250).** In the same turn the agent tells the operator to run `/opt/termlink/runme.sh`, it starts a background wait (`run_in_background`) that sleeps until a `latest.log` newer than the hand-over shows `finished rc=`, so the run is read whether or not the operator says "running". Without it, the operator becomes the transport for the fact that something ran: observed twice here on 2026-10-02, when nothing was read until "running" was typed. "running" stays a valid trigger: read the log at once and keep watching. When the watch fires, act on it (report each action, remediate FAILED lines); do not merely narrate it. **Two watches, not one, and re-armed after every session restart (operator, 2026-10-05).** A finish-only watch is silent for the whole run, so the operator again had to type "running" to tell the agent a run it was supposedly watching had started. Arm a START watch (fires as soon as `latest.log` points at a new file, then report "runme started, reading it") alongside the FINISH watch. Background watches die when the session ends or compacts, and the harness marks them "stopped": after `/resume` or any compaction, re-arm both at once whenever a hand-over is outstanding. Any new script handed to the operator follows the same contract: one file, a full path, and its own log at a stated path.

- **Operator decisions are taken ONE AT A TIME (operator standing instruction, 2026-10-01).** When a decision is needed from the operator, present ONE question: its background, the options and a recommendation. Then stop and leave room for the answer and for any Q&A about it. **Do not move to the next question until the operator says so**, even if they answer: wait for an explicit "next" (or equivalent). Never batch several decisions into one message or ask for a combined reply like "SQ-8 1, SQ-10 a+b, …". Questions about a pending decision are answered in place, with the same question still open. This applies to sovereign-question walk-throughs, inception go/no-go, and any `needs input:` that carries more than one decision.

- **Every list item carries a unique label the operator can cite (operator standing instruction, 2026-10-03, T-3329).** Never write a plain `-`/`*` bullet list in text meant for the operator. Label items hierarchically: top level `1`, `2`, `3`; the next level appends a letter, `1a`, `1b`; the level after appends another, `1aa`, `1ab`. Numbering is **unique across the whole message**: do not restart at 1 in each section, continue (a later section starts at 4 if the earlier one ended at 3), so "3" or "2b" names exactly one item. Decision options keep their letters `A`–`D` (already unique). Why: the operator answers by reference ("do 2b, skip 3"), often by voice, and an unlabelled bullet forces them to quote the text back. Tables and code blocks are exempt; a list inside a table cell is not. This applies to replies, decision briefs, handover text written for the operator, and messages to peer agents.

- **State budget stop conditions by threshold NAME, never by number (T-3192).**
  *Fixed upstream in AEF 1.8.3 (re-vendored 2026-10-06, T-3370):* the governance section
  below now states the ladder as percentages of `FW_CONTEXT_WINDOW` (75 / 85 / 95 %) and names
  `budget-gate.sh` as the source of truth; `scripts/check-budget-ladder-drift.sh` guards that
  prose against the gate. `checkpoint.sh budget` is now a native verb. **Superseded again by
  AEF 1.8.13 (T-4226, T-3489):** the bands are no longer fixed percentages. `lib/budget_ladder.py`
  derives them from one spawn reserve R (650K / 700K / 750K used at the 800K window), and
  `budget-gate.sh` and `checkpoint.sh` both read it; the drift guard now runs that module for
  the live numbers. The history below is kept because the lesson about literal numbers in
  mandates still applies (the 75/85/95% figures in it are the pre-1.8.13 form).
  Before 1.8.3 the governance section stated *"Escalation ladder:
  120K ok→warn, 150K warn→urgent, 170K urgent→critical (BLOCK)"*. Those are fixed figures
  from a 200K-window era. **The code has not worked that way for some time**:
  `agents/context/budget-gate.sh:103-108` reads
  `CONTEXT_WINDOW=$(fw_config_int "CONTEXT_WINDOW" 300000)` and derives every threshold as a
  **percentage** — `TOKEN_WARN` 75%, `TOKEN_URGENT` 85%, `TOKEN_CRITICAL` 95%.
  This project sets `CONTEXT_WINDOW: 800000` (`.framework.yaml:15`), so the **real** lines are
  **600,000 / 680,000 / 760,000**, not 120K/150K/170K — the documented ladder is wrong by
  roughly 4.5×, and wrong in the dangerous direction: it reads as "you are critical" at 21%
  of the real window.
  **Why this is load-bearing and not a typo.** An autonomous mandate was written against the
  documented numbers and carried the stop condition "context reaches ~300k". 300000 is not a
  threshold at all — it is `budget-gate.sh`'s **default window size**, quoted as if it were a
  budget. At an 800K window that is 37%, *below even the first warning*, so the stop condition
  fired before the run had done any work. A literal number in a mandate is what went stale.
  **State stop conditions by threshold NAME, never by number** — e.g. *"stop at `TOKEN_WARN`
  (the Amber band, from `lib/budget_ladder.py` for the configured `CONTEXT_WINDOW`)"*. That auto-scales when the window changes and cannot rot.
  Check the live value with `bin/fw config get CONTEXT_WINDOW` and current usage with
  `.agentic-framework/agents/context/checkpoint.sh status` (note: there is **no** `budget`
  subcommand in the vendored copy, despite `/resume` step 6 calling one — use `status`).
  The stale text is in the framework-managed region (`## Core Principle` to end of file is
  replaced wholesale by `fw upgrade`, T-2015), so it cannot be fixed here — it is filed
  upstream. This note lives **above** that line deliberately, so it survives the re-vendor
  that will otherwise restore the wrong numbers.

- **NEVER use a git worktree unless the operator explicitly instructs it.** Do not create one, do not `cd` into one, do not route work through one. This is an operator standing instruction (2026-09-24), and it is load-bearing rather than stylistic: **a single read-only `cd` into a worktree re-roots the whole session into it.** `CLAUDE_PROJECT_DIR` is unset here, so the T-559 boundary gate falls back to the shell's cwd — and once that cwd is a worktree, `cd /opt/termlink` and even `git -C /opt/termlink` are both refused as "another project". There is no way out from inside; the session must be restarted from `/opt/termlink`. Measured cost in one session: three separate gates refused ordinary work, and none of the refusals could be satisfied — the worktree-corpus commit gate cites `fw integrate run`, which was **never built** (`lib/integrate.py` exists; its docstring calls the mutating verb "a later slice"); the pre-push audit FAILS on a cron unit named from the worktree's own basename (T-2815), and its suggested `fw cron install` would write a spurious `agentic-audit-<worktree-name>` into `/etc/cron.d`. If you need to read a file in a worktree, use `git show <branch>:<path>` against the shared object store — never `cd`.
- **Use the vendored path, not global `fw`** — in this consumer project the executable lives at `.agentic-framework/bin/fw`, never `bin/fw` (that only exists inside the framework repo itself). The global `fw` may resolve to a different install. Correct: cd into the project and call `.agentic-framework/bin/fw <cmd>`.
- **DEFER outcomes set `revisit_at` (T-1451, G-053)** — When `fw inception decide T-XXX defer` is the outcome, also set `revisit_at: YYYY-MM-DD` in the task frontmatter to the date when the decision should be reconsidered. Pair with `revisit_evidence_needed: <one-line>` to specify what evidence makes the revisit actionable. The daily G-053 cron (T-1452) will surface ripe revisits in the handover banner — without this field, the deferral has no structural reminder.
- *(Optional, recommended for any non-trivial session)* Opt into agent-presence so peers can reach you: `/be-reachable` (T-1841). Idempotent — safe to run unconditionally; stops cleanly on `/be-reachable stop` before session end. Skip on throw-away sessions (<2 min) or hosts that should not appear on the fleet.
- If you ran `/be-reachable` at session start, stop it: `/be-reachable stop` (the heartbeat is detached via `nohup setsid` so it will otherwise outlive the session)
- **`.context/working/.budget-status` is a SINGLE path shared by every concurrently
  dispatched `claude -p` worker (T-3127, G-087-inverted).** The Automated Monitoring
  section below documents `.budget-status` as the cached status a PostToolUse hook
  refreshes for a single continuously-running session — that model breaks under
  orchestrated multi-worker dispatch (e.g. a `[Review, Audit, procAsFit] x N` run):
  each worker is a short-lived separate process, so the file holds whichever
  worker wrote it last, and a healthy worker can read another worker's near-exhausted
  count (measured: two T-3093 workers independently read ~504K against a real
  per-session figure of ~169K). Since several mandates carry a hard context-stop
  threshold (e.g. ~300k tokens), trusting this file can make a worker report a
  context stop it never hit and leave its queue unworked — indistinguishable in the
  handback from a genuine stop. **If you are one of several concurrently dispatched
  workers, read `.agentic-framework/agents/context/checkpoint.sh status` instead** —
  it parses the session's own transcript and is per-process, not shared. This is also
  why `fw audit`'s CTL-003 ("budget-status stale") WARN is not itself fixable by any
  single worker: the check and its writer (`budget-gate.sh`) are vendored and assume
  the single-session model (filed upstream per G-062, tracked at T-3127/T-3128).

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
| `termlink fleet verify` | all profiles in `~/.termlink/hubs.toml` | Fleet rollup. Drift dominates. `--exit-on-drift-only` for cron alerting on rotations only. `--profile <name>` (T-3391) checks one profile and exits per that profile alone (unknown name = exit 2); duplicate addresses across profiles print a warning. |
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

### Comms round-trip self-test (T-2482 — "why is there still no response?")

The substrate has 11 failure-detecting canaries + four observability arcs, but
all detect breakage **after the fact**. There was no single on-demand command to
**affirmatively prove the charter's core round-trip works right now** and, when it
doesn't, name whether the break is in discover / send / consume.
`scripts/comms-selftest.sh --peer <id>` is that staged prover — the affirmative
complement to the canaries ("prove it works" vs "detect when it broke"):

```bash
bash scripts/comms-selftest.sh --peer <agent_id> --discover-only   # side-effect-free: reachable + armed?
bash scripts/comms-selftest.sh --peer <agent_id> [--json]          # full: fires one synthetic proof-ping
```

Three PASS/FAIL stages — **DISCOVER** (peer LIVE + armed, reuses
`diagnose-unconsumed.sh`), **SEND** (durable turn written, reuses `agent-send.sh`),
**CONSUME** (peer posted a receipt). A DISCOVER fail stops before a pointless send
(`dead` / `unwakeable`); a CONSUME fail carries the G-083 class
(`busy-or-manual`). Exit **0** proven / **1** broken (names the stage) / **2**
tooling (fail-closed). The SEND stage is bounded by `COMMS_SELFTEST_SEND_TIMEOUT`
(default 120s) so it never hangs — no receipt in time renders `CONSUME=FAIL(timeout)`.
Test hooks `TERMLINK_DIAGNOSE_TEST_PRESENCE_JSON` + `COMMS_SELFTEST_TEST_SEND_RC`
(PL-213). See `docs/operations/comms-selftest.md`.

### Session self-test — proving the "control terminal sessions" verb (T-2485)

The charter names four core verbs (discover / exchange durable messages / claim work
/ control terminal sessions). Affirmative on-demand provers existed for three —
`comms-selftest.sh` (T-2482, discover+exchange) and `substrate-smoke.sh` (T-2151,
claim work) — but the **fourth**, the founding verb ("TermLink began as a
cross-terminal session-control tool"), had none. The gap was acknowledged in-code:
`agent-conversation-selftest.sh` says "does NOT validate: PTY inject", and
comms-selftest only checks the `pty_session` presence *flag*, never that a command
actually injects and runs. `scripts/session-selftest.sh` closes it — the 4th sibling
completing prover coverage of all four verbs (an on-demand PROVER, **not** a cron
canary, so it adds no canary breadth):

```bash
bash scripts/session-selftest.sh [--json] [--ttl <secs>] [--hub <addr>]
```

Three staged PASS/FAIL checks against a real session — **SPAWN** (a tmux-backed
scratch session registers), **EXEC** (`termlink exec <s> 'echo <sentinel>' --json`
returns ok + exit_code 0 with the sentinel in stdout — the inject→run→capture
round-trip), **CLEANUP** (best-effort `signal TERM` + `clean`, never fatal). Unlike
doorbell-wake (needs a live peer), `exec --json` makes it deterministic + local. The
EXEC stage retries ~5s to absorb the spawn→ready race (`spawn` returns before the
shell is exec-ready). Exit **0** proven / **1** broken (names the stage) / **2**
tooling (fail-closed — hub-down/missing-dep is never "proven"). Test hooks
`TERMLINK_SESSION_SELFTEST_TEST_SPAWN_RC` + `TERMLINK_SESSION_SELFTEST_TEST_EXEC_JSON`
+ `SESSION_SELFTEST_SENTINEL` (PL-213). See `docs/operations/session-selftest.md`.

### Session-to-session message prover (T-2876) — assert on the receiver, never the sender

The fourth prover, and the only one covering the rail Claude Code sessions actually use
to reach each other. Its three siblings cover the charter's other rails —
`comms-selftest.sh` (T-2482) TermLink discover + exchange, `session-selftest.sh` (T-2485)
PTY control, `substrate-smoke.sh` (T-2151) claim work — and none of them touch this one.
T-2875 found a peer agent had declared it structurally impossible while it was working
the whole time.

**It never trusts the sender.** `SendMessage` returns `{"success":true}` the moment the
message is queued, which says nothing about whether the target received it. Three findings
in two days share that shape: a hub reported `injected` while the PTY got nothing (T-2873);
a config looked authoritative and was never read (T-2874); a send succeeded while the target
sat blocked (T-2875). So the script reads the RECEIVER's own transcript
(`~/.claude/projects/*/<sessionId>.jsonl`).

Four outcomes that are byte-identical from the sender's side:

| verdict | evidence | meaning | exit |
|---|---|---|---|
| `DELIVERED` | sentinel is a `user` turn | the rail works | 0 |
| `BLOCKED` | sentinel queued, target at rest | **delivery was fine**; target never drained | 1 |
| `ENQUEUED` | sentinel queued, target still busy | may yet drain — retry with a longer timeout | 1 |
| `UNDELIVERED` | sentinel absent | the rail is broken | 1 |

The BLOCKED/UNDELIVERED split is the whole point: collapsing them is the T-2875
misdiagnosis, a working rail declared impossible because the target could not act.

**Two premises died on first execution, and both are worth remembering.** The draft keyed
BLOCKED off a `waitingFor` field in `claude agents --json` — **that field does not exist**
(0 of 56 agent records carried it), so the branch was dead code. Its fallback,
`state == "blocked"`, is the ordinary RESTING state of a finished background session (43 of
those same 56), so keying off it would have classified almost every broken rail as "target
merely stuck" — the exact inversion of the bug the prover exists to prevent. `state`/`status`
now only ANNOTATE an already-queued message; they never decide whether delivery happened.

**Reachability is `status`, not presence in the listing.** A session appears in
`claude agents --json` without being addressable. The measured predicate is the presence of
a `status` field — that set matched the SendMessage peer list exactly (15 of 58 records).
`claude --bg` produces a record with no `status`, no transcript, and SendMessage answers
"No agent named ... is reachable". So `prepare --spawn` polls until the target is genuinely
reachable and fails closed otherwise, and `prepare --existing MATCH` targets a session that
already is — which is how the live pass is actually run.

```bash
bash scripts/session-message-selftest.sh prepare --existing <id-or-name>   # prints recipient + sentinel
#   ... then SEND: SendMessage to="<name>" message="<sentinel>"   (agent tool call — there is no CLI verb)
bash scripts/session-message-selftest.sh assert --session-id <sid> --sentinel <text> --id <short>
bash scripts/session-message-selftest.sh cleanup --id <short>              # only after --spawn
```

`prepare`/`cleanup` spawn and stop real sessions, so the script is deliberately **not**
marked `# guard-layer: source`; `assert` alone is a pure read. Exit 0 proven / 1 broken
(naming the stage) / 2 tooling — **fail-closed**: a receiver whose transcript cannot be read
exits 2 and never a delivery verdict. Test seams (PL-213) `SESSION_MSG_TEST_AGENTS_JSON` +
`SESSION_MSG_TEST_TRANSCRIPT_DIR` run the whole suite with no live session and no network.
Fixtures: `bash tests/session-message-selftest-fixtures.sh` (71 assertions, weighted to the
firing cases; 8 mutants pinned, including removing the BLOCKED branch, dropping the
reachability gate, and collapsing ENQUEUED into UNDELIVERED).

**Live-proven** 2026-09-01 against a real peer: DELIVERED rc 0; a control sentinel that was
never sent, UNDELIVERED rc 1; an unobservable receiver, TOOLING rc 2.


### Shipped == Live gate (T-2480, G-069 prevention)

Capabilities were repeatedly recorded **closed=shipped** while dark in the field
for weeks (arc-004 push-transport; .107/.122 stale binaries) — "shipped" meant
*code merged*, not *capability live on the hubs that need it*. Detection is
covered on a daily cron (T-2359 binary-floor / T-2387 waker / T-2415 capability
canaries) but was retrospective; there was **no gate at closure time**.

`scripts/arc-live-probe.sh` is the synchronous "is this capability/version
actually served RIGHT NOW" probe an arc-closing task drops into its
`## Verification` block, so the existing P-011 verification gate mechanically
blocks `work-completed` until the fleet genuinely serves it:

```bash
bash scripts/arc-live-probe.sh --hub <primary-addr> --min-version "$(cat VERSION)" --capability cv-keys
```

Exit **0** = live-confirmed, **1** = shipped-but-not-live (the G-069 firing
class), **2** = tooling-error (**fail-closed** — an unreachable/unparseable hub is
NEVER reported live). Capabilities: `cv-keys` (doorbell discovery prereq, reuses
the T-2415 probe) or `field:<name>` (a named non-null field on the hub's
fleet-doctor entry). Test hooks `TERMLINK_ARC_PROBE_TEST_DOCTOR_JSON` +
`TERMLINK_ARC_PROBE_TEST_CVKEYS_{RC,OUT}` (PL-213). This ships the
**advisory-by-convention** form; the mandatory-block inside AEF `arc.sh arc_close`
(T-2477 IW-1) is a deferred human governance decision (cross-repo per G-062). See
`docs/operations/shipped-equals-live-gate.md`.

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

## Worktree Policy — OPT-IN ONLY (operator directive, 2026-08-25)

**The default is the main checkout. Do not create a git worktree unless the
operator has explicitly instructed one, in this session, in their own words.**

This overrides every other passage in this file that reads as "use a worktree for parallel work".

| Situation | What to do |
|---|---|
| Ordinary work, any size | Main checkout. No worktree. |
| Parallel work that edits files | Serialise it in the main checkout. Converging writes are a reason to go sequential, not a reason to branch the filesystem. |
| Genuinely major piece of work | You MAY **suggest** a worktree, once, as a suggestion. Silence or anything short of a clear yes = main checkout. |
| Operator explicitly says to use one | Use it (`fw worktree create`). Land it on the project's development branch from the main checkout (`fw integrate run <dev-branch> --push`), never straight onto the release branch. |

**"It would be cleaner in isolation" is the rationalisation this rule exists to
catch.** So is "the dispatch protocol says to". Neither is an instruction.

The mechanism is not removed and not gated; only the default changed. This binds on
agent discipline, so do not read the absence of a gate as permission. Worktrees have
been a repeated source of damage: branch/worktree divergence, commits stranded in
directories later torn down, and handed-off `cd …/worktrees/…` commands that fail
because the path is gone.

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
- Web pages load: `curl -sf "$(cat .context/working/watchtower.url 2>/dev/null || echo http://localhost:$(.agentic-framework/bin/fw config get PORT 2>/dev/null || echo 3000))/page"` — never hard-code `:3000`; the triple file `.context/working/watchtower.{pid,port,url}` is the source of truth for Watchtower's port
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
1. Check for existing task or create new one following `default.md` template
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

**Path resolution:** in this project the launcher is `.agentic-framework/bin/fw` (the vendored copy — there is no `bin/fw` at the project root). It finds the framework via its own location and `.framework.yaml` in the project root.

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
- **Before proposing the next unit of work, check context budget** (`checkpoint.sh budget`: band and free tokens)
- Green: proceed normally; spawn workers
- Amber: propose only small, bounded tasks and small workers; commit first
- Red: propose only wrap-up actions (commit, learnings, handover); spawn none
- Stop: handover immediately, no new work
- The bands are one ladder from the spawn reserve R, not fixed percentages: at the 800K default they sit at 650K / 700K / 750K used (Amber / Red / Stop); at 975K, 825K / 875K / 925K. Absolute token counts move with `FW_CONTEXT_WINDOW` and must not be written down as if fixed.
- **This applies especially in autonomous mode** — without a human to catch the mistake, proposing work that can't complete in remaining context risks losing all uncommitted work

### Automated Monitoring (Claude Code)
- **Primary enforcement:** A PreToolUse hook runs `budget-gate.sh` which reads **actual token usage** from the session JSONL transcript and **blocks** Write/Edit/Bash at critical level (exit code 2)
- **Fallback:** A PostToolUse hook runs `checkpoint.sh` for warnings and auto-handover (T-136)
- Escalation ladder, from one spawn reserve R (source of truth: `lib/budget_ladder.py`, read by
  `agents/context/budget-gate.sh` and `checkpoint.sh`): **Amber** (warn, note), **Red** (urgent,
  warning), **Stop** (critical, **BLOCK**). Free tokens: Stop at free ≤ R, Red at free ≤ (1.5 + 0.5f)·R,
  Amber at free ≤ (2 + f)·R. At the 800K default: 650K / 700K / 750K used. At 975K: 825K / 875K / 925K.
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
| Token headroom | Keep the **spawn reserve** free before dispatching: `min(50K, max(20K, 20K + 5% × (CONTEXT_WINDOW − 200K)))` — 20K at 200K, 35K at 500K, 50K at 800K+ (covers result ingestion). Spawn by band: Green workers; Amber small bounded workers only; Red and Stop none |
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

**Structural enforcement (T-139):** The `budget-gate.sh` PreToolUse hook reads actual token usage from the session transcript and **blocks** Write/Edit/Bash tool calls when context reaches critical level (the Stop band: 750K used at the 800K default; the bands are one ladder from the spawn reserve). At critical, only git commit, fw handover, and read operations are allowed. The hook writes `.context/working/.budget-status` with current level (ok/warn/urgent/critical) for fast caching. PostToolUse `checkpoint.sh` remains as fallback for warnings and auto-handover.

### Operator Commands Ship as ONE `runme.sh` Line — standing directive (T-3675)

Whenever the operator has to run something, do NOT paste a block. Wrap it and hand over one line:

```
.agentic-framework/bin/fw runme new <name> --desc "why" -- '<cmd 1>' '<cmd 2>' …
# prints:   bash /abs/path/.context/runme/<name>/runme.sh
.agentic-framework/bin/fw runme watch <name>   # in the background: wait for START, follow to EXIT
```

The script runs `set -euo pipefail`, echoes each command, timestamps all output into `run.log`
beside it, and marks `RUNME START` / `RUNME EXIT <code>`. Hand off exactly the printed line (full
absolute path), then watch the log yourself — never ask the operator to paste output back. No
`run.log` means it was not started, whatever the chat says. Answer a failed run with a new runme,
not ad-hoc instructions. Watchtower URL handoffs (reviews, inception decisions) stay URLs. Keep
`.context/runme/` gitignored.

### Copy-Pasteable Commands (T-609)
When giving the human a command to run (Tier 0 approvals, inception decisions, verification steps, Human AC instructions), the command MUST be:

1. **Single-line, copy-pasteable** — works when pasted into any terminal, from any directory
2. **Prefixed with `cd`** — always include `cd /path/to/project &&` so directory context is explicit
3. **Use `.agentic-framework/bin/fw` not `fw`** — the global `fw` may resolve to a different install, and this project has no `bin/fw` at its root (that path exists only in the framework repo)
4. **No bare multi-line** — if multiple commands are needed, chain with `&&` on one line

### Inception Discipline
When the active task has `workflow_type: inception`:
1. **State the phase** — Say "This is an inception/exploration task" before doing any work
2. **Present the filled template** for review before executing any spikes or prototypes
3. **Do not write build artifacts** (production code, full apps) before `fw inception decide T-XXX go`
4. **The commit-msg hook enforces this** — after 2 exploration commits, further commits are blocked until a decision is recorded
5. After a GO decision, **create separate build tasks** for implementation — do not continue building under the inception task ID
6. **Research artifact first (C-001)** — When starting inception work, first file at least one question (`- **IW-1: …**`) under the task's `## Open Questions`: the T-2194 gate blocks every write outside `.tasks/`, `.context/`, `.claude/` and `.git/`, the artifact included, until one exists. Then create `docs/reports/T-XXX-*.md` BEFORE conducting research. Update the file incrementally as dialogue produces findings. Commit after each dialogue segment. The thinking trail IS the artifact — conversations are ephemeral, files are permanent.
7. **Dialogue log (C-001 extension)** — For phases involving human dialogue, include a `## Dialogue Log` section in the research artifact. Record: questions the human posed, answers given, course corrections, and the outcome/decision that resulted.

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

### AC Classification Guidance (T-954, T-3557, T-3963)
Who closes a criterion is decided by ONE ruleset, `lib/delegation.py` in the framework
(`classify()`, `CLASS_TO_DELEGATION`, `CARVE_OUTS`). Do not re-implement it; ask it:

- `fw task classify-ac --text '- [ ] [REVIEW] ...'` (or `--file F`, or stdin; `--json`,
  `--workflow-type`, `--render-surface`) — classifies criteria before a task exists.
- `fw task delegate T-XXX --dry-run [--json]` — classifies a task's open Human criteria.

The three classes that stay **operator-only**: tier-0 or bypass approval, acts in the world
(outside the repo / the operator's machine), and sovereignty fields. Render-surface, taste,
inception-decision and unclassified criteria are **reviewer-judged** (an independent reviewer
may escalate to the operator); deterministic ones are **reviewer-closeable**. A criterion
whose audience is agents (stderr wording, gate messages) belongs under `### Agent`.

When writing a criterion: if its Expected clause is a shell check, make it an Agent
criterion with the command in `## Verification`; keep `[REVIEW]` for genuine judgement.

Project-specific operator-only classes are not configurable yet; a project-declared
extension point is proposed in framework inception T-3968. Until it is ruled on, such
criteria are reviewer-judged and the reviewer may escalate.

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
5. If the change is visual (CSS, HTML layout, contenteditable, font/density/theme/language modes): see **"Visual Verification for UI Changes"** below — DOM-rect math is not sufficient
6. Do NOT call `fw task update --status work-completed` until all pass
7. The verification gate (P-011) enforces this structurally — this rule makes you check BEFORE hitting the gate

### Visual Verification for UI Changes
DOM measurements (`getBoundingClientRect`, `scrollWidth`, `clientWidth`) confirm geometry — they do NOT confirm rendered output. Font hinting, kerning, ellipsis behaviour, sub-pixel rounding, and contenteditable span spacing can add 3-10px that DOM math misses entirely.

**Before claiming a UI/CSS change is fixed:**
1. **Take element-level Playwright screenshots** of the affected component using `browser_take_screenshot` with a `target` selector (not full-viewport — too zoomed-out to inspect)
2. **Screenshot every visual mode the change can affect:** font types (mono/sans/serif), themes (light/dark/contrast), densities (compact/normal/cozy), languages (de/nl/en), widths (narrow/wide if responsive)
3. **READ each screenshot** with the Read tool — actually look at the rendered output, don't assume
4. **Verify the symptom is gone AND no new visual regression appeared** — fixing one mode can break another
5. Only then check the AC box and commit

**The test:** "Did I look at a rendered screenshot of the change, in every mode it can affect?" If no, you haven't verified — you've measured.

**Gate enforcement (opt-in):** `fw hook-enable --event PreToolUse --matcher Bash --name check-visual-verification` blocks `git commit` when `.css`/`.html` files are staged unless the active task has a `## Visual Verification` section with at least one screenshot reference. Enable in projects that do CSS/HTML work.

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
| **Auto-restart** | **`claude-fw [args...]`** | Wrapper: runs claude, auto-restarts on handover signal |

## Auto-Restart (T-179)

When context budget hits critical, `checkpoint.sh` auto-generates a handover and writes `.context/working/.restart-requested`. If the user started their session via `claude-fw` (instead of `claude`), the wrapper detects this signal on exit and auto-restarts **a fresh session** — not `claude -c` (T-3166). The previous conversation is not restored, because restoring it would hand back the very context the restart exists to free. The wrapper leaves a `.auto-restart-pending` sentinel, and the SessionStart hook (`post-compact-resume.sh`, source `startup`) injects the handover, any armed directive and the loop counter into the new session.

**Flow:** Budget critical → auto-handover → signal file → claude exits → wrapper detects → `sleep 3` → new `claude` session → handover injected → `/resume` ready.

**Restart mode:** `FW_RESTART_MODE=fresh` is the default. `FW_RESTART_MODE=continue` keeps the transcript instead: the wrapper resumes this session's recorded conversation (`--resume <id>`, never a bare `-c`, T-4497). It does not free context, so the next budget check can be critical again. The wrapper prints which mode fired (`Restart mode: …`) on every restart.

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

**Do not end a session without generating a handover.**
