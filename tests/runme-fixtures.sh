#!/usr/bin/env bash
# T-3052 — fixtures for runme.sh, the operator-action script.
# guard-layer-timeout: 900  # 137 full runme runs at ~1.3 s each; measured 183-303 s on this host under load (T-3571)
#
# The whole reason runme.sh exists is that a human running a command and assuming
# it worked is the silent-failure class this repo guards against everywhere else.
# So the script verifies itself — and that self-verification has to be proven,
# or it is just a longer assumption.
#
# Hermetic (PL-213): RUNME_CRON_DIR points every install at a scratch directory,
# so nothing here writes real host state.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RUNME="$REPO_ROOT/runme.sh"

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); printf '  \033[0;32mPASS\033[0m  %s\n' "$1"; }
bad() { FAIL=$((FAIL+1)); printf '  \033[0;31mFAIL\033[0m  %s\n' "$1"; [ $# -gt 1 ] && printf '        %s\n' "$2"; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

run() { RUNME_CRON_DIR="$TMP/cron" bash "$RUNME" "$@" 2>&1; }

mkdir -p "$TMP/cron"

# T-3311 (action 21): fake `fleet governor-status --json`. $FAKE_SWEEP_FILE holds the
# hub's interval; default 3600 = already on, so every other case skips action 21
# without touching a real hub.
export FAKE_SWEEP_FILE="$TMP/sweep.interval" RUNME_SWEEP_GOV="$TMP/fake-gov"
echo 3600 > "$FAKE_SWEEP_FILE"
cat > "$TMP/fake-gov" <<'EOS'
#!/usr/bin/env bash
printf '{"hubs":[{"name":"%s","retention_sweep_interval_secs":%s}]}\n' "${RUNME_SWEEP_HUB:-ring20-management}" "$(cat "$FAKE_SWEEP_FILE")"
EOS
chmod +x "$TMP/fake-gov"

# T-3477 (action 22): ring20's deep-inception pack. A scratch git repo whose .gitignore covers
# work/*, a fake pack (two files + SHA256SUMS), and a default state of "already imported, drop
# gone" so every other case skips action 22 without touching /var/tmp or the real repo.
PACK_REPO="$TMP/packrepo"; mkdir -p "$PACK_REPO/work" "$TMP/packsrc/pack"
git -C "$PACK_REPO" init -q 2>/dev/null; printf 'work/*\n' > "$PACK_REPO/.gitignore"
printf 'plan\n' > "$TMP/packsrc/pack/plan.md"; printf 'chain: v2\n' > "$TMP/packsrc/pack/role-chain.yaml"
# Shaped like ring20's real pack: SHA256SUMS lists itself first, hashed while still empty.
( cd "$TMP/packsrc/pack" && : > SHA256SUMS && sha256sum ./SHA256SUMS ./plan.md ./role-chain.yaml > SHA256SUMS.new && mv SHA256SUMS.new SHA256SUMS && echo >> SHA256SUMS )
tar -czf "$TMP/pack-good.tgz" -C "$TMP/packsrc" pack
export RUNME_PACK_GITROOT="$PACK_REPO" RUNME_PACK_DEST="$PACK_REPO/work/pack" RUNME_PACK_SRC="$TMP/pack-drop.tgz"
export RUNME_PACK_SHA="$(sha256sum "$TMP/pack-good.tgz" | cut -d' ' -f1)"
mkdir -p "$RUNME_PACK_DEST" && tar -xzf "$TMP/pack-good.tgz" -C "$RUNME_PACK_DEST" && chmod -R go-rwx "$RUNME_PACK_DEST"

# T-2988 (action 9): stateful fake termlink for health:ring20-fedprobe. State is
# "<kind> <value> <count>" in $FAKE_FEDPROBE_STATE; default = already bounded, so
# every other case below skips action 9 without touching a real hub.
export FAKE_FEDPROBE_STATE="$TMP/fedprobe.state"
echo "latest_per_cv_key 0 1" > "$FAKE_FEDPROBE_STATE"
cat > "$TMP/fake-tl-fedprobe" <<'EOS'
#!/usr/bin/env bash
read -r kind value count < "$FAKE_FEDPROBE_STATE"
case "$1 $2" in
  "channel info")
    [ -n "${FAKE_FEDPROBE_MISSING:-}" ] && { echo "unknown topic" >&2; exit 1; }
    printf '{"topic":"%s","count":%s,"retention":{"kind":"%s","value":%s}}\n' "$3" "$count" "$kind" "$value" ;;
  "channel set-retention")
    spec="$5"
    if [ "$spec" = "latest-per-cv-key" ]; then echo "latest_per_cv_key 0 $count" > "$FAKE_FEDPROBE_STATE"
    else echo "${spec%%:*} ${spec#*:} $count" > "$FAKE_FEDPROBE_STATE"; fi ;;
  "channel sweep")
    [ -n "${FAKE_FEDPROBE_NOSWEEP:-}" ] && exit 0
    [ "$kind" = "messages" ] && [ "$count" -gt "$value" ] && count=$value
    [ "$kind" = "latest_per_cv_key" ] && count=1   # one probe source, every record tagged
    echo "$kind $value $count" > "$FAKE_FEDPROBE_STATE" ;;
esac
EOS
chmod +x "$TMP/fake-tl-fedprobe"
export RUNME_FEDPROBE_TL="$TMP/fake-tl-fedprobe"

# T-3443 (action 18): stateful fake termlink for one project inbox. State is
# "<kind> <value> <count> <newest offset> <min receipt|none>" in $FAKE_INBOX_STATE;
# default = already capped, so every other case skips action 18 without a real hub.
export FAKE_INBOX_STATE="$TMP/inbox.state"
echo "messages 1000 1000 4999 4999" > "$FAKE_INBOX_STATE"
echo 4999 > "$TMP/inbox.marker"
cat > "$TMP/fake-tl-inbox" <<'EOS'
#!/usr/bin/env bash
read -r kind value count newest minr < "$FAKE_INBOX_STATE"
case "$1 $2" in
  "channel info")
    [ -n "${FAKE_INBOX_MISSING:-}" ] && { echo "unknown topic" >&2; exit 1; }
    if [ "$minr" = none ]; then rc='[]'; else rc="[{\"sender_id\":\"a\",\"up_to\":$minr}]"; fi
    [ "$value" = 0 ] && v=null || v=$value
    printf '{"topic":"%s","count":%s,"retention":{"kind":"%s","value":%s},"receipts":%s}\n' "$3" "$count" "$kind" "$v" "$rc" ;;
  "channel subscribe")
    printf '{"msg_type":"note","offset":%s}\n' "$newest" ;;
  "channel set-retention")
    spec="$5"; echo "${spec%%:*} ${spec#*:} $count $newest $minr" > "$FAKE_INBOX_STATE" ;;
  "channel sweep")
    [ -n "${FAKE_INBOX_NOSWEEP:-}" ] && exit 0
    [ "$kind" = "messages" ] && [ "$count" -gt "$value" ] && count=$value
    echo "$kind $value $count $newest $minr" > "$FAKE_INBOX_STATE" ;;
esac
EOS
chmod +x "$TMP/fake-tl-inbox"
export RUNME_INBOX_TL="$TMP/fake-tl-inbox"
export RUNME_INBOX_TOPICS="inbox:t/010|$TMP/inbox.marker"

# T-3426 (action 19): fake W1 kill-switch drill; default = already passed (marker present).
cat > "$TMP/fake-drill" <<'EOS'
#!/usr/bin/env bash
echo "drill ran" >> "$FAKE_DRILL_RAN"; echo "W1 kill-switch drill: 8 passed, 0 failed (probe m1)"; exit "${FAKE_DRILL_RC:-0}"
EOS
chmod +x "$TMP/fake-drill"
export FAKE_DRILL_RAN="$TMP/drill.ran"
export RUNME_W1_DRILL="$TMP/fake-drill"
export RUNME_W1_DRILL_DONE="$TMP/drill.passed"
echo "2026-10-16T00:00:00Z earlier pass" > "$RUNME_W1_DRILL_DONE"

# T-3440 5c (action 20): scratch arc dir + scratch spec dir + a fake attach that
# writes a valid scoring block (unless FAKE_ATTACH_LIE is set). Default state:
# all three drivers already carry a spec, so every other case skips the action.
export RUNME_ARCS_DIR="$TMP/arcs" RUNME_DRIVER_SPECS="$TMP/specs" RUNME_ARC_ATTACH="$TMP/fake-attach"
mkdir -p "$RUNME_ARCS_DIR" "$RUNME_DRIVER_SPECS"
for d in AS_COORDINATION_TRUTH AS_FAILURE_OBSERVABILITY AS_RESOURCE_FOOTPRINT AS_DELIVERY_TRUTH AS_FAILURE_SURFACED AS_OPERATOR_SIMPLICITY; do
    printf 'kind: signals\nlevels:\n  3: {keywords: [x]}\n' > "$RUNME_DRIVER_SPECS/$d.yaml"
done
arcs_specced() {  # write both scratch arcs with (1) or without (0) scoring blocks
    local arc drivers d
    for arc in arc-substrate-fitness arc-011; do
        case $arc in
            arc-011) drivers="AS_DELIVERY_TRUTH AS_FAILURE_SURFACED AS_OPERATOR_SIMPLICITY" ;;
            *)       drivers="AS_COORDINATION_TRUTH AS_FAILURE_OBSERVABILITY AS_RESOURCE_FOOTPRINT" ;;
        esac
        { echo "id: $arc"; echo "scoped_drivers:"
          for d in $drivers; do
              echo "  - name: $d"; echo "    weight: 4"
              [ "$1" = 1 ] && echo "    scoring: {kind: signals, levels: {3: {keywords: [x]}}}"
          done; } > "$RUNME_ARCS_DIR/$arc.yaml"
    done
}
arcs_specced 1
cat > "$TMP/fake-attach" <<'EOS'
#!/usr/bin/env bash
[ -n "${FAKE_ATTACH_LIE:-}" ] && exit 0
python3 - "$RUNME_ARCS_DIR/$1.yaml" "$2" <<'PY'
import sys, yaml
d = yaml.safe_load(open(sys.argv[1]))
for s in d["scoped_drivers"]:
    if s["name"] == sys.argv[2]:
        s["scoring"] = {"kind": "signals", "levels": {3: {"keywords": ["x"]}}}
yaml.safe_dump(d, open(sys.argv[1], "w"))
PY
EOS
chmod +x "$TMP/fake-attach"

# T-3325 (action 10): stateful fake stale-sidecar detector. State file holds the
# detector's JSON; default = nothing stale, so every other case skips the action.
# The fake supervisor flips stale -> current unless FAKE_SIDECAR_NOFIX is set. The
# fake kill only records pids, so no real process is ever signalled.
export FAKE_SIDECAR_STATE="$TMP/sidecar.state" FAKE_SIDECAR_KILLED="$TMP/sidecar.killed"
echo '{"stale_count":0,"current_count":3,"stale":[]}' > "$FAKE_SIDECAR_STATE"
printf '#!/usr/bin/env bash\ncat "$FAKE_SIDECAR_STATE"\n' > "$TMP/fake-sidecar-detect"
printf '#!/usr/bin/env bash\necho "$1" >> "$FAKE_SIDECAR_KILLED"\n' > "$TMP/fake-sidecar-kill"
printf '#!/usr/bin/env bash\n[ -n "${FAKE_SIDECAR_NOFIX:-}" ] && exit 0\necho %s > "$FAKE_SIDECAR_STATE"\n' \
    "'{\"stale_count\":0,\"current_count\":2,\"stale\":[]}'" > "$TMP/fake-sidecar-supervisor"
chmod +x "$TMP/fake-sidecar-detect" "$TMP/fake-sidecar-kill" "$TMP/fake-sidecar-supervisor"
export RUNME_SIDECAR_DETECT="$TMP/fake-sidecar-detect" RUNME_SIDECAR_KILL="$TMP/fake-sidecar-kill" \
       RUNME_SIDECAR_SUPERVISOR="$TMP/fake-sidecar-supervisor"

# T-3272: the closure action goes through RUNME_TASKS_DIR + RUNME_FW. Exported for
# EVERY invocation below (including the --decide cases, which do a real run) so no
# fixture can ever close a real task.
CLOSE_IDS="T-3132 T-3128 T-3130"
mkdir -p "$TMP/tasks/active" "$TMP/tasks/completed"
for id in $CLOSE_IDS; do printf -- '---\nid: %s\nstatus: started-work\n---\n' "$id" > "$TMP/tasks/active/$id-fixture.md"; done
# T-3275: T-3132 and T-3130 were `captured` on the real tree and the operator's
# first run failed on them. Model that state for one of them.
sed -i 's/^status: .*/status: captured/' "$TMP/tasks/active/T-3132-fixture.md"
cat > "$TMP/fake-fw" <<'EOF'
#!/usr/bin/env bash
# fake `fw task update <id> --status <s> [...]` enforcing the REAL transition rule:
# captured -> work-completed is refused ("Invalid transition"), as fw does.
[ "$1 $2" = "task review" ] && exit 0      # T-3285: review marker step (a no-op here)
if [ "$1" = "watchtower" ]; then            # T-3568 (action 11c): stale iff the marker file exists
    case "$2" in
        current) [ -e "$TMP_WT_STALE" ] && exit 1; exit 0 ;;
        restart) [ -n "${FAKE_WT_RESTART_NOOP:-}" ] || rm -f "$TMP_WT_STALE"; exit 0 ;;
        status)  exit 0 ;;
    esac
    exit 9
fi
if [ "$1 $2" = "cron install" ]; then       # T-3336: writes ONLY the fixture's audit cron file
    [ -n "${FAKE_CRON_NOINSTALL:-}" ] && exit 0
    # T-3554: like the real verb, deploy the registry source verbatim (action 11b case).
    if [ -n "${FAKE_CRON_COPY_SRC:-}" ]; then cp "$RUNME_AUDIT_CRON_SRC" "$RUNME_AUDIT_CRON_FILE"; exit 0; fi
    echo '20 * * * * root cd "/x" && "/x/fw" index reindex' >> "$RUNME_AUDIT_CRON_FILE"
    echo '3,8 * * * * root cd "/x" && "/x/fw" sidecar sweep' >> "$RUNME_AUDIT_CRON_FILE"; exit 0
fi
if [ "$1 $2" = "inception decide" ]; then   # T-3285: record like fw does, on disk
    f=$(ls "$RUNME_TASKS_DIR"/active/"$3"-*.md 2>/dev/null | head -1); [ -n "$f" ] || exit 1
    printf '\n## Decision\n\n**Decision**: %s\n' "$(printf '%s' "$4" | tr '[:lower:]' '[:upper:]')" >> "$f"; exit 0
fi
[ "$1 $2" = "task update" ] || exit 9
id="$3"; new="$5"
f=$(ls "$RUNME_TASKS_DIR"/active/"$id"-*.md 2>/dev/null | head -1)
[ -n "$f" ] || exit 1
if [ "$4" = "--owner" ]; then   # T-3592: hand-back; FAKE_HANDBACK_REFUSE=1 claims success, changes nothing
    [ "${FAKE_HANDBACK_REFUSE:-0}" = 1 ] && exit 0
    sed -i "s/^owner: .*/owner: $5/" "$f"; exit 0
fi
cur=$(sed -n 's/^status: //p' "$f")
if [ "$cur" = "captured" ] && [ "$new" = "work-completed" ]; then
    echo "ERROR: Invalid transition 'captured' → 'work-completed'"; exit 1
