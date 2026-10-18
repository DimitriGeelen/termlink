#!/usr/bin/env bash
# runme.sh — pending operator actions for 010-termlink.
#
# Run this from the project root instead of pasting commands:
#
#     sudo ./runme.sh --dry-run     # read what it intends to do
#     sudo ./runme.sh --decide T-XXXX=go        # record ONE human decision
#     sudo ./runme.sh --disable-mismatch-plugins  # opt-in plugin cleanup
#     sudo ./runme.sh               # do it
#
# WHY THIS EXISTS. Operator steps were being handed over as copy-pasteable shell
# lines. That puts correct transcription on the human, leaves no record of what
# was asked, and — worst — cannot check itself: the operator runs it and both
# sides assume it worked. That is the same silent-failure class this repo's whole
# guard layer exists to prevent, reproduced in the one place a human is acting.
#
# So every action here VERIFIES itself after running and this script exits
# non-zero if any verification fails. A green run is evidence, not an assumption.
#
# IDEMPOTENT: each action checks whether it is already done and skips if so. Safe
# to re-run at any time; that is the point, not a caveat.
#
# Actions are removed once they are permanently done and no longer pending, so a
# short file means little is waiting on you.
#
# Exit: 0 all actions done+verified · 1 an action failed verification · 2 refused
set -uo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_ROOT" || { echo "runme: cannot cd to $PROJECT_ROOT" >&2; exit 2; }

# Test seam (PL-213): fixtures point the installs at a scratch dir so this script's
# own logic — install, idempotence, tamper-detection, verification — can be proven
# without writing to real host state. Default is the real path.
# T-3299: every default below that touches REAL host state goes through `seam`.
# Twice in one session a test reached a real default because its seam was
# missing — a hub test reaped ~10k real files (T-3293), and a fixture run
# overwrote /etc/systemd/system/termlink-hub.service (T-3299). Under
# RUNME_FIXTURE=1 (exported by tests/runme-fixtures.sh) reaching a real default
# ABORTS the run (exit 2) before anything is touched, so a new action is
# protected even when its fixture seam was forgotten.
seam() {  # seam <VAR> <RUNME_SEAM> <real default>
    if [ -n "${!2+x}" ]; then printf -v "$1" '%s' "${!2}"; return; fi
    if [ "${RUNME_FIXTURE:-0}" = "1" ]; then
        echo "runme: FIXTURE run reached the REAL default for $2 ($3) — refusing before touching host state (T-3299)" >&2
        exit 2
    fi
    printf -v "$1" '%s' "$3"
}

seam CRON_DIR RUNME_CRON_DIR "/etc/cron.d"
# Keep the drift checker pointed at the same place, or the verification step would
# arbitrate against a directory this run never touched.
[ -n "${RUNME_CRON_DIR:-}" ] && export CRON_DRIFT_INSTALLED_DIR="$RUNME_CRON_DIR"
# T-3272 seams: the closure action reads/writes task files through these, so the
# fixtures can prove it against a scratch tree and a fake fw — never real tasks.
seam TASKS_DIR RUNME_TASKS_DIR "$PROJECT_ROOT/.tasks"
seam FW RUNME_FW "$PROJECT_ROOT/.agentic-framework/bin/fw"

RUNME_ARGS="$*"   # captured before parsing shifts them away (logged, T-3273)
DRY_RUN=0
DECIDE=""
DISABLE_MISMATCH=0
SIDECAR_DELIVERY=""
SIDECAR_WHY=""
while [ $# -gt 0 ]; do
    case "$1" in
        --dry-run) DRY_RUN=1 ;;
        --decide)  shift; [ $# -ge 1 ] || { echo "runme: --decide needs <ID>=<verdict>" >&2; exit 2; }; DECIDE="$1" ;;
        --disable-mismatch-plugins) DISABLE_MISMATCH=1 ;;
        # T-3438 (arc-011 OQ-e, SI-23): the fleet kill switch for this host's
        # sidecars. Runs ONLY the switch (no other action), every flip read back.
        --sidecar-delivery) shift; case "${1:-}" in on|off) SIDECAR_DELIVERY="$1" ;; *) echo "runme: --sidecar-delivery needs on|off" >&2; exit 2 ;; esac ;;
        --why) shift; [ $# -ge 1 ] || { echo "runme: --why needs a reason" >&2; exit 2; }; SIDECAR_WHY="$1" ;;
        -h|--help) sed -n '2,27p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "runme: unknown arg: $1" >&2; exit 2 ;;
    esac
    shift
done

# T-3273 — LOG EVERY RUN. The operator runs this; the agent reads the log afterwards
# to confirm what happened, remediate any FAILED line, and ask for a re-run. So the
# log holds the COMPLETE stdout+stderr, a header saying what ran, and a final rc line.
# It is written for refusals and dry runs too — a refusal is an outcome worth reading.
# Logging never blocks the actions: an unwritable log dir falls back, and says so.
LOG_DIR="${RUNME_LOG_DIR:-$PROJECT_ROOT/.context/working/runme-logs}"
LOG_NOTE=""
if ! mkdir -p "$LOG_DIR" 2>/dev/null || [ ! -w "$LOG_DIR" ]; then
    LOG_NOTE="log dir $LOG_DIR not writable — using fallback"
    LOG_DIR="${TMPDIR:-/tmp}/runme-logs-$(id -u)"; mkdir -p "$LOG_DIR" 2>/dev/null
fi
RUNME_LOG="$LOG_DIR/runme-$(date -u +%Y%m%dT%H%M%SZ)-$$.log"
if : > "$RUNME_LOG" 2>/dev/null; then
    ln -sfn "$(basename "$RUNME_LOG")" "$LOG_DIR/latest.log" 2>/dev/null || true
    exec > >(tee -a "$RUNME_LOG") 2>&1
    # The rc line is the reader's success/failure signal; print it on every exit path.
    trap 'rc=$?; printf "\n=== runme.sh finished rc=%s at %s — log: %s ===\n" "$rc" "$(date -u +%FT%TZ)" "$RUNME_LOG"' EXIT
else
    LOG_NOTE="${LOG_NOTE:+$LOG_NOTE; }could not create any log file — output is terminal-only"
    RUNME_LOG=""
fi
printf '=== runme.sh start %s  args=[%s]  uid=%s  head=%s ===\n' \
    "$(date -u +%FT%TZ)" "$RUNME_ARGS" "$(id -u)" "$(git -C "$PROJECT_ROOT" rev-parse --short HEAD 2>/dev/null || echo '?')"
[ -n "$RUNME_LOG" ] && printf 'log: %s\n' "$RUNME_LOG"
[ -n "$LOG_NOTE" ] && printf 'note: %s\n' "$LOG_NOTE"

# Refuse rather than half-apply. A partial application is worse than none: it
# leaves the host in a state nobody described, and the operator believes it ran.
# T-3443: a FIXTURE run may opt out (RUNME_FIXTURE=1 plus RUNME_FIXTURE_ALLOW_NONROOT=1).
# Safe because fixture mode aborts before reaching any real host default (T-3299),
# and needed because CI runners are not root: without it 76 fixture cases failed on
# this refusal there, so the guard layer stayed red for weeks for nothing they test.
FIXTURE_NONROOT=0
[ "${RUNME_FIXTURE:-0}" = "1" ] && [ "${RUNME_FIXTURE_ALLOW_NONROOT:-0}" = "1" ] && FIXTURE_NONROOT=1
if [ "$DRY_RUN" = "0" ] && [ -z "$DECIDE" ] && [ "$DISABLE_MISMATCH" = "0" ] && [ "$(id -u)" != "0" ] && [ "$FIXTURE_NONROOT" = "0" ]; then
    echo "runme: needs root (these write to /etc/cron.d). Nothing was applied." >&2
    echo "       Run:  sudo ./runme.sh          (or ./runme.sh --dry-run to preview)" >&2
    exit 2
fi

DONE=0; SKIPPED=0; FAILED=0
INSTALLED_NAMES=()   # basenames this run is responsible for

say()  { printf '%s\n' "$*"; }
head2() { printf '\n\033[1m%s\033[0m\n' "$*"; }

# ---------------------------------------------------------------------------
# KILL SWITCH — `runme.sh --sidecar-delivery off|on --why "<reason>"` (T-3438;
# OQ-e ruled A, SI-23). Flips delivery on EVERY sidecar in the host's admin-socket
# list, each flip read back by the client (`admin delivery ... --all --list`), and
# runs nothing else. Off stops new hand-overs within one tick; mail keeps arriving
# and is stored. Seams (fixtures only): RUNME_SIDECAR_TL, RUNME_SIDECAR_ADMIN_LIST.
# ---------------------------------------------------------------------------
if [ -n "$SIDECAR_DELIVERY" ]; then
    seam SC_TL RUNME_SIDECAR_TL "$(command -v termlink || echo /root/.cargo/bin/termlink)"
    seam SC_ADMIN_LIST RUNME_SIDECAR_ADMIN_LIST "$PROJECT_ROOT/.context/working/sidecar-admin-sockets.list"
    head2 "Kill switch: sidecar delivery $SIDECAR_DELIVERY on every listed sidecar (T-3438)"
    if [ -z "$SIDECAR_WHY" ]; then say "  FAILED  --why \"<reason>\" is required (the reason is logged on each sidecar)"; exit 1; fi
    if [ ! -s "$SC_ADMIN_LIST" ]; then say "  FAILED  no sidecars listed in $SC_ADMIN_LIST (W1 not provisioned?) — nothing switched"; exit 1; fi
    if [ "$DRY_RUN" = "1" ]; then say "  [DRY]   would set delivery $SIDECAR_DELIVERY on: $(tr '\n' ' ' < "$SC_ADMIN_LIST")"; exit 0; fi
    if "$SC_TL" sidecar admin delivery "$SIDECAR_DELIVERY" --why "$SIDECAR_WHY" --all --list "$SC_ADMIN_LIST"; then
        say "  OK      delivery $SIDECAR_DELIVERY on every listed sidecar, each read back"; exit 0
    else
        say "  FAILED  at least one sidecar did not confirm delivery $SIDECAR_DELIVERY (lines above name it)"; exit 1
    fi
fi

# install_crontab <source-basename> <installed-path>
# Installs a git-tracked crontab, then VERIFIES the installed copy matches source.
install_crontab() {
    local src=".context/cron/$1" dest="$2"
    INSTALLED_NAMES+=("$(basename "$dest")")
    if [ ! -r "$src" ]; then
        say "  FAILED  source missing: $src"; FAILED=$((FAILED+1)); return
    fi
    if [ -f "$dest" ] && cmp -s "$src" "$dest"; then
        say "  skip    already installed and identical: $dest"; SKIPPED=$((SKIPPED+1)); return
    fi
    if [ "$DRY_RUN" = "1" ]; then
        say "  [DRY]   would install $src -> $dest"; DONE=$((DONE+1)); return
    fi
    if ! cp "$src" "$dest"; then
        say "  FAILED  could not copy $src -> $dest"; FAILED=$((FAILED+1)); return
    fi
    chmod 644 "$dest" 2>/dev/null || true
    # Verify rather than assume. cp can succeed onto a full disk or a read-only
    # remount and still leave the destination wrong.
    if cmp -s "$src" "$dest"; then
        say "  OK      installed and verified: $dest"; DONE=$((DONE+1))
    else
        say "  FAILED  copied but destination does not match source: $dest"; FAILED=$((FAILED+1))
    fi
}

# ---------------------------------------------------------------------------
# ACTION 1 — notify-rail cron (T-3050 supervisor + T-3051 liveness canary)
#
# The arc-003 deterministic wake rail was measured dark for 82 days (T-3049):
# nothing started it and nothing noticed. The supervisor is the trigger it never
# had (autostart + self-heal, every 5 min); the canary is independent detection
# for when the supervisor cannot fix it. Until these are installed the rail stays
# alive only for as long as the session that started it by hand, and dies on
# reboot.
#
# No cron reload is needed — cron re-reads /etc/cron.d within a minute. (Note
# `systemctl reload cron` is NOT valid on this host: "Job type reload is not
# applicable for unit cron.service".)
# ---------------------------------------------------------------------------
head2 "1. Cron installs (notify-rail T-3050/T-3051/T-3068, arc-claim-drift T-3288, session-leak T-3296)"
install_crontab notify-sidecar-supervisor.crontab "$CRON_DIR/termlink-notify-sidecar-supervisor"
install_crontab notify-sidecar-canary.crontab     "$CRON_DIR/termlink-notify-sidecar-canary"

# T-3068 — the WAKE trigger (path A). Without this the rail delivers and
# confirms, but nothing reads the flag durably: the consumers this starts are
# the only thing that climbs L2 -> L3, and only cron restarts them after a
# reboot or a crash. Proven live before being offered here: a real message
# produced stage=delivered from both sidecars and then stage=read
# (evidence=wake-consumer) from the supervised consumer, correctly signed as the
# receiving agent, with no runaway.
install_crontab notify-wake-supervisor.crontab    "$CRON_DIR/termlink-notify-wake-supervisor"

# T-3288 — the only place arc-003's "no silent loss" and arc-004's push-wake
# claims can be re-verified. CI skips both provers (no hub, no key), and the
# host's other guard-layer run is WARN-tier only, so without this daily job
# nothing re-checks a closed arc's claim unless someone runs it by hand.
install_crontab arc-claim-drift-canary.crontab    "$CRON_DIR/termlink-arc-claim-drift-canary"

# T-3296 (T-3291 S4) — daily detection for the session leak: zombie register
# sessions and orphaned data sockets, both invisible for weeks before T-3291.
install_crontab session-leak-canary.crontab       "$CRON_DIR/termlink-session-leak-canary"

# T-3425 (operator ruling A) — `/` lost its execute bits on 2026-10-12 (an agent's
# `tmux -S /proc/<pid>/cwd`) and the host could not boot. Checks at boot and hourly
# that `/` is a 755 root-owned directory; alerts only, never repairs.
install_crontab root-mode-canary.crontab          "$CRON_DIR/termlink-root-mode-canary"

