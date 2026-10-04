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

# T-3310 (arc-012 step 5, D1 backstop) — fires if the hub opts out of "forever
# needs an owner", or is still not enforcing it after 2026-11-15; files one task.
install_crontab forever-owner-canary.crontab      "$CRON_DIR/termlink-forever-owner-canary"
install_crontab agent-send-suites-canary.crontab  "$CRON_DIR/termlink-agent-send-suites-canary"
install_crontab vector-index-canary.crontab       "$CRON_DIR/termlink-vector-index-canary"

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
    local st; st="$("$HUB_BIN" hub status --json 2>/dev/null)"
    case "$st" in *"\"pid\":$npid"*'"status":"running"'*|*'"status":"running"'*"\"pid\":$npid"*) : ;;
        *) problems="$problems hub-status-not-running-as-$npid;";; esac
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
FLEET_HUBS="${RUNME_FLEET_HUBS:-ring20-management ring20-dashboard}"
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

head2 "6. Fleet hubs reachable from here (T-3290; .121 added by T-3323)"
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