fi
sed -i "s/^status: .*/status: $new/" "$f"
[ "$new" = "work-completed" ] && mv "$f" "$RUNME_TASKS_DIR/completed/"
exit 0
EOF
printf '#!/usr/bin/env bash\nexit 0\n' > "$TMP/noop-fw"   # claims success, moves nothing
chmod +x "$TMP/fake-fw" "$TMP/noop-fw"
export RUNME_TASKS_DIR="$TMP/tasks" RUNME_FW="$TMP/fake-fw"
export TMP_WT_STALE="$TMP/watchtower-stale"   # T-3568: absent = Watchtower current, so other cases skip 11c
# T-3336 (action 11) + T-3370 (action 13): default = reindex AND sidecar sweep already
# scheduled, so every other case skips both.
export RUNME_AUDIT_CRON_FILE="$TMP/agentic-audit-termlink"
# T-3554 (action 11b): default source = the installed file itself, so every other case skips it.
export RUNME_AUDIT_CRON_SRC="$RUNME_AUDIT_CRON_FILE"
cron_both() { printf '%s\n' '20 * * * * root cd "/x" && "/x/fw" index reindex' '3,8 * * * * root cd "/x" && "/x/fw" sidecar sweep' > "$RUNME_AUDIT_CRON_FILE"; }
cron_both
# T-3351 (action 14): temp tick ledger + a fake human_ac_ticks.py that appends an
# operator-ack row (or nothing, under FAKE_TICK_NOWRITE=1). Default = T-3349 already
# recorded, so every other case skips the action. The real ledger is never touched.
export RUNME_HUMAN_TICKS_LEDGER="$TMP/human-ac-ticks.jsonl" RUNME_HUMAN_TICKS_PY="$TMP/fake-ticks.py"
cat > "$RUNME_HUMAN_TICKS_PY" <<'EOF'
import json, os, sys
if os.environ.get("FAKE_TICK_NOWRITE") == "1":
    sys.exit(0)
task = sys.argv[2]
with open(os.environ["RUNME_HUMAN_TICKS_LEDGER"], "a") as f:
    f.write(json.dumps({"task": task, "via": "operator-ack", "by": "fixture"}, sort_keys=True) + "\n")
EOF
ticks_recorded() { : > "$RUNME_HUMAN_TICKS_LEDGER"; for t in T-3349 T-3351 T-3388; do echo "{\"by\": \"fixture\", \"task\": \"$t\", \"via\": \"operator-ack\"}" >> "$RUNME_HUMAN_TICKS_LEDGER"; done; }   # every tick runme lists (T-3393)
ticks_recorded
# T-3416 (action 15): fake testdev claude + fake installer. FAKE_CLAUDE_NEW is the
# version the installer leaves (empty = installs nothing). Default = current version
# already installed, so every other case skips the action. No real user, download or
# binary is touched.
export RUNME_TESTDEV_CLAUDE="$TMP/testdev-claude" RUNME_TESTDEV_INSTALL="bash $TMP/fake-claude-install"
fake_claude() { printf '#!/usr/bin/env bash\necho "%s (Claude Code)"\n' "$1" > "$RUNME_TESTDEV_CLAUDE"; chmod +x "$RUNME_TESTDEV_CLAUDE"; }
cat > "$TMP/fake-claude-install" <<'EOF'
#!/usr/bin/env bash
[ -n "${FAKE_CLAUDE_NEW:-}" ] || exit 0
printf '#!/usr/bin/env bash\necho "%s (Claude Code)"\n' "$FAKE_CLAUDE_NEW" > "$RUNME_TESTDEV_CLAUDE"; chmod +x "$RUNME_TESTDEV_CLAUDE"
EOF
fake_claude 2.1.296
# T-3421 (action 16): a fake router and one fake claude-fw on PATH. Default = that path
# already IS the router, so every other case skips the action once. No real PATH file.
export RUNME_CLAUDE_FW_ROUTER="$TMP/claude-fw-router" RUNME_CLAUDE_FW_TARGETS="$TMP/cfw-a" RUNME_CLAUDE_FW_BACKUP_DIR="$TMP/cfw-backups"
printf '#!/bin/bash\n# claude-fw-router (fixture)\nexec true "$@"\n' > "$RUNME_CLAUDE_FW_ROUTER"; chmod 755 "$RUNME_CLAUDE_FW_ROUTER"
cfw_router() { cp "$RUNME_CLAUDE_FW_ROUTER" "$1"; chmod 755 "$1"; }
cfw_old()    { printf '#!/bin/bash\n# old fixed copy of claude-fw\nexit 0\n' > "$1"; chmod 755 "$1"; }
cfw_router "$TMP/cfw-a"
# T-3343 (action 12): fake stray hub. Default = no pidfile in the stray dir, so every
# other case skips the action. The fake termlink records a stop and then answers
# status with the canonical pid; the fake liveness reads a state file; no real
# process or hub is ever touched.
mkdir -p "$TMP/stray" "$TMP/canon" "$TMP/proc/4242" "$TMP/guardbin"
echo 906 > "$TMP/canon/hub.pid"
printf 'termlink\0hub\0start\0' > "$TMP/proc/4242/cmdline"
export FAKE_STRAY_STATE="$TMP/stray.state"; echo alive > "$FAKE_STRAY_STATE"
cat > "$TMP/fake-stray-tl" <<'EOS'
#!/usr/bin/env bash
case "$1 $2" in
  "hub start") [ -n "${FAKE_STRAY_NOGUARD:-}" ] || echo "      --allow-second-hub" ;;
  "hub stop") [ -n "${FAKE_STRAY_NOSTOP:-}" ] || echo dead > "$FAKE_STRAY_STATE" ;;
  "hub status") if grep -q dead "$FAKE_STRAY_STATE"; then echo '{"pid":906}'; else echo '{"pid":4242}'; fi ;;
esac
EOS
printf '#!/usr/bin/env bash\ngrep -q alive "$FAKE_STRAY_STATE"\n' > "$TMP/fake-stray-alive"
cp "$TMP/fake-stray-tl" "$TMP/guardbin/termlink"
chmod +x "$TMP/fake-stray-tl" "$TMP/fake-stray-alive" "$TMP/guardbin/termlink"
echo "rescued" > "$TMP/rescue-report.md"
export RUNME_STRAY_HUB_DIR="$TMP/stray" RUNME_CANON_HUB_DIR="$TMP/canon" RUNME_STRAY_TL="$TMP/fake-stray-tl" \
       RUNME_STRAY_ALIVE="$TMP/fake-stray-alive" RUNME_STRAY_PROC_ROOT="$TMP/proc" \
       RUNME_STRAY_GUARD_BINS="$TMP/guardbin/termlink" RUNME_STRAY_RESCUE_REPORT="$TMP/rescue-report.md"
# T-3276: the real runme.sh carries no pending closures or decisions once they are
# done, so the fixtures feed the machinery through its test seams instead.
export RUNME_TEST_CLOSES="T-3132|fixture approved closure
T-3128|fixture approved closure
T-3130|fixture approved closure"
export RUNME_TEST_DECISIONS="T-9055"
# T-3285: approved inception decisions go through their own seam (SET replaces the real list).
printf -- '---\nid: T-9284\nworkflow_type: inception\n---\n' > "$TMP/tasks/active/T-9284-fixture.md"
export RUNME_TEST_APPROVED_DECISIONS="T-9284|go|fixture approved decision"
# T-3592 action 2b: hand-backs go through their own seam (SET replaces the real list).
# Default = empty, so ordinary cases run no hand-back and never touch a real task.
export RUNME_TEST_HANDBACKS=""
# T-3287: the termlink install action must NEVER build or touch ~/.cargo/bin in a
# fixture. A fake "built" binary that accepts --no-create (exit 3, writes nothing),
# and an "old" one that rejects it, as installed binaries would.
mkdir -p "$TMP/bin"
cat > "$TMP/tl-new" <<'EOS'
#!/usr/bin/env bash
[ "$1" = "--version" ] && { echo "termlink 9.9.9"; exit 0; }
case " $* " in *" --no-create "*) echo '{"ok":false,"error":"no_identity"}'; exit 3 ;; esac
exit 0
EOS
cat > "$TMP/tl-old" <<'EOS'
#!/usr/bin/env bash
[ "$1" = "--version" ] && { echo "termlink 0.0.1"; exit 0; }
echo "error: unexpected argument '--no-create' found" >&2; exit 2
EOS
chmod +x "$TMP/tl-new" "$TMP/tl-old"
cp "$TMP/tl-old" "$TMP/bin/termlink"
# T-3299: fixture mode — runme ABORTS (exit 2) if any action reaches a REAL
# host default because its seam is missing, instead of touching the host.
export RUNME_FIXTURE=1
# T-3443: let the real-run cases run as a non-root CI user too (fixture mode only;
# case 11 unsets this to test the real non-root refusal).
export RUNME_FIXTURE_ALLOW_NONROOT=1
# T-3435 (P3): sidecar supervision — scratch unit dir, settings dir, a fake
# systemd marker dir, and a fake systemctl that logs calls and resolves the
# template only when the unit file is really in the scratch unit dir.
SCU="$TMP/sidecar-sup"; mkdir -p "$SCU/units" "$SCU/run-systemd"
cat > "$SCU/systemctl" <<EOS
#!/usr/bin/env bash
echo "\$*" >> "$SCU/calls"
case "\$1" in
    daemon-reload) exit 0 ;;
    cat) [ -f "$SCU/units/termlink-sidecar@.service" ] && [ ! -e "$SCU/broken" ] ;;
    *) exit 1 ;;
esac
EOS
chmod +x "$SCU/systemctl"
# T-3438 (P7): W1 provisioning and the kill switch — fakes only.
cat > "$SCU/w1" <<'EOS'
#!/usr/bin/env bash
echo "$*" >> "$FAKE_W1_CALLS"
case "$1" in
    --check) echo "  ready      fake"; exit "${FAKE_W1_CHECK_RC:-0}" ;;
    --apply) echo "  OK      fake key register written and verified: /x"; exit "${FAKE_W1_APPLY_RC:-0}" ;;
esac
EOS
cat > "$SCU/tl" <<'EOS'
#!/usr/bin/env bash
echo "$*" >> "$FAKE_W1_CALLS"
exit "${FAKE_KS_RC:-0}"
EOS
chmod +x "$SCU/w1" "$SCU/tl"
printf '/x/sidecar/claude-termlink.admin.sock\n' > "$SCU/admin.list"
export RUNME_W1_GO= RUNME_W1_PROVISION="bash $SCU/w1" FAKE_W1_CALLS="$SCU/w1-calls" \
       RUNME_SIDECAR_TL="$SCU/tl" RUNME_SIDECAR_ADMIN_LIST="$SCU/admin.list"
export RUNME_SIDECAR_UNIT_DIR="$SCU/units" RUNME_SIDECAR_ENV_DIR="$SCU/env" \
       RUNME_SIDECAR_SYSTEMCTL="$SCU/systemctl" RUNME_SIDECAR_SYSTEMD_RUN="$SCU/run-systemd"
# T-3303 action 8: a scratch Claude local-settings file, already carrying the
# env key by default (=> skip); never the real .claude/settings.local.json.
printf '{"permissions":{"allow":["Bash(ls:*)"]},"env":{"TERMLINK_AGENT_ID":"claude-termlink"}}\n' > "$TMP/claude-local.json"
export RUNME_CLAUDE_LOCAL_SETTINGS="$TMP/claude-local.json"
# Build seams: SKIP_BUILD means these are never used, but fixture mode requires
# every host default to be explicit. `false` as cargo can never build anything.
export RUNME_BUILD_STAMP_DIR="$TMP/build-stamps" RUNME_BUILD_REPO="$REPO_ROOT" RUNME_CARGO=false
export RUNME_SKIP_BUILD=1 RUNME_TERMLINK_SRC="$TMP/tl-new" RUNME_TERMLINK_DEST="$TMP/bin/termlink"
# T-3290: the hub-restart action must NEVER reach the real systemctl, /proc or
# /var/lib/termlink. A fake systemctl keeps MainPID in a state file; `restart`
# bumps the pid and points its fake /proc exe at the installed binary (or, with
# FAKE_HUB_RESTART_MODE, misbehaves). Default state: hub already on the installed
# binary, so ordinary cases see a skip.
H="$TMP/hub"; mkdir -p "$H/proc" "$H/rt"
printf 'secret-bytes' > "$H/rt/hub.secret"; printf 'cert-bytes' > "$H/rt/hub.cert.pem"
cat > "$H/systemctl" <<'EOS'
#!/usr/bin/env bash
H="${FAKE_HUB_DIR:?}"; pid=$(cat "$H/pid")
case "$1" in
  # T-3587: NRestarts lives in a file (default 0, so other cases skip action 5b);
  # reset-failed zeroes it unless FAKE_HUB_RESET_STICKY=1.
  show) if [ "${3:-}" = "NRestarts" ]; then cat "$H/nrestarts" 2>/dev/null || echo 0; else echo "$pid"; fi ;;
  reset-failed) echo reset-failed >> "$H/resets"; [ "${FAKE_HUB_RESET_STICKY:-0}" = 1 ] || echo 0 > "$H/nrestarts" ;;
  is-active) echo "${FAKE_HUB_ACTIVE:-active}" ;;
  restart)
    echo restart >> "$H/restarts"
    case "${FAKE_HUB_RESTART_MODE:-ok}" in
      fail) exit 1 ;;
      rotate) printf 'new-secret' > "$H/rt/hub.secret" ;;
    esac
    n=$((pid+1)); echo "$n" > "$H/pid"; mkdir -p "$H/proc/$n"
    ln -sfn "${FAKE_HUB_NEW_EXE:-$FAKE_HUB_BIN}" "$H/proc/$n/exe"
    if [ -n "${FAKE_HUB_MOUNTINFO:-}" ]; then printf '%s\n' "$FAKE_HUB_MOUNTINFO" > "$H/proc/$n/mountinfo"; fi ;;
esac
EOS
chmod +x "$H/systemctl"
cat > "$H/hubbin" <<'EOS'
#!/usr/bin/env bash
[ "$1" = "--version" ] && { echo "termlink 9.9.9"; exit 0; }
if [ "$1 $2" = "hub status" ]; then
    # T-3450: FAKE_HUB_STATUS_LAG=N reports "stopped" for the first N reads (the new hub has not
    # written its pidfile yet), then running.
    n=$(( $(cat "$FAKE_HUB_DIR/status-reads" 2>/dev/null || echo 0) + 1 )); echo "$n" > "$FAKE_HUB_DIR/status-reads"
    if [ "$n" -le "${FAKE_HUB_STATUS_LAG:-0}" ]; then echo '{"ok":true,"status":"stopped"}'; exit 0; fi
    echo "{\"ok\":true,\"pid\":$(cat "$FAKE_HUB_DIR/pid"),\"status\":\"running\"}"; exit 0
fi
exit 0
EOS
chmod +x "$H/hubbin"
hub_state() { # hub_state <pid> <exe-target>
    echo "$1" > "$H/pid"; mkdir -p "$H/proc/$1"; ln -sfn "$2" "$H/proc/$1/exe"; rm -f "$H/restarts" "$H/status-reads"
    printf 'secret-bytes' > "$H/rt/hub.secret"
}
export FAKE_HUB_DIR="$H" FAKE_HUB_BIN="$H/hubbin"
export RUNME_HUB_SYSTEMCTL="$H/systemctl" RUNME_HUB_PROC="$H/proc" RUNME_HUB_RUNTIME_DIR="$H/rt" \
       RUNME_HUB_BIN="$H/hubbin" RUNME_HUB_WAIT_SECS=2