# T-3310 (arc-012 step 5, D1 backstop) — fires if the hub opts out of "forever
# needs an owner", or is still not enforcing it after 2026-11-15; files one task.
install_crontab forever-owner-canary.crontab      "$CRON_DIR/termlink-forever-owner-canary"
install_crontab agent-send-suites-canary.crontab  "$CRON_DIR/termlink-agent-send-suites-canary"
install_crontab vector-index-canary.crontab       "$CRON_DIR/termlink-vector-index-canary"

# T-3432 (arc-011 FX-5 ruling C) — daily: files sent as ordinary messages outside the artifact path.
install_crontab file-shaped-posts-canary.crontab  "$CRON_DIR/termlink-file-shaped-posts-canary"

# T-3383 (arc-011 PH-0 6.2.1.a (1)) — per-minute hub outage sampler for the H-1 / PN-1 re-check,
# plus its 15-min liveness canary (a dead collector would silently shorten the 7-day run).
install_crontab hub-outage-collector.crontab     "$CRON_DIR/termlink-hub-outage-collector"

# T-3435 (arc-011 PH-2 row P3, OQ-f ruling D+) — supervision for project sidecars:
# the systemd template unit (Restart=always, starts only while the project's
# SIDECAR_KEEP_RUNNING is on and it was not explicitly stopped), the settings dir
# for per-project env files (written in row P7), and the 10-minute cron backstop
# that repairs THROUGH systemd and records each time it had to act. Installing
# these starts nothing: no project is configured until row P7.
# Seams (fixtures only): RUNME_SIDECAR_UNIT_DIR, RUNME_SIDECAR_ENV_DIR,
# RUNME_SIDECAR_SYSTEMCTL, RUNME_SIDECAR_SYSTEMD_RUN.
seam SC_UNIT_DIR RUNME_SIDECAR_UNIT_DIR "/etc/systemd/system"
seam SC_ENV_DIR RUNME_SIDECAR_ENV_DIR "/etc/termlink-sidecar"
seam SC_SYSTEMCTL RUNME_SIDECAR_SYSTEMCTL "systemctl"
seam SC_SYSTEMD_RUN RUNME_SIDECAR_SYSTEMD_RUN "/run/systemd/system"
install_sidecar_supervision() {
    local src="systemd-templates/termlink-sidecar@.service" dest="$SC_UNIT_DIR/termlink-sidecar@.service"
    if [ -d "$SC_ENV_DIR" ]; then
        say "  skip    settings dir exists: $SC_ENV_DIR"; SKIPPED=$((SKIPPED+1))
    elif [ "$DRY_RUN" = "1" ]; then
        say "  [DRY]   would create $SC_ENV_DIR (0755)"; DONE=$((DONE+1))
    elif install -d -m 0755 "$SC_ENV_DIR" && [ -d "$SC_ENV_DIR" ]; then
        say "  OK      created and verified: $SC_ENV_DIR"; DONE=$((DONE+1))
    else
        say "  FAILED  could not create $SC_ENV_DIR"; FAILED=$((FAILED+1))
    fi
    if [ ! -d "$SC_SYSTEMD_RUN" ]; then
        say "  skip    no systemd on this host: the cron backstop is the sidecar supervisor"; SKIPPED=$((SKIPPED+1)); return
    fi
    if [ ! -r "$src" ]; then
        say "  FAILED  source missing: $src"; FAILED=$((FAILED+1)); return
    fi
    if [ -f "$dest" ] && cmp -s "$src" "$dest"; then
        say "  skip    already installed and identical: $dest"; SKIPPED=$((SKIPPED+1)); return
    fi
    if [ "$DRY_RUN" = "1" ]; then
        say "  [DRY]   would install $src -> $dest and daemon-reload"; DONE=$((DONE+1)); return
    fi
    if ! cp "$src" "$dest" || ! chmod 644 "$dest" || ! cmp -s "$src" "$dest"; then
        say "  FAILED  could not install $dest"; FAILED=$((FAILED+1)); return
    fi
    if ! "$SC_SYSTEMCTL" daemon-reload; then
        say "  FAILED  installed $dest but daemon-reload failed"; FAILED=$((FAILED+1)); return
    fi
    # Verify systemd itself now knows the template, not only that the file exists.
    if "$SC_SYSTEMCTL" cat termlink-sidecar@verify.service >/dev/null 2>&1; then
        say "  OK      installed, reloaded and verified: $dest"; DONE=$((DONE+1))
    else
        say "  FAILED  $dest installed but systemd does not resolve termlink-sidecar@<project>"; FAILED=$((FAILED+1))
    fi
}
install_sidecar_supervision
install_crontab sidecar-backstop.crontab          "$CRON_DIR/termlink-sidecar-backstop"

# T-3443 (operator ruling, decision 5 B): this host no longer expects the legacy
# push-wake rail armed (--expect-armed dropped); agents not expected armed are named
# in .context/checks/waker-not-expected-allowlist. Revisit at arc-011 W2.
install_crontab waker-liveness-canary.crontab     "$CRON_DIR/termlink-waker-liveness-canary"

# close_task <T-ID> <reason>
# Closes a task the operator has ALREADY approved closing, then VERIFIES it
# landed in completed/ with status work-completed.
close_task() {
    local id="$1" reason="$2" done_f act_f
    done_f=$(ls "$TASKS_DIR"/completed/"$id"-*.md 2>/dev/null | head -1)
    if [ -n "$done_f" ] && grep -q '^status: work-completed' "$done_f"; then
        say "  skip    already closed: $id"; SKIPPED=$((SKIPPED+1)); return
    fi
    act_f=$(ls "$TASKS_DIR"/active/"$id"-*.md 2>/dev/null | head -1)
    if [ -z "$act_f" ]; then
        say "  FAILED  $id is in neither active/ nor completed/"; FAILED=$((FAILED+1)); return
    fi
    if [ "$DRY_RUN" = "1" ]; then
        say "  [DRY]   would close $id (--skip-acceptance-criteria: $reason)"; DONE=$((DONE+1)); return
    fi
    : > /tmp/.runme-close-"$id"
    # T-3275: fw refuses captured -> work-completed ("Invalid transition"); a
    # captured task must pass through started-work first. Measured on the
    # operator's first run: T-3132 and T-3130 failed exactly this way.
    if grep -q '^status: captured' "$act_f"; then
        "$FW" task update "$id" --status started-work >>/tmp/.runme-close-"$id" 2>&1
    fi
    # Narrow bypass: only the one agent AC the operator approved skipping —
    # not the deprecated blanket --force, which would also skip verification.
    "$FW" task update "$id" --status work-completed --skip-acceptance-criteria --reason "$reason" >>/tmp/.runme-close-"$id" 2>&1
    done_f=$(ls "$TASKS_DIR"/completed/"$id"-*.md 2>/dev/null | head -1)
    # Verify the result, not fw's exit code: the close must be visible on disk.
    if [ -n "$done_f" ] && grep -q '^status: work-completed' "$done_f"; then
        say "  OK      closed and verified: $id"; DONE=$((DONE+1))
    else
        say "  FAILED  $id did not land in completed/ as work-completed (fw output: /tmp/.runme-close-$id)"
        FAILED=$((FAILED+1))
    fi
}

# ---------------------------------------------------------------------------
# ACTION 2 — closures the operator has ALREADY approved (T-3272, T-3276)
#
# One "T-ID|reason" per entry. A closure belongs here only when the operator has
# approved it and the approval is recorded in the task's Updates; completion then
# needs a bypass of the approved AC (--skip-acceptance-criteria), which the agent
# cannot run, so the operator does, by running this script. Remove entries once the
# log shows them closed (a finished action is removed, CLAUDE.md runme rule).
# History: T-3132/T-3128/T-3130 (SQ-3) closed 2026-09-30, log runme-20260930T103733Z.
# RUNME_TEST_CLOSES is a FIXTURE seam (newline-separated entries), never set by hand.
# ---------------------------------------------------------------------------
APPROVED_CLOSES=()
[ -n "${RUNME_TEST_CLOSES:-}" ] && mapfile -t APPROVED_CLOSES <<< "$RUNME_TEST_CLOSES"
if [ "${#APPROVED_CLOSES[@]}" -gt 0 ]; then
    head2 "2. Approved closures"
    for entry in "${APPROVED_CLOSES[@]}"; do
        [ -n "$entry" ] && close_task "${entry%%|*}" "${entry#*|}"
    done
fi

# ---------------------------------------------------------------------------
# ACTION 3 — inception decisions the operator has ALREADY made (T-3285)
#
# One "T-ID|verdict|rationale" per entry (verdict: go | no-go | defer). An entry
# belongs here only when the operator stated the ruling and it is recorded in the
# task's Updates. Recording it is Tier 0 (human authority), so the operator carries
# it out by running this script. Verified on disk: the task file must then carry
# "**Decision**: <VERDICT>". Remove the entry once the log shows it recorded.
# RUNME_TEST_APPROVED_DECISIONS is a FIXTURE seam; when SET (even empty) it
# replaces the real list, so fixtures never touch a real task.
# ---------------------------------------------------------------------------
# T-3284 = go recorded 2026-09-30 (log runme-20260930T185754Z, rc=0)
# T-3291 = go recorded 2026-09-30 (log runme-20260930T215045Z, rc=0)
# T-3302 = go, T-3006 = no-go, T-3200 = no-go recorded 2026-10-01 (log runme-20261001T132546Z, rc=0)
# T-3304 = go recorded 2026-10-01 (log runme-20261001T193212Z, rc=0)
# T-3319 = go recorded 2026-10-02 (log runme-20261002T175105Z, rc=0)
APPROVED_DECISIONS=()
if [ -n "${RUNME_TEST_APPROVED_DECISIONS+x}" ]; then
    APPROVED_DECISIONS=()
    [ -n "$RUNME_TEST_APPROVED_DECISIONS" ] && mapfile -t APPROVED_DECISIONS <<< "$RUNME_TEST_APPROVED_DECISIONS"
fi

# record_decision <T-ID> <verdict> <rationale>
record_decision() {
    local id="$1" verdict="$2" why="$3" f want
    want="$(printf '%s' "$verdict" | tr '[:lower:]' '[:upper:]')"
    case "$verdict" in go|no-go|defer) : ;; *) say "  FAILED  $id: verdict must be go/no-go/defer (got '$verdict')"; FAILED=$((FAILED+1)); return ;; esac
    f=$(ls "$TASKS_DIR"/active/"$id"-*.md "$TASKS_DIR"/completed/"$id"-*.md 2>/dev/null | head -1)
    [ -n "$f" ] || { say "  FAILED  $id: no task file in active/ or completed/"; FAILED=$((FAILED+1)); return; }
    if grep -q "^\*\*Decision\*\*: $want\$" "$f"; then
        say "  skip    already recorded: $id = $verdict"; SKIPPED=$((SKIPPED+1)); return
    fi
    if [ "$DRY_RUN" = "1" ]; then
        say "  [DRY]   would record $id = $verdict"; DONE=$((DONE+1)); return
    fi
    # T-973: decide refuses without a review marker, which `fw task review` creates. The
    # operator running this script IS the human review (the ruling is recorded in the
    # task), so review runs here, then decide. Review's exit code is not trusted either
    # way — decide enforces the marker, and the on-disk check below is the verdict.
    "$FW" task review "$id" >/tmp/.runme-decide-"$id" 2>&1
    "$FW" inception decide "$id" "$verdict" --rationale "$why" >>/tmp/.runme-decide-"$id" 2>&1
    f=$(ls "$TASKS_DIR"/active/"$id"-*.md "$TASKS_DIR"/completed/"$id"-*.md 2>/dev/null | head -1)
    if [ -n "$f" ] && grep -q "^\*\*Decision\*\*: $want\$" "$f"; then
        say "  OK      recorded and verified: $id = $verdict"; DONE=$((DONE+1))
    else
        say "  FAILED  $id: decision not found in the task file after recording (fw output: /tmp/.runme-decide-$id)"
        FAILED=$((FAILED+1))
    fi
}

if [ "${#APPROVED_DECISIONS[@]}" -gt 0 ]; then
    head2 "3. Approved inception decisions"
    for entry in "${APPROVED_DECISIONS[@]}"; do
        [ -n "$entry" ] || continue
        e_id="${entry%%|*}"; rest="${entry#*|}"
        record_decision "$e_id" "${rest%%|*}" "${rest#*|}"
    done
fi

# ---------------------------------------------------------------------------
# ACTION 4 — install the current termlink build into every termlink on this host
# (T-3287, widened by T-3289)
#
# ~/.cargo/bin/termlink is what the guard layer's provers, systemd, and the arc
# canary call. But it is NOT the only copy: /usr/local/bin/termlink and
# ~/.local/bin/termlink sat at 0.12.13 while cargo moved on, and 32 of the 33
# termlink cron jobs declare PATH=/usr/local/bin:... — so nearly every canary and
# the notify-rail sidecars executed six-day-stale code. check-installed-binary-drift
# reported "DRIFT: 0.12.13 0.12.21" and, being WARN-tier, nothing acted on it.
#
# Policy: ~/.cargo/bin is always installed. The other paths are REFRESHED only
# where a termlink already exists — this fixes stale copies without spreading new
# ones onto a host that never had them.
# Idempotent per path: skipped when it already reports the build's version AND
# accepts --no-create. Verified per path on disk after installing: the version
# matches, and a real `--resolve --no-create` probe against a throwaway HOME exits
# 3 (no key) WITHOUT writing one. Replacing a binary does not touch a process
# already running it (install unlinks + creates); crons pick the new one up on
# their next run. Restarting the hub / MCP server is a separate decision.
# Seams (fixtures only): RUNME_TERMLINK_SRC (built binary); RUNME_TERMLINK_DEST
# (ONE install path, as before) or RUNME_TERMLINK_DESTS (space-separated list,
# first = always-install, rest = refresh-if-present); RUNME_SKIP_BUILD=1.
# ---------------------------------------------------------------------------
seam TL_SRC RUNME_TERMLINK_SRC "$PROJECT_ROOT/target/local-fast/termlink"

