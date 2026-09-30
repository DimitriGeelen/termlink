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
CRON_DIR="${RUNME_CRON_DIR:-/etc/cron.d}"
# Keep the drift checker pointed at the same place, or the verification step would
# arbitrate against a directory this run never touched.
[ -n "${RUNME_CRON_DIR:-}" ] && export CRON_DRIFT_INSTALLED_DIR="$RUNME_CRON_DIR"
# T-3272 seams: the closure action reads/writes task files through these, so the
# fixtures can prove it against a scratch tree and a fake fw — never real tasks.
TASKS_DIR="${RUNME_TASKS_DIR:-$PROJECT_ROOT/.tasks}"
FW="${RUNME_FW:-$PROJECT_ROOT/.agentic-framework/bin/fw}"

RUNME_ARGS="$*"   # captured before parsing shifts them away (logged, T-3273)
DRY_RUN=0
DECIDE=""
DISABLE_MISMATCH=0
while [ $# -gt 0 ]; do
    case "$1" in
        --dry-run) DRY_RUN=1 ;;
        --decide)  shift; [ $# -ge 1 ] || { echo "runme: --decide needs <ID>=<verdict>" >&2; exit 2; }; DECIDE="$1" ;;
        --disable-mismatch-plugins) DISABLE_MISMATCH=1 ;;
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
if [ "$DRY_RUN" = "0" ] && [ -z "$DECIDE" ] && [ "$DISABLE_MISMATCH" = "0" ] && [ "$(id -u)" != "0" ]; then
    echo "runme: needs root (these write to /etc/cron.d). Nothing was applied." >&2
    echo "       Run:  sudo ./runme.sh          (or ./runme.sh --dry-run to preview)" >&2
    exit 2
fi

DONE=0; SKIPPED=0; FAILED=0
INSTALLED_NAMES=()   # basenames this run is responsible for

say()  { printf '%s\n' "$*"; }
head2() { printf '\n\033[1m%s\033[0m\n' "$*"; }

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
head2 "1. Cron installs (notify-rail T-3050/T-3051/T-3068, arc-claim-drift T-3288)"
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
APPROVED_DECISIONS=(
    "T-3291|go|Operator 2026-09-30: GO on all four slices — S1 reap orphan .sock.data (cleanup deletes all three files + sweep reaps orphans), S2 register --shell exits with its shell, S3 upstream dispatch/claude-fw exit + one-time runme reap of idle zombies, S4 session-leak canary; docs/reports/T-3291-session-lifecycle-leak-rca.md"
)
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
TL_SRC="${RUNME_TERMLINK_SRC:-$PROJECT_ROOT/target/release/termlink}"
if [ -n "${RUNME_TERMLINK_DESTS:-}" ]; then
    TL_DESTS="$RUNME_TERMLINK_DESTS"
elif [ -n "${RUNME_TERMLINK_DEST:-}" ]; then
    TL_DESTS="$RUNME_TERMLINK_DEST"
else
    TL_DESTS="${HOME:-/root}/.cargo/bin/termlink /usr/local/bin/termlink ${HOME:-/root}/.local/bin/termlink"
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
    if [ "${RUNME_SKIP_BUILD:-0}" != "1" ] && [ "$DRY_RUN" = "0" ]; then
        local cargo; cargo="$(command -v cargo || echo "${HOME:-/root}/.cargo/bin/cargo")"
        say "  build   $cargo build --release -p termlink (a no-op when up to date)"
        if ! (cd "$PROJECT_ROOT" && "$cargo" build --release -p termlink) >/tmp/.runme-build.log 2>&1; then
            say "  FAILED  cargo build --release failed (see /tmp/.runme-build.log)"; FAILED=$((FAILED+1)); return
        fi
    fi
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
HUB_SYSTEMCTL="${RUNME_HUB_SYSTEMCTL:-systemctl}"
HUB_PROC="${RUNME_HUB_PROC:-/proc}"
HUB_RT="${RUNME_HUB_RUNTIME_DIR:-/var/lib/termlink}"
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
    local st; st="$("$HUB_BIN" hub status --json 2>/dev/null)"
    case "$st" in *"\"pid\":$npid"*'"status":"running"'*|*'"status":"running"'*"\"pid\":$npid"*) : ;;
        *) problems="$problems hub-status-not-running-as-$npid;";; esac
    if [ -z "$problems" ]; then
        say "  OK      restarted and verified: pid $pid -> $npid, exe $nexe ($("$HUB_BIN" --version 2>/dev/null)); secret + cert unchanged"; DONE=$((DONE+1))
    else
        say "  FAILED  hub restart verification:$problems"; FAILED=$((FAILED+1))
    fi
}

head2 "5. Local hub onto the installed binary (T-3290)"
restart_hub