hub_state 100 "$H/hubbin"
# T-3299 action 5a: tracked vs installed unit (identical by default => skip);
# NEVER the real /etc/systemd/system unit. RW path is a scratch dir checked
# against the fake hub's mountinfo.
printf 'unit-v1\n' > "$H/unit.src"; cp "$H/unit.src" "$H/unit.dest"; mkdir -p "$H/rwpool"
export RUNME_HUB_UNIT_SRC="$H/unit.src" RUNME_HUB_UNIT_DEST="$H/unit.dest" RUNME_HUB_RW_PATH="$H/rwpool"
# T-3290 action 6: the fleet upgrade must NEVER reach the real fleet. Fake fleet
# doctor reads a hub version from a state file; fake deploy records the call and
# (FAKE_DEPLOY_MODE) sets the new version, fails, or rotates the secret; fake tofu
# honours FAKE_TOFU_RC. The fake musl binary reports the same version as tl-new.
F="$TMP/fleet"; mkdir -p "$F"
cat > "$F/doctor" <<'EOS'
#!/usr/bin/env bash
v=$(cat "$FAKE_FLEET_DIR/ver"); st=$(cat "$FAKE_FLEET_DIR/status" 2>/dev/null || echo ok)
echo "{\"hubs\":[{\"hub\":\"fake-hub\",\"address\":\"10.0.0.1:9100\",\"hub_version\":\"$v\",\"status\":\"$st\"}]}"
EOS
# The swap (not the staging) changes the served version: via the fake remote's
# install+restart when the hub has a unit (FAKE_UNIT=1), else via --swap-restart.
cat > "$F/swap" <<'EOS'
#!/usr/bin/env bash
case "${FAKE_DEPLOY_MODE:-ok}" in
  auth) echo "9.9.9" > "$FAKE_FLEET_DIR/ver"; echo "auth-mismatch" > "$FAKE_FLEET_DIR/status" ;;
  nochange) : ;;
  *) echo "9.9.9" > "$FAKE_FLEET_DIR/ver" ;;
esac
EOS
cat > "$F/deploy" <<'EOS'
#!/usr/bin/env bash
echo "$*" >> "$FAKE_FLEET_DIR/deploys"
[ "${FAKE_DEPLOY_MODE:-ok}" = "fail" ] && exit 4
case " $* " in *" --swap-restart "*) bash "$FAKE_FLEET_DIR/swap" ;; esac
exit 0
EOS
cat > "$F/remote" <<'EOS'
#!/usr/bin/env bash
# T-3615 action 23: the rollup's probe/install run LOCALLY against scratch paths (the
# RUNME_REMOTE_ROLLUP_* seams), with every rollup input seamed, so nothing real is read
# or written. Logged apart so action 6's "dry run sent no remote command" check holds.
case "$2" in *RUNME-ROLLUP*)
    echo "$2" >> "$FAKE_FLEET_DIR/rollup-cmds"
    [ "${FAKE_RR_DOWN:-0}" = "1" ] && exit 3
    TELEMETRY_QUEUE_LOG=/nonexistent TELEMETRY_RECONNECT_LOG=/nonexistent TELEMETRY_BREAKER_LOG=/nonexistent \
    TELEMETRY_OUTAGE_SAMPLES=/nonexistent TELEMETRY_INVOCATION_SINKS=/nonexistent TELEMETRY_DISPATCH_DIRS=/nonexistent \
    TELEMETRY_CLOSE_STATES=/nonexistent TELEMETRY_VERDICTS=/nonexistent TELEMETRY_APPLIED=/nonexistent \
    TELEMETRY_STANDING_UNITS_JSON="$FAKE_FLEET_DIR/rr-units" TELEMETRY_QUEUE_STATUS_JSON="$FAKE_FLEET_DIR/rr-qs" \
    TELEMETRY_USER_HOMES="" TELEMETRY_PROJECT_ROOT=none TELEMETRY_STANDING_STATE="$FAKE_FLEET_DIR/rr-nrestarts" \
    TERMLINK_RUNTIME_DIR="$FAKE_FLEET_DIR/rr-runtime" bash -c "$2"
    exit $? ;;
esac
echo "$2" >> "$FAKE_FLEET_DIR/remote-cmds"
case "$2" in
  *"t3311-sweep.conf"*)   # T-3311 action 21: drop-in + restart; must match BEFORE the plain unit probe
    [ "${FAKE_UNIT:-1}" = "1" ] || { echo NO-UNIT; exit 0; }
    [ "${FAKE_SWEEP_APPLIES:-1}" = "1" ] && echo 3600 > "$FAKE_SWEEP_FILE"
    echo RUNME-SWEEP-OK ;;
  *"systemctl cat termlink-hub"*) [ "${FAKE_UNIT:-1}" = "1" ] && echo "/usr/local/bin/termlink" ;;
  *"systemctl restart termlink-hub"*)
    [ "${FAKE_SWAP_CONFIRM:-1}" = "1" ] || { echo "install: cannot create regular file: Text file busy"; exit 1; }
    bash "$FAKE_FLEET_DIR/swap"; echo RUNME-SWAP-OK ;;
esac
EOS
cat > "$F/tofu" <<'EOS'
#!/usr/bin/env bash
exit "${FAKE_TOFU_RC:-0}"
EOS
cp "$TMP/tl-new" "$F/musl"
chmod +x "$F/doctor" "$F/deploy" "$F/tofu" "$F/musl" "$F/swap" "$F/remote"
fleet_state() { echo "$1" > "$F/ver"; echo ok > "$F/status"; rm -f "$F/deploys" "$F/remote-cmds"; }
export FAKE_FLEET_DIR="$F" RUNME_FLEET_HUBS=fake-hub RUNME_MUSL_SRC="$F/musl" \
       RUNME_FLEET_DEPLOY="$F/deploy" RUNME_FLEET_DOCTOR="$F/doctor" RUNME_TOFU="$F/tofu" RUNME_FLEET_WAIT_SECS=2 \
       RUNME_FLEET_REMOTE="$F/remote"
# T-3615 action 23: the remote rollup install, all into scratch. FAKE_RR_AUTH_RC is the
# exit code of `termlink channel list --hub <.107>` on the "remote" (0 = credential present).
echo none > "$F/rr-units"; echo '{}' > "$F/rr-qs"; mkdir -p "$F/rr-cron"
cat > "$F/rr-tl" <<'EOS'
#!/usr/bin/env bash
case "$1 $2" in "channel list") exit "${FAKE_RR_AUTH_RC:-0}" ;; "channel queue-status") echo '{}' ;; esac
exit 0
EOS
chmod +x "$F/rr-tl"
# The install cases run as if the credential decision (T-3615 Q3) were recorded; the held
# default has its own case (RUNME_ROLLUP_CREDENTIAL_DECISION set but empty).
export RUNME_ROLLUP_CREDENTIAL_DECISION="fixture: decided"
export RUNME_ROLLUP_SRC="$REPO_ROOT/scripts/telemetry-rollup.sh" RUNME_REMOTE_ROLLUP_ROOT="$F/rr-root" \
       RUNME_REMOTE_ROLLUP_CRON="$F/rr-cron/termlink-telemetry-rollup" RUNME_REMOTE_TL="$F/rr-tl"
fleet_state 9.9.9
# T-3297 action 7: the zombie reap must NEVER see or signal a real process. A
# fake detector prints canned JSON (FAKE_ZOMBIES: "pid:name ..."), a fake /proc
# holds cmdline files, and a fake kill deletes the pid's /proc entry (the process
# "exits") unless it is listed in FAKE_STUCK.
Z="$TMP/zombie"; mkdir -p "$Z/proc"
cat > "$Z/detect" <<'EOS'
#!/usr/bin/env bash
python3 - "${FAKE_ZOMBIES:-}" <<'EOP'
import json,sys
zs=[]
for tok in sys.argv[1].split():
    pid,name=tok.split(":",1); zs.append({"pid":int(pid),"name":name,"age_s":200000,"tmux_session":"tl-"+name})
print(json.dumps({"zombie":zs}))
EOP
EOS
cat > "$Z/kill" <<'EOS'
#!/usr/bin/env bash
pid="${!#}"; echo "$pid" >> "$FAKE_ZOMBIE_DIR/killed"
case " ${FAKE_STUCK:-} " in *" $pid "*) exit 0 ;; esac
rm -rf "$FAKE_ZOMBIE_DIR/proc/$pid"
EOS
chmod +x "$Z/detect" "$Z/kill"
zombie_proc() { # zombie_proc <pid> <name>  -> fake /proc/<pid>/cmdline for a register session
    mkdir -p "$Z/proc/$1"; printf '/root/.cargo/bin/termlink\0register\0--name\0%s\0--shell\0' "$2" > "$Z/proc/$1/cmdline"; }
zombie_reset() { rm -rf "$Z/proc" "$Z/killed"; mkdir -p "$Z/proc"; }
export FAKE_ZOMBIE_DIR="$Z" RUNME_ZOMBIE_DETECT="$Z/detect" RUNME_ZOMBIE_PROC="$Z/proc" \
       RUNME_ZOMBIE_KILL="$Z/kill" RUNME_ZOMBIE_WAIT_SECS=1 FAKE_ZOMBIES=""
# T-3273: logs go to scratch (world-writable so the non-root case logs here too).
mkdir -p "$TMP/logs" && chmod 1777 "$TMP/logs"
export RUNME_LOG_DIR="$TMP/logs"
# T-3571: no wall-clock polling waits in fixture runs (the fakes change state per call).
export RUNME_NAP=0

echo "T-3052 runme.sh fixtures"
echo ""