# --- Cached builds (T-3292) -------------------------------------------------
# Builds dominated runme (11-32 min per run). Two causes, both fixed here:
#  * build.rs re-derives the version from git on EVERY commit (it watches
#    .git/logs/HEAD), so even a docs-only commit recompiled four crates and redid
#    `release`'s fat single-unit LTO link. runme now records the commit each build
#    came from and SKIPS cargo when nothing under crates/, Cargo.toml or Cargo.lock
#    changed since (committed or in the working tree). The installed version string
#    may then trail HEAD across docs-only commits — same code, by construction.
#  * `release` is lto=true + codegen-units=1. Host installs use `local-fast`
#    (thin LTO, 16 units, incremental). Published binaries (release.yml) keep
#    `release`.
# Seams (fixtures only): RUNME_CARGO, RUNME_BUILD_REPO, RUNME_BUILD_STAMP_DIR;
# RUNME_SKIP_BUILD=1 skips building entirely and treats outputs as current.
seam BUILD_STAMP_DIR RUNME_BUILD_STAMP_DIR "$PROJECT_ROOT/.context/working/runme-build-stamps"
seam BUILD_REPO RUNME_BUILD_REPO "$PROJECT_ROOT"
seam CARGO RUNME_CARGO "$(command -v cargo || echo "${HOME:-/root}/.cargo/bin/cargo")"
CODE_PATHS="crates Cargo.toml Cargo.lock"

build_is_current() {  # <label> <output-binary> -> 0 when <output> was built from the current code
    [ "${RUNME_SKIP_BUILD:-0}" = "1" ] && return 0
    local stamp="$BUILD_STAMP_DIR/$1" built head
    [ -x "$2" ] && [ -f "$stamp" ] || return 1
    built="$(cat "$stamp")"; head="$(git -C "$BUILD_REPO" rev-parse HEAD 2>/dev/null)" || return 1
    git -C "$BUILD_REPO" diff --quiet "$built" "$head" -- $CODE_PATHS 2>/dev/null || return 1
    [ -z "$(git -C "$BUILD_REPO" status --porcelain -- $CODE_PATHS 2>/dev/null)" ]
}

cached_build() {  # <label> <output-binary> <cargo args...> -> 0 built or current; 1 failed (counted)
    local label="$1" out="$2"; shift 2
    if build_is_current "$label" "$out"; then
        [ "${RUNME_SKIP_BUILD:-0}" = "1" ] || say "  cached  $label build is current (no change under crates/, Cargo.toml, Cargo.lock since $(cut -c1-9 "$BUILD_STAMP_DIR/$label")) — not rebuilding"
        return 0
    fi
    if [ "$DRY_RUN" = "1" ]; then say "  [DRY]   would build ($label): cargo $*"; return 0; fi
    local head t0; head="$(git -C "$BUILD_REPO" rev-parse HEAD 2>/dev/null)"; t0=$(date +%s)
    say "  build   ($label) cargo $*"
    if ! (cd "$PROJECT_ROOT" && "$CARGO" "$@") >"/tmp/.runme-build-$label.log" 2>&1; then
        say "  FAILED  $label build failed (see /tmp/.runme-build-$label.log)"; FAILED=$((FAILED+1)); return 1
    fi
    mkdir -p "$BUILD_STAMP_DIR" && printf '%s\n' "$head" > "$BUILD_STAMP_DIR/$label"
    say "  built   $label in $(( $(date +%s) - t0 ))s"
}
if [ -n "${RUNME_TERMLINK_DESTS:-}" ]; then
    TL_DESTS="$RUNME_TERMLINK_DESTS"
elif [ -n "${RUNME_TERMLINK_DEST:-}" ]; then
    TL_DESTS="$RUNME_TERMLINK_DEST"
else
    seam TL_DESTS RUNME_TERMLINK_DESTS "${HOME:-/root}/.cargo/bin/termlink /usr/local/bin/termlink ${HOME:-/root}/.local/bin/termlink"
fi

tl_accepts_no_create() {  # <binary> -> 0 if the flag is accepted and nothing is minted
    local bin="$1" h rc=0
    h="$(mktemp -d)" || return 1
    env -u TERMLINK_IDENTITY_FILE -u TERMLINK_IDENTITY_DIR HOME="$h" TERMLINK_AGENT_ID=runme-probe \
        "$bin" agent identity --resolve --no-create --json >/dev/null 2>&1 || rc=$?
    local minted=0; [ -e "$h/.termlink" ] && minted=1
    rm -rf "$h"
    [ "$rc" = "3" ] && [ "$minted" = "0" ]
}

install_termlink_to() {  # <dest> <want-version>
    local dest="$1" want="$2" have
    have="$("$dest" --version 2>/dev/null || true)"
    if [ "$have" = "$want" ] && tl_accepts_no_create "$dest"; then
        say "  skip    already installed: $dest ($have, accepts --no-create)"; SKIPPED=$((SKIPPED+1)); return
    fi
    if [ "$DRY_RUN" = "1" ]; then
        say "  [DRY]   would install $TL_SRC ($want) -> $dest (now: ${have:-absent})"; DONE=$((DONE+1)); return
    fi
    mkdir -p "$(dirname "$dest")"
    if ! install -m 755 "$TL_SRC" "$dest"; then
        say "  FAILED  could not install $TL_SRC -> $dest"; FAILED=$((FAILED+1)); return
    fi
    have="$("$dest" --version 2>/dev/null || true)"
    if [ "$have" = "$want" ] && tl_accepts_no_create "$dest"; then
        say "  OK      installed and verified: $dest ($have; --resolve --no-create exits 3, writes nothing)"; DONE=$((DONE+1))
    else
        say "  FAILED  installed $dest reports '${have:-nothing}' (want '$want') or rejects --no-create"; FAILED=$((FAILED+1))
    fi
}

install_termlink() {
    cached_build host "$TL_SRC" build --profile local-fast -p termlink || return
    if [ ! -x "$TL_SRC" ]; then
        if [ "$DRY_RUN" = "1" ]; then say "  [DRY]   would build and install $TL_SRC -> $TL_DESTS"; DONE=$((DONE+1)); return; fi
        say "  FAILED  built binary missing: $TL_SRC"; FAILED=$((FAILED+1)); return
    fi
    local want dest first=1
    want="$("$TL_SRC" --version 2>/dev/null)"
    for dest in $TL_DESTS; do
        if [ "$first" = "1" ]; then
            first=0
        elif [ ! -e "$dest" ]; then
            say "  note    no termlink at $dest — not creating one (refresh-if-present)"
            continue
        fi
        install_termlink_to "$dest" "$want"
    done
}

head2 "4. termlink binary — every copy on this host (T-3287, T-3289)"
install_termlink

# ---------------------------------------------------------------------------
# ACTION 5 — restart the local hub onto the installed binary (T-3290)
#
# Installing a binary does not change a running process. The hub kept serving
# 0.12.13 from a replaced file (/proc/<pid>/exe -> "... (deleted)") while every
# install path moved on; preflight check 5 flags exactly this. Restart ONLY
# through the systemd unit (G-070: a detached hub escapes supervision).
#
# Safe by measurement: runtime_dir is /var/lib/termlink (disk-backed), so the
# secret and TLS cert persist (persist-if-present); sessions are rediscovered
# from their registration files (supervisor.rs reads sessions_dir), not held in
# hub memory. Lost on restart: in-memory cv_index (repopulates within one
# heartbeat), dedupe LRU, rate/governor counters; TCP peers reconnect; posts
# during the gap go to each sender's offline queue.
# Idempotent: skipped when the running MainPID's exe IS the installed binary.
# Verified after: unit active, NEW MainPID, its exe is the installed path and not
# "(deleted)", `termlink hub status` reports that pid running, and hub.secret +
# hub.cert.pem hashes are byte-identical to before (a change = PL-021 rotation,
# every peer would need re-auth) — any miss is FAILED.
# Seams (fixtures only): RUNME_HUB_SYSTEMCTL, RUNME_HUB_PROC, RUNME_HUB_RUNTIME_DIR,
# RUNME_HUB_BIN, RUNME_HUB_WAIT_SECS.
# ---------------------------------------------------------------------------
HUB_UNIT=termlink-hub
seam HUB_SYSTEMCTL RUNME_HUB_SYSTEMCTL "systemctl"
seam HUB_PROC RUNME_HUB_PROC "/proc"
seam HUB_RT RUNME_HUB_RUNTIME_DIR "/var/lib/termlink"
HUB_BIN="${RUNME_HUB_BIN:-$(set -- $TL_DESTS; echo "$1")}"
HUB_WAIT="${RUNME_HUB_WAIT_SECS:-30}"

hub_mainpid() { "$HUB_SYSTEMCTL" show -p MainPID --value "$HUB_UNIT" 2>/dev/null; }
hub_exe()     { readlink "$HUB_PROC/$1/exe" 2>/dev/null; }
hub_hash()    { sha256sum "$HUB_RT/$1" 2>/dev/null | cut -d' ' -f1; }

restart_hub() {
    local pid exe
    pid="$(hub_mainpid)"
    if [ -z "$pid" ] || [ "$pid" = "0" ]; then
        say "  skip    $HUB_UNIT is not running under systemd here — nothing to restart"; SKIPPED=$((SKIPPED+1)); return
    fi
    exe="$(hub_exe "$pid")"
    if [ "$exe" = "$HUB_BIN" ]; then
        say "  skip    hub (pid $pid) already runs the installed binary: $exe"; SKIPPED=$((SKIPPED+1)); return
    fi
    if [ "$DRY_RUN" = "1" ]; then
        say "  [DRY]   would restart $HUB_UNIT (pid $pid runs '${exe:-unknown}', installed is $HUB_BIN)"; DONE=$((DONE+1)); return
    fi
    local sec0 crt0; sec0="$(hub_hash hub.secret)"; crt0="$(hub_hash hub.cert.pem)"
    if [ -z "$sec0" ] || [ -z "$crt0" ]; then
        say "  FAILED  cannot read $HUB_RT/hub.secret or hub.cert.pem — refusing to restart blind"; FAILED=$((FAILED+1)); return
    fi
    say "  restart $HUB_UNIT (pid $pid runs '$exe')"
    if ! "$HUB_SYSTEMCTL" restart "$HUB_UNIT"; then
        say "  FAILED  systemctl restart $HUB_UNIT failed"; FAILED=$((FAILED+1)); return
    fi
    local i npid="" nexe="" active=""
    for i in $(seq 1 "$HUB_WAIT"); do
        active="$("$HUB_SYSTEMCTL" is-active "$HUB_UNIT" 2>/dev/null)"; npid="$(hub_mainpid)"
        if [ "$active" = "active" ] && [ -n "$npid" ] && [ "$npid" != "0" ] && [ "$npid" != "$pid" ]; then break; fi
        sleep 1
    done
    nexe="$(hub_exe "$npid")"
    local problems=""
    [ "$active" = "active" ] || problems="$problems unit=${active:-unknown};"
    { [ -n "$npid" ] && [ "$npid" != "$pid" ]; } || problems="$problems mainpid-unchanged($pid);"
    [ "$nexe" = "$HUB_BIN" ] || problems="$problems exe='${nexe:-none}';"
    [ "$(hub_hash hub.secret)" = "$sec0" ] || problems="$problems hub.secret-CHANGED(peers-need-reauth);"
    [ "$(hub_hash hub.cert.pem)" = "$crt0" ] || problems="$problems hub.cert-CHANGED(tofu-drift);"
    # T-3450: systemd reports the unit active before the new hub has written its pidfile and
    # socket, so one read of `hub status` raced the startup (a false FAILED on 2026-10-17 at
    # load ~30). Re-read within the same bounded window; fail only if it never reads running.
    local st="" ok_status=0
    for i in $(seq 1 "$HUB_WAIT"); do
        st="$("$HUB_BIN" hub status --json 2>/dev/null)"
        case "$st" in *"\"pid\":$npid"*'"status":"running"'*|*'"status":"running"'*"\"pid\":$npid"*) ok_status=1; break ;; esac
        sleep 1
    done
    [ "$ok_status" = 1 ] || problems="$problems hub-status-not-running-as-$npid;"
    if [ -z "$problems" ]; then
        say "  OK      restarted and verified: pid $pid -> $npid, exe $nexe ($("$HUB_BIN" --version 2>/dev/null)); secret + cert unchanged"; DONE=$((DONE+1))
    else
        say "  FAILED  hub restart verification:$problems"; FAILED=$((FAILED+1))
    fi
}

# ---------------------------------------------------------------------------
# ACTION 5a — hub systemd unit (T-3299)
#
# termlink-hub.service runs with ProtectSystem=strict, so everything outside
# ReadWritePaths is read-only in the hub's namespace. T-3295 made the hub sweep
# the legacy /tmp/termlink-0 session pool, but /tmp was read-only for it: every
# delete failed with EROFS and, until T-3299, nothing said so. The tracked unit
# now adds `-/tmp/termlink-0` to ReadWritePaths. Installed when it differs;
# daemon-reload + restart; verified in the hub's OWN mount table
# (/proc/<pid>/mountinfo) that the pool is mounted read-write, and the unit is
# active. Idempotent: identical unit => skip.
# Seams (fixtures only): RUNME_HUB_UNIT_SRC, RUNME_HUB_UNIT_DEST, RUNME_HUB_RW_PATH
# (+ RUNME_HUB_SYSTEMCTL / RUNME_HUB_PROC from action 5).
# ---------------------------------------------------------------------------
seam HUB_UNIT_SRC RUNME_HUB_UNIT_SRC "$PROJECT_ROOT/.context/systemd/termlink-hub.service"
seam HUB_UNIT_DEST RUNME_HUB_UNIT_DEST "/etc/systemd/system/termlink-hub.service"
seam HUB_RW_PATH RUNME_HUB_RW_PATH "/tmp/termlink-0"