# ---------------------------------------------------------------------------
# ACTION 6 — upgrade remote fleet hubs we have a foothold on (T-3290; operator
# authorized forced upgrades of .122 and .121, 2026-09-30)
#
# runme's install actions only touch THIS host. .122 (ring20-management) served
# 0.11.1411. It is upgraded through scripts/fleet-deploy-binary.sh (T-1420):
# stream the musl-static build over one of .122's remote-exec sessions, --probe
# that it executes there (PL-100), then --swap-restart (.122 is watchdog-launched,
# not systemd). .121 is NOT here: no foothold from this host (no remote sessions,
# SSH publickey-denied) — it is asked to upgrade via its own operator agent.
#
# TRAP guarded: fleet-deploy-binary ships target/x86_64-unknown-linux-musl, which
# sat at 0.11.679 (July) — deploying it would DOWNGRADE .122. The musl build is
# rebuilt from HEAD first, and nothing ships unless it reports the SAME version as
# the glibc build installed above.
# Idempotent: skipped when fleet doctor already reports the hub at that version.
# Verified after: fleet doctor status ok (the HMAC secret still authenticates — no
# rotation) AND hub_version == the build; `tofu verify` exits 0 (cert unchanged).
# Seams (fixtures only): RUNME_FLEET_HUBS, RUNME_MUSL_SRC, RUNME_FLEET_DEPLOY,
# RUNME_FLEET_DOCTOR, RUNME_TOFU, RUNME_FLEET_WAIT_SECS (+ RUNME_SKIP_BUILD).
# ---------------------------------------------------------------------------
FLEET_HUBS="${RUNME_FLEET_HUBS:-ring20-management}"
MUSL_SRC="${RUNME_MUSL_SRC:-$PROJECT_ROOT/target/x86_64-unknown-linux-musl/release/termlink}"
FLEET_DEPLOY="${RUNME_FLEET_DEPLOY:-bash $PROJECT_ROOT/scripts/fleet-deploy-binary.sh}"
FLEET_DOCTOR="${RUNME_FLEET_DOCTOR:-termlink fleet doctor --json}"
TOFU="${RUNME_TOFU:-termlink tofu verify}"
FLEET_WAIT="${RUNME_FLEET_WAIT_SECS:-90}"

fleet_hub_field() {  # <hub-name> <field> -> value from fleet doctor, empty if absent
    timeout 60 $FLEET_DOCTOR 2>/dev/null | python3 -c '
import json,sys
try: d=json.load(sys.stdin)
except Exception: sys.exit(0)
for h in d.get("hubs",[]):
    if h.get("hub")==sys.argv[1]: print(h.get(sys.argv[2]) or ""); break
' "$1" "$2"
}

upgrade_fleet_hubs() {
    if [ "${RUNME_SKIP_BUILD:-0}" != "1" ] && [ "$DRY_RUN" = "0" ]; then
        local cargo; cargo="$(command -v cargo || echo "${HOME:-/root}/.cargo/bin/cargo")"
        say "  build   $cargo build --release --target x86_64-unknown-linux-musl -p termlink"
        if ! (cd "$PROJECT_ROOT" && "$cargo" build --release --target x86_64-unknown-linux-musl -p termlink) >/tmp/.runme-musl-build.log 2>&1; then
            say "  FAILED  musl build failed (see /tmp/.runme-musl-build.log)"; FAILED=$((FAILED+1)); return
        fi
    fi
    local want mver
    want="$("$TL_SRC" --version 2>/dev/null | awk '{print $2}')"
    mver="$("$MUSL_SRC" --version 2>/dev/null | awk '{print $2}')"
    if [ -z "$want" ] || [ "$mver" != "$want" ]; then
        if [ "$DRY_RUN" = "1" ]; then
            say "  [DRY]   would rebuild musl (now '${mver:-absent}', build is '${want:-unknown}') before any deploy"
        else
            say "  FAILED  musl binary reports '${mver:-nothing}', build is '${want:-unknown}' — refusing to ship (not the current code)"; FAILED=$((FAILED+1)); return
        fi
    fi
    local hub have addr i st
    for hub in $FLEET_HUBS; do
        have="$(fleet_hub_field "$hub" hub_version)"; addr="$(fleet_hub_field "$hub" address)"
        if [ -n "$want" ] && [ "$have" = "$want" ]; then
            say "  skip    $hub already serves $have"; SKIPPED=$((SKIPPED+1)); continue
        fi
        if [ -z "$addr" ]; then
            say "  FAILED  $hub not found or unreachable in fleet doctor — cannot upgrade"; FAILED=$((FAILED+1)); continue
        fi
        if [ "$DRY_RUN" = "1" ]; then
            say "  [DRY]   would deploy musl ${want:-?} to $hub ($addr, now ${have:-unknown}) with --probe --swap-restart"; DONE=$((DONE+1)); continue
        fi
        say "  deploy  $hub ($addr): ${have:-unknown} -> $want"
        if ! $FLEET_DEPLOY "$hub" --binary "$MUSL_SRC" --probe --swap-restart >/tmp/.runme-deploy-"$hub".log 2>&1; then
            say "  FAILED  fleet-deploy-binary for $hub failed (see /tmp/.runme-deploy-$hub.log)"; FAILED=$((FAILED+1)); continue
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

head2 "6. Fleet hubs reachable from here (T-3290) — .121 has no foothold"
upgrade_fleet_hubs

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