# ---------------------------------------------------------------------------
# 1-2. --dry-run reports intent and changes NOTHING. This is the mode a human
#      reads before granting root, so it must never have a side effect.
# ---------------------------------------------------------------------------
out=$(run --dry-run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "would install"; then ok "--dry-run reports what it would install"
else bad "--dry-run reports intent" "rc=$rc: $out"; fi
if [ -z "$(ls -A "$TMP/cron")" ]; then ok "--dry-run wrote nothing"
else bad "--dry-run wrote nothing" "$(ls -A "$TMP/cron")"; fi
# T-3272: dry-run names each approved close and performs none.
if echo "$out" | grep -q "would close T-3132" && [ "$(ls "$TMP/tasks/active" | grep -cE '^T-31(32|28|30)-')" = "3" ] && [ -z "$(ls -A "$TMP/tasks/completed")" ]; then
    ok "--dry-run reports the approved closes and closes nothing"
else bad "--dry-run closes nothing" "$(ls "$TMP/tasks/active" "$TMP/tasks/completed"): $out"; fi
if echo "$out" | grep -q "would record T-9284 = go" && ! grep -q '^\*\*Decision\*\*' "$TMP/tasks/active/T-9284-fixture.md"; then
    ok "--dry-run reports the approved decision and records nothing"
else bad "--dry-run decision" "$out"; fi
if echo "$out" | grep -q "would install $TMP/tl-new" && cmp -s "$TMP/tl-old" "$TMP/bin/termlink"; then
    ok "--dry-run reports the termlink install and replaces nothing"
else bad "--dry-run termlink install" "$out"; fi
# T-3273: the run left a readable log — full output, header, rc line — and
# latest.log resolves to it. The agent reads this after the operator runs it.
LOGF="$TMP/logs/latest.log"
if [ -f "$LOGF" ] && grep -q '^=== runme.sh start .*args=\[--dry-run\]' "$LOGF" \
   && grep -q 'would close T-3132' "$LOGF" && grep -q '=== runme.sh finished rc=0' "$LOGF"; then
    ok "every run is logged: header + full output + rc line, via latest.log"
else bad "run is logged" "$(ls -la "$TMP/logs"; cat "$LOGF" 2>/dev/null | head -5)"; fi
if echo "$out" | grep -q "^log: $TMP/logs/runme-"; then ok "the log's full path is printed to the operator"
else bad "log path printed" "$out"; fi

# T-3574: one run at a time. The lock lives in the (scratch) log dir.
LOCK="$TMP/logs/.runme.lock"
if [ ! -e "$LOCK" ]; then ok "lock: a finished run leaves no lock behind"
else bad "lock: released at exit" "$(ls -la "$LOCK")"; fi
# A live runme.sh holds the lock: a second start refuses, applies nothing and leaves
# latest.log on the running run's log.
bash -c 'exec -a runme.sh sleep 60' & HOLDER=$!
mkdir -p "$LOCK"; echo "$HOLDER" > "$LOCK/pid"; echo "$TMP/logs/the-running-one.log" > "$LOCK/log"
before="$(readlink "$TMP/logs/latest.log")"
out=$(run --dry-run); rc=$?
if [ "$rc" = "2" ] && echo "$out" | grep -q "already in progress (pid $HOLDER, log $TMP/logs/the-running-one.log)" \
   && ! echo "$out" | grep -q "would install" && [ "$(readlink "$TMP/logs/latest.log")" = "$before" ] \
   && grep -q "already in progress (pid $HOLDER" "$TMP/logs/refused.log" && [ -d "$LOCK" ]; then
    ok "lock: a second start refuses (exit 2), names the holder, does nothing, keeps latest.log"
else bad "lock: second start refused" "rc=$rc latest=$(readlink "$TMP/logs/latest.log") before=$before: $out"; fi
kill "$HOLDER" 2>/dev/null; wait "$HOLDER" 2>/dev/null
# The holder is gone: the lock is stale and is taken over, then released at exit.
out=$(run --dry-run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "taking over a stale lock (pid $HOLDER is gone)" && [ ! -e "$LOCK" ]; then
    ok "lock: a stale lock (holder gone) is taken over and released"
else bad "lock: stale takeover" "rc=$rc: $out"; fi
# Control for the name check: a live process that is NOT a runme.sh (pid reuse) is stale too.
sleep 60 & OTHER=$!
mkdir -p "$LOCK"; echo "$OTHER" > "$LOCK/pid"
out=$(run --dry-run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "taking over a stale lock (pid $OTHER" && [ ! -e "$LOCK" ]; then
    ok "lock: a live pid that is not a runme.sh does not hold the lock (pid reuse)"
else bad "lock: pid-reuse control" "rc=$rc: $out"; fi
kill "$OTHER" 2>/dev/null; wait "$OTHER" 2>/dev/null

# T-3237: cases 3-9 perform a REAL run, and runme.sh refuses non-root by design
# (case 11 pins that). A GitHub runner is non-root, so there these cases can only
# fail on the refusal they are not testing. Skip them ONLY when CI is set AND we
# are not root (T-3234 pattern) — non-root anywhere else still fails loudly.
# T-3443: no longer skipped — RUNME_FIXTURE_ALLOW_NONROOT lets fixture mode run as
# a non-root CI user, so cases 3-9 are asserted everywhere.
REAL_RUN=1
# The summary's already-done count is derived from runme.sh itself, never a literal:
# a literal went stale when T-3068 added a third crontab and failed on every host.
EXPECT_N=$(( $(grep -c '^install_crontab ' "$RUNME") + $(printf '%s\n' "$RUNME_TEST_CLOSES" | grep -c '|') + $(printf '%s\n' "$RUNME_TEST_APPROVED_DECISIONS" | grep -c '|')  + 38 ))   # +2 T-3615 (action 1b background-class resync, action 23 fleet rollup); +2 operator tick records (T-3611: T-3006, T-3200); +1 hub flap ack (T-3587 action 5b); +1 watchtower onto current (T-3568 action 11c); +1 audit crontab sync (T-3554 action 11b); +1 ring20 pack (T-3477 action 22); +1 .122 retention sweeper (T-3311 action 21); +6 driver specs (T-3440 5c: arc-substrate-fitness 3, arc-011 3); +1 W1 drill (T-3426); +1 inbox cap (T-3443); +1 W1 readiness/wait (T-3438); +2 sidecar supervision: settings dir + unit template (T-3435); +1 claude-fw router (T-3421); +1 testdev claude (T-3416); +5 operator tick records (T-3351, T-3393: T-3349, T-3351, T-3388; T-3460: T-3291, T-3302); +1 sidecar sweep cron (T-3370); +1 stray-hub stop (T-3343); +1 reindex schedule (T-3336); +1 termlink install (T-3287); +1 hub unit (T-3299); +1 hub restart (T-3290); +1 fleet hub (T-3290 action 6); +1 zombie reap (T-3297); +1 identity env (T-3303); +1 fedprobe bound (T-2988); +1 sidecar restart (T-3325)

if [ "$REAL_RUN" = "1" ]; then
# ---------------------------------------------------------------------------
# 3-4. A real run installs and VERIFIES. The installed copy must match source
#      byte-for-byte — cp can succeed onto a full disk or a read-only remount
#      and still leave the destination wrong, which is the case "assume success"
#      misses.
# ---------------------------------------------------------------------------
out=$(run); rc=$?
if [ "$rc" = "0" ]; then ok "real run exits 0 when everything installs and verifies"
else bad "real run exits 0" "rc=$rc: $out"; fi
if cmp -s "$REPO_ROOT/.context/cron/notify-sidecar-supervisor.crontab" "$TMP/cron/termlink-notify-sidecar-supervisor"; then
    ok "installed copy is byte-identical to the git-tracked source"
else bad "installed copy matches source"; fi
# T-3272: a default run (no flags) closes all three and verifies each on disk.
n_closed=0
for id in $CLOSE_IDS; do
    f=$(ls "$TMP/tasks/completed/$id"-*.md 2>/dev/null | head -1)
    [ -n "$f" ] && grep -q '^status: work-completed' "$f" && n_closed=$((n_closed+1))
done
if [ "$n_closed" = "3" ] && [ "$(echo "$out" | grep -c 'closed and verified')" = "3" ]; then
    ok "default run closes the 3 approved tasks and verifies each in completed/"
else bad "default run closes approved tasks" "closed=$n_closed: $out"; fi
if echo "$out" | grep -q "recorded and verified: T-9284 = go" && grep -q '^\*\*Decision\*\*: GO$' "$TMP/tasks/active/T-9284-fixture.md"; then
    ok "default run records the approved decision and verifies it on disk"
else bad "default run records decision" "$out"; fi
if echo "$out" | grep -q "installed and verified: $TMP/bin/termlink (termlink 9.9.9" && cmp -s "$TMP/tl-new" "$TMP/bin/termlink"; then
    ok "default run installs the new termlink and verifies version + --no-create on disk"
else bad "termlink install" "$out"; fi

if cmp -s .context/cron/hub-outage-collector.crontab "$TMP/cron/termlink-hub-outage-collector"; then
    ok "default run installs the hub-outage collector + canary crontab and verifies it on disk (T-3383)"
else bad "hub-outage-collector crontab install" "$out"; fi
if cmp -s .context/cron/root-mode-canary.crontab "$TMP/cron/termlink-root-mode-canary"; then
    ok "default run installs the root-mode canary crontab and verifies it on disk (T-3425)"
else bad "root-mode-canary crontab install" "$out"; fi
if cmp -s systemd-templates/termlink-sidecar@.service "$SCU/units/termlink-sidecar@.service" \
   && grep -q '^daemon-reload' "$SCU/calls" && [ -d "$SCU/env" ] \
   && echo "$out" | grep -q "installed, reloaded and verified: $SCU/units/termlink-sidecar@.service"; then
    ok "default run installs the sidecar unit template, reloads systemd, verifies systemd resolves it, creates the settings dir (T-3435)"
else bad "sidecar unit template install" "$out"; fi
if cmp -s .context/cron/sidecar-backstop.crontab "$TMP/cron/termlink-sidecar-backstop"; then
    ok "default run installs the sidecar cron backstop and verifies it on disk (T-3435)"
else bad "sidecar-backstop crontab install" "$out"; fi

# ---------------------------------------------------------------------------
# 5-6. IDEMPOTENCE. Re-running must do nothing and say so — the operator has to
#      be able to re-run without wondering whether they already did.
# ---------------------------------------------------------------------------
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "already installed and identical"; then ok "second run skips as already-done"
else bad "idempotent second run" "rc=$rc: $out"; fi
if echo "$out" | grep -qE "0 done, ${EXPECT_N} already-done"; then ok "summary counts the skips rather than re-claiming the work"
else bad "summary counts skips" "$out"; fi

# ---------------------------------------------------------------------------
# 7. TAMPER / DRIFT. If the installed copy has been edited out-of-band, the
#    script must re-install rather than trust its presence.
# ---------------------------------------------------------------------------
echo "# tampered out of band" >> "$TMP/cron/termlink-notify-sidecar-canary"
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "installed and verified.*termlink-notify-sidecar-canary"; then
    ok "a drifted installed copy is re-installed, not trusted"
else bad "drifted copy re-installed" "rc=$rc: $out"; fi

# ---------------------------------------------------------------------------
# 8-9. A missing source is a FAILED action and a non-zero exit — never a quiet
#      skip. A script that reports success for work it did not do is worse than
#      the copy-paste it replaced.
# ---------------------------------------------------------------------------
STASH="$TMP/stash.crontab"
mv "$REPO_ROOT/.context/cron/notify-sidecar-canary.crontab" "$STASH"
out=$(run); rc=$?
mv "$STASH" "$REPO_ROOT/.context/cron/notify-sidecar-canary.crontab"
if [ "$rc" = "1" ]; then ok "missing source => exit 1 (not a silent skip)"
else bad "missing source => exit 1" "rc=$rc: $out"; fi
if echo "$out" | grep -q "FAILED  source missing"; then ok "missing source names the file"
else bad "missing source named" "$out"; fi

# T-3272: fw exiting 0 is NOT evidence. A fw that claims success but moves nothing
# must be a FAILED close and exit 1 — the verification reads disk, not exit codes.
mv "$TMP/tasks/completed/T-3130-fixture.md" "$TMP/tasks/active/"
sed -i 's/^status: .*/status: started-work/' "$TMP/tasks/active/T-3130-fixture.md"
out=$(RUNME_FW="$TMP/noop-fw" run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "FAILED  T-3130 did not land"; then
    ok "a close that fw reports but disk does not show => FAILED, exit 1"
else bad "unverified close is a failure" "rc=$rc: $out"; fi
# T-3273: a failed run's log says so — the agent reading it must see rc=1 and the FAILED line.
sleep 1   # tee flushes the trap line asynchronously after the script exits
if grep -q 'FAILED  T-3130 did not land' "$TMP/logs/latest.log" && grep -q '=== runme.sh finished rc=1' "$TMP/logs/latest.log"; then
    ok "a failed run's log carries the FAILED line and rc=1"
else bad "failed run logged" "$(tail -5 "$TMP/logs/latest.log" 2>/dev/null)"; fi
printf -- '---\nid: T-9285\n---\n' > "$TMP/tasks/active/T-9285-fixture.md"
out=$(RUNME_FW="$TMP/noop-fw" RUNME_TEST_APPROVED_DECISIONS="T-9285|no-go|fixture" run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "FAILED  T-9285: decision not found"; then
    ok "a decision fw reports but disk does not show => FAILED, exit 1"
else bad "unverified decision is a failure" "rc=$rc: $out"; fi
out=$(RUNME_TERMLINK_SRC="$TMP/tl-old" RUNME_TERMLINK_DEST="$TMP/bin/termlink-2" run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "rejects --no-create"; then
    ok "installing a binary that rejects --no-create => FAILED, exit 1"
else bad "bad termlink install is a failure" "rc=$rc: $out"; fi

fi

# ---------------------------------------------------------------------------
# 10. Unknown argument is refused rather than ignored — a typo'd flag must not
#     silently run the full thing.
# ---------------------------------------------------------------------------
out=$(run --no-such-flag); rc=$?
if [ "$rc" = "2" ]; then ok "unknown argument => exit 2"
else bad "unknown arg => 2" "rc=$rc: $out"; fi

# ---------------------------------------------------------------------------
# 11. Non-root refusal. Skipped rather than faked when the sandbox cannot drop
#     privileges — a fixture that silently tests nothing is worse than an
#     honest skip (T-3105).
# ---------------------------------------------------------------------------
# T-3237: when we are ALREADY non-root (a CI runner), no privilege drop is needed —
# run it as ourselves; that is the real non-root refusal. runuser only works as root.
if [ "$(id -u)" != "0" ]; then
    out=$(env -u RUNME_FIXTURE_ALLOW_NONROOT RUNME_CRON_DIR="$TMP/cron" bash "$RUNME" 2>&1); rc=$?
    if [ "$rc" = "2" ] && echo "$out" | grep -q "needs root"; then ok "non-root run refuses with exit 2 and applies nothing"
    else bad "non-root refusal" "rc=$rc: $out"; fi
elif command -v runuser >/dev/null 2>&1 && id nobody >/dev/null 2>&1; then
    chmod -R a+rx "$TMP" 2>/dev/null || true
    out=$(runuser -u nobody -- env -u RUNME_FIXTURE_ALLOW_NONROOT RUNME_CRON_DIR="$TMP/cron" bash "$RUNME" 2>&1); rc=$?
    if [ "$rc" = "2" ] && echo "$out" | grep -q "needs root"; then ok "non-root run refuses with exit 2 and applies nothing"
    else bad "non-root refusal" "rc=$rc: $out"; fi
else
    echo "  SKIP  non-root refusal (no runuser/nobody available) — NOT asserted"
fi

# ---------------------------------------------------------------------------
# 12. DECISIONS ARE LISTED, NEVER EXECUTED by a default run. This is the
#     load-bearing property of the whole section: a Tier 0 gate exists to require
#     a human, so a script that recorded decisions on its own would be exactly the
#     laundering the gate prevents.
# ---------------------------------------------------------------------------
out=$(run --dry-run)
if echo "$out" | grep -q "Pending decisions"; then ok "default run LISTS pending decisions"
else bad "lists pending decisions" "$out"; fi
if echo "$out" | grep -q "nothing here runs by default"; then ok "the listing states that nothing runs by default"
else bad "listing states non-execution" "$out"; fi
# T-3285: APPROVED decisions (action 3) do run; a PENDING, undecided one (T-9055) never does.
if echo "$out" | grep -q "would record T-9055"; then bad "default run must not record a pending decision" "$out"
else ok "default run records NO pending (undecided) decision"; fi

# ---------------------------------------------------------------------------
# 13. A typo must never resolve to a default verdict. Both halves are checked,
#     because a wrong id and a wrong verdict fail for different reasons and either
#     one silently defaulting would record a decision the human did not make.
# ---------------------------------------------------------------------------
rc=0; RUNME_CRON_DIR="$TMP/cron" bash "$RUNME" --decide T-9999=go >/dev/null 2>&1 || rc=$?
if [ "$rc" = "2" ]; then ok "unknown decision id => exit 2, nothing recorded"
else bad "unknown id refused" "rc=$rc"; fi
rc=0; vout=$(RUNME_CRON_DIR="$TMP/cron" bash "$RUNME" --decide T-9055=maybe 2>&1) || rc=$?
# T-3276: must be refused FOR THE VERDICT — not because the id is unknown, which
# would pass this case vacuously once the real pending list is empty.
if [ "$rc" = "2" ] && echo "$vout" | grep -q "verdict must be go, no-go or defer"; then ok "verdict outside go/no-go/defer => exit 2 (refused for the verdict)"
else bad "bad verdict refused" "rc=$rc: $vout"; fi
rc=0; RUNME_CRON_DIR="$TMP/cron" bash "$RUNME" --decide T-9055 >/dev/null 2>&1 || rc=$?
if [ "$rc" = "2" ]; then ok "malformed --decide (no '=') => exit 2"
else bad "malformed refused" "rc=$rc"; fi

# ---------------------------------------------------------------------------
# 14. The plugin cleanup is OPT-IN: absent from a default run, since it is a
#     recommendation the operator has not approved, not a repair.
# ---------------------------------------------------------------------------
if echo "$out" | grep -qi "disabling purpose-mismatch"; then bad "plugin disable must be opt-in" "$out"
else ok "plugin cleanup absent from a default run"; fi

# ---------------------------------------------------------------------------
# 15. T-3276 — the REAL shape: with no approved closures and no pending decisions
#     (seams unset), a run shows no closures section and says decisions are none.
# ---------------------------------------------------------------------------
eout=$(env -u RUNME_TEST_DECISIONS RUNME_TEST_CLOSES= RUNME_TEST_APPROVED_DECISIONS= RUNME_CRON_DIR="$TMP/cron" bash "$RUNME" --dry-run 2>&1); rc=$?
if [ "$rc" = "0" ] && ! echo "$eout" | grep -q "Approved closures" && ! echo "$eout" | grep -q "Approved inception decisions" && echo "$eout" | grep -q "none pending"; then
    ok "empty lists: no closures section, decisions read 'none pending'"
else bad "empty lists render cleanly" "rc=$rc: $eout"; fi

# ---------------------------------------------------------------------------
# 13. T-3289 — every termlink copy on the host. First destination is always
#     installed; the rest are refreshed only where a copy already exists (a stale
#     /usr/local/bin/termlink was what 32 of 33 crons ran). All paths are scratch.
# ---------------------------------------------------------------------------
M="$TMP/multi"; mkdir -p "$M/cargo" "$M/usrlocal" "$M/dotlocal"
cp "$TMP/tl-old" "$M/usrlocal/termlink"          # stale copy that must be refreshed
DESTS="$M/cargo/termlink $M/usrlocal/termlink $M/dotlocal/termlink"
out=$(RUNME_TERMLINK_DESTS="$DESTS" run); rc=$?
if [ "$rc" = "0" ] && cmp -s "$TMP/tl-new" "$M/cargo/termlink" && cmp -s "$TMP/tl-new" "$M/usrlocal/termlink" \
   && echo "$out" | grep -q "installed and verified: $M/usrlocal/termlink (termlink 9.9.9"; then
    ok "multi-dest: always-install path + stale existing copy both installed and verified"
else bad "multi-dest install" "rc=$rc: $out"; fi
if [ ! -e "$M/dotlocal/termlink" ] && echo "$out" | grep -q "no termlink at $M/dotlocal/termlink — not creating one"; then
    ok "multi-dest: absent optional path is NOT created, and the log says so"
else bad "refresh-if-present must not create" "$(ls -la "$M/dotlocal"): $out"; fi
out=$(RUNME_TERMLINK_DESTS="$DESTS" run); rc=$?
if [ "$rc" = "0" ] && [ "$(echo "$out" | grep -c "skip    already installed: $M/")" = "2" ]; then
    ok "multi-dest: second run skips both installed paths (idempotent)"
else bad "multi-dest idempotence" "rc=$rc: $out"; fi
: > "$M/blocker"                                   # a FILE where a directory must be
cp "$TMP/tl-old" "$M/usrlocal/termlink"
out=$(RUNME_TERMLINK_DESTS="$M/cargo/termlink $M/usrlocal/termlink $M/blocker/sub/termlink" run); rc=$?
# the blocker path does not exist, so refresh-if-present skips it; force it as the FIRST (always) dest:
out2=$(RUNME_TERMLINK_DESTS="$M/blocker/sub/termlink $M/usrlocal/termlink" run 2>&1); rc2=$?
if [ "$rc" = "0" ] && [ "$rc2" = "1" ] && echo "$out2" | grep -q "FAILED  could not install $TMP/tl-new -> $M/blocker/sub/termlink" \
   && cmp -s "$TMP/tl-new" "$M/usrlocal/termlink"; then
    ok "multi-dest: one failing path => FAILED naming it, exit 1, the other path still installed"
else bad "per-path failure" "rc=$rc rc2=$rc2: $out2"; fi

# ---------------------------------------------------------------------------
# 14. T-3290 — hub restart. All through the fake systemctl / proc / runtime dir.
# ---------------------------------------------------------------------------
hub_state 200 "$H/hubbin"
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "skip    hub (pid 200) already runs the installed binary" && [ ! -e "$H/restarts" ]; then
    ok "hub: already on the installed binary => skip, no restart"
else bad "hub idempotent skip" "rc=$rc: $out"; fi
hub_state 300 "/old/termlink (deleted)"
out=$(run --dry-run); rc=$?
if echo "$out" | grep -q "would restart termlink-hub (pid 300 runs '/old/termlink (deleted)'" && [ ! -e "$H/restarts" ]; then
    ok "hub: --dry-run reports the restart and does NOT perform it"
else bad "hub dry-run" "rc=$rc: $out"; fi
hub_state 400 "/old/termlink (deleted)"
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "OK      restarted and verified: pid 400 -> 401" && [ "$(wc -l < "$H/restarts")" = "1" ]; then
    ok "hub: stale hub restarted once, new pid on the installed exe, secret+cert unchanged => OK"
else bad "hub restart" "rc=$rc: $out"; fi
hub_state 450 "/old/termlink (deleted)"
out=$(FAKE_HUB_STATUS_LAG=1 run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "OK      restarted and verified: pid 450 -> 451"; then
    ok "hub (T-3450): hub status lagging one read behind the restart is re-read, not a false FAILED"
else bad "hub status lag" "rc=$rc: $out"; fi
hub_state 460 "/old/termlink (deleted)"
out=$(FAKE_HUB_STATUS_LAG=999 run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "hub-status-not-running-as-461"; then
    ok "hub (T-3450): hub status that never reads running still FAILS, named"
else bad "hub status never running" "rc=$rc: $out"; fi
# T-3587 action 5b: acknowledge flap residue with reset-failed (hub already current, so 5 skips).
hub_state 100 "$H/hubbin"; rm -f "$H/resets"
echo 3 > "$H/nrestarts"; out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "skip    termlink-hub NRestarts=3 (max 5)" && [ ! -e "$H/resets" ]; then
    ok "flap ack: NRestarts at or below the max is a skip, nothing reset"
else bad "flap ack skip" "rc=$rc: $out"; fi
echo 433 > "$H/nrestarts"; out=$(run --dry-run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "would run: systemctl reset-failed termlink-hub (NRestarts=433" && [ ! -e "$H/resets" ]; then
    ok "flap ack: --dry-run names the reset and performs none"
else bad "flap ack dry-run" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "OK      acknowledged: termlink-hub NRestarts 433 -> 0" && [ "$(wc -l < "$H/resets")" = "1" ]; then
    ok "flap ack: active unit above the max is reset once and verified (433 -> 0)"
else bad "flap ack reset" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "skip    termlink-hub NRestarts=0" && [ "$(wc -l < "$H/resets")" = "1" ]; then
    ok "flap ack: second run skips (idempotent)"
else bad "flap ack idempotent" "rc=$rc: $out"; fi
echo 433 > "$H/nrestarts"; rm -f "$H/resets"; out=$(FAKE_HUB_ACTIVE=activating run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "live crash loop, not residue; not acknowledging" && [ ! -e "$H/resets" ]; then
    ok "flap ack: a unit that is not active is a live incident => FAILED, never reset"
else bad "flap ack not active" "rc=$rc: $out"; fi
out=$(FAKE_HUB_RESET_STICKY=1 run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "reset-failed did not clear the counter: NRestarts=433"; then
    ok "flap ack: a reset that does not clear the counter is FAILED, not OK"
else bad "flap ack sticky" "rc=$rc: $out"; fi
echo 0 > "$H/nrestarts"; rm -f "$H/resets"
# T-3592 action 2b: hand human-owned tasks back to the agent (scratch tasks only).
printf -- '---\nid: T-9592\nstatus: captured\nowner: human\n---\n' > "$TMP/tasks/active/T-9592-fixture.md"
printf -- '---\nid: T-9593\nstatus: captured\nowner: agent\n---\n' > "$TMP/tasks/active/T-9593-fixture.md"
out=$(RUNME_TEST_HANDBACKS="T-9592 T-9593 T-9594" run --dry-run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "would hand T-9592 back" && grep -q '^owner: human' "$TMP/tasks/active/T-9592-fixture.md"; then
    ok "handback: --dry-run names the hand-back and changes nothing"
else bad "handback dry-run" "rc=$rc: $out"; fi
out=$(RUNME_TEST_HANDBACKS="T-9592 T-9593 T-9594" run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "OK      handed back and verified: T-9592 owner: agent" \
   && echo "$out" | grep -q "skip    T-9593 already owner: agent" && echo "$out" | grep -q "skip    T-9594 is no longer active" \
   && grep -q '^owner: agent' "$TMP/tasks/active/T-9592-fixture.md"; then
    ok "handback: human-owned task handed back and verified; agent-owned and missing ones skip"
else bad "handback run" "rc=$rc: $out"; fi
sed -i 's/^owner: .*/owner: human/' "$TMP/tasks/active/T-9592-fixture.md"
out=$(FAKE_HANDBACK_REFUSE=1 RUNME_TEST_HANDBACKS="T-9592" run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "FAILED  T-9592 still not owner: agent"; then
    ok "handback: fw that claims success but changes nothing => FAILED, not OK"
else bad "handback refuse" "rc=$rc: $out"; fi
rm -f "$TMP/tasks/active/T-9592-fixture.md" "$TMP/tasks/active/T-9593-fixture.md"
hub_state 500 "/old/termlink (deleted)"
out=$(FAKE_HUB_RESTART_MODE=rotate run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "hub.secret-CHANGED"; then
    ok "hub: secret rotated by the restart => FAILED naming it (PL-021 class)"
else bad "hub secret rotation must fail" "rc=$rc: $out"; fi
hub_state 600 "/old/termlink (deleted)"
out=$(FAKE_HUB_NEW_EXE="/old/termlink (deleted)" run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "exe='/old/termlink (deleted)'"; then
    ok "hub: restarted onto the wrong binary => FAILED"
else bad "hub wrong exe must fail" "rc=$rc: $out"; fi
hub_state 700 "/old/termlink (deleted)"
out=$(FAKE_HUB_RESTART_MODE=fail run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "FAILED  systemctl restart termlink-hub failed"; then
    ok "hub: systemctl restart failure => FAILED, exit 1"
else bad "hub restart failure" "rc=$rc: $out"; fi
hub_state 800 "/old/termlink (deleted)"
out=$(FAKE_HUB_ACTIVE=failed run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "unit=failed"; then
    ok "hub: unit not active after restart => FAILED"
else bad "hub inactive must fail" "rc=$rc: $out"; fi
hub_state 100 "$H/hubbin"

# ---------------------------------------------------------------------------
# 15. T-3290 action 6 — remote fleet upgrade, all through fakes.
# ---------------------------------------------------------------------------
fleet_state 9.9.9
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "skip    fake-hub already serves 9.9.9" && [ ! -e "$F/deploys" ]; then
    ok "fleet: hub already at a current-code version => skip, no deploy"
else bad "fleet idempotent skip" "rc=$rc: $out"; fi
fleet_state 0.11.1
out=$(run --dry-run); rc=$?
if echo "$out" | grep -q "would stage musl 9.9.9 on fake-hub (10.0.0.1:9100, now 0.11.1)" && [ ! -e "$F/deploys" ] && [ ! -e "$F/remote-cmds" ]; then
    ok "fleet: --dry-run reports the upgrade and does NOT perform it"
else bad "fleet dry-run" "rc=$rc: $out"; fi
fleet_state 0.11.1
out=$(FAKE_UNIT=1 run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "OK      upgraded and verified: fake-hub serves 9.9.9" \
   && grep -q -- "--binary $F/musl --dst /tmp/termlink.new --probe" "$F/deploys" && ! grep -q -- "--swap-restart" "$F/deploys" \
   && grep -q "install -m 755 /tmp/termlink.new /usr/local/bin/termlink" "$F/remote-cmds" && grep -q "systemctl restart termlink-hub" "$F/remote-cmds"; then
    ok "fleet: systemd-supervised hub => stage+probe, install over the unit's binary, restart THROUGH systemd (G-070), no --swap-restart"
else bad "fleet systemd path" "rc=$rc: $out $(cat "$F/deploys" "$F/remote-cmds" 2>/dev/null)"; fi
fleet_state 0.11.1
out=$(FAKE_UNIT=0 run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "OK      upgraded and verified" && grep -q -- "--swap-restart" "$F/deploys" \
   && ! grep -q "systemctl restart" "$F/remote-cmds"; then
    ok "fleet: hub with no unit (watchdog-launched) => --swap-restart, never systemctl"
else bad "fleet watchdog path" "rc=$rc: $out $(cat "$F/deploys" 2>/dev/null)"; fi
fleet_state 0.11.1
out=$(FAKE_SWAP_CONFIRM=0 run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "install/restart on fake-hub did not confirm: install: cannot create"; then
    ok "fleet: unconfirmed systemd install/restart => FAILED naming the remote error"
else bad "fleet unconfirmed swap" "rc=$rc: $out"; fi
fleet_state 0.11.1
cp "$TMP/tl-old" "$F/musl"; printf '#!/usr/bin/env bash\nexit 1\n' > "$F/musl"
out=$(run); rc=$?
cp "$TMP/tl-new" "$F/musl"
if [ "$rc" = "1" ] && echo "$out" | grep -q "musl binary $F/musl does not run — nothing shipped" && [ ! -e "$F/deploys" ]; then
    ok "fleet: musl binary that does not run => FAILED, nothing shipped"
else bad "fleet broken musl" "rc=$rc: $out"; fi
fleet_state 0.11.1
out=$(FAKE_DEPLOY_MODE=fail run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "FAILED  staging on fake-hub failed" && [ ! -e "$F/remote-cmds" ]; then
    ok "fleet: staging/probe failure => FAILED, no swap attempted"
else bad "fleet staging failure" "rc=$rc: $out"; fi
fleet_state 0.11.1
out=$(FAKE_DEPLOY_MODE=auth run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "status='auth-mismatch'"; then
    ok "fleet: hub up but auth broken after the swap (secret rotated) => FAILED"
else bad "fleet auth regression must fail" "rc=$rc: $out"; fi
fleet_state 0.11.1
out=$(FAKE_TOFU_RC=1 run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "tofu verify failed"; then
    ok "fleet: cert changed (tofu verify fails) => FAILED"
else bad "fleet cert rotation must fail" "rc=$rc: $out"; fi
fleet_state 9.9.9

# ---------------------------------------------------------------------------
# 16. T-3292 — cached builds. A fake cargo counts invocations; a scratch git repo
#     stands in for the project (RUNME_BUILD_REPO) so commits are controlled.
#     The fleet hub is current, so only the HOST build is ever considered.
# ---------------------------------------------------------------------------
B="$TMP/build"; mkdir -p "$B/repo/crates/x" "$B/stamps"
printf '#!/usr/bin/env bash\necho "$*" >> "%s/cargo-calls"\n' "$B" > "$B/cargo"; chmod +x "$B/cargo"
git -C "$B/repo" init -q && git -C "$B/repo" config user.email f@x && git -C "$B/repo" config user.name f
echo a > "$B/repo/crates/x/lib.rs"; echo d > "$B/repo/README"; git -C "$B/repo" add -A && git -C "$B/repo" commit -qm init
brun() { env -u RUNME_SKIP_BUILD RUNME_CARGO="$B/cargo" RUNME_BUILD_REPO="$B/repo" RUNME_BUILD_STAMP_DIR="$B/stamps" \
             RUNME_CRON_DIR="$TMP/cron" bash "$RUNME" "$@" 2>&1; }
calls() { [ -f "$B/cargo-calls" ] && wc -l < "$B/cargo-calls" || echo 0; }
out=$(brun); rc=$?
if [ "$rc" = "0" ] && [ "$(calls)" = "1" ] && grep -q -- "--profile local-fast -p termlink" "$B/cargo-calls" \
   && [ "$(cat "$B/stamps/host")" = "$(git -C "$B/repo" rev-parse HEAD)" ]; then
    ok "build cache: no record yet => cargo builds with the local-fast profile and records HEAD"
else bad "first build" "rc=$rc calls=$(calls): $out"; fi
out=$(brun); rc=$?
if [ "$rc" = "0" ] && [ "$(calls)" = "1" ] && echo "$out" | grep -q "cached  host build is current"; then
    ok "build cache: nothing changed => no cargo, says cached"
else bad "cached rerun" "rc=$rc calls=$(calls): $out"; fi
echo d2 > "$B/repo/README"; git -C "$B/repo" commit -qam docs
out=$(brun); rc=$?
if [ "$rc" = "0" ] && [ "$(calls)" = "1" ]; then
    ok "build cache: docs-only commit => still no rebuild (the every-commit rebuild is gone)"
else bad "docs-only commit must not rebuild" "rc=$rc calls=$(calls): $out"; fi
echo b > "$B/repo/crates/x/lib.rs"
out=$(brun); rc=$?
if [ "$rc" = "0" ] && [ "$(calls)" = "2" ]; then
    ok "build cache: uncommitted change under crates/ => rebuild"
else bad "dirty crates must rebuild" "rc=$rc calls=$(calls): $out"; fi
git -C "$B/repo" commit -qam code
out=$(brun); rc=$?
if [ "$rc" = "0" ] && [ "$(calls)" = "3" ]; then
    ok "build cache: committed change under crates/ since the recorded build => rebuild"
else bad "code commit must rebuild" "rc=$rc calls=$(calls): $out"; fi

# ---------------------------------------------------------------------------
# 17. T-3297 — zombie reap, all through a fake detector / proc / kill.
# ---------------------------------------------------------------------------
zombie_reset; zombie_proc 501 z-one; zombie_proc 502 z-two
out=$(FAKE_ZOMBIES="501:z-one 502:z-two" run --dry-run); rc=$?
if echo "$out" | grep -q "targets 2 zombie session(s) — listed before any is touched" && echo "$out" | grep -q "pid 501  z-one" \
   && echo "$out" | grep -q "would SIGTERM these 2" && [ ! -e "$Z/killed" ]; then
    ok "zombie: --dry-run lists every target and signals nothing"
else bad "zombie dry-run" "rc=$rc: $out"; fi
out=$(FAKE_ZOMBIES="501:z-one 502:z-two" run); rc=$?
lst=$(echo "$out" | grep -n "pid 502  z-two" | cut -d: -f1); okl=$(echo "$out" | grep -n "OK      reaped and verified: 2 session" | cut -d: -f1)
if [ "$rc" = "0" ] && [ -n "$lst" ] && [ -n "$okl" ] && [ "$lst" -lt "$okl" ] && [ "$(wc -l < "$Z/killed")" = "2" ]; then
    ok "zombie: targets listed before acting, both SIGTERMed and verified gone"
else bad "zombie reap" "rc=$rc: $out"; fi
if ! echo "$out" | grep -q "No such file or directory"; then
    ok "zombie: an exited session's re-check prints no shell error into the log (T-3301)"
else bad "zombie re-check stderr noise" "$(echo "$out" | grep -m2 'No such file')"; fi
zombie_reset; zombie_proc 601 z-real
mkdir -p "$Z/proc/602"; printf 'vim\0notes.txt\0' > "$Z/proc/602/cmdline"   # pid 602 was reused
out=$(FAKE_ZOMBIES="601:z-real 602:z-gone" run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "pid 602 is no longer register 'z-gone' — not signalled" \
   && ! grep -qx 602 "$Z/killed" && grep -qx 601 "$Z/killed" && [ -d "$Z/proc/602" ]; then
    ok "zombie: a reused pid (cmdline no longer that session) is skipped, never signalled"
else bad "zombie pid-reuse guard" "rc=$rc: $out $(cat "$Z/killed" 2>/dev/null)"; fi
zombie_reset; zombie_proc 701 z-stuck
out=$(FAKE_ZOMBIES="701:z-stuck" FAKE_STUCK=701 run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "still running after SIGTERM (not escalated to SIGKILL): 701(z-stuck)" \
   && [ "$(wc -l < "$Z/killed")" = "1" ]; then
    ok "zombie: a session that ignores SIGTERM => FAILED naming it, signalled once, no SIGKILL"
else bad "zombie stuck" "rc=$rc: $out"; fi
zombie_reset
out=$(FAKE_ZOMBIES="" run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "skip    no zombie sessions" && [ ! -e "$Z/killed" ]; then
    ok "zombie: no zombies => skip (idempotent)"
else bad "zombie idempotent" "rc=$rc: $out"; fi
out=$(RUNME_ZOMBIE_DETECT=false run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "zombie detector could not run — nothing reaped"; then
    ok "zombie: detector failure => FAILED, nothing reaped (fail-closed)"
else bad "zombie detector failure" "rc=$rc: $out"; fi

# ---------------------------------------------------------------------------
# 18. T-3299 — hub unit install, through the fake systemctl / proc.
# ---------------------------------------------------------------------------
hub_state 900 "$H/hubbin"; printf 'unit-v2\n' > "$H/unit.src"
out=$(run --dry-run); rc=$?
if echo "$out" | grep -q "would install $H/unit.src -> $H/unit.dest, daemon-reload, restart termlink-hub" && grep -q unit-v1 "$H/unit.dest" && [ ! -e "$H/restarts" ]; then
    ok "hub unit: --dry-run reports the install and changes nothing"
else bad "hub unit dry-run" "rc=$rc: $out"; fi
hub_state 910 "$H/hubbin"
out=$(FAKE_HUB_MOUNTINFO="36 25 0:1 / $H/rwpool rw,relatime - btrfs /dev/x rw" run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "OK      unit installed and verified: termlink-hub active (pid 911); $H/rwpool read-write" \
   && cmp -s "$H/unit.src" "$H/unit.dest"; then
    ok "hub unit: differing unit installed, hub restarted, pool verified read-write in the hub's namespace"
else bad "hub unit install" "rc=$rc: $out"; fi
hub_state 920 "$H/hubbin"; printf 'unit-v3\n' > "$H/unit.src"
out=$(FAKE_HUB_MOUNTINFO="36 25 0:1 / $H/rwpool ro,relatime - btrfs /dev/x ro" run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "rwpool-not-writable-in-hub-namespace"; then
    ok "hub unit: pool still read-only for the hub after install => FAILED (the T-3299 defect)"
else bad "hub unit ro must fail" "rc=$rc: $out"; fi
cp "$H/unit.src" "$H/unit.dest"; hub_state 100 "$H/hubbin"

# ---------------------------------------------------------------------------
# 19. T-3299 — fixture mode refuses real defaults. Dropping one seam must abort
#     the whole run before any action touches the host.
# ---------------------------------------------------------------------------
before="$(sha256sum /etc/systemd/system/termlink-hub.service 2>/dev/null)"
out=$(env -u RUNME_HUB_UNIT_DEST RUNME_CRON_DIR="$TMP/cron" bash "$RUNME" 2>&1); rc=$?
after="$(sha256sum /etc/systemd/system/termlink-hub.service 2>/dev/null)"
if [ "$rc" = "2" ] && echo "$out" | grep -q "FIXTURE run reached the REAL default for RUNME_HUB_UNIT_DEST" && [ "$before" = "$after" ]; then
    ok "fixture mode: a missing seam aborts the run (exit 2) and the real unit file is untouched"
else bad "fixture guard" "rc=$rc: $(echo "$out" | tail -3)"; fi

# ---------------------------------------------------------------------------
# 20. T-3303 — identity env merged into Claude local settings, scratch file only.
# ---------------------------------------------------------------------------
CL="$TMP/claude-local.json"
printf '{"permissions":{"allow":["Bash(ls:*)"]},"enableAllProjectMcpServers":true,"env":{"KEEP":"1"}}\n' > "$CL"
out=$(run --dry-run); rc=$?
if echo "$out" | grep -q "would add env.TERMLINK_AGENT_ID=claude-termlink" && ! grep -q TERMLINK_AGENT_ID "$CL"; then
    ok "identity env: --dry-run reports and changes nothing"
else bad "identity env dry-run" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "OK      set and verified: $CL" && python3 -c "
import json,sys; d=json.load(open('$CL'))
assert d['env']=={'KEEP':'1','TERMLINK_AGENT_ID':'claude-termlink'}, d
assert d['permissions']=={'allow':['Bash(ls:*)']} and d['enableAllProjectMcpServers'] is True"; then
    ok "identity env: key added, existing env and all other keys preserved"
else bad "identity env merge" "rc=$rc: $out $(cat "$CL")"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "skip    $CL already sets TERMLINK_AGENT_ID=claude-termlink"; then
    ok "identity env: second run skips (idempotent)"
else bad "identity env idempotent" "rc=$rc: $out"; fi

# ---------------------------------------------------------------------------
# 21. T-2988 — bound health:ring20-fedprobe (fake termlink, scratch state only).
# ---------------------------------------------------------------------------
# State at the first run's end (messages:100, 104 records) -> latest-per-cv-key.
echo "messages 100 104" > "$FAKE_FEDPROBE_STATE"
out=$(run --dry-run); rc=$?
if echo "$out" | grep -q "would set health:ring20-fedprobe retention messages -> latest-per-cv-key" && grep -q "^messages 100 104$" "$FAKE_FEDPROBE_STATE"; then
    ok "fedprobe: --dry-run reports and changes nothing"
else bad "fedprobe dry-run" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "OK      health:ring20-fedprobe now keeps the latest record per cv_key; swept to 1 records (verified)" && grep -q "^latest_per_cv_key 0 1$" "$FAKE_FEDPROBE_STATE"; then
    ok "fedprobe: latest-per-cv-key set, swept to one record, verified"
else bad "fedprobe bound" "rc=$rc: $out $(cat "$FAKE_FEDPROBE_STATE")"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "skip    health:ring20-fedprobe already keeps the latest record per cv_key"; then
    ok "fedprobe: second run skips (idempotent)"
else bad "fedprobe idempotent" "rc=$rc: $out"; fi
echo "forever 0 2777" > "$FAKE_FEDPROBE_STATE"
out=$(FAKE_FEDPROBE_NOSWEEP=1 run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep -q "FAILED  health:ring20-fedprobe: after set-retention + sweep it reads 'latest_per_cv_key 0 2777'"; then
    ok "fedprobe: a sweep that does not take effect is FAILED, not OK"
else bad "fedprobe no-sweep" "rc=$rc: $out"; fi
out=$(FAKE_FEDPROBE_MISSING=1 run); rc=$?
if echo "$out" | grep -q "skip    health:ring20-fedprobe not found on the local hub"; then
    ok "fedprobe: missing topic is a skip"
else bad "fedprobe missing" "rc=$rc: $out"; fi
echo "latest_per_cv_key 0 1" > "$FAKE_FEDPROBE_STATE"

# ---------------------------------------------------------------------------
# 22. T-3325 — restart stale notify-sidecars (fake detector/kill/supervisor only).
# ---------------------------------------------------------------------------
STALE2='{"stale_count":2,"current_count":0,"stale":[{"pid":111,"agent_id":"a1"},{"pid":222,"agent_id":"a2"}]}'
echo "$STALE2" > "$FAKE_SIDECAR_STATE"; : > "$FAKE_SIDECAR_KILLED"
out=$(run --dry-run); rc=$?
if echo "$out" | grep -q "would restart 2 stale sidecar(s): a1 a2" && [ ! -s "$FAKE_SIDECAR_KILLED" ]; then
    ok "sidecars: --dry-run reports and signals nothing"
else bad "sidecars dry-run" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "OK      restarted 2 sidecar(s) onto current code: a1 a2 (verified: 0 stale, 2 current)" \
   && [ "$(tr '\n' ' ' < "$FAKE_SIDECAR_KILLED")" = "111 222 " ]; then
    ok "sidecars: stale pids signalled, supervisor restarts them, verified"
else bad "sidecars restart" "rc=$rc: $out killed=$(cat "$FAKE_SIDECAR_KILLED")"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "skip    all 2 notify-sidecar(s) already run the current code"; then
    ok "sidecars: second run skips (idempotent)"
else bad "sidecars idempotent" "rc=$rc: $out"; fi
echo "$STALE2" > "$FAKE_SIDECAR_STATE"
out=$(FAKE_SIDECAR_NOFIX=1 run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep -q "FAILED  after restart: stale=2"; then
    ok "sidecars: a restart that leaves them stale is FAILED, not OK"
else bad "sidecars no-fix" "rc=$rc: $out"; fi
echo "not json" > "$FAKE_SIDECAR_STATE"
out=$(run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep -q "FAILED  check-stale-sidecar-code gave no readable result"; then
    ok "sidecars: unreadable detector output is FAILED (fail-closed), never a skip"
else bad "sidecars unreadable" "rc=$rc: $out"; fi
echo '{"stale_count":0,"current_count":3,"stale":[]}' > "$FAKE_SIDECAR_STATE"

# ---------------------------------------------------------------------------
# 23. T-3336 — schedule the hourly vector reindex (fake fw cron install, temp file only).
# ---------------------------------------------------------------------------
: > "$RUNME_AUDIT_CRON_FILE"
out=$(run --dry-run); rc=$?
if echo "$out" | grep -q "would run: fw cron install" && [ ! -s "$RUNME_AUDIT_CRON_FILE" ]; then
    ok "reindex: --dry-run reports and installs nothing"
else bad "reindex dry-run" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "OK      hourly vector reindex installed and verified" && grep -q 'index reindex' "$RUNME_AUDIT_CRON_FILE"; then
    ok "reindex: installed and verified on disk"
else bad "reindex install" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "skip    .* already schedules the hourly vector reindex"; then
    ok "reindex: second run skips (idempotent)"
else bad "reindex idempotent" "rc=$rc: $out"; fi
: > "$RUNME_AUDIT_CRON_FILE"
out=$(FAKE_CRON_NOINSTALL=1 run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep -q "FAILED  fw cron install ran but"; then
    ok "reindex: an install that leaves no line is FAILED, not OK"
else bad "reindex no-install" "rc=$rc: $out"; fi
cron_both

# 25b. T-3554 — deploy the regenerated audit crontab when source and installed differ.
SRC_SAVED="$RUNME_AUDIT_CRON_SRC"
export RUNME_AUDIT_CRON_SRC="$TMP/agentic-audit-src.crontab"
{ cat "$RUNME_AUDIT_CRON_FILE"; echo '# regenerated by fw upgrade'; } > "$RUNME_AUDIT_CRON_SRC"
out=$(run --dry-run); rc=$?
if echo "$out" | grep -q "would run: fw cron install (deploy the regenerated registry crontab" && ! cmp -s "$RUNME_AUDIT_CRON_SRC" "$RUNME_AUDIT_CRON_FILE"; then
    ok "audit crontab: --dry-run reports and installs nothing"
else bad "audit crontab dry-run" "rc=$rc: $out"; fi
out=$(FAKE_CRON_NOINSTALL=1 run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep -q "FAILED  fw cron install ran but .* still differs"; then
    ok "audit crontab: an install that leaves it different is FAILED, not OK"
else bad "audit crontab no-install" "rc=$rc: $out"; fi
out=$(FAKE_CRON_COPY_SRC=1 run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "OK      registry crontab deployed and verified identical" && cmp -s "$RUNME_AUDIT_CRON_SRC" "$RUNME_AUDIT_CRON_FILE"; then
    ok "audit crontab: deployed and verified identical on disk"
else bad "audit crontab install" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "skip    .* already equals the registry source"; then
    ok "audit crontab: second run skips (idempotent)"
else bad "audit crontab idempotent" "rc=$rc: $out"; fi
export RUNME_AUDIT_CRON_SRC="$SRC_SAVED"
cron_both

# 25c. T-3568 — restart a stale Watchtower onto current web/ code; skip a current one.
: > "$TMP_WT_STALE"
out=$(run --dry-run); rc=$?
if echo "$out" | grep -q "would run: fw watchtower restart" && [ -e "$TMP_WT_STALE" ]; then
    ok "watchtower: --dry-run reports and restarts nothing"
else bad "watchtower dry-run" "rc=$rc: $out"; fi
out=$(FAKE_WT_RESTART_NOOP=1 run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep -q "FAILED  fw watchtower restart ran but Watchtower is still stale"; then
    ok "watchtower: a restart that leaves it stale is FAILED, not OK"
else bad "watchtower restart-noop" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "OK      Watchtower restarted onto the current web/ code" && [ ! -e "$TMP_WT_STALE" ]; then
    ok "watchtower: stale -> restarted and verified current"
else bad "watchtower restart" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "skip    Watchtower already serves the current web/ code"; then
    ok "watchtower: second run skips (idempotent)"
else bad "watchtower idempotent" "rc=$rc: $out"; fi

# 25. T-3370 — install the cron jobs AEF 1.8.3 added (sidecar sweep), temp file only.
echo '20 * * * * root cd "/x" && "/x/fw" index reindex' > "$RUNME_AUDIT_CRON_FILE"
out=$(run --dry-run); rc=$?
if echo "$out" | grep -q "would run: fw cron install (adds the 5-minute 'fw sidecar sweep'" && ! grep -q 'sidecar sweep' "$RUNME_AUDIT_CRON_FILE"; then
    ok "sidecar sweep: --dry-run reports and installs nothing"
else bad "sweep dry-run" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "OK      sidecar ack-ledger sweep installed and verified" && grep -q 'sidecar sweep' "$RUNME_AUDIT_CRON_FILE"; then
    ok "sidecar sweep: installed and verified on disk"
else bad "sweep install" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "skip    .* already schedules the sidecar ack-ledger sweep"; then
    ok "sidecar sweep: second run skips (idempotent)"
else bad "sweep idempotent" "rc=$rc: $out"; fi
echo '20 * * * * root cd "/x" && "/x/fw" index reindex' > "$RUNME_AUDIT_CRON_FILE"
out=$(FAKE_CRON_NOINSTALL=1 run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep -q "FAILED  fw cron install ran but .* has no 'sidecar sweep' line"; then
    ok "sidecar sweep: an install that leaves no line is FAILED, not OK"
else bad "sweep no-install" "rc=$rc: $out"; fi
cron_both

# 24. T-3343 — stop the stray hub (fake termlink, fake liveness, fake /proc only).
out=$(run); rc=$?
if echo "$out" | grep -q "skip    no live hub recorded in $TMP/stray"; then
    ok "stray hub: no pidfile in the stray dir -> skip"
else bad "stray none" "rc=$rc: $out"; fi
echo 4242 > "$TMP/stray/hub.pid"; echo alive > "$FAKE_STRAY_STATE"
out=$(run --dry-run); rc=$?
if echo "$out" | grep -q "would stop stray hub pid 4242" && grep -q alive "$FAKE_STRAY_STATE"; then
    ok "stray hub: --dry-run reports and stops nothing"
else bad "stray dry-run" "rc=$rc: $out"; fi
out=$(FAKE_STRAY_NOGUARD=1 run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep -q "FAILED  installed termlink lacks the T-3340 guard" && grep -q alive "$FAKE_STRAY_STATE"; then
    ok "stray hub: refuses while an installed termlink lacks the guard"
else bad "stray noguard" "rc=$rc: $out"; fi
out=$(RUNME_STRAY_RESCUE_REPORT="$TMP/no-such-report.md" run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep -q "FAILED  rescue report" && grep -q alive "$FAKE_STRAY_STATE"; then
    ok "stray hub: refuses before the mail is rescued"
else bad "stray noreport" "rc=$rc: $out"; fi
printf 'bash\0sleep\0' > "$TMP/proc/4242/cmdline"
out=$(run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep -q "is not a termlink hub process" && grep -q alive "$FAKE_STRAY_STATE"; then
    ok "stray hub: a reused pid that is not a termlink hub is never stopped"
else bad "stray foreign pid" "rc=$rc: $out"; fi
printf 'termlink\0hub\0start\0' > "$TMP/proc/4242/cmdline"
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "OK      stray hub pid 4242 stopped; a client without TERMLINK_RUNTIME_DIR now resolves the canonical hub pid 906"; then
    ok "stray hub: stopped and the canonical hub verified"
else bad "stray stop" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "skip    no live hub recorded"; then
    ok "stray hub: second run skips (idempotent)"
else bad "stray idempotent" "rc=$rc: $out"; fi
echo alive > "$FAKE_STRAY_STATE"
out=$(FAKE_STRAY_NOSTOP=1 run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep -q "still alive 15 s after 'hub stop'"; then
    ok "stray hub: a hub that will not stop is FAILED, never killed hard"
else bad "stray nostop" "rc=$rc: $out"; fi
rm -f "$TMP/stray/hub.pid"; echo alive > "$FAKE_STRAY_STATE"

# 26. T-3351 — record the operator's own Human tick (T-3349), temp ledger only.
: > "$RUNME_HUMAN_TICKS_LEDGER"
out=$(run --dry-run); rc=$?
if echo "$out" | grep -q "would record that you ticked T-3349 Human AC#1" && [ ! -s "$RUNME_HUMAN_TICKS_LEDGER" ]; then
    ok "operator tick: --dry-run reports and records nothing"
else bad "tick dry-run" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "OK      recorded that you ticked T-3349 Human AC#1 (verified" \
   && grep -q '"via": "operator-ack"' "$RUNME_HUMAN_TICKS_LEDGER"; then
    ok "operator tick: recorded and verified in the ledger"
else bad "tick record" "rc=$rc: $out"; fi
# T-3393: every listed tick is recorded, one row per task, not just the first.
if echo "$out" | grep -q "OK      recorded that you ticked T-3351 Human AC#1" \
   && echo "$out" | grep -q "OK      recorded that you ticked T-3388 Human AC#1" \
   && echo "$out" | grep -q "OK      recorded that you ticked T-3291 Human AC#1" \
   && echo "$out" | grep -q "OK      recorded that you ticked T-3302 Human AC#1" \
   && echo "$out" | grep -q "OK      recorded that you ticked T-3006 Human AC#1" \
   && echo "$out" | grep -q "OK      recorded that you ticked T-3200 Human AC#1" \
   && [ "$(grep -c '"via": "operator-ack"' "$RUNME_HUMAN_TICKS_LEDGER")" = "7" ]; then
    ok "operator tick: all seven listed ticks recorded, one ledger row each"
else bad "tick record all" "rows=$(grep -c operator-ack "$RUNME_HUMAN_TICKS_LEDGER"): $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "skip    T-3349 Human AC#1: your tick is already recorded"; then
    ok "operator tick: second run skips (idempotent)"
else bad "tick idempotent" "rc=$rc: $out"; fi
: > "$RUNME_HUMAN_TICKS_LEDGER"
out=$(FAKE_TICK_NOWRITE=1 run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep -q "FAILED  human_ac_ticks ack T-3349 --ac 1 ran but"; then
    ok "operator tick: an ack that records nothing is FAILED, not OK"
else bad "tick nowrite" "rc=$rc: $out"; fi
ticks_recorded

# 27. T-3416 — current Claude Code for testdev (fake binary and installer only).
rm -f "$RUNME_TESTDEV_CLAUDE"
out=$(run --dry-run); rc=$?
if echo "$out" | grep -q "would install Claude Code for testdev" && [ ! -e "$RUNME_TESTDEV_CLAUDE" ]; then
    ok "testdev claude: --dry-run reports and installs nothing"
else bad "testdev dry-run" "rc=$rc: $out"; fi
fake_claude 2.1.27
out=$(FAKE_CLAUDE_NEW=2.1.300 run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "OK      testdev now has Claude Code 2.1.300" && "$RUNME_TESTDEV_CLAUDE" --version | grep -q 2.1.300; then
    ok "testdev claude: an old version is replaced and verified by running it"
else bad "testdev install" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "skip    testdev already has Claude Code 2.1.300"; then
    ok "testdev claude: second run skips (idempotent)"
else bad "testdev idempotent" "rc=$rc: $out"; fi
fake_claude 2.1.27
out=$(FAKE_CLAUDE_NEW= run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep -q "FAILED  the installer ran but .* reports '2.1.27', need >= 2.1.296"; then
    ok "testdev claude: an installer that leaves the old version is FAILED, not OK"
else bad "testdev old stays" "rc=$rc: $out"; fi
rm -f "$RUNME_TESTDEV_CLAUDE"
out=$(FAKE_CLAUDE_NEW= run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep -q "FAILED  the installer ran but .* reports 'nothing'"; then
    ok "testdev claude: an installer that leaves no binary is FAILED, not OK"
else bad "testdev none" "rc=$rc: $out"; fi
fake_claude 2.1.296

# 28. T-3421 — claude-fw on PATH becomes the router (fake files only).
cfw_old "$TMP/cfw-a"; rm -f "$TMP/cfw-b"; rm -rf "$RUNME_CLAUDE_FW_BACKUP_DIR"
out=$(RUNME_CLAUDE_FW_TARGETS="$TMP/cfw-a $TMP/cfw-b" run --dry-run); rc=$?
if echo "$out" | grep -q "would replace $TMP/cfw-a" && grep -q "old fixed copy" "$TMP/cfw-a" && [ ! -e "$TMP/cfw-b" ]; then
    ok "claude-fw router: --dry-run reports and changes nothing"
else bad "cfw dry-run" "rc=$rc: $out"; fi
out=$(RUNME_CLAUDE_FW_TARGETS="$TMP/cfw-a $TMP/cfw-b" run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "OK      $TMP/cfw-a is now the claude-fw router" && cmp -s "$TMP/cfw-a" "$RUNME_CLAUDE_FW_ROUTER" \
   && ls "$RUNME_CLAUDE_FW_BACKUP_DIR" | grep -q cfw-a && grep -q "old fixed copy" "$RUNME_CLAUDE_FW_BACKUP_DIR"/*cfw-a*; then
    ok "claude-fw router: a fixed copy is replaced, verified on disk, and backed up"
else bad "cfw replace" "rc=$rc: $out"; fi
if echo "$out" | grep -q "skip    $TMP/cfw-b does not exist here" && [ ! -e "$TMP/cfw-b" ]; then
    ok "claude-fw router: an absent path is left alone, never created"
else bad "cfw absent" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "skip    $TMP/cfw-a is already the claude-fw router"; then
    ok "claude-fw router: second run skips (idempotent)"
else bad "cfw idempotent" "rc=$rc: $out"; fi
cfw_old "$TMP/cfw-a"
out=$(RUNME_CLAUDE_FW_ROUTER="$TMP/no-such-router" run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep -q "FAILED  the router .* is missing" && grep -q "old fixed copy" "$TMP/cfw-a"; then
    ok "claude-fw router: a missing router is FAILED and nothing is touched"
else bad "cfw no router" "rc=$rc: $out"; fi
cfw_router "$TMP/cfw-a"

# T-3435 (P3): sidecar supervision install — systemd must RESOLVE the template,
# a host without systemd skips the unit and says cron supervises.
rm -f "$SCU/units/termlink-sidecar@.service"; touch "$SCU/broken"
out=$(run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep -q "FAILED  .*termlink-sidecar@.service installed but systemd does not resolve"; then
    ok "sidecar unit: a template systemd does not resolve is FAILED, not OK"
else bad "sidecar unit unresolved" "rc=$rc: $out"; fi
rm -f "$SCU/broken" "$SCU/units/termlink-sidecar@.service"
printf '#!/usr/bin/env bash\nexit 1\n' > "$SCU/systemctl-fail"; chmod +x "$SCU/systemctl-fail"
out=$(RUNME_SIDECAR_SYSTEMCTL="$SCU/systemctl-fail" run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep -q "FAILED  installed .* but daemon-reload failed"; then
    ok "sidecar unit: a failed daemon-reload is FAILED"
else bad "sidecar unit daemon-reload" "rc=$rc: $out"; fi
rm -f "$SCU/units/termlink-sidecar@.service"
out=$(RUNME_SIDECAR_SYSTEMD_RUN="$SCU/no-systemd" run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "no systemd on this host: the cron backstop is the sidecar supervisor" \
   && [ ! -e "$SCU/units/termlink-sidecar@.service" ]; then
    ok "sidecar unit: a host without systemd skips the unit and names cron as the supervisor"
else bad "sidecar no-systemd" "rc=$rc: $out"; fi
run >/dev/null 2>&1

# T-3438 (P7): W1 — readiness every run, switch-on only on a recorded go.
: > "$SCU/w1-calls"; out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "W1 is ready; waiting for the operator's go" && grep -qx -- "--check" "$SCU/w1-calls" && ! grep -q -- "--apply" "$SCU/w1-calls"; then
    ok "W1: without a recorded go only the read-only check runs"
else bad "W1 default" "rc=$rc: $out"; fi
: > "$SCU/w1-calls"; out=$(RUNME_W1_GO="2026-10-13|fixture" run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "OK      W1 pair provisioned and running" && grep -qx -- "--apply" "$SCU/w1-calls"; then
    ok "W1: with a recorded go it provisions and reports OK"
else bad "W1 go" "rc=$rc: $out"; fi
out=$(RUNME_W1_GO="2026-10-13|fixture" FAKE_W1_APPLY_RC=1 run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep -q "FAILED  W1 provisioning"; then
    ok "W1: a failed provisioning is FAILED, not OK"
else bad "W1 apply fail" "rc=$rc: $out"; fi

# T-3438: the kill switch runs alone, needs a reason, reads each flip back.
: > "$SCU/w1-calls"; out=$(run --sidecar-delivery off --why "fixture drill"); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "OK      delivery off on every listed sidecar" \
   && grep -q "sidecar admin delivery off --why fixture drill --all --list $SCU/admin.list" "$SCU/w1-calls" \
   && ! echo "$out" | grep -q "1. Cron installs"; then
    ok "kill switch: flips every listed sidecar and runs no other action"
else bad "kill switch" "rc=$rc: $out"; fi
out=$(run --sidecar-delivery off); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q -- "--why"; then ok "kill switch: refuses without a reason"
else bad "kill switch no why" "rc=$rc: $out"; fi
out=$(FAKE_KS_RC=1 run --sidecar-delivery off --why x); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "FAILED  at least one sidecar did not confirm"; then ok "kill switch: an unconfirmed flip is FAILED"
else bad "kill switch unconfirmed" "rc=$rc: $out"; fi
out=$(RUNME_SIDECAR_ADMIN_LIST="$SCU/none.list" run --sidecar-delivery on --why x); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "no sidecars listed"; then ok "kill switch: an empty list switches nothing and says so"
else bad "kill switch empty list" "rc=$rc: $out"; fi


# ---------------------------------------------------------------------------
# 23. T-3443 — cap leftover forever project inboxes (fake termlink, scratch state only).
# ---------------------------------------------------------------------------
echo "forever 0 1042 1041 969" > "$FAKE_INBOX_STATE"; echo 1040 > "$TMP/inbox.marker"
out=$(run --dry-run); rc=$?
if echo "$out" | grep -q "would set inbox:t/010 retention forever -> messages 1000 and sweep offsets <= 41" && grep -q "^forever 0 1042 1041 969$" "$FAKE_INBOX_STATE"; then
    ok "inbox: --dry-run reports the cut and changes nothing"
else bad "inbox dry-run" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "OK      inbox:t/010 now keeps the newest 1000; swept to 1000 records, every reader was past offset 41 (verified)" && grep -q "^messages 1000 1000 " "$FAKE_INBOX_STATE"; then
    ok "inbox: readers past the cut -> capped, swept, verified"
else bad "inbox cap" "rc=$rc: $out $(cat "$FAKE_INBOX_STATE")"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "skip    inbox:t/010 already keeps the newest 1000"; then ok "inbox: second run skips (idempotent)"
else bad "inbox idempotent" "rc=$rc: $out"; fi
echo "forever 0 1500 1499 300" > "$FAKE_INBOX_STATE"
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "HELD    inbox:t/010: refused — a reader is behind the cut point 499 (hub receipt at 300)" && echo "$out" | grep -q " 1 held, 0 failed" && grep -q "^forever 0 1500 1499 300$" "$FAKE_INBOX_STATE"; then
    ok "inbox: a hub reader behind the cut is HELD (rc 0, T-3606), nothing changed"
else bad "inbox receipt behind" "rc=$rc: $out"; fi
echo "forever 0 1500 1499 1499" > "$FAKE_INBOX_STATE"; echo 200 > "$TMP/inbox.marker"
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "HELD .*alert marker at 200" && grep -q "^forever 0 1500 " "$FAKE_INBOX_STATE"; then
    ok "inbox: our own alert marker behind the cut is HELD"
else bad "inbox marker behind" "rc=$rc: $out"; fi
echo "forever 0 1500 1499 none" > "$FAKE_INBOX_STATE"
out=$(RUNME_INBOX_TOPICS="inbox:t/aef|" run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "HELD    inbox:t/aef: refused — no reader position is known" && grep -q "^forever 0 1500 " "$FAKE_INBOX_STATE"; then
    ok "inbox: no known reader position is HELD, never assumed read"
else bad "inbox no readers" "rc=$rc: $out"; fi
echo "forever 0 1500 1499 1499" > "$FAKE_INBOX_STATE"; printf 'x' > "$TMP/inbox.marker"
out=$(run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "FAILED  inbox:t/010: alert marker unreadable" && grep -q "^forever 0 1500 " "$FAKE_INBOX_STATE"; then
    ok "inbox: an unreadable alert marker is FAILED (could not look), not HELD"
else bad "inbox marker unreadable" "rc=$rc: $out"; fi
echo "forever 0 1500 1499 1499" > "$FAKE_INBOX_STATE"; echo 1499 > "$TMP/inbox.marker"
out=$(FAKE_INBOX_NOSWEEP=1 run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "FAILED  inbox:t/010: after set-retention + sweep it reads 'messages 1000 1500'"; then
    ok "inbox: a sweep that does not take effect is FAILED, not OK"
else bad "inbox no-sweep" "rc=$rc: $out"; fi
echo "forever 0 182 181 none" > "$FAKE_INBOX_STATE"
out=$(RUNME_INBOX_TOPICS="inbox:t/small|" run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "OK      inbox:t/small now keeps the newest 1000; 182 records, nothing deleted (verified)"; then
    ok "inbox: under the cap with no known reader -> policy set, nothing deleted, said so"
else bad "inbox small" "rc=$rc: $out"; fi
echo "messages 1000 1000 4999 4999" > "$FAKE_INBOX_STATE"
out=$(FAKE_INBOX_MISSING=1 run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "FAILED  inbox:t/010: cannot read it on the local hub"; then ok "inbox: an unreadable topic is FAILED, never a skip"
else bad "inbox missing" "rc=$rc: $out"; fi
echo "messages 1000 1000 4999 4999" > "$FAKE_INBOX_STATE"; echo 4999 > "$TMP/inbox.marker"


# ---------------------------------------------------------------------------
# 24. T-3426 — the W1 kill-switch drill runs once and records a pass (fake drill only).
# ---------------------------------------------------------------------------
rm -f "$RUNME_W1_DRILL_DONE" "$FAKE_DRILL_RAN"
out=$(run --dry-run); rc=$?
if echo "$out" | grep -q "would run the W1 kill-switch drill" && [ ! -f "$FAKE_DRILL_RAN" ]; then ok "drill: --dry-run runs nothing"
else bad "drill dry-run" "rc=$rc: $out"; fi
out=$(FAKE_DRILL_RC=1 run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "FAILED  W1 kill-switch drill rc=1" && [ ! -s "$RUNME_W1_DRILL_DONE" ]; then ok "drill: a failed drill is FAILED and records no pass"
else bad "drill fail" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "OK      W1 kill-switch drill passed" && grep -q "8 passed, 0 failed" "$RUNME_W1_DRILL_DONE"; then ok "drill: a pass is recorded"
else bad "drill pass" "rc=$rc: $out"; fi
: > "$FAKE_DRILL_RAN"; out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep -q "skip    the W1 kill-switch drill already passed" && [ ! -s "$FAKE_DRILL_RAN" ]; then ok "drill: once passed it never runs again (idempotent)"
else bad "drill idempotent" "rc=$rc: $out"; fi

# ---------------------------------------------------------------------------
# 25. T-3440 5c — attach driver scoring specs (scratch arc + fake attach only).
# ---------------------------------------------------------------------------
arcs_specced 0
out=$(run --dry-run); rc=$?
if echo "$out" | grep -q "would attach .*AS_FAILURE_OBSERVABILITY" && ! grep -q "scoring:" "$RUNME_ARCS_DIR/arc-substrate-fitness.yaml"; then ok "specs: --dry-run attaches nothing"
else bad "specs dry-run" "rc=$rc: $out"; fi
out=$(FAKE_ATTACH_LIE=1 run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep -q "attach reported success but no valid scoring block is stored"; then ok "specs: a claimed attach that stored nothing is FAILED"
else bad "specs lie" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && [ "$(echo "$out" | grep -c "OK      attached and verified")" = "6" ] && grep -q "scoring:" "$RUNME_ARCS_DIR/arc-011.yaml"; then ok "specs: all six attached and verified on disk, both arcs"
else bad "specs attach" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && [ "$(echo "$out" | grep -c "already carries a scoring spec")" = "6" ]; then ok "specs: idempotent (skips once attached)"
else bad "specs idempotent" "rc=$rc: $out"; fi

# ---------------------------------------------------------------------------
# T-3311 action 21 — retention sweeper on the .122 hub, through fakes only.
# ---------------------------------------------------------------------------
sweep_case() { echo "$1" > "$FAKE_SWEEP_FILE"; rm -f "$F/remote-cmds"; }
export RUNME_SWEEP_HUB=fake-hub
sweep_case 3600
out=$(run); s21=$(echo "$out" | sed -n '/^.*21\. Retention sweeper/,/^.*Verification/p')
if echo "$s21" | grep -q "skip    fake-hub already sweeps every 3600s" && ! grep -q "t3311" "$F/remote-cmds" 2>/dev/null; then
    ok "sweeper: already on => skip, nothing sent to the hub"
else bad "sweeper skip" "$s21"; fi
sweep_case 0
out=$(run --dry-run); s21=$(echo "$out" | sed -n '/^.*21\. Retention sweeper/,/^.*Verification/p')
if echo "$out" | grep -q "21. Retention sweeper on for the .122 hub — THE .122 HUB RESTARTS" \
   && echo "$s21" | grep -q "would add TERMLINK_SWEEP_INTERVAL_SECS=3600 .* and RESTART that hub" \
   && ! grep -q "t3311" "$F/remote-cmds" 2>/dev/null; then
    ok "sweeper: heading says the hub restarts; --dry-run sends nothing"
else bad "sweeper dry-run" "$s21"; fi
sweep_case 0
out=$(run); s21=$(echo "$out" | sed -n '/^.*21\. Retention sweeper/,/^.*Verification/p')
if echo "$s21" | grep -q "OK      fake-hub restarted and sweeps every 3600s" \
   && grep -q "termlink-hub.service.d/t3311-sweep.conf" "$F/remote-cmds" && grep -q "daemon-reload" "$F/remote-cmds" \
   && grep -q "systemctl restart termlink-hub" "$F/remote-cmds"; then
    ok "sweeper: drop-in written, daemon-reload, restart through the unit, verified 3600"
else bad "sweeper apply" "$s21 // $(cat "$F/remote-cmds" 2>/dev/null)"; fi
sweep_case 0
out=$(FAKE_UNIT=0 run); rc=$?; s21=$(echo "$out" | sed -n '/^.*21\. Retention sweeper/,/^.*Verification/p')
if [ "$rc" = "1" ] && echo "$s21" | grep -q "FAILED  fake-hub has no termlink-hub.service — nothing changed"; then
    ok "sweeper: no unit => FAILED, nothing changed"
else bad "sweeper no unit" "rc=$rc: $s21"; fi
sweep_case 0
out=$(FAKE_SWEEP_APPLIES=0 run); rc=$?; s21=$(echo "$out" | sed -n '/^.*21\. Retention sweeper/,/^.*Verification/p')
if [ "$rc" = "1" ] && echo "$s21" | grep -q "FAILED  fake-hub after restart: interval='0' (want 3600)"; then
    ok "sweeper: restart that does not take effect => FAILED (verified, not trusted)"
else bad "sweeper not applied" "rc=$rc: $s21"; fi
sweep_case 3600; unset RUNME_SWEEP_HUB

# ---------------------------------------------------------------------------
# T-3477 action 22 — ring20's pack: verify before and after, delete the drop only when
# the copy verifies, never place it where git would track it.
# ---------------------------------------------------------------------------
s22() { echo "$1" | sed -n '/^.*22\. ring20.s deep-inception pack/,/^.*Verification/p'; }
pack_reset() { rm -rf "$RUNME_PACK_DEST"; rm -f "$RUNME_PACK_SRC"; }
out=$(run); x=$(s22 "$out")
if echo "$x" | grep -q "skip    pack already in .* verified; the /var/tmp drop is gone"; then ok "pack: already imported, drop gone => skip"
else bad "pack skip" "$x"; fi
pack_reset; cp "$TMP/pack-good.tgz" "$RUNME_PACK_SRC"
out=$(run --dry-run); x=$(s22 "$out")
if echo "$x" | grep -q "\[DRY\]   would unpack" && [ -e "$RUNME_PACK_SRC" ] && [ ! -d "$RUNME_PACK_DEST" ]; then ok "pack: --dry-run changes nothing"
else bad "pack dry-run" "$x"; fi
out=$(run); x=$(s22 "$out")
if echo "$x" | grep -q "OK      pack unpacked into" && [ ! -e "$RUNME_PACK_SRC" ] && [ -f "$RUNME_PACK_DEST/pack/plan.md" ]; then
    ok "pack: unpacked, verified, drop deleted"
else bad "pack apply" "$x"; fi
if [ -z "$(find "$RUNME_PACK_DEST" -perm /077 -print -quit)" ]; then ok "pack: unpacked tree is owner-only on disk (tar does not leave it 755)"
else bad "pack mode" "$(find "$RUNME_PACK_DEST" -perm /077 | head -3)"; fi
chmod -R go+rX "$RUNME_PACK_DEST"
out=$(run); rc=$?; x=$(s22 "$out")
if [ "$rc" = "0" ] && echo "$x" | grep -q "OK      .* was group/other-readable; now owner-only" && [ -z "$(find "$RUNME_PACK_DEST" -perm /077 -print -quit)" ]; then
    ok "pack: imported but readable => tightened to owner-only"
else bad "pack tighten" "rc=$rc: $x"; fi
pack_reset; printf 'not the pack' > "$RUNME_PACK_SRC"
out=$(run); rc=$?; x=$(s22 "$out")
if [ "$rc" = "1" ] && echo "$x" | grep -q "FAILED  .* sha256 .* does not match" && [ -e "$RUNME_PACK_SRC" ] && [ ! -d "$RUNME_PACK_DEST" ]; then
    ok "pack: wrong sha256 => FAILED, nothing copied, drop kept"
else bad "pack wrong sha" "rc=$rc: $x"; fi
pack_reset; mkdir -p "$TMP/packbad/pack"; cp "$TMP/packsrc/pack/"* "$TMP/packbad/pack/"; printf 'tampered\n' > "$TMP/packbad/pack/plan.md"
tar -czf "$RUNME_PACK_SRC" -C "$TMP/packbad" pack
out=$(RUNME_PACK_SHA="$(sha256sum "$RUNME_PACK_SRC" | cut -d' ' -f1)" run); rc=$?; x=$(s22 "$out")
if [ "$rc" = "1" ] && echo "$x" | grep -q "FAILED  unpacked files do not match the SHA256SUMS inside" && [ -e "$RUNME_PACK_SRC" ]; then
    ok "pack: contents not matching SHA256SUMS => FAILED, drop kept"
else bad "pack tampered" "rc=$rc: $x"; fi
pack_reset; cp "$TMP/pack-good.tgz" "$RUNME_PACK_SRC"; mkdir -p "$TMP/plainrepo"; git -C "$TMP/plainrepo" init -q 2>/dev/null
out=$(RUNME_PACK_GITROOT="$TMP/plainrepo" RUNME_PACK_DEST="$TMP/plainrepo/pack" run); rc=$?; x=$(s22 "$out")
if [ "$rc" = "1" ] && echo "$x" | grep -q "FAILED  .* is not gitignored" && [ ! -d "$TMP/plainrepo/pack" ] && [ -e "$RUNME_PACK_SRC" ]; then
    ok "pack: destination git would track => FAILED, nothing placed"
else bad "pack not ignored" "rc=$rc: $x"; fi
pack_reset; mkdir -p "$RUNME_PACK_DEST" && tar -xzf "$TMP/pack-good.tgz" -C "$RUNME_PACK_DEST" && chmod -R go-rwx "$RUNME_PACK_DEST"

# ---------------------------------------------------------------------------
# T-3615 action 1b — the background-class line on installed crontabs it does not manage.
# ---------------------------------------------------------------------------
s1b() { echo "$1" | sed -n '/^.*1b\. Background-class line/,/^.*2\. /p'; }
grep -vx 'TERMLINK_WORKER_CLASS=background' "$REPO_ROOT/.context/cron/heartbeat.crontab" > "$TMP/cron/termlink-heartbeat"
{ grep -vx 'TERMLINK_WORKER_CLASS=background' "$REPO_ROOT/.context/cron/presence-sweep.crontab"; echo '# hand edit'; } > "$TMP/cron/termlink-presence-sweep"
cp "$TMP/cron/termlink-presence-sweep" "$TMP/presence.before"
out=$(run --dry-run); x=$(s1b "$out")
if echo "$x" | grep -q "would add the background-class line: .*heartbeat.crontab" && ! grep -q TERMLINK_WORKER_CLASS "$TMP/cron/termlink-heartbeat"; then
    ok "1b: --dry-run names the env-only drift and writes nothing"
else bad "1b dry-run" "$x"; fi
out=$(run); rc=$?; x=$(s1b "$out")
if [ "$rc" = "0" ] && echo "$x" | grep -q "OK      1 installed crontab(s) now carry TERMLINK_WORKER_CLASS=background" \
   && cmp -s "$REPO_ROOT/.context/cron/heartbeat.crontab" "$TMP/cron/termlink-heartbeat"; then
    ok "1b: an installed copy differing ONLY by the class line is reinstalled and verified identical"
else bad "1b resync" "rc=$rc: $x"; fi
if echo "$x" | grep -q "left alone .*termlink-presence-sweep" && cmp -s "$TMP/presence.before" "$TMP/cron/termlink-presence-sweep"; then
    ok "1b: an installed copy with any other drift is named and left untouched"
else bad "1b other drift" "$x"; fi
rm -f "$TMP/cron/termlink-presence-sweep"
out=$(run); x=$(s1b "$out")
if echo "$x" | grep -q "skip    every installed crontab already carries"; then ok "1b: second run skips (idempotent)"
else bad "1b idempotent" "$x"; fi
rm -f "$TMP/cron/termlink-heartbeat"

# ---------------------------------------------------------------------------
# T-3615 action 23 — the daily telemetry rollup on the fleet hubs, through fakes.
# ---------------------------------------------------------------------------
s23() { echo "$1" | sed -n '/^.*23\. Daily telemetry rollup/,/^.*Verification/p'; }
rr_reset() { rm -rf "$F/rr-root" "$F/rollup-cmds"; rm -f "$RUNME_REMOTE_ROLLUP_CRON"; }
rr_reset
out=$(RUNME_ROLLUP_CREDENTIAL_DECISION="" run); rc=$?; x=$(s23 "$out")
if [ "$rc" = "0" ] && echo "$x" | grep -q "HELD    waiting for the operator's credential decision (T-3615 Q3)" \
   && [ ! -e "$F/rr-root" ] && [ ! -e "$RUNME_REMOTE_ROLLUP_CRON" ] && [ ! -s "$F/rollup-cmds" ] \
   && echo "$out" | grep -qE " 1 held, 0 failed"; then
    ok "rollup: default (no credential decision recorded) is HELD, rc 0, nothing probed or written (T-3615 Q3)"
else bad "rollup held default" "rc=$rc: $x"; fi
out=$(run --dry-run); x=$(s23 "$out")
if echo "$x" | grep -q "would install the rollup on fake-hub" && [ ! -e "$F/rr-root" ] && [ ! -e "$RUNME_REMOTE_ROLLUP_CRON" ]; then
    ok "rollup: --dry-run probes, reports the install, writes nothing"
else bad "rollup dry-run" "$x"; fi
out=$(run); rc=$?; x=$(s23 "$out")
if [ "$rc" = "0" ] && echo "$x" | grep -q "OK      fake-hub: rollup installed and verified" \
   && cmp -s "$REPO_ROOT/scripts/telemetry-rollup.sh" "$F/rr-root/scripts/telemetry-rollup.sh" \
   && grep -q '^TELEMETRY_HUB=192.168.10.107:9100$' "$RUNME_REMOTE_ROLLUP_CRON" && grep -q '^TELEMETRY_HOST=fake-hub$' "$RUNME_REMOTE_ROLLUP_CRON" \
   && grep -q '^TERMLINK_WORKER_CLASS=background$' "$RUNME_REMOTE_ROLLUP_CRON" \
   && grep -q "^30 2 \* \* \* root cd $F/rr-root && bash scripts/telemetry-rollup.sh --catch-up 7 " "$RUNME_REMOTE_ROLLUP_CRON"; then
    ok "rollup: installs script + cron file (posts to .107, host label, background class, --catch-up 7) and verifies both plus a remote dry run"
else bad "rollup install" "rc=$rc: $x"; fi
: > "$F/rollup-cmds"
out=$(run); rc=$?; x=$(s23 "$out")
if [ "$rc" = "0" ] && echo "$x" | grep -q "skip    fake-hub already runs the current rollup daily" && ! grep -q RUNME-ROLLUP-INSTALL "$F/rollup-cmds" 2>/dev/null; then
    ok "rollup: second run skips (sha256 identical), no install command sent"
else bad "rollup idempotent" "rc=$rc: $x"; fi
echo "# stale" >> "$F/rr-root/scripts/telemetry-rollup.sh"; : > "$F/rollup-cmds"
out=$(run); rc=$?; x=$(s23 "$out")
if [ "$rc" = "0" ] && echo "$x" | grep -q "OK      fake-hub: rollup installed" && cmp -s "$REPO_ROOT/scripts/telemetry-rollup.sh" "$F/rr-root/scripts/telemetry-rollup.sh"; then
    ok "rollup: a drifted remote script is reinstalled"
else bad "rollup drift" "rc=$rc: $x"; fi
rr_reset
out=$(FAKE_RR_AUTH_RC=1 run); rc=$?; x=$(s23 "$out")
if [ "$rc" = "1" ] && echo "$x" | grep -q "FAILED  fake-hub cannot authenticate to .107" && echo "$x" | grep -q "Operator question" \
   && [ ! -e "$F/rr-root" ] && [ ! -e "$RUNME_REMOTE_ROLLUP_CRON" ]; then
    ok "rollup: no credential for .107 => FAILED naming the operator question, nothing written"
else bad "rollup no auth" "rc=$rc: $x"; fi
out=$(FAKE_RR_DOWN=1 run); rc=$?; x=$(s23 "$out")
if [ "$rc" = "1" ] && echo "$x" | grep -q "FAILED  fake-hub: no answer over remote exec" && [ ! -e "$F/rr-root" ]; then
    ok "rollup: unreachable hub => FAILED, nothing written"
else bad "rollup down" "rc=$rc: $x"; fi
out=$(RUNME_REMOTE_TL="$F/no-such-termlink" run); rc=$?; x=$(s23 "$out")
if [ "$rc" = "1" ] && echo "$x" | grep -q "FAILED  fake-hub lacks python3 or termlink" && [ ! -e "$F/rr-root" ]; then
    ok "rollup: termlink missing on the remote => FAILED, nothing written"
else bad "rollup no termlink" "rc=$rc: $x"; fi
printf '#!/bin/sh\nexit 1\n' > "$F/bad-src.sh"; rr_reset
out=$(RUNME_ROLLUP_SRC="$F/bad-src.sh" run); rc=$?; x=$(s23 "$out")
if [ "$rc" = "1" ] && echo "$x" | grep -q "FAILED  fake-hub after install: .*dry run failed"; then
    ok "rollup: files in place but the installed rollup's dry run fails => FAILED (verified, not assumed)"
else bad "rollup verify dry run" "rc=$rc: $x"; fi
rr_reset
mkdir -p "$F/rr-root/scripts"; out=$(run); rc=$?
if [ "$rc" = "0" ]; then ok "rollup: restored to installed state for later cases"; else bad "rollup restore" "rc=$rc: $(s23 "$out")"; fi

echo ""
echo "----------------------------------------"
printf 'T-3052 fixtures: %d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" = "0" ] || exit 1
exit 0