install_hub_unit() {
    if [ ! -f "$HUB_UNIT_SRC" ]; then
        say "  FAILED  tracked unit missing: $HUB_UNIT_SRC"; FAILED=$((FAILED+1)); return
    fi
    if cmp -s "$HUB_UNIT_SRC" "$HUB_UNIT_DEST"; then
        say "  skip    unit already installed and identical: $HUB_UNIT_DEST"; SKIPPED=$((SKIPPED+1)); return
    fi
    if [ "$DRY_RUN" = "1" ]; then
        say "  [DRY]   would install $HUB_UNIT_SRC -> $HUB_UNIT_DEST, daemon-reload, restart $HUB_UNIT"; DONE=$((DONE+1)); return
    fi
    local old_pid; old_pid="$(hub_mainpid)"
    if ! install -m 644 "$HUB_UNIT_SRC" "$HUB_UNIT_DEST"; then
        say "  FAILED  could not install $HUB_UNIT_DEST"; FAILED=$((FAILED+1)); return
    fi
    "$HUB_SYSTEMCTL" daemon-reload
    if ! "$HUB_SYSTEMCTL" restart "$HUB_UNIT"; then
        say "  FAILED  systemctl restart $HUB_UNIT failed after installing the unit"; FAILED=$((FAILED+1)); return
    fi
    local i pid="" active=""
    for i in $(seq 1 "$HUB_WAIT"); do
        active="$("$HUB_SYSTEMCTL" is-active "$HUB_UNIT" 2>/dev/null)"; pid="$(hub_mainpid)"
        [ "$active" = "active" ] && [ -n "$pid" ] && [ "$pid" != "0" ] && [ "$pid" != "$old_pid" ] && break
        sleep 1
    done
    local problems=""
    cmp -s "$HUB_UNIT_SRC" "$HUB_UNIT_DEST" || problems="$problems installed-unit-differs;"
    [ "$active" = "active" ] || problems="$problems unit=${active:-unknown};"
    if [ -d "$HUB_RW_PATH" ]; then
        awk -v p="$HUB_RW_PATH" '$5==p && $6 ~ /^rw/ {f=1} END{exit !f}' "$HUB_PROC/$pid/mountinfo" 2>/dev/null \
            || problems="$problems $HUB_RW_PATH-not-writable-in-hub-namespace;"
    fi
    if [ -z "$problems" ]; then
        say "  OK      unit installed and verified: $HUB_UNIT active (pid $pid); $HUB_RW_PATH read-write in the hub's namespace"; DONE=$((DONE+1))
    else
        say "  FAILED  hub unit verification:$problems"; FAILED=$((FAILED+1))
    fi
}

head2 "5a. Hub systemd unit (T-3299) — legacy session pool writable"
install_hub_unit

head2 "5. Local hub onto the installed binary (T-3290)"
restart_hub

# ---------------------------------------------------------------------------
# ACTION 6 — upgrade remote fleet hubs we have a foothold on (T-3290; operator
# authorized forced upgrades of .122 and .121, 2026-09-30)
#
# runme's install actions only touch THIS host. .122 (ring20-management) served
# 0.11.1411. It is upgraded through scripts/fleet-deploy-binary.sh (T-1420):
# stream the musl-static build over one of .122's remote-exec sessions and
# --probe that it executes there (PL-100). The swap then follows how the hub is
# supervised: if the host has a termlink-hub.service, install over the unit's
# ExecStart binary and `systemctl restart` (G-070; the first run found .122 had
# moved from a watchdog to systemd, and fleet-deploy-binary's guard rightly
# refused --swap-restart); otherwise --swap-restart for a watchdog-launched hub.
# .121 (ring20-dashboard) joined on 2026-10-02 (T-3323): it now has a root
# remote-exec session, and its hub runs detached (no systemd unit), so it takes the
# --swap-restart path. That path relaunches with the RUNNING hub's runtime dir and
# arguments, read from /proc, so the secret and cert stay where they are (PL-021).
# .132 (greenfield) joined on 2026-10-13 (operator: "upgrade 132 too"): it served
# 0.12.61; root remote-exec sessions exist there, and its hub runs detached
# (`hub start --tcp 0.0.0.0:9100`, no systemd unit), so it also takes --swap-restart.
# .141 (laptop-141) is deliberately NOT listed: a customer machine, operator ruling.
#
# Build cost (T-3292): the musl build runs only when some hub is behind. A hub is
# current if it serves the version of any current-code build (host or musl).
# The 0.11.679 musl that sat in target/ (shipping it would have DOWNGRADED .122)
# has no build record, so it is always rebuilt before anything ships.
# Verified after: fleet doctor status ok (the HMAC secret still authenticates) AND
# hub_version == the deployed musl version; `tofu verify` exits 0 (cert unchanged).
# Seams (fixtures only): RUNME_FLEET_HUBS, RUNME_MUSL_SRC, RUNME_FLEET_DEPLOY,
# RUNME_FLEET_DOCTOR, RUNME_FLEET_REMOTE, RUNME_TOFU, RUNME_FLEET_WAIT_SECS.
# ---------------------------------------------------------------------------
# greenfield (.132) was out between its first swap (2026-10-13: the hub moved to a
# fresh runtime dir, new cert + secret) and T-3437, which resolves the running
# hub's real dir from its own environment, refuses to stop it when that cannot be
# confirmed, and fails the deploy if the cert changes. Back in from 2026-10-14.
FLEET_HUBS="${RUNME_FLEET_HUBS:-ring20-management ring20-dashboard greenfield}"
seam MUSL_SRC RUNME_MUSL_SRC "$PROJECT_ROOT/target/x86_64-unknown-linux-musl/local-fast/termlink"
seam FLEET_DEPLOY RUNME_FLEET_DEPLOY "bash $PROJECT_ROOT/scripts/fleet-deploy-binary.sh"
seam FLEET_DOCTOR RUNME_FLEET_DOCTOR "termlink fleet doctor --json"
seam FLEET_REMOTE RUNME_FLEET_REMOTE ""
seam TOFU RUNME_TOFU "termlink tofu verify"
FLEET_WAIT="${RUNME_FLEET_WAIT_SECS:-90}"
STAGED=/tmp/termlink.new

fleet_hub_field() {  # <hub-name> <field> -> value from fleet doctor, empty if absent
    timeout 60 $FLEET_DOCTOR 2>/dev/null | python3 -c '
import json,sys
try: d=json.load(sys.stdin)
except Exception: sys.exit(0)
for h in d.get("hubs",[]):
    if h.get("hub")==sys.argv[1]: print(h.get(sys.argv[2]) or ""); break
' "$1" "$2"
}

fleet_remote() {  # <hub> <shell command> -> stdout of the command run on that hub's host
    if [ -n "$FLEET_REMOTE" ]; then $FLEET_REMOTE "$1" "$2"; return; fi
    local sid; sid="$(timeout 30 termlink remote list "$1" 2>/dev/null | awk 'NR==3 {print $1}')"
    [ -n "$sid" ] || return 3
    timeout 70 termlink remote exec --timeout 60 "$1" "$sid" "$2"
}

upgrade_fleet_hubs() {
    local host_ver musl_ver="" hub have addr pending=""
    host_ver="$("$TL_SRC" --version 2>/dev/null | awk '{print $2}')"
    build_is_current musl "$MUSL_SRC" && musl_ver="$("$MUSL_SRC" --version 2>/dev/null | awk '{print $2}')"
    # Which hubs are behind? A hub is current if it serves the version of ANY
    # current-code build (host or musl): across docs-only commits the two version
    # strings can differ while the code is identical, and treating that as "behind"
    # would redeploy on every run.
    for hub in $FLEET_HUBS; do
        have="$(fleet_hub_field "$hub" hub_version)"
        if [ -n "$have" ] && { [ "$have" = "$host_ver" ] || [ "$have" = "$musl_ver" ]; }; then
            say "  skip    $hub already serves $have (current code)"; SKIPPED=$((SKIPPED+1))
        else
            pending="$pending $hub"
        fi
    done
    [ -n "$pending" ] || return 0
    # Only now is the musl build worth its cost (T-3292). A musl binary with no
    # build record — like the 0.11.679 one in target/ — is never "current", so the
    # downgrade trap stays closed: it is rebuilt before anything ships.
    cached_build musl "$MUSL_SRC" build --profile local-fast --target x86_64-unknown-linux-musl -p termlink || return
    local want; want="$("$MUSL_SRC" --version 2>/dev/null | awk '{print $2}')"
    if [ -z "$want" ]; then
        if [ "$DRY_RUN" = "1" ]; then want="(to be built)"; else
            say "  FAILED  musl binary $MUSL_SRC does not run — nothing shipped"; FAILED=$((FAILED+1)); return; fi
    fi
    local i st unitbin out
    for hub in $pending; do
        have="$(fleet_hub_field "$hub" hub_version)"; addr="$(fleet_hub_field "$hub" address)"
        if [ -z "$addr" ]; then
            say "  FAILED  $hub not found or unreachable in fleet doctor — cannot upgrade"; FAILED=$((FAILED+1)); continue
        fi
        if [ "$DRY_RUN" = "1" ]; then
            say "  [DRY]   would stage musl $want on $hub ($addr, now ${have:-unknown}), probe it, then restart its hub (systemd unit if present)"; DONE=$((DONE+1)); continue
        fi
        say "  deploy  $hub ($addr): ${have:-unknown} -> $want"
        # Stage + probe only; the swap is chosen below by how the hub is supervised.
        if ! $FLEET_DEPLOY "$hub" --binary "$MUSL_SRC" --dst "$STAGED" --probe >/tmp/.runme-deploy-"$hub".log 2>&1; then
            say "  FAILED  staging on $hub failed (see /tmp/.runme-deploy-$hub.log)"; FAILED=$((FAILED+1)); continue
        fi
        unitbin="$(fleet_remote "$hub" 'systemctl cat termlink-hub 2>/dev/null | sed -n "s/^ExecStart=\([^ ]*\).*/\1/p" | head -1' 2>/dev/null | tr -d '\r' | tail -1)"
        if [ -n "$unitbin" ]; then
            # G-070: a systemd-supervised hub is restarted THROUGH its unit. The
            # restart is delayed and detached so this exec (which rides on that
            # hub) returns before the hub goes down.
            say "  swap    $hub runs under termlink-hub.service ($unitbin): install + systemctl restart"
            out="$(fleet_remote "$hub" "install -m 755 $STAGED $unitbin && (setsid sh -c 'sleep 2; systemctl restart termlink-hub' </dev/null >/dev/null 2>&1 &) && echo RUNME-SWAP-OK" 2>&1)"
            case "$out" in *RUNME-SWAP-OK*) : ;; *)
                say "  FAILED  install/restart on $hub did not confirm: $(printf '%s' "$out" | tail -1)"; FAILED=$((FAILED+1)); continue ;; esac
        else
            say "  swap    $hub has no termlink-hub unit: fleet-deploy-binary --swap-restart (watchdog-launched hub)"
            if ! $FLEET_DEPLOY "$hub" --binary "$MUSL_SRC" --dst "$STAGED" --probe --swap-restart >>/tmp/.runme-deploy-"$hub".log 2>&1; then
                say "  FAILED  swap-restart on $hub failed (see /tmp/.runme-deploy-$hub.log)"; FAILED=$((FAILED+1)); continue
            fi
        fi
        for i in $(seq 1 "$FLEET_WAIT"); do
            [ "$(fleet_hub_field "$hub" hub_version)" = "$want" ] && break; sleep 1
        done
        have="$(fleet_hub_field "$hub" hub_version)"; st="$(fleet_hub_field "$hub" status)"
        if [ "$have" = "$want" ] && [ "$st" = "ok" ] && $TOFU "$addr" >/dev/null 2>&1; then
            say "  OK      upgraded and verified: $hub serves $have; fleet doctor ok (secret still authenticates); tofu verify ok (cert unchanged)"; DONE=$((DONE+1))
        else
            say "  FAILED  $hub after deploy: version='${have:-none}' status='${st:-none}' (want $want/ok), or tofu verify failed — see /tmp/.runme-deploy-$hub.log"; FAILED=$((FAILED+1))
        fi
    done
}

head2 "6. Fleet hubs reachable from here (T-3290; .121 added by T-3323, .132 on 2026-10-13)"
upgrade_fleet_hubs

# ---------------------------------------------------------------------------
# ACTION 7 — one-time reap of idle zombie `register --shell` sessions
# (T-3297, T-3291 S3b; operator GO on T-3291 recorded 2026-09-30)
#
# T-3291 measured ~480 idle, detached tmux shells nobody had touched since
# creation (~4 GB). T-3294 now ends a session when its shell ends, but a shell
# left idle at its prompt — the vendored dispatch never exits it (S3a, upstream)
# — stays a zombie. Targets come from scripts/lib/session-zombies.py, the SAME
# detector as the daily session-leak canary, so this terminates exactly what the
# canary flags: a tmux pane, older than 24 h, shell with no child, session not
# attached, launcher not alive. Systemd-supervised agents are never targets.
#
# Safety: every target is listed BEFORE any is touched; right before each signal
# /proc/<pid>/cmdline is re-read and must still be `termlink register --name
# <that name>` (pid reuse guard) — a mismatch is skipped and reported. SIGTERM
# only, never SIGKILL: a post-T-3293 register removes its own files on SIGTERM
# and its tmux pane closes; an older binary dies by default and the hub sweep
# removes what it leaves. A target still alive after the bound is FAILED, not
# escalated. Idempotent: no zombies => skip.
# Seams (fixtures only): RUNME_ZOMBIE_DETECT (command printing detector JSON),
# RUNME_ZOMBIE_PROC (proc root), RUNME_ZOMBIE_KILL (signal command),
# RUNME_ZOMBIE_WAIT_SECS.
# ---------------------------------------------------------------------------
seam ZOMBIE_DETECT RUNME_ZOMBIE_DETECT "python3 $PROJECT_ROOT/scripts/lib/session-zombies.py"
seam ZOMBIE_PROC RUNME_ZOMBIE_PROC "/proc"
seam ZOMBIE_KILL RUNME_ZOMBIE_KILL "kill -TERM"
ZOMBIE_WAIT="${RUNME_ZOMBIE_WAIT_SECS:-10}"

zombie_still_is() {  # <pid> <name> -> 0 if the pid is still that register session
    local cl
    # T-3301: the redirect sits inside a group whose stderr is discarded — a bare
    # `< file 2>/dev/null` reports the failed open BEFORE the 2> applies, which
    # printed one bash error per exited session into the operator's log.
    cl="$( { tr '\0' ' ' < "$ZOMBIE_PROC/$1/cmdline"; } 2>/dev/null )" || return 1
    case "$cl" in *termlink*" register "*"--name $2 "*) return 0 ;; esac
    return 1
}

reap_zombie_sessions() {
    local json list n
    if ! json="$($ZOMBIE_DETECT 2>/dev/null)"; then
        say "  FAILED  zombie detector could not run — nothing reaped"; FAILED=$((FAILED+1)); return
    fi
    list="$(printf '%s' "$json" | python3 -c '
import json,sys
for z in json.load(sys.stdin).get("zombie",[]):
    print(z["pid"], z["name"], z["age_s"]//86400, z.get("tmux_session") or "-")')" || {
        say "  FAILED  zombie detector output unreadable — nothing reaped"; FAILED=$((FAILED+1)); return; }
    n="$(printf '%s' "$list" | grep -c . || true)"
    if [ "$n" = "0" ]; then
        say "  skip    no zombie sessions (idle >24h, detached, shell childless)"; SKIPPED=$((SKIPPED+1)); return
    fi
    say "  targets $n zombie session(s) — listed before any is touched:"
    printf '%s\n' "$list" | while read -r pid name days tm; do
        say "            pid $pid  $name  ${days}d  tmux=$tm"
    done
    if [ "$DRY_RUN" = "1" ]; then
        say "  [DRY]   would SIGTERM these $n session(s) after re-verifying each pid"; DONE=$((DONE+1)); return
    fi
    local pid name days tm i gone=0 skipped=0 stuck=""
    while read -r pid name days tm; do
        if ! zombie_still_is "$pid" "$name"; then
            skipped=$((skipped+1)); say "  skip    pid $pid is no longer register '$name' — not signalled"; continue
        fi
        $ZOMBIE_KILL "$pid" 2>/dev/null || true
        for i in $(seq 1 "$ZOMBIE_WAIT"); do
            zombie_still_is "$pid" "$name" || break
            sleep 1
        done
        if zombie_still_is "$pid" "$name"; then stuck="$stuck $pid($name)"; else gone=$((gone+1)); fi
    done <<< "$list"
    if [ -z "$stuck" ]; then
        local note=""; [ "$skipped" -gt 0 ] && note="; $skipped skipped (pid no longer that session)"
        say "  OK      reaped and verified: $gone session(s) exited on SIGTERM$note"; DONE=$((DONE+1))
    else
        say "  FAILED  still running after SIGTERM (not escalated to SIGKILL):$stuck — $gone exited"; FAILED=$((FAILED+1))
    fi
}

head2 "7. Zombie register sessions — one-time reap (T-3297, T-3291 S3b)"
reap_zombie_sessions

# ---------------------------------------------------------------------------
# ACTION 8 — this project's Claude sessions sign as claude-termlink's own key
# (T-3303; operator GO on T-3302 option B, 2026-10-01)
#
# Every session here signed with the shared host key (d1993c2c3ec44c94), so on
# the wire this agent, pen-agent and 126 other sessions were indistinguishable
# and attribution fell back to unsigned metadata. Setting
# TERMLINK_AGENT_ID=claude-termlink in the project's Claude settings env makes
# its sessions resolve the per-agent key (6738c073bbcc587a), as the systemd
# agents already do with --identity-key. Takes effect for NEW sessions.
# The enforcement-config file .claude/settings.json is protected (B-005) and
# untouched; this merges ONE key into .claude/settings.local.json and verifies
# every other key is unchanged. Idempotent. Both sidecar mailboxes stay watched
# through the transition (T-3303 AC2).
# Seam (fixtures only): RUNME_CLAUDE_LOCAL_SETTINGS.
# ---------------------------------------------------------------------------
seam CLAUDE_LOCAL_SETTINGS RUNME_CLAUDE_LOCAL_SETTINGS "$PROJECT_ROOT/.claude/settings.local.json"

set_agent_identity_env() {
    local f="$CLAUDE_LOCAL_SETTINGS" res
    res="$(python3 - "$f" "$DRY_RUN" <<'EOP'
import json, os, sys, copy
p, dry = sys.argv[1], sys.argv[2] == "1"
d = json.load(open(p)) if os.path.exists(p) else {}
if d.get("env", {}).get("TERMLINK_AGENT_ID") == "claude-termlink":
    print("skip"); sys.exit(0)
if dry:
    print("dry"); sys.exit(0)
before = copy.deepcopy(d)
d.setdefault("env", {})["TERMLINK_AGENT_ID"] = "claude-termlink"
tmp = p + ".runme-tmp"
with open(tmp, "w") as fh:
    json.dump(d, fh, indent=2); fh.write("\n")
os.replace(tmp, p)
after = json.load(open(p))
others_same = {k: v for k, v in after.items() if k != "env"} == {k: v for k, v in before.items() if k != "env"}
env_same = {k: v for k, v in after.get("env", {}).items() if k != "TERMLINK_AGENT_ID"} == before.get("env", {})
print("ok" if after["env"]["TERMLINK_AGENT_ID"] == "claude-termlink" and others_same and env_same else "mismatch")
EOP
)" || res="error"
    case "$res" in
        skip) say "  skip    $f already sets TERMLINK_AGENT_ID=claude-termlink"; SKIPPED=$((SKIPPED+1)) ;;
        dry)  say "  [DRY]   would add env.TERMLINK_AGENT_ID=claude-termlink to $f (all other keys preserved)"; DONE=$((DONE+1)) ;;
        ok)   say "  OK      set and verified: $f env.TERMLINK_AGENT_ID=claude-termlink; every other key unchanged (new sessions sign as 6738c073)"; DONE=$((DONE+1)) ;;
        *)    say "  FAILED  could not set or verify TERMLINK_AGENT_ID in $f ($res)"; FAILED=$((FAILED+1)) ;;
    esac
}

head2 "8. Claude sessions sign with claude-termlink's own key (T-3303, T-3302 GO option B)"
set_agent_identity_env

# ---------------------------------------------------------------------------
# ACTION 9 — bound health:ring20-fedprobe to its last 100 records
# (T-2988; operator ruling SQ-10 a+b, 2026-10-01)
#
# ring20's federation round-trip probe posts ~90-100 tokens a day to this hub,
# retention "forever" (2,777 records when measured), and nothing here reads it.
# The operator ruled: bound it and sweep once. The last 100 posts (~1 day) stay
# readable, so ring20's own round-trip check is unaffected. If ring20 later tags
# its posts with a stable metadata.cv_key, this becomes latest-per-cv-key.
# Idempotent: skips when already "messages 100" with <= 100 records; verifies
# by re-reading retention and count after the sweep. A missing topic is a skip.
# Seam (fixtures only): RUNME_FEDPROBE_TL (the termlink CLI).
# ---------------------------------------------------------------------------
seam FEDPROBE_TL RUNME_FEDPROBE_TL "termlink"
FEDPROBE_TOPIC="health:ring20-fedprobe"
FEDPROBE_KEEP=100

fedprobe_state() {  # -> "<kind> <value> <count>" or "missing"
    $FEDPROBE_TL channel info "$FEDPROBE_TOPIC" --json 2>/dev/null | python3 -c '
import json, sys
try:
    d = json.load(sys.stdin)
except Exception:
    print("missing"); sys.exit(0)
r = d.get("retention") or {}
if "count" not in d:
    print("missing"); sys.exit(0)
print(r.get("kind", "?"), r.get("value", 0), d.get("count", 0))'
}

bound_fedprobe() {
    local st kind value count
    st="$(fedprobe_state)"
    if [ "$st" = "missing" ] || [ -z "$st" ]; then
        say "  skip    $FEDPROBE_TOPIC not found on the local hub (nothing to bound)"; SKIPPED=$((SKIPPED+1)); return
    fi
    read -r kind value count <<< "$st"
    if [ "$kind" = "messages" ] && [ "$value" = "$FEDPROBE_KEEP" ] && [ "$count" -le "$FEDPROBE_KEEP" ]; then
        say "  skip    $FEDPROBE_TOPIC already keeps the last $FEDPROBE_KEEP ($count records)"; SKIPPED=$((SKIPPED+1)); return
    fi
    if [ "$DRY_RUN" = "1" ]; then
        say "  [DRY]   would set $FEDPROBE_TOPIC retention $kind -> messages $FEDPROBE_KEEP and sweep ($count records now)"; DONE=$((DONE+1)); return
    fi
    $FEDPROBE_TL channel set-retention "$FEDPROBE_TOPIC" --retention "messages:$FEDPROBE_KEEP" >/dev/null 2>&1
    $FEDPROBE_TL channel sweep "$FEDPROBE_TOPIC" >/dev/null 2>&1
    read -r kind value count <<< "$(fedprobe_state)"
    if [ "$kind" = "messages" ] && [ "$value" = "$FEDPROBE_KEEP" ] && [ "${count:-999999}" -le "$FEDPROBE_KEEP" ]; then
        say "  OK      $FEDPROBE_TOPIC now keeps the last $FEDPROBE_KEEP; swept to $count records (verified)"; DONE=$((DONE+1))
    else
        say "  FAILED  $FEDPROBE_TOPIC: after set-retention + sweep it reads '$kind $value $count'"; FAILED=$((FAILED+1))
    fi
}

head2 "9. Bound health:ring20-fedprobe to its last $FEDPROBE_KEEP records (T-2988, SQ-10)"
bound_fedprobe

# ---------------------------------------------------------------------------
# ACTION 10 — restart notify-sidecars that run stale code (T-3325)
#
# A sidecar is a long-lived process: editing scripts/notify-sidecar.sh changes nothing
# for one already running (T-3205). T-3325 changed it so a sidecar wakes only for mail
# addressed to its agent (five-level circuit, operator ruling Q1 = C). Until each
# sidecar restarts, it keeps waking the wrong agent. The supervisor never restarts a
# live sidecar, and a restart posts receipts peers can see and can inject into live
# prompts, so it is an operator action, here.
# Idempotent: skipped when check-stale-sidecar-code reports none stale. Otherwise each
# stale pid gets SIGTERM; one supervisor sweep starts the missing ones from
# notify-sidecar-agents.conf; then the detector runs again. Verified: stale_count is 0
# and at least as many sidecars run current code as were restarted. Fail-closed: an
# unreadable detector result is FAILED, never a skip.
# Seams (fixtures only): RUNME_SIDECAR_DETECT, RUNME_SIDECAR_KILL, RUNME_SIDECAR_SUPERVISOR.
# ---------------------------------------------------------------------------
seam SIDECAR_DETECT RUNME_SIDECAR_DETECT "bash $PROJECT_ROOT/scripts/check-stale-sidecar-code.sh --json"
seam SIDECAR_KILL RUNME_SIDECAR_KILL "kill"
seam SIDECAR_SUPERVISOR RUNME_SIDECAR_SUPERVISOR "bash $PROJECT_ROOT/scripts/notify-sidecar-supervisor.sh --quiet"

sidecar_state() {  # -> "<stale_count> <current_count> <pid:agent,...>" or "error"
    $SIDECAR_DETECT 2>/dev/null | python3 -c '
import json, sys
try:
    d = json.load(sys.stdin)
    s = d["stale_count"]; c = d["current_count"]
except Exception:
    print("error"); sys.exit(0)
print(s, c, ",".join("%s:%s" % (x.get("pid"), x.get("agent_id")) for x in d.get("stale", [])) or "-")'
}

restart_stale_sidecars() {
    local st stale current list pid agents="" n=0 i
    st="$(sidecar_state)"
    if [ "$st" = "error" ] || [ -z "$st" ]; then
        say "  FAILED  check-stale-sidecar-code gave no readable result; cannot tell which sidecars are stale"; FAILED=$((FAILED+1)); return
    fi
    read -r stale current list <<< "$st"
    if [ "$stale" = "0" ]; then
        say "  skip    all $current notify-sidecar(s) already run the current code"; SKIPPED=$((SKIPPED+1)); return
    fi
    for i in ${list//,/ }; do agents="$agents ${i#*:}"; done
    if [ "$DRY_RUN" = "1" ]; then
        say "  [DRY]   would restart $stale stale sidecar(s):$agents"; DONE=$((DONE+1)); return
    fi
    for i in ${list//,/ }; do
        pid="${i%%:*}"
        $SIDECAR_KILL "$pid" 2>/dev/null && n=$((n+1))
        for _ in 1 2 3 4 5 6 7 8 9 10; do kill -0 "$pid" 2>/dev/null || break; sleep 1; done
    done
    $SIDECAR_SUPERVISOR >/dev/null 2>&1
    sleep 2
    read -r stale current list <<< "$(sidecar_state)"
    if [ "$stale" = "0" ] && [ "${current:-0}" -ge "$n" ] 2>/dev/null; then
        say "  OK      restarted $n sidecar(s) onto current code:$agents (verified: 0 stale, $current current)"; DONE=$((DONE+1))
    else
        say "  FAILED  after restart: stale=$stale current=$current (wanted 0 stale, >= $n current). Check: bash scripts/check-stale-sidecar-code.sh"; FAILED=$((FAILED+1))
    fi
}

head2 "10. Restart notify-sidecars onto the current code (T-3325 addressing)"
restart_stale_sidecars

# ---------------------------------------------------------------------------
# ACTION 11 — schedule the hourly vector-index reindex (T-3336)
#
# The vector database behind `fw ask` / RAG stopped at T-2508 (early August): this
# project's .context/cron-registry.yaml never had `index-reindex-hourly`, and the
# reindex verb could not import the framework's web/ module in a vendored checkout
# anyway (fixed in T-3336). Installs the registry crontab via `fw cron install`
# (its dry run adds exactly this one line). Idempotent: skipped when the installed
# file already carries `fw" index reindex`. Verified on disk after installing.
# Seams (fixtures only): RUNME_AUDIT_CRON_FILE, RUNME_FW.
# ---------------------------------------------------------------------------
seam AUDIT_CRON_FILE RUNME_AUDIT_CRON_FILE "/etc/cron.d/agentic-audit-termlink"

schedule_reindex() {
    if grep -q 'index reindex' "$AUDIT_CRON_FILE" 2>/dev/null; then
        say "  skip    $AUDIT_CRON_FILE already schedules the hourly vector reindex"; SKIPPED=$((SKIPPED+1)); return
    fi
    if [ "$DRY_RUN" = "1" ]; then
        say "  [DRY]   would run: fw cron install (adds the hourly 'fw index reindex' line to $AUDIT_CRON_FILE)"; DONE=$((DONE+1)); return
    fi
    (cd "$PROJECT_ROOT" && $FW cron install >/dev/null 2>&1)
    if grep -q 'index reindex' "$AUDIT_CRON_FILE" 2>/dev/null; then
        say "  OK      hourly vector reindex installed and verified in $AUDIT_CRON_FILE"; DONE=$((DONE+1))
    else
        say "  FAILED  fw cron install ran but $AUDIT_CRON_FILE has no 'index reindex' line"; FAILED=$((FAILED+1))
    fi
}

head2 "11. Schedule the hourly vector-index reindex (T-3336)"
schedule_reindex

# ---------------------------------------------------------------------------
# ACTION 12 — stop the stray second hub at /tmp/termlink-0 (T-3343, ruling B)
#
# T-3340: an agent's `hub restart` without TERMLINK_RUNTIME_DIR left a second hub
# running at /tmp/termlink-0 beside the canonical /var/lib/termlink one; clients
# without the env var used it, and mail posted there never reached its readers. The
# operator ruled B: rescue the stranded mail first (done, T-3343 report), then stop.
# Preconditions, each FAILED (never skipped) when unmet: the pid is a termlink hub;
# every installed termlink carries the T-3340 guard (`hub start --allow-second-hub`
# exists), so nothing can recreate the stray hub; the rescue report exists.
# Graceful `hub stop` against the stray dir; no SIGKILL. Verified: the pid is gone AND
# a client without TERMLINK_RUNTIME_DIR now resolves the canonical hub's pid.
# Idempotent: skipped when no live hub is recorded in the stray dir.
# Seams (fixtures only): RUNME_STRAY_HUB_DIR, RUNME_CANON_HUB_DIR, RUNME_STRAY_TL,
#   RUNME_STRAY_ALIVE, RUNME_STRAY_PROC_ROOT, RUNME_STRAY_GUARD_BINS, RUNME_STRAY_RESCUE_REPORT.
# ---------------------------------------------------------------------------
seam STRAY_DIR RUNME_STRAY_HUB_DIR "/tmp/termlink-0"
seam CANON_DIR RUNME_CANON_HUB_DIR "/var/lib/termlink"
seam STRAY_TL RUNME_STRAY_TL "termlink"
seam STRAY_ALIVE RUNME_STRAY_ALIVE "kill -0"
seam STRAY_PROC RUNME_STRAY_PROC_ROOT "/proc"
seam STRAY_GUARD_BINS RUNME_STRAY_GUARD_BINS "${HOME:-/root}/.cargo/bin/termlink /usr/local/bin/termlink ${HOME:-/root}/.local/bin/termlink"
seam STRAY_REPORT RUNME_STRAY_RESCUE_REPORT "$PROJECT_ROOT/docs/reports/T-3343-stray-hub-rescue.md"

stop_stray_hub() {
    local pid canon_pid b missing="" now_pid i
    pid="$(tr -cd '0-9' 2>/dev/null < "$STRAY_DIR/hub.pid")"
    if [ -z "$pid" ] || ! $STRAY_ALIVE "$pid" 2>/dev/null; then
        say "  skip    no live hub recorded in $STRAY_DIR (already stopped)"; SKIPPED=$((SKIPPED+1)); return
    fi
    if ! tr '\0' ' ' < "$STRAY_PROC/$pid/cmdline" 2>/dev/null | grep -q 'termlink.*hub'; then
        say "  FAILED  pid $pid in $STRAY_DIR/hub.pid is not a termlink hub process; refusing to stop it"; FAILED=$((FAILED+1)); return
    fi
    for b in $STRAY_GUARD_BINS; do
        [ -x "$b" ] || continue
        "$b" hub start --help 2>/dev/null | grep -q -- '--allow-second-hub' || missing="$missing $b"
    done
    if [ -n "$missing" ]; then
        say "  FAILED  installed termlink lacks the T-3340 guard:$missing — run action 4 first (re-run this script)"; FAILED=$((FAILED+1)); return
    fi
    if [ ! -s "$STRAY_REPORT" ]; then
        say "  FAILED  rescue report $STRAY_REPORT missing — stranded mail must be rescued first (T-3343)"; FAILED=$((FAILED+1)); return
    fi
    canon_pid="$(tr -cd '0-9' 2>/dev/null < "$CANON_DIR/hub.pid")"
    if [ "$DRY_RUN" = "1" ]; then
        say "  [DRY]   would stop stray hub pid $pid ($STRAY_DIR); canonical hub is pid ${canon_pid:-?} ($CANON_DIR)"; DONE=$((DONE+1)); return
    fi
    TERMLINK_RUNTIME_DIR="$STRAY_DIR" $STRAY_TL hub stop >/dev/null 2>&1
    for i in $(seq 1 15); do $STRAY_ALIVE "$pid" 2>/dev/null || break; sleep 1; done
    if $STRAY_ALIVE "$pid" 2>/dev/null; then
        say "  FAILED  stray hub pid $pid still alive 15 s after 'hub stop' (no SIGKILL by design; inspect it)"; FAILED=$((FAILED+1)); return
    fi
    now_pid="$(env -u TERMLINK_RUNTIME_DIR $STRAY_TL hub status --json 2>/dev/null | python3 -c 'import json,sys
try: print(json.load(sys.stdin).get("pid",""))
except Exception: print("")')"
    if [ -n "$canon_pid" ] && [ "$now_pid" = "$canon_pid" ]; then
        say "  OK      stray hub pid $pid stopped; a client without TERMLINK_RUNTIME_DIR now resolves the canonical hub pid $canon_pid (verified)"; DONE=$((DONE+1))
    else
        say "  FAILED  stray hub pid $pid stopped, but a client without TERMLINK_RUNTIME_DIR resolves pid '${now_pid:-none}', not the canonical ${canon_pid:-?}"; FAILED=$((FAILED+1))
    fi
}

head2 "12. Stop the stray second hub at /tmp/termlink-0 (T-3343, mail rescued first)"
stop_stray_hub

# ---------------------------------------------------------------------------
# ACTION 13 — install the cron jobs AEF 1.8.3 added (T-3370, operator ruling A)
#
# The re-vendor to AEF 1.8.3 added `sidecar-sweep-5m` to .context/cron-registry.yaml:
# it walks the peer-consult ack ledger and escalates unacknowledged consults along the
# retry ladder. Without the installed line, consults we send are never re-posted or
# escalated. `fw cron install` was measured (dry run, 2026-10-06) to add exactly this
# one line. Idempotent: skipped when the installed file already carries `sidecar sweep`.
# Verified on disk after installing. Seams (fixtures only): RUNME_AUDIT_CRON_FILE, RUNME_FW.
# ---------------------------------------------------------------------------
schedule_sidecar_sweep() {
    if grep -q 'sidecar sweep' "$AUDIT_CRON_FILE" 2>/dev/null; then
        say "  skip    $AUDIT_CRON_FILE already schedules the sidecar ack-ledger sweep"; SKIPPED=$((SKIPPED+1)); return
    fi
    if [ "$DRY_RUN" = "1" ]; then
        say "  [DRY]   would run: fw cron install (adds the 5-minute 'fw sidecar sweep' line to $AUDIT_CRON_FILE)"; DONE=$((DONE+1)); return
    fi
    (cd "$PROJECT_ROOT" && $FW cron install >/dev/null 2>&1)
    if grep -q 'sidecar sweep' "$AUDIT_CRON_FILE" 2>/dev/null; then
        say "  OK      sidecar ack-ledger sweep installed and verified in $AUDIT_CRON_FILE"; DONE=$((DONE+1))
    else
        say "  FAILED  fw cron install ran but $AUDIT_CRON_FILE has no 'sidecar sweep' line"; FAILED=$((FAILED+1))
    fi
}

head2 "13. Install the cron jobs AEF 1.8.3 added: sidecar ack-ledger sweep (T-3370)"
schedule_sidecar_sweep

# ---------------------------------------------------------------------------
# ACTION 14 — record the operator's own Human ticks in the provenance ledger (T-3351)
#
# The operator ticked T-3349 Human AC#1 ("Sign off step 1") in Watchtower on 2026-10-07.
# Watchtower wrote no provenance row (.context/reviews/human-ac-ticks.jsonl did not
# exist), so the pre-push audit FAILs on an "unprovenanced ### Human tick" and every push
# is blocked. Only the operator may record that tick: the framework refuses it under
# agent control. Running it here, from the operator's terminal, is that record.
# The decision is already made (the operator ticked the box); this only records who did.
# Idempotent: skipped when the ledger already holds an operator-ack row for the task.
# Verified on disk. Seams (fixtures only): RUNME_HUMAN_TICKS_LEDGER, RUNME_HUMAN_TICKS_PY.
# ---------------------------------------------------------------------------
HUMAN_TICKS_LEDGER="${RUNME_HUMAN_TICKS_LEDGER:-$PROJECT_ROOT/.context/reviews/human-ac-ticks.jsonl}"
HUMAN_TICKS_PY="${RUNME_HUMAN_TICKS_PY:-$PROJECT_ROOT/.agentic-framework/lib/human_ac_ticks.py}"
# T-3351:1 (accept step-2 residual risks) and T-3388:1 (sign off requirements v0.5.2): ticked by the operator in
# Watchtower on 2026-10-11, which again wrote no provenance row (T-3393).
# T-3291:1 and T-3302:1 ("approve go/no-go decision"): ticked by `fw inception decide` when the operator ruled GO
# (2026-09-30 and 2026-10-01; both files record "Operator ...: GO" with his rationale). That output sat
# uncommitted until T-3451 committed it under the agent's name on 2026-10-18 (T-3460), blocking pushes.
OPERATOR_TICKS="T-3349:1 T-3351:1 T-3388:1 T-3291:1 T-3302:1"   # task:Human-AC-number, space-separated; each one the operator ticked themself

has_operator_ack() { grep -F "\"task\": \"$1\"" "$HUMAN_TICKS_LEDGER" 2>/dev/null | grep -qF '"via": "operator-ack"'; }

ack_operator_ticks() {
    local spec task ac
    for spec in $OPERATOR_TICKS; do
        task="${spec%%:*}"; ac="${spec##*:}"
        if has_operator_ack "$task"; then
            say "  skip    $task Human AC#$ac: your tick is already recorded in $HUMAN_TICKS_LEDGER"; SKIPPED=$((SKIPPED+1)); continue
        fi
        if [ "$DRY_RUN" = "1" ]; then
            say "  [DRY]   would record that you ticked $task Human AC#$ac (human_ac_ticks ack)"; DONE=$((DONE+1)); continue
        fi
        # No env scrubbing: under agent control (CLAUDECODE/AI_AGENT set) the framework refuses, and must.
        (cd "$PROJECT_ROOT" && python3 "$HUMAN_TICKS_PY" ack "$task" --ac "$ac" >/dev/null 2>&1)
        if has_operator_ack "$task"; then
            say "  OK      recorded that you ticked $task Human AC#$ac (verified in $HUMAN_TICKS_LEDGER)"; DONE=$((DONE+1))
        else
            say "  FAILED  human_ac_ticks ack $task --ac $ac ran but $HUMAN_TICKS_LEDGER has no operator-ack row for $task"; FAILED=$((FAILED+1))
        fi
    done
}

head2 "14. Record your own ticks (T-3349 step 1, T-3351 step 2, T-3388 step 1 v0.5.2, T-3291 and T-3302 GO reviews) so pushes are not blocked"
ack_operator_ticks

# ---------------------------------------------------------------------------
# ACTION 15 — current Claude Code for the test user `testdev` (T-3416, arc-011 B14 prep)
#
# B14 runs two real Claude Code sessions as the unprivileged `testdev` user (OQ-3). You
# logged testdev in on 2026-10-12, but its `claude` is an old npm install (2.1.27) while
# the transcript evidence the sidecar relies on was measured on 2.1.296. This installs
# the native Claude Code for testdev with the official installer, run AS testdev, into
# testdev's own home. Nothing of yours is copied: not the binary, not the login.
# Idempotent: skipped when testdev's native claude already reports >= the minimum.
# Verified by running the installed binary's --version after installing.
# Seams (fixtures only): RUNME_TESTDEV_CLAUDE, RUNME_TESTDEV_INSTALL.
# ---------------------------------------------------------------------------
seam TESTDEV_CLAUDE RUNME_TESTDEV_CLAUDE "/home/testdev/.local/bin/claude"
seam TESTDEV_INSTALL RUNME_TESTDEV_INSTALL "su - testdev -c 'curl -fsSL https://claude.ai/install.sh | bash'"
TESTDEV_CLAUDE_MIN="2.1.296"

testdev_claude_version() {  # -> version number, or empty when absent/broken
    [ -x "$TESTDEV_CLAUDE" ] || return 0
    "$TESTDEV_CLAUDE" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1
}
version_at_least() {  # <have> <want> -> 0 when have >= want
    [ -n "$1" ] && [ "$(printf '%s\n%s\n' "$2" "$1" | sort -V | head -1)" = "$2" ]
}

install_testdev_claude() {
    local have
    have="$(testdev_claude_version)"
    if version_at_least "$have" "$TESTDEV_CLAUDE_MIN"; then
        say "  skip    testdev already has Claude Code $have at $TESTDEV_CLAUDE (>= $TESTDEV_CLAUDE_MIN)"; SKIPPED=$((SKIPPED+1)); return
    fi
    if [ "$DRY_RUN" = "1" ]; then
        say "  [DRY]   would install Claude Code for testdev (official installer, run as testdev) into $TESTDEV_CLAUDE; now: ${have:-none}"; DONE=$((DONE+1)); return
    fi
    say "  ...     installing Claude Code for testdev (downloads from claude.ai; ${have:-no native install yet})"
    bash -c "$TESTDEV_INSTALL" 2>&1 | tail -5 | sed 's/^/          /'
    have="$(testdev_claude_version)"
    if version_at_least "$have" "$TESTDEV_CLAUDE_MIN"; then
        say "  OK      testdev now has Claude Code $have at $TESTDEV_CLAUDE (verified by running it)"; DONE=$((DONE+1))
    else
        say "  FAILED  the installer ran but $TESTDEV_CLAUDE reports '${have:-nothing}', need >= $TESTDEV_CLAUDE_MIN"; FAILED=$((FAILED+1))
    fi
}

head2 "15. Current Claude Code for the test user testdev (T-3416, needed for the arc-011 two-agent test)"
install_testdev_claude

# ---------------------------------------------------------------------------
# ACTION 16 — `claude-fw` on PATH becomes AEF's router, not a stale fixed copy (T-3421)
#
# After the 2026-10-12 framework upgrade (T-3420), fw doctor warns that the claude-fw
# on your PATH (/root/.local/bin/claude-fw and /usr/bin/claude-fw) are full copies of
# the old 1.8.9 launcher: claude-fw sessions run that old launcher until it is
# replaced, and every future upgrade would leave them behind again. AEF's
# claude-fw-router is a small file that finds the project you start it in and runs
# THAT project's own, current claude-fw, so it never goes stale. You ruled A.
# For each path that exists and is not already the router: a backup copy is kept in
# .context/working/claude-fw-backups/, then the file is replaced (rm -f first, so a
# symlink is never written through) and checked on disk. A path that does not exist
# is left alone. Takes effect the next time a claude-fw session starts.
# Seams (fixtures only): RUNME_CLAUDE_FW_TARGETS, RUNME_CLAUDE_FW_ROUTER,
# RUNME_CLAUDE_FW_BACKUP_DIR.
# ---------------------------------------------------------------------------
seam CLAUDE_FW_TARGETS RUNME_CLAUDE_FW_TARGETS "/root/.local/bin/claude-fw /usr/bin/claude-fw"
seam CLAUDE_FW_ROUTER RUNME_CLAUDE_FW_ROUTER "$PROJECT_ROOT/.agentic-framework/bin/claude-fw-router"
seam CLAUDE_FW_BACKUP_DIR RUNME_CLAUDE_FW_BACKUP_DIR "$PROJECT_ROOT/.context/working/claude-fw-backups"

install_claude_fw_router() {
    if [ ! -x "$CLAUDE_FW_ROUTER" ]; then
        say "  FAILED  the router $CLAUDE_FW_ROUTER is missing or not executable (re-vendor incomplete?)"; FAILED=$((FAILED+1)); return
    fi
    local t backup
    for t in $CLAUDE_FW_TARGETS; do
        if [ ! -e "$t" ] && [ ! -L "$t" ]; then
            say "  skip    $t does not exist here; nothing to replace"; SKIPPED=$((SKIPPED+1)); continue
        fi
        if [ ! -L "$t" ] && cmp -s "$t" "$CLAUDE_FW_ROUTER"; then
            say "  skip    $t is already the claude-fw router"; SKIPPED=$((SKIPPED+1)); continue
        fi
        if [ "$DRY_RUN" = "1" ]; then
            say "  [DRY]   would replace $t (a fixed copy) with the claude-fw router, keeping a backup"; DONE=$((DONE+1)); continue
        fi
        mkdir -p "$CLAUDE_FW_BACKUP_DIR"
        backup="$CLAUDE_FW_BACKUP_DIR/$(printf '%s' "$t" | tr '/' '_').$(date -u +%Y%m%dT%H%M%SZ)"
        if ! cp -pL "$t" "$backup" 2>/dev/null; then
            say "  FAILED  could not back up $t to $backup; left unchanged"; FAILED=$((FAILED+1)); continue
        fi
        rm -f "$t" && cp "$CLAUDE_FW_ROUTER" "$t" && chmod 755 "$t"
        if [ ! -L "$t" ] && cmp -s "$t" "$CLAUDE_FW_ROUTER" && [ -x "$t" ] && bash -n "$t" 2>/dev/null; then
            say "  OK      $t is now the claude-fw router (checked on disk; old copy kept at $backup)"; DONE=$((DONE+1))
        else
            say "  FAILED  $t is not the router after replacing it; the old copy is at $backup"; FAILED=$((FAILED+1))
        fi
    done
}

head2 "16. claude-fw on your PATH becomes the router, so upgrades can never leave it stale again (T-3421)"
install_claude_fw_router

# ---------------------------------------------------------------------------
# ACTION 17 — arc-011 wave W1: the pair's sidecars (T-3438, row P7)
#
# Every run reports W1 READINESS (read-only: keys present and matching the
# register, hub answers with its id, the installed termlink has the sidecar
# verbs). Provisioning and switch-on run ONLY when the operator's W1 go is
# recorded below (rulings P7-1..P7-4 are made; the go itself is a separate
# operator decision: "each wave move is a runme.sh action the operator runs; the
# agent never switches a receiver itself", ARCH 3). Provisioning writes the key
# register, instance ids, store roots with the production marker, configs,
# per-agent settings, the admin-socket list and SIDECAR_KEEP_RUNNING: on, then
# starts both sidecars under systemd and verifies each in production mode with a
# fresh heartbeat. Idempotent: a second run changes nothing.
# Seams (fixtures only): RUNME_W1_PROVISION, RUNME_W1_GO.
# ---------------------------------------------------------------------------
# The recorded operator go for W1: "<date>|<where it is recorded>". Empty = not given.
ARC011_W1_GO="${RUNME_W1_GO-2026-10-14|T-3426 Decisions: W1 go, operator ruling B}"
seam W1_PROVISION RUNME_W1_PROVISION "bash $PROJECT_ROOT/scripts/arc011-w1-provision.sh"
arc011_w1() {
    local out rc
    out=$($W1_PROVISION --check 2>&1); rc=$?
    printf '%s\n' "$out" | sed 's/^/  /'
    if [ -z "$ARC011_W1_GO" ]; then
        if [ "$rc" = 0 ]; then say "  skip    W1 is ready; waiting for the operator's go (nothing provisioned or started)"
        else say "  skip    W1 not ready yet (see above); waiting for the operator's go in any case"; fi
        SKIPPED=$((SKIPPED+1)); return
    fi
    if [ "$DRY_RUN" = "1" ]; then
        say "  [DRY]   would provision and start the W1 pair (go: $ARC011_W1_GO)"; DONE=$((DONE+1)); return
    fi
    out=$($W1_PROVISION --apply 2>&1); rc=$?
    printf '%s\n' "$out"
    if [ "$rc" = 0 ]; then
        if grep -qE "written and verified|SIDECAR_KEEP_RUNNING (set|added)" <<< "$out"; then
            say "  OK      W1 pair provisioned and running (go: $ARC011_W1_GO)"; DONE=$((DONE+1))
        else
            say "  skip    W1 pair already provisioned and running"; SKIPPED=$((SKIPPED+1))
        fi
    else
        say "  FAILED  W1 provisioning (lines above name each failed step)"; FAILED=$((FAILED+1))
    fi
}
head2 "17. arc-011 W1: the pair's sidecars (T-3438; readiness every run, switch-on only on your go)"
arc011_w1

# ---------------------------------------------------------------------------
# ACTION 18 — cap leftover forever project inboxes at the newest 1000 (T-3443, decision 6 B)
#
# T-3310 made "newest 1000" the default for inbox:* topics; inboxes auto-created before it
# fell through to forever by omission, and the forever-archival canary fires on them.
# Operator ruling 2026-10-15 (decision 6, option B): cap ours now, every other project's
# only after its owner confirms. Sweeping deletes everything older than the newest 1000,
# so each topic is capped ONLY when every known reader is already past the cut point:
# every hub read-receipt, plus this project's own alert marker where one is named. A
# topic with no reader positions, or one reader behind the cut, is REFUSED (FAILED, with
# the reader named) and left untouched — no mail is dropped to quiet a canary.
# Idempotent: a topic already at messages:1000 with <= 1000 records is a skip.
# Verified: retention reads messages 1000 and the count is <= 1000 after the sweep.
# Entries, one per line: "<topic>|<marker file or empty>". Add a project's inbox here only with its
# owner's confirmation recorded in T-3443.
# Seams (fixtures only): RUNME_INBOX_TL, RUNME_INBOX_TOPICS.
# ---------------------------------------------------------------------------
seam INBOX_TL RUNME_INBOX_TL "termlink"
INBOX_KEEP=1000
# 055 confirmed 2026-10-16 (msg 8873e607); 1409-sprind's operator 2026-10-16 (msg 2faa422d); 832's operator 2026-10-16 (msg 026fa4ce). T-3443 Decisions.
# AEF's operator approved 2026-10-16 (msgs d3ed390d, 2443f180). Its inbox carries no hub
# read-receipts, so its reader position is the one AEF stated (read through offset 1461,
# the newest then), kept in a git-tracked file; the cut check reads it like a marker.
INBOX_TOPICS="${RUNME_INBOX_TOPICS-inbox:cacc73ea32b121dd/010-termlink|$PROJECT_ROOT/.context/working/session-alerts/inbox_cacc73ea32b121dd_010_termlink.seen
inbox:cacc73ea32b121dd/055-agentic-fleet-cockpit|
inbox:cacc73ea32b121dd/1409-sprind|
inbox:cacc73ea32b121dd/832-Workflow-designer|
inbox:cacc73ea32b121dd/999-Agentic-Engineering-Framework|$PROJECT_ROOT/.context/checks/inbox-reader-attested/999-Agentic-Engineering-Framework}"

inbox_state() {  # <topic> -> "<kind> <value> <count> <min_receipt|none>" or "missing"
    $INBOX_TL channel info "$1" --json 2>/dev/null | python3 -c '
import json, sys
try:
    d = json.load(sys.stdin)
except Exception:
    print("missing"); sys.exit(0)
if "count" not in d:
    print("missing"); sys.exit(0)
r = d.get("retention") or {}
ups = [x.get("up_to") for x in d.get("receipts") or [] if isinstance(x.get("up_to"), int)]
print(r.get("kind", "?"), r.get("value") or 0, d.get("count", 0), min(ups) if ups else "none")'
}

inbox_newest() {  # <topic> -> newest offset, or empty
    $INBOX_TL channel subscribe "$1" --from-latest --once --json 2>/dev/null | python3 -c '
import json, sys
for line in sys.stdin:
    try:
        print(json.loads(line)["offset"]); break
    except Exception:
        continue'
}

bound_one_inbox() {  # <topic> <marker file or empty>
    local topic="$1" marker="$2" st kind value count minr newest cut m behind=""
    st="$(inbox_state "$topic")"
    if [ "$st" = "missing" ] || [ -z "$st" ]; then
        say "  FAILED  $topic: cannot read it on the local hub (nothing changed)"; FAILED=$((FAILED+1)); return
    fi
    read -r kind value count minr <<< "$st"
    if [ "$kind" = "messages" ] && [ "$value" = "$INBOX_KEEP" ] && [ "$count" -le "$INBOX_KEEP" ]; then
        say "  skip    $topic already keeps the newest $INBOX_KEEP ($count records)"; SKIPPED=$((SKIPPED+1)); return
    fi
    newest="$(inbox_newest "$topic")"
    case "$newest" in ''|*[!0-9]*) say "  FAILED  $topic: cannot read its newest offset (nothing changed)"; FAILED=$((FAILED+1)); return ;; esac
    cut=$((newest - INBOX_KEEP))   # offsets <= cut are deleted by the sweep
    if [ "$cut" -ge 0 ]; then
        if [ "$minr" = "none" ] && [ -z "$marker" ]; then
            say "  FAILED  $topic: refused — no reader position is known, so deleting offsets 0..$cut could drop unread mail (nothing changed)"; FAILED=$((FAILED+1)); return
        fi
        [ "$minr" != "none" ] && [ "$minr" -lt "$cut" ] && behind="hub receipt at $minr"
        if [ -n "$marker" ]; then
            m="$(tr -dc '0-9' < "$marker" 2>/dev/null)"
            if [ -z "$m" ]; then behind="${behind:+$behind, }alert marker unreadable ($marker)"
            elif [ "$m" -lt "$cut" ]; then behind="${behind:+$behind, }alert marker at $m"; fi
        fi
        if [ -n "$behind" ]; then
            say "  FAILED  $topic: refused — a reader is behind the cut point $cut ($behind); nothing changed"; FAILED=$((FAILED+1)); return
        fi
    fi
    if [ "$DRY_RUN" = "1" ]; then
        say "  [DRY]   would set $topic retention $kind -> messages $INBOX_KEEP and sweep offsets <= $cut ($count records; readers at ${minr} / marker ${m:-n/a})"; DONE=$((DONE+1)); return
    fi
    $INBOX_TL channel set-retention "$topic" --retention "messages:$INBOX_KEEP" >/dev/null 2>&1
    $INBOX_TL channel sweep "$topic" >/dev/null 2>&1
    read -r kind value count minr <<< "$(inbox_state "$topic")"
    if [ "$kind" = "messages" ] && [ "$value" = "$INBOX_KEEP" ] && [ "${count:-999999}" -le "$INBOX_KEEP" ]; then
        if [ "$cut" -ge 0 ]; then
            say "  OK      $topic now keeps the newest $INBOX_KEEP; swept to $count records, every reader was past offset $cut (verified)"; DONE=$((DONE+1))
        else
            say "  OK      $topic now keeps the newest $INBOX_KEEP; $count records, nothing deleted (verified)"; DONE=$((DONE+1))
        fi
    else
        say "  FAILED  $topic: after set-retention + sweep it reads '$kind $value $count'"; FAILED=$((FAILED+1))
    fi
}

bound_project_inboxes() {
    local entry
    while IFS= read -r entry; do
        [ -n "$entry" ] || continue
        bound_one_inbox "${entry%%|*}" "${entry#*|}"
    done <<< "$INBOX_TOPICS"
}
head2 "18. Cap leftover forever project inboxes at the newest $INBOX_KEEP, only where every reader is past the cut (T-3443, decision 6 B)"
bound_project_inboxes

# ---------------------------------------------------------------------------
# ACTION 19 — the W1 kill-switch drill on the real pair (T-3426; design 6.2.3.b, R-8.e)
#
# scripts/arc011-w1-killswitch-drill.sh switches delivery off on both W1 sidecars, sends one
# probe, proves the partner refuses to hand it over (delivery-off), switches delivery on and
# proves the same claim now passes (the control), then SIGKILLs the partner's MainPID and
# proves systemd brings back a sidecar that still holds the probe. Delivery is put back on
# on every exit path. Runs once: a pass is recorded in the marker and later runs skip.
# A failed drill is FAILED and is re-run on the next runme.
# Seams (fixtures only): RUNME_W1_DRILL, RUNME_W1_DRILL_DONE.
# ---------------------------------------------------------------------------
seam W1_DRILL RUNME_W1_DRILL "bash $PROJECT_ROOT/scripts/arc011-w1-killswitch-drill.sh"
seam W1_DRILL_DONE RUNME_W1_DRILL_DONE "$PROJECT_ROOT/.context/working/w1-killswitch-drill.passed"
w1_killswitch_drill() {
    local out rc
    if [ -s "$W1_DRILL_DONE" ]; then
        say "  skip    the W1 kill-switch drill already passed ($(head -1 "$W1_DRILL_DONE"))"; SKIPPED=$((SKIPPED+1)); return
    fi
    if [ "$DRY_RUN" = "1" ]; then
        say "  [DRY]   would run the W1 kill-switch drill (delivery off, probe refused, on, control, SIGKILL + restart)"; DONE=$((DONE+1)); return
    fi
    out=$($W1_DRILL 2>&1); rc=$?
    printf '%s\n' "$out" | sed 's/^/  /'
    if [ "$rc" = 0 ]; then
        printf '%s %s\n' "$(date -u +%FT%TZ)" "$(tail -1 <<< "$out")" > "$W1_DRILL_DONE"
        say "  OK      W1 kill-switch drill passed (recorded in $W1_DRILL_DONE)"; DONE=$((DONE+1))
    else
        say "  FAILED  W1 kill-switch drill rc=$rc (the lines above name the step)"; FAILED=$((FAILED+1))
    fi
}
head2 "19. arc-011 W1: kill-switch drill on the pair, with a control and a SIGKILL restart (T-3426)"
w1_killswitch_drill

# ---------------------------------------------------------------------------
# ACTION 20 — attach scoring specs to arc-substrate-fitness's three drivers
# (T-3440 5c, operator ruling A′ 2026-10-18; operator "ja" to attaching after
# seeing the score spread in docs/reports/T-3440-driver-specs/README.md).
#
# Attaching a spec to an approved scoped driver is the operator's act, so it
# runs here, from the operator's own run, with --i-am-human. It needs the T-3472
# fix (approve-driver now stores the spec and attaches to an existing driver).
# Idempotent: a driver whose arc entry already carries a `scoring:` block is
# skipped. Verified on disk: after attaching, the arc YAML must hold a valid
# block the estimator reads, else FAILED.
# Seams (fixtures only): RUNME_ARCS_DIR, RUNME_ARC_ATTACH (command taking
# <arc> <driver> <spec-file>), RUNME_DRIVER_SPECS (spec dir).
# ---------------------------------------------------------------------------
seam ARCS_DIR RUNME_ARCS_DIR "$PROJECT_ROOT/.context/arcs"
seam DRIVER_SPECS RUNME_DRIVER_SPECS "$PROJECT_ROOT/docs/reports/T-3440-driver-specs"
seam ARC_ATTACH RUNME_ARC_ATTACH "arc_attach_real"
arc_attach_real() { "$FW" arc approve-driver "$1" "$2" --scoring-file "$3" --i-am-human; }

arc_driver_has_scoring() {  # <arc> <driver> -> 0 when the stored entry carries a valid scoring block
    FWR="$PROJECT_ROOT/.agentic-framework" python3 - "$ARCS_DIR/$1.yaml" "$2" <<'PY'
import importlib.util, os, sys, yaml
try:
    d = yaml.safe_load(open(sys.argv[1])) or {}
except Exception:
    sys.exit(2)
e = next((s for s in d.get("scoped_drivers") or [] if s.get("name") == sys.argv[2]), None)
b = (e or {}).get("scoring")
if not isinstance(b, dict) or not b:
    sys.exit(1)
p = os.path.join(os.environ["FWR"], "agents/termlink/bvp-estimator/estimator.py")
if os.path.isfile(p):
    s = importlib.util.spec_from_file_location("e", p); m = importlib.util.module_from_spec(s); s.loader.exec_module(m)
    sys.exit(1 if m.validate_scoring_spec(b) else 0)
sys.exit(0)
PY
}

# arc-011's three were added on the operator's second "ja" (2026-10-19), after the
# narrowed specs' spread (docs/reports/T-3440-driver-specs/README.md).
DRIVER_SPEC_PAIRS="arc-substrate-fitness:AS_COORDINATION_TRUTH arc-substrate-fitness:AS_FAILURE_OBSERVABILITY arc-substrate-fitness:AS_RESOURCE_FOOTPRINT arc-011:AS_DELIVERY_TRUTH arc-011:AS_FAILURE_SURFACED arc-011:AS_OPERATOR_SIMPLICITY"
attach_driver_specs() {
    local pair arc d spec
    for pair in $DRIVER_SPEC_PAIRS; do
        arc="${pair%%:*}"; d="${pair#*:}"
        spec="$DRIVER_SPECS/$d.yaml"
        if arc_driver_has_scoring "$arc" "$d"; then
            say "  skip    $arc $d already carries a scoring spec"; SKIPPED=$((SKIPPED+1)); continue
        fi
        if [ ! -f "$spec" ]; then
            say "  FAILED  $arc $d: spec file missing ($spec)"; FAILED=$((FAILED+1)); continue
        fi
        if [ "$DRY_RUN" = "1" ]; then
            say "  [DRY]   would attach $spec to $arc $d"; DONE=$((DONE+1)); continue
        fi
        if ! $ARC_ATTACH "$arc" "$d" "$spec" >/tmp/.runme-attach-"$d".log 2>&1; then
            say "  FAILED  attaching $d refused: $(tail -1 /tmp/.runme-attach-"$d".log)"; FAILED=$((FAILED+1)); continue
        fi
        if arc_driver_has_scoring "$arc" "$d"; then
            say "  OK      attached and verified: $arc $d now carries a valid scoring spec"; DONE=$((DONE+1))
        else
            say "  FAILED  $arc $d: attach reported success but no valid scoring block is stored"; FAILED=$((FAILED+1))
        fi
    done
}

head2 "20. Scoring specs for the value drivers of arc-substrate-fitness and arc-011 (T-3440 5c, T-3472)"
attach_driver_specs

# ---------------------------------------------------------------------------
# Verification — the project's own drift checker is the arbiter, not this script.
# Using the repo's existing check rather than a bespoke one means this cannot
# quietly disagree with what `fw audit` will say five minutes from now.
# ---------------------------------------------------------------------------
# ---------------------------------------------------------------------------
# PENDING DECISIONS — human authority required
#
# LISTED by default, never executed. That distinction is the whole point: a Tier 0
# gate exists to require a human, so a script recording decisions on its own would
# be precisely the laundering the gate prevents. Dispatching them to a TermLink
# peer would be the same thing by another route — a peer doing what was blocked
# here bypasses the operator's decision rather than honouring it.
#
# Recording one is deliberate and per-item:
#     /opt/termlink/runme.sh --decide T-XXXX=go
# The operator names BOTH the item and the verdict. No default verdict, no batch.
# ---------------------------------------------------------------------------
# Pending (undecided) human decisions, space-separated IDs. Empty = none pending.
# History: T-3055 was listed here and has since been decided and completed.
# RUNME_TEST_DECISIONS is a FIXTURE seam, never set by hand.
PENDING_DECISIONS="${RUNME_TEST_DECISIONS:-}"
decision_ids() { echo "$PENDING_DECISIONS"; }

decision_blurb() {
    case "$1" in
        *) echo "(no description)" ;;
    esac
}

if [ -n "$DECIDE" ]; then
    head2 "Recording decision"
    d_id="${DECIDE%%=*}"; d_verdict="${DECIDE#*=}"
    if [ "$d_id" = "$DECIDE" ] || [ -z "$d_verdict" ]; then
        say "  FAILED  --decide needs <ID>=<verdict>, e.g. T-1234=go"; exit 2
    fi
    if ! decision_ids | tr " " "\n" | grep -qx -- "$d_id"; then
        say "  FAILED  unknown decision id '$d_id'. Pending: $(decision_ids)"; exit 2
    fi
    case "$d_verdict" in
        go|no-go|defer) : ;;
        *) say "  FAILED  verdict must be go, no-go or defer (got '$d_verdict')"; exit 2 ;;
    esac
    if [ "$DRY_RUN" = "1" ]; then
        say "  [DRY]   would record $d_id = $d_verdict"
    elif "$FW" inception decide "$d_id" "$d_verdict" --rationale "Operator decision via runme.sh --decide $DECIDE"; then
        say "  OK      recorded $d_id = $d_verdict"; DONE=$((DONE + 1))
    else
        say "  FAILED  fw refused the decision (output above)"; FAILED=$((FAILED + 1))
    fi
else
    head2 "Pending decisions (human authority — nothing here runs by default)"
    [ -z "$(decision_ids)" ] && say "  none pending"
    for d in $(decision_ids); do
        say "  $d  $(decision_blurb "$d" | head -1)"
        decision_blurb "$d" | tail -n +2 | sed "s/^/    /"
        say "    -> $PROJECT_ROOT/runme.sh --decide $d=go"
    done
fi

# ---------------------------------------------------------------------------
# OPT-IN — disable the purpose-mismatch plugins (T-3055)
# Never part of a default run: a recommendation, not a repair, and unapproved.
# ---------------------------------------------------------------------------
if [ "$DISABLE_MISMATCH" = "1" ]; then
    head2 "Disabling purpose-mismatch plugins (measured 1,835 always-on tok)"
    for pl in chrome-devtools-mcp modern-web-guidance superdesign frontend-design; do
        if [ "$DRY_RUN" = "1" ]; then
            say "  [DRY]   would disable $pl"
        elif claude plugin disable "$pl" >/dev/null 2>&1; then
            say "  OK      disabled $pl"; DONE=$((DONE + 1))
        else
            say "  FAILED  could not disable $pl"; FAILED=$((FAILED + 1))
        fi
    done
fi

head2 "Verification"
if [ "$DRY_RUN" = "1" ]; then
    say "  [DRY]   skipped (nothing was changed)"
else
    if bash scripts/check-cron-install-drift.sh > /tmp/.runme-drift 2>&1; then
        say "  OK      check-cron-install-drift.sh reports no uninstalled crontabs"
    else
        # Scope the verdict to what THIS script promised. The drift checker
        # arbitrates over every git-tracked crontab, so an unrelated uninstalled
        # one would otherwise make runme.sh report failure for work it never
        # undertook — and an operator script that cries wolf gets ignored, which
        # is the same fate as the guards this repo keeps having to rescue.
        ours=0
        for n in "${INSTALLED_NAMES[@]}"; do
            grep -q -- "$n" /tmp/.runme-drift && ours=1
        done
        if [ "$ours" = "1" ]; then
            say "  FAILED  a crontab THIS script installed is still reported as drifted:"
            sed 's/^/          /' /tmp/.runme-drift | head -12
            FAILED=$((FAILED+1))
        else
            say "  OK      none of this script's crontabs are drifted"
            say "  note    other crontabs are uninstalled — NOT this script's payload,"
            say "          reported so it is visible rather than hidden:"
            grep -E "MISSING|UNINSTALLED" /tmp/.runme-drift | sed 's/^/          /' | head -6
        fi
    fi
fi

head2 "Summary"
say "  $DONE done, $SKIPPED already-done, $FAILED failed"
if [ "$FAILED" -gt 0 ]; then
    say ""
    say "  Something did not verify. Nothing here is destructive and it is safe to"
    say "  re-run, but report the FAILED lines rather than assuming it took."
    exit 1
fi
[ "$DRY_RUN" = "1" ] && say "  (dry run — nothing was changed)"
exit 0
