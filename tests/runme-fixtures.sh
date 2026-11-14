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
# T-3627: grep that reads ALL its input. Under pipefail, `producer | grep -q` returns 141 when grep
# matches and exits while the producer is still writing (L-387), which read as "no match" at random.
grep_all() { grep "$@" >/dev/null; }

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
# T-3618 action 25: a fake `crontab` over a scratch file (no ntfy lines by default,
# so ordinary runs skip; the action-25 cases at the end seed them).
CT="$TMP/crontab-root"
cat > "$TMP/crontab" <<EOS
#!/usr/bin/env bash
case "\$1" in
    -l) [ -e "$CT" ] || exit 1; [ -n "\${FAKE_CT_DRIFT:-}" ] && { cat "$CT"; echo "* * * * * injected"; exit 0; }; cat "$CT" ;;
    -)  [ -n "\${FAKE_CT_REFUSE:-}" ] && exit 1; cat > "$CT" ;;
    *)  cp "\$1" "$CT" ;;
esac
EOS
chmod +x "$TMP/crontab"
printf '0 6 * * * /opt/x/update.sh\n# other job\n' > "$CT"
export RUNME_CRONTAB="$TMP/crontab" RUNME_CRONTAB_BACKUP="$TMP/crontab.bak.T-476"
# T-3657 action 26: a fake fw answering `sidecar waiting --json` / `sidecar drop` over a
# scratch id file. Empty by default, so ordinary runs skip; the action-26 cases seed it.
# The fake refuses a drop under CLAUDECODE=1 without --i-am-human, exactly as the real one.
export FAKE_WAIT_IDS="$TMP/waiting.ids" FAKE_DROP_LOG="$TMP/drops.log"
: > "$FAKE_WAIT_IDS"
cat > "$TMP/fake-drop-fw" <<'EOS'
#!/usr/bin/env bash
[ "$1" = sidecar ] || exit 3
case "$2" in
  waiting) [ -n "${FAKE_WAIT_UNREADABLE:-}" ] && { echo "boom" >&2; exit 1; }
           python3 -c 'import json,sys; print(json.dumps({"items":[{"id":l.strip(),"side":"inbound"} for l in open(sys.argv[1]) if l.strip()]}))' "$FAKE_WAIT_IDS" ;;
  drop)    id="$3"; reason=""; human=0; shift 3
           while [ $# -gt 0 ]; do case "$1" in --reason) reason="$2"; shift ;; --i-am-human) human=1 ;; esac; shift; done
           if [ "${CLAUDECODE:-}" = 1 ] && [ "$human" = 0 ]; then echo "operator-only — refused under CLAUDECODE=1" >&2; exit 2; fi
           [ "$human" = 1 ] && echo "FORGED $id" >> "$FAKE_DROP_LOG"
           [ "${FAKE_DROP_REFUSE:-}" = "$id" ] && { echo "refused $id" >&2; exit 1; }
           printf '%s|%s\n' "$id" "$reason" >> "$FAKE_DROP_LOG"
           grep -vxF -- "$id" "$FAKE_WAIT_IDS" > "$FAKE_WAIT_IDS.n"; mv "$FAKE_WAIT_IDS.n" "$FAKE_WAIT_IDS" ;;
esac
EOS
chmod +x "$TMP/fake-drop-fw"
printf 'aaaa-1111\tT-3656: e2e residue: selftest conv x\nbbbb-2222\tT-3656: answered: reply "ok" posted 2026-10-06\ncccc-3333\tT-3656: answered: gone already\n' > "$TMP/drop-list.tsv"
export RUNME_DROP_FW="$TMP/fake-drop-fw" RUNME_DROP_LIST="$TMP/drop-list.tsv"
# T-3685 action 28: a fake `su - testdev -c` that answers OK only for the GOOD token in the
# scratch token file, and fake terminal files. Default = a working token, so ordinary runs skip.
TD="$TMP/testdev"; mkdir -p "$TD/home/.config/claude"
TD_GOOD="sk-ant-oat01-FIXTUREgoodTOKEN0123456789"
cat > "$TD/run" <<'EOS'
#!/usr/bin/env bash
case "$1" in
  *setup-token*) echo "fake setup-token: open https://claude.ai/oauth/fixture and log in"; echo "fake setup-token printed a token" ;;
  *"claude -p"*) if ! grep -q "testdev's own Claude temp dir" "${FAKE_TD_PROFILE:?}" 2>/dev/null; then echo "EACCES: permission denied, mkdir '/var/tmp/claude/claude-1001'"
                 elif [ "$(cat "${FAKE_TD_TOKEN_FILE:?}" 2>/dev/null)" = "${FAKE_TD_GOOD:?}" ]; then echo OK; else echo "Failed to authenticate: OAuth session expired"; fi ;;
  *) echo "fake su: unexpected command: $1" >&2; exit 9 ;;
esac
EOS
chmod +x "$TD/run"
# Fake script(1): the recording gets $FAKE_TD_SETUP_OUT (what setup-token "printed"), the terminal a line.
cat > "$TD/script" <<'EOS'
#!/usr/bin/env bash
rec="${@: -1}"; cat "${FAKE_TD_SETUP_OUT:?}" > "$rec" 2>/dev/null || : > "$rec"
cp "$rec" "${FAKE_TD_SETUP_OUT}.seen-rec" 2>/dev/null || true; echo "fake setup-token: open https://claude.ai/oauth/fixture and log in"
EOS
chmod +x "$TD/script"
printf 'Welcome to setup-token\nOpen the link above and log in.\n' > "$TD/setup-out-notoken"
export FAKE_TD_SETUP_OUT="$TD/setup-out-notoken" RUNME_TESTDEV_SCRIPT="$TD/script" RUNME_TESTDEV_REC_DIR="$TD/rec"
export FAKE_TD_TOKEN_FILE="$TD/home/.config/claude/oauth-token" FAKE_TD_GOOD="$TD_GOOD" FAKE_TD_PROFILE="$TD/home/.profile"
export RUNME_TESTDEV_RUN="$TD/run" RUNME_TESTDEV_TOKEN_FILE="$TD/home/.config/claude/oauth-token" \
       RUNME_TESTDEV_PROFILE="$TD/home/.profile" RUNME_TESTDEV_TTY_IN="$TD/tty-in" RUNME_TESTDEV_TTY_OUT="$TD/tty-out" \
       RUNME_TESTDEV_OWNER="$(id -u):$(id -g)"
printf '%s\n' "$TD_GOOD" > "$TD/home/.config/claude/oauth-token"; chmod 600 "$TD/home/.config/claude/oauth-token"
TD_TMPLINE="# T-3685: testdev's own Claude temp dir (/etc/environment's /var/tmp/claude is root's)"
printf '%s\nexport CLAUDE_CODE_TMPDIR="$HOME/.cache/claude-tmp"\n' "$TD_TMPLINE" > "$TD/home/.profile"
# T-3664 action 27: fake systemctl/ss/docker over a scratch daemon.json. Default = already closed
# (fd:// only, nothing on :2375), so ordinary runs skip; the action-27 cases seed the open state.
DK="$TMP/docker"; mkdir -p "$DK"
export FAKE_DK="$DK" RUNME_DOCKER_CONF="$DK/daemon.json" RUNME_DOCKER_SYSTEMCTL="$DK/systemctl" RUNME_DOCKER_SS="$DK/ss" RUNME_DOCKER_CLI="$DK/docker"
printf '{\n    "hosts": ["fd://"]\n}\n' > "$DK/daemon.json"
cat > "$DK/systemctl" <<'EOS'
#!/usr/bin/env bash
D="$FAKE_DK"
case "$1" in
  show) echo "ExecStart={ path=/usr/bin/dockerd ; argv[]=/usr/bin/dockerd ${FAKE_DK_EXECARGS:---containerd=/run/containerd/containerd.sock} ; }" ;;
  is-active) [ -n "${FAKE_DK_INACTIVE:-}" ] && exit 3; exit 0 ;;
  restart) echo restart >> "$D/restarts"
           if grep -q 'tcp://' "$D/daemon.json" || [ -n "${FAKE_DK_PORT_STAYS:-}" ]; then : > "$D/listen"; else rm -f "$D/listen"; fi
           grep -vxF -- "${FAKE_DK_LOST:-@none@}" "$D/names.before" > "$D/names" || true ;;
esac
EOS
cat > "$DK/ss" <<'EOS'
#!/usr/bin/env bash
echo "LISTEN 0      4096   127.0.0.1:9100 0.0.0.0:*"
[ -e "$FAKE_DK/listen" ] && echo "LISTEN 0      4096   *:2375 *:*"
exit 0
EOS
cat > "$DK/docker" <<'EOS'
#!/usr/bin/env bash
[ -n "${FAKE_DK_PS_BROKEN:-}" ] && exit 1
[ "$1" = ps ] || exit 3
if [ "${2:-}" = --format ]; then cat "$FAKE_DK/names" 2>/dev/null; else echo "CONTAINER ID   IMAGE"; fi
EOS
chmod +x "$DK/systemctl" "$DK/ss" "$DK/docker"
printf 'watchtower\nsprechloop-dev\nopen-webui\n' > "$DK/names"
# T-3778 action 11a: a scratch AEF source repo (commit A = what is vendored; tag v1.8.16-be.49
# changes the one preserved file and carries T-5105; a side-branch commit for the ancestry
# control) and a scratch project repo with a bare origin, a fake project fw (config set HOST,
# watchtower restart, doctor) and a fake curl. aefu_state done|fresh rebuilds the project.
# Default = done, so ordinary runs skip 11a. Nothing outside $TMP is read or written.
AE="$TMP/aefu"; mkdir -p "$AE"
aefu_g() { git -c user.name=fx -c user.email=fx@example.invalid -c init.defaultBranch=main -c commit.gpgsign=false "$@"; }
mkdir -p "$AE/src/bin" "$AE/src/lib" "$AE/src/agents/termlink"
cat > "$AE/src/bin/fw" <<'EOS'
#!/usr/bin/env bash
# fake AEF `fw upgrade [--dry-run] <project>` (T-3778 fixture); knobs FAKE_AEFU_*
[ "$1" = upgrade ] || exit 9
[ "$2" = --dry-run ] && { echo "3 change(s) would be made"; exit 0; }
P="$2"; S="$(cd "$(dirname "$0")/.." && pwd)"
[ -n "${FAKE_AEFU_UP_RC:-}" ] && { echo "x" > "$P/.agentic-framework/lib/half-written.py"; echo "upgrade broke"; exit "$FAKE_AEFU_UP_RC"; }
echo "new upstream file" > "$P/.agentic-framework/lib/new-upstream.py"
[ -n "${FAKE_AEFU_OLDVER:-}" ] || echo "1.8.15+2.g$(git -C "$S" rev-parse --short=9 HEAD)" > "$P/.agentic-framework/VERSION"
cp "$S/agents/termlink/termlink.sh" "$P/.agentic-framework/agents/termlink/termlink.sh"
echo "upstream toolkit copy" > "$P/scripts/tool.sh.upstream"
[ -n "${FAKE_AEFU_CLOBBER:-}" ] && sed -i 's/^project rule$/project rule CLOBBERED/' "$P/CLAUDE.md"
[ -n "${FAKE_AEFU_TOUCH_KEEP:-}" ] && echo "upstream overwrote it" > "$P/.agentic-framework/lib/keep.sh"
echo "21 change(s) applied"
EOS
chmod +x "$AE/src/bin/fw"
printf '1.8.15\n' > "$AE/src/VERSION"; printf 'upstream v1\n' > "$AE/src/lib/keep.sh"; printf 'old scan\n' > "$AE/src/agents/termlink/termlink.sh"
aefu_g -C "$AE/src" init -q && aefu_g -C "$AE/src" add -A && aefu_g -C "$AE/src" commit -q -m "A: vendored"
AEFU_A="$(git -C "$AE/src" rev-parse HEAD)"
aefu_g -C "$AE/src" checkout -q -b side && echo side > "$AE/src/side.txt" && aefu_g -C "$AE/src" add -A && aefu_g -C "$AE/src" commit -q -m "S: side"
AEFU_S="$(git -C "$AE/src" rev-parse HEAD)"; aefu_g -C "$AE/src" checkout -q main
printf 'upstream v1\nupstream v2 line\n' > "$AE/src/lib/keep.sh"; printf 'T-5105 linear scan\n' > "$AE/src/agents/termlink/termlink.sh"
aefu_g -C "$AE/src" add -A && aefu_g -C "$AE/src" commit -q -m "B: be.49" && aefu_g -C "$AE/src" tag v1.8.16-be.49
AEFU_B="$(git -C "$AE/src" rev-parse HEAD)"
mkdir -p "$AE/merges/lib"
printf 'upstream v1\nlocal fix T-1\nupstream v2 line\n' > "$AE/merges/lib/keep.sh.merged"
printf '# path\tours\ttheirs\tmerged\nlib/keep.sh\t%s\t%s\t%s\n' "$(printf 'upstream v1\nlocal fix T-1\n' | sha256sum | cut -d' ' -f1)" \
    "$(sha256sum < "$AE/src/lib/keep.sh" | cut -d' ' -f1)" "$(sha256sum < "$AE/merges/lib/keep.sh.merged" | cut -d' ' -f1)" > "$AE/merges/MANIFEST.tsv"
cat > "$AE/proj-fw" <<'EOS'
#!/usr/bin/env bash
# fake project fw for action 11a: config set HOST, watchtower restart, doctor
P="$(cd "$(dirname "$0")/../.." && pwd)"
case "$1" in
  config) [ "$2 $3" = "set HOST" ] || exit 9; [ -n "${FAKE_AEFU_NOHOST:-}" ] && exit 0
          sed -i '/^HOST:/d' "$P/.framework.yaml"; echo "HOST: $4" >> "$P/.framework.yaml" ;;
  watchtower) [ "$2" = restart ] || exit 9; echo restart >> "$FAKE_AEFU_WTUP.restarts"; [ -n "${FAKE_AEFU_WT_DEAD:-}" ] || : > "$FAKE_AEFU_WTUP" ;;
  doctor) echo "PASS  all fine"   # the FAIL appears only once the upgrade has written its files
          [ -n "${FAKE_AEFU_DOCTOR_FAIL:-}" ] && [ -e "$P/.agentic-framework/lib/new-upstream.py" ] && echo "FAIL  something new"; exit 0 ;;
  *) exit 9 ;;
esac
EOS
printf '#!/usr/bin/env bash\n[ -e "$FAKE_AEFU_WTUP" ] && grep -qx "HOST: 0.0.0.0" "$FAKE_AEFU_PROJ/.framework.yaml"\n' > "$AE/curl"
printf '#!/usr/bin/env bash\n[ -z "${FAKE_AEFU_DIV_DIRTY:-}" ]\n' > "$AE/divcheck"
chmod +x "$AE/proj-fw" "$AE/curl" "$AE/divcheck"
aefu_state() {  # done|fresh [vendored-commit]
    local P="$AE/proj" v="${2:-$AEFU_A}"
    rm -rf "$P" "$AE/origin.git" "$AE/wtup" "$AE/wtup.restarts" "$AE/work"; mkdir -p "$AE/work"
    aefu_g init -q --bare "$AE/origin.git"
    mkdir -p "$P/.agentic-framework/bin" "$P/.agentic-framework/lib" "$P/.agentic-framework/agents/termlink" "$P/scripts"
    printf '# proj\nproject rule\n## Core Principle\ngovernance\n' > "$P/CLAUDE.md"
    printf 'PORT: 3003\n' > "$P/.framework.yaml"
    printf 'files:\n  - path: lib/keep.sh\n    reason: fixture\n' > "$P/.fwvendor-preserve.yaml"
    printf '.context/working/\n' > "$P/.gitignore"
    cp "$AE/proj-fw" "$P/.agentic-framework/bin/fw"
    echo "1.8.15+1.g${v:0:9}" > "$P/.agentic-framework/VERSION"
    printf 'upstream v1\nlocal fix T-1\n' > "$P/.agentic-framework/lib/keep.sh"
    printf 'old scan\n' > "$P/.agentic-framework/agents/termlink/termlink.sh"
    printf '#!/bin/sh\n' > "$P/scripts/existing.sh"
    aefu_g -C "$P" init -q; git -C "$P" config user.name fx; git -C "$P" config user.email fx@example.invalid
    git -C "$P" config commit.gpgsign false; git -C "$P" remote add origin "$AE/origin.git"
    aefu_g -C "$P" add -A && aefu_g -C "$P" commit -q -m "T-1: fixture project"
    if [ "$1" = done ]; then
        echo "1.8.15+2.g${AEFU_B:0:9}" > "$P/.agentic-framework/VERSION"
        cp "$AE/merges/lib/keep.sh.merged" "$P/.agentic-framework/lib/keep.sh"
        printf 'T-5105 linear scan\n' > "$P/.agentic-framework/agents/termlink/termlink.sh"
        echo "HOST: 0.0.0.0" >> "$P/.framework.yaml"; : > "$AE/wtup"
        aefu_g -C "$P" add -A && aefu_g -C "$P" commit -q -m "T-3778: fw upgrade fixture"
    fi
    aefu_g -C "$P" push -q origin HEAD:main 2>/dev/null
}
aefu_state done
export RUNME_AEFU_SRC="$AE/src" RUNME_AEFU_PROJECT="$AE/proj" RUNME_AEFU_WORKROOT="$AE/work" RUNME_AEFU_DIVCHECK="$AE/divcheck" \
       RUNME_AEFU_URL="http://fixture.invalid:3003" RUNME_AEFU_CURL="$AE/curl" RUNME_AEFU_MERGES="$AE/merges" \
       FAKE_AEFU_WTUP="$AE/wtup" FAKE_AEFU_PROJ="$AE/proj"
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
# T-3647 (action 4b): never the real systemctl or /proc. No units by default, so other
# cases see nothing from 4b; its own cases below set RUNME_W1_UNITS.
W1R="$TMP/w1r"; mkdir -p "$W1R/proc"
export RUNME_W1_UNITS="" RUNME_W1_SYSTEMCTL="$W1R/systemctl" RUNME_W1_PROC="$W1R/proc" RUNME_W1_BIN="$W1R/termlink-new"
# T-3651 (action 4c): scratch configs, sockets, status client and audit log; no re-pin recorded.
mkdir -p "$W1R/conf"
export RUNME_W1_REPIN_MAIN="" RUNME_W1_CONF_DIR="$W1R/conf" RUNME_W1_SOCK_DIR="$W1R/sock" \
       RUNME_W1_STATUS_TL="$W1R/status-tl" RUNME_W1_PIN_AUDIT="$W1R/repins.jsonl"
# T-3654 (action 4d): the project the W1 sidecars are given; scratch env files live in $W1R/conf.
export RUNME_W1_MAIN_PROJECT="010-termlink"
# T-3698 (action 4e): no release named by default (list-only); scratch release root and state;
# the REAL install.sh, fed from a local file:// "release" built in the 4e cases below.
export RUNME_REL_TAG="" RUNME_REL_ROOT="$TMP/rel-root" RUNME_REL_INSTALLER="$REPO_ROOT/install.sh" \
       RUNME_REL_BASE_URL="file://$TMP/rel-pub" RUNME_REL_STATE="$TMP/rel-state"
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
# T-3615 actions 22b/24: token and secret commands run as the fleet host's root, whose HOME is
# scratch ($F/tok-remote/root); logged apart (they carry a fake token, never a real one).
case "$2" in *RUNME-TOKEN-*|*RUNME-SECRET-*)
    echo "$2" >> "$FAKE_FLEET_DIR/tok-cmds"
    [ "${FAKE_TOK_DOWN:-0}" = "1" ] && exit 3
    HOME="$FAKE_FLEET_DIR/tok-remote/root" bash -c "$2"
    exit $? ;;
esac
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
# T-3615 action 23 + 22b/24: the "remote" termlink. channel list/post authenticate the way the
# real CLI does: the profile at --hub in $HOME/.termlink/hubs.toml; a secret_file wins (a missing
# file is an error), else token_file, else nothing. Each call logs which credential it used, so a
# case can assert the TOKEN was what authenticated. FAKE_RR_AUTH_RC overrides `channel list`;
# FAKE_TOK_POST_FAIL=1 makes a post come back queued, not accepted.
echo none > "$F/rr-units"; echo '{}' > "$F/rr-qs"; mkdir -p "$F/rr-cron"
cat > "$F/rr-tl" <<'EOS'
#!/usr/bin/env bash
hub=""; a=("$@"); for ((i = 0; i < ${#a[@]}; i++)); do [ "${a[$i]}" = --hub ] && hub="${a[$((i + 1))]}"; done
cred() { python3 - "$HOME/.termlink/hubs.toml" "$hub" <<'P'
import os, re, sys
try:
    t = open(sys.argv[1]).read()
except OSError:
    print("none"); sys.exit()
for sec in re.split(r'(?m)^\[', t)[1:]:
    kv = dict(re.findall(r'(?m)^(\w+)\s*=\s*"([^"]*)"', sec))
    if kv.get("address") != sys.argv[2]:
        continue
    if "secret_file" in kv:
        ok = os.path.exists(os.path.expanduser(kv["secret_file"])) and not os.environ.get("FAKE_SECRET_STALE")
        print("secret" if ok else "none"); break
    print("token" if kv.get("token_file") and os.path.exists(kv["token_file"]) else "none"); break
else:
    print("none")
P
}
case "$1 $2" in
  "channel list")
    [ -n "${FAKE_RR_AUTH_RC:-}" ] && exit "$FAKE_RR_AUTH_RC"
    c="$(cred)"; echo "list $c $HOME" >> "$FAKE_FLEET_DIR/tok-auth"; [ "$c" != none ]; exit $? ;;
  "channel post")
    c="$(cred)"; echo "post $c $HOME" >> "$FAKE_FLEET_DIR/tok-auth"
    [ "$c" = none ] && { echo "auth failed" >&2; exit 1; }
    [ "${FAKE_TOK_POST_FAIL:-0}" = 1 ] && { echo "Queued to $3 (offline)"; exit 0; }
    echo "Posted to $3 — offset=7"; exit 0 ;;
  "channel queue-status") echo '{}' ;;
  "identity show") echo '{"action":"loaded","fingerprint":"0123456789abcdef","ok":true}' ;;
esac
exit 0
EOS
# The .107 side: a fake mint (a fresh fake token per call, logged) and a fake local termlink.
cat > "$F/tok-mint" <<'EOS'
#!/usr/bin/env bash
[ "${FAKE_MINT_FAIL:-0}" = 1 ] && { echo "cannot read hub.secret" >&2; exit 1; }
out=""; while [ $# -gt 0 ]; do [ "$1" = --out ] && out="$2"; shift; done
echo "fake-token-$(date +%s%N)-$RANDOM" > "$out"; chmod 600 "$out"
echo mint >> "$FAKE_FLEET_DIR/tok-mints"
echo "{\"ok\":true,\"out\":\"$out\",\"scope\":\"${FAKE_MINT_SCOPE:-interact}\",\"expires_at\":$(( $(date +%s) + ${FAKE_MINT_TTL:-604800} )),\"ttl_secs\":604800,\"clamped\":true}"
EOS
printf '#!/usr/bin/env bash\necho "$*" >> "$FAKE_FLEET_DIR/tok-local"\nexit 0\n' > "$F/tok-ltl"
chmod +x "$F/rr-tl" "$F/tok-mint" "$F/tok-ltl"
TR="$F/tok-remote/root"   # the fleet host's root HOME, in scratch
tok_reset() {  # a host holding .107's secret in its profile, no token yet; .107 has no record
    rm -rf "$F/tok-remote" "$F/tok-state" "$F/tok-auth" "$F/tok-mints" "$F/tok-cmds"
    mkdir -p "$TR/.termlink/secrets" "$F/tok-refs"
    echo "fixture-not-a-secret" > "$TR/.termlink/secrets/107-hub.hex"; chmod 600 "$TR/.termlink/secrets/107-hub.hex"
    printf '# fleet host hubs.toml (fixture)\n[hubs.local]\naddress = "127.0.0.1:9100"\nsecret_file = "/nonexistent/local.hex"\n\n[hubs.home-107]\naddress = "192.168.10.107:9100"\nsecret_file = "%s"\nscope = "interact"\n' \
        "$TR/.termlink/secrets/107-hub.hex" > "$TR/.termlink/hubs.toml"
}
tok_reset
export RUNME_TOKEN_REMINT="bash $REPO_ROOT/scripts/fleet-token-remint.sh" RUNME_TOKEN_STATE_DIR="$F/tok-state" \
       RUNME_TOKEN_MINT="$F/tok-mint" RUNME_TOKEN_LOCAL_TL="$F/tok-ltl" RUNME_HUB_RUNTIME_DIR="${RUNME_HUB_RUNTIME_DIR:-$F/tok-rt}" \
       RUNME_TOKEN_REMOTE_TOKEN="$TR/.termlink/secrets/107-hub.token" RUNME_TOKEN_REMOTE_HOME="$TR/.termlink/token-only-home" \
       RUNME_TOKEN_REMOTE_HUBS="$TR/.termlink/hubs.toml" RUNME_TOKEN_REMOTE_ID="$TR/.termlink" \
       RUNME_TOKEN_REMOTE_OWNER="$(id -un)" RUNME_SECRET_REMOVAL_HUBS="" RUNME_SECRET_REF_DIRS="$F/tok-refs"
# T-3756 action 22c: the local design user is the fixture's own user with a scratch home. The
# "runuser" is a fake that logs its ARGV (the token must never be in it) and runs the stdin
# script with HOME = that home; FAKE_LT_DOWN=1 makes it answer nothing. The user's PATH termlink
# is a canned old 0.11 build.
LTU="$(id -un)"; LH="$F/lt-home"
cat > "$F/lt-run" <<'EOS'
#!/usr/bin/env bash
echo "argv: $*" >> "$FAKE_FLEET_DIR/lt-argv"
[ "${FAKE_LT_DOWN:-0}" = 1 ] && { cat >/dev/null; exit 3; }
HOME="$2" bash -s
EOS
printf '#!/usr/bin/env bash
echo /home/u/.local/bin/termlink
echo "termlink 0.11.931"
echo TOKEN-FILE-NO
' > "$F/lt-pathtl"
chmod +x "$F/lt-run" "$F/lt-pathtl"
lt_reset() {  # the user's own hubs.toml: two profiles and its old secret copy; no token, no record
    rm -rf "$LH"; rm -f "$F/tok-state/local-$LTU.state" "$F/lt-argv"; mkdir -p "$LH/.termlink/secrets"
    echo "fixture-user-secret-copy" > "$LH/.termlink/secrets/hub.secret"; chmod 600 "$LH/.termlink/secrets/hub.secret"
    printf '# user hubs.toml (fixture)\n[hubs.ring20]\naddress = "192.168.10.122:9100"\nsecret_file = "/nonexistent/x.hex"\n\n[hubs.home]\naddress = "192.168.10.107:9100"\nsecret_file = "%s"\n' \
        "$LH/.termlink/secrets/hub.secret" > "$LH/.termlink/hubs.toml"
}
lt_reset
export RUNME_LOCALTOK_USER="$LTU" RUNME_LOCALTOK_HOME="$LH" RUNME_LOCALTOK_RUN="$F/lt-run" \
       RUNME_LOCALTOK_TL="$F/rr-tl" RUNME_LOCALTOK_PATHTL="$F/lt-pathtl"
# T-3781 action 22e: the hub link of the same scratch user, in its own scratch home. Its own fake
# mint (logged to hl-mints, so 22b/22c mint counts are untouched), a fake "runuser" that logs its
# ARGV and runs the stdin script with HOME = that home (FAKE_HL_DOWN=1: no answer), and a fake
# user-side termlink whose `register --json` prints a linked record with owner FAKE_HL_OWNER
# (default: the user) and stays up until killed. Every other case sees a verified, unchanged,
# not-yet-due link (skip), so 22e never mints or fails outside its own cases.
HLH="$F/hl-home"; HLS="$F/hl-state"
cat > "$F/hl-mint" <<'EOS'
#!/usr/bin/env bash
[ "${FAKE_HL_MINT_FAIL:-0}" = 1 ] && exit 1
args="$*"
out=""; while [ $# -gt 0 ]; do [ "$1" = --out ] && out="$2"; shift; done
echo "fake-claim-token-$(date +%s%N)-$RANDOM" > "$out"; chmod 600 "$out"
echo "mint $args" >> "$FAKE_FLEET_DIR/hl-mints"
echo "{\"ok\":true,\"out\":\"$out\",\"scope\":\"observe\",\"expires_at\":$(( $(date +%s) + 604800 ))}"
EOS
cat > "$F/hl-run" <<'EOS'
#!/usr/bin/env bash
echo "argv: $*" >> "$FAKE_FLEET_DIR/hl-argv"
[ "${FAKE_HL_DOWN:-0}" = 1 ] && { cat >/dev/null; exit 3; }
HOME="$2" bash -s
EOS
cat > "$F/hl-tl" <<'EOS'
#!/usr/bin/env bash
[ "${1:-}" = register ] || exit 0
echo "{\"ok\":true,\"hub_link\":{\"owner\":\"${FAKE_HL_OWNER:-$(id -un)}\",\"name\":\"x\"}}"
echo "register $HOME" >> "$FAKE_FLEET_DIR/hl-registers"
exec sleep 30
EOS
chmod +x "$F/hl-mint" "$F/hl-run" "$F/hl-tl"
hl_skip_state() {  # a verified link on record matching the installed files, 5 days left
    rm -rf "$HLH" "$HLS"; mkdir -p "$HLH/.termlink" "$HLS"
    echo "fixture-claim-token" > "$HLH/.termlink/hub-register.token"; chmod 600 "$HLH/.termlink/hub-register.token"
    echo "127.0.0.1:9100" > "$HLH/.termlink/hub-register.addr"
    printf 'expires_at=%s\ntoken_sha256=%s\n' "$(( $(date +%s) + 432000 ))" "$(sha256sum < "$HLH/.termlink/hub-register.token" | cut -d' ' -f1)" > "$HLS/$LTU.state"
}
hl_skip_state
export RUNME_HUBLINK_USER="$LTU" RUNME_HUBLINK_HOME="$HLH" RUNME_HUBLINK_RUN="$F/hl-run" \
       RUNME_HUBLINK_MINT="$F/hl-mint" RUNME_HUBLINK_STATE_DIR="$HLS" RUNME_HUBLINK_TL="$F/hl-tl"
# T-3758 action 22d: the user's scratch home holds the old 0.11.931 copy in ~/.local/bin (mode 750).
# The fake "runuser" logs its argv and runs the stdin script with HOME = that home and a login-like
# PATH: ~/bin, ~/.local/bin, ~/.cargo/bin (T-3762), then the "system" dir with the current fake termlink (0.12.77), whose
# `channel list` logs who called it and exits FAKE_STL_AUTH_RC. FAKE_STL_DOWN=1 answers nothing.
SH="$F/stl-home"; SSYS="$F/stl-sys"; mkdir -p "$SSYS"
cat > "$F/stl-run" <<'EOS'
#!/usr/bin/env bash
echo "argv: $*" >> "$FAKE_FLEET_DIR/stl-argv"
[ "${FAKE_STL_DOWN:-0}" = 1 ] && { cat >/dev/null; exit 3; }
sys="$FAKE_STL_SYS:"; [ "${FAKE_STL_NOSYS:-0}" = 1 ] && sys=""   # T-3762: no current termlink on PATH
HOME="$2" PATH="$2/bin:$2/.local/bin:$2/.cargo/bin:${sys}$2/after:/usr/bin:/bin" bash -s
EOS
cat > "$SSYS/termlink" <<'EOS'
#!/usr/bin/env bash
[ "${1:-}" = --version ] && { echo "termlink 0.12.77"; exit 0; }
[ "${1:-} ${2:-}" = "channel list" ] && { echo "list $HOME $*" >> "$FAKE_FLEET_DIR/stl-auth"; exit "${FAKE_STL_AUTH_RC:-0}"; }
exit 0
EOS
chmod +x "$F/stl-run" "$SSYS/termlink"
stl_old() { printf '#!/usr/bin/env bash\necho "termlink %s"\n' "$1" > "$2"; chmod 750 "$2"; }   # <version> <path>
stl_reset() { rm -rf "$SH" "$F/stl-argv" "$F/stl-auth"; mkdir -p "$SH/.local/bin"; stl_old 0.11.931 "$SH/.local/bin/termlink"; }
stl_reset
export FAKE_STL_SYS="$SSYS" RUNME_STALETL_RUN="$F/stl-run" RUNME_STALETL_HOME="$SH" RUNME_STALETL_CUR="$SSYS/termlink"
# The install cases run with the recorded ruling (Q3 = A); the held default has its own case
# (RUNME_ROLLUP_CREDENTIAL_DECISION set but empty).
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
if [ "$rc" = "0" ] && echo "$out" | grep_all "would install"; then ok "--dry-run reports what it would install"
else bad "--dry-run reports intent" "rc=$rc: $out"; fi
if [ -z "$(ls -A "$TMP/cron")" ]; then ok "--dry-run wrote nothing"
else bad "--dry-run wrote nothing" "$(ls -A "$TMP/cron")"; fi
# T-3272: dry-run names each approved close and performs none.
if echo "$out" | grep_all "would close T-3132" && [ "$(ls "$TMP/tasks/active" | grep -cE '^T-31(32|28|30)-')" = "3" ] && [ -z "$(ls -A "$TMP/tasks/completed")" ]; then
    ok "--dry-run reports the approved closes and closes nothing"
else bad "--dry-run closes nothing" "$(ls "$TMP/tasks/active" "$TMP/tasks/completed"): $out"; fi
if echo "$out" | grep_all "would record T-9284 = go" && ! grep -q '^\*\*Decision\*\*' "$TMP/tasks/active/T-9284-fixture.md"; then
    ok "--dry-run reports the approved decision and records nothing"
else bad "--dry-run decision" "$out"; fi
if echo "$out" | grep_all "would install $TMP/tl-new" && cmp -s "$TMP/tl-old" "$TMP/bin/termlink"; then
    ok "--dry-run reports the termlink install and replaces nothing"
else bad "--dry-run termlink install" "$out"; fi
# T-3273: the run left a readable log — full output, header, rc line — and
# latest.log resolves to it. The agent reads this after the operator runs it.
LOGF="$TMP/logs/latest.log"
if [ -f "$LOGF" ] && grep -q '^=== runme.sh start .*args=\[--dry-run\]' "$LOGF" \
   && grep -q 'would close T-3132' "$LOGF" && grep -q '=== runme.sh finished rc=0' "$LOGF"; then
    ok "every run is logged: header + full output + rc line, via latest.log"
else bad "run is logged" "$(ls -la "$TMP/logs"; cat "$LOGF" 2>/dev/null | head -5)"; fi
if echo "$out" | grep_all "^log: $TMP/logs/runme-"; then ok "the log's full path is printed to the operator"
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
if [ "$rc" = "2" ] && echo "$out" | grep_all "already in progress (pid $HOLDER, log $TMP/logs/the-running-one.log)" \
   && ! echo "$out" | grep_all "would install" && [ "$(readlink "$TMP/logs/latest.log")" = "$before" ] \
   && grep -q "already in progress (pid $HOLDER" "$TMP/logs/refused.log" && [ -d "$LOCK" ]; then
    ok "lock: a second start refuses (exit 2), names the holder, does nothing, keeps latest.log"
else bad "lock: second start refused" "rc=$rc latest=$(readlink "$TMP/logs/latest.log") before=$before: $out"; fi
kill "$HOLDER" 2>/dev/null; wait "$HOLDER" 2>/dev/null
# The holder is gone: the lock is stale and is taken over, then released at exit.
out=$(run --dry-run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "taking over a stale lock (pid $HOLDER is gone)" && [ ! -e "$LOCK" ]; then
    ok "lock: a stale lock (holder gone) is taken over and released"
else bad "lock: stale takeover" "rc=$rc: $out"; fi
# Control for the name check: a live process that is NOT a runme.sh (pid reuse) is stale too.
sleep 60 & OTHER=$!
mkdir -p "$LOCK"; echo "$OTHER" > "$LOCK/pid"
out=$(run --dry-run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "taking over a stale lock (pid $OTHER" && [ ! -e "$LOCK" ]; then
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
EXPECT_N=$(( $(grep -c '^install_crontab ' "$RUNME") + $(printf '%s\n' "$RUNME_TEST_CLOSES" | grep -c '|') + $(printf '%s\n' "$RUNME_TEST_APPROVED_DECISIONS" | grep -c '|')  + 48 ))   # +1 hub link (T-3781 action 22e); +1 vendored framework at be.49 (T-3778 action 11a); +1 stale user termlink moved aside (T-3758 action 22d); +1 local design user token (T-3756 action 22c); +1 release list-only (T-3698 action 4e); +1 testdev long-lived token (T-3685 action 28); +1 docker tcp 2375 closed (T-3664 action 27); +1 drop T-3656 waiting (T-3657 action 26); +1 ntfy cron lines (T-3618 action 25); +3 T-3615 (action 1b background-class resync, action 23 fleet rollup, action 22b fleet token); +2 operator tick records (T-3611: T-3006, T-3200); +1 hub flap ack (T-3587 action 5b); +1 watchtower onto current (T-3568 action 11c); +1 audit crontab sync (T-3554 action 11b); +1 ring20 pack (T-3477 action 22); +1 .122 retention sweeper (T-3311 action 21); +6 driver specs (T-3440 5c: arc-substrate-fitness 3, arc-011 3); +1 W1 drill (T-3426); +1 inbox cap (T-3443); +1 W1 readiness/wait (T-3438); +2 sidecar supervision: settings dir + unit template (T-3435); +1 claude-fw router (T-3421); +1 testdev claude (T-3416); +5 operator tick records (T-3351, T-3393: T-3349, T-3351, T-3388; T-3460: T-3291, T-3302); +1 sidecar sweep cron (T-3370); +1 stray-hub stop (T-3343); +1 reindex schedule (T-3336); +1 termlink install (T-3287); +1 hub unit (T-3299); +1 hub restart (T-3290); +1 fleet hub (T-3290 action 6); +1 zombie reap (T-3297); +1 identity env (T-3303); +1 fedprobe bound (T-2988); +1 sidecar restart (T-3325)

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
if echo "$out" | grep_all "recorded and verified: T-9284 = go" && grep -q '^\*\*Decision\*\*: GO$' "$TMP/tasks/active/T-9284-fixture.md"; then
    ok "default run records the approved decision and verifies it on disk"
else bad "default run records decision" "$out"; fi
if echo "$out" | grep_all "installed and verified: $TMP/bin/termlink (termlink 9.9.9" && cmp -s "$TMP/tl-new" "$TMP/bin/termlink"; then
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
   && echo "$out" | grep_all "installed, reloaded and verified: $SCU/units/termlink-sidecar@.service"; then
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
if [ "$rc" = "0" ] && echo "$out" | grep_all "already installed and identical"; then ok "second run skips as already-done"
else bad "idempotent second run" "rc=$rc: $out"; fi
if echo "$out" | grep_all -E "0 done, ${EXPECT_N} already-done"; then ok "summary counts the skips rather than re-claiming the work"
else bad "summary counts skips" "$out"; fi

# ---------------------------------------------------------------------------
# 7. TAMPER / DRIFT. If the installed copy has been edited out-of-band, the
#    script must re-install rather than trust its presence.
# ---------------------------------------------------------------------------
echo "# tampered out of band" >> "$TMP/cron/termlink-notify-sidecar-canary"
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "installed and verified.*termlink-notify-sidecar-canary"; then
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
if echo "$out" | grep_all "FAILED  source missing"; then ok "missing source names the file"
else bad "missing source named" "$out"; fi

# T-3272: fw exiting 0 is NOT evidence. A fw that claims success but moves nothing
# must be a FAILED close and exit 1 — the verification reads disk, not exit codes.
mv "$TMP/tasks/completed/T-3130-fixture.md" "$TMP/tasks/active/"
sed -i 's/^status: .*/status: started-work/' "$TMP/tasks/active/T-3130-fixture.md"
out=$(RUNME_FW="$TMP/noop-fw" run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep_all "FAILED  T-3130 did not land"; then
    ok "a close that fw reports but disk does not show => FAILED, exit 1"
else bad "unverified close is a failure" "rc=$rc: $out"; fi
# T-3273: a failed run's log says so — the agent reading it must see rc=1 and the FAILED line.
sleep 1   # tee flushes the trap line asynchronously after the script exits
if grep -q 'FAILED  T-3130 did not land' "$TMP/logs/latest.log" && grep -q '=== runme.sh finished rc=1' "$TMP/logs/latest.log"; then
    ok "a failed run's log carries the FAILED line and rc=1"
else bad "failed run logged" "$(tail -5 "$TMP/logs/latest.log" 2>/dev/null)"; fi
printf -- '---\nid: T-9285\n---\n' > "$TMP/tasks/active/T-9285-fixture.md"
out=$(RUNME_FW="$TMP/noop-fw" RUNME_TEST_APPROVED_DECISIONS="T-9285|no-go|fixture" run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep_all "FAILED  T-9285: decision not found"; then
    ok "a decision fw reports but disk does not show => FAILED, exit 1"
else bad "unverified decision is a failure" "rc=$rc: $out"; fi
out=$(RUNME_TERMLINK_SRC="$TMP/tl-old" RUNME_TERMLINK_DEST="$TMP/bin/termlink-2" run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep_all "rejects --no-create"; then
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
    if [ "$rc" = "2" ] && echo "$out" | grep_all "needs root"; then ok "non-root run refuses with exit 2 and applies nothing"
    else bad "non-root refusal" "rc=$rc: $out"; fi
elif command -v runuser >/dev/null 2>&1 && id nobody >/dev/null 2>&1; then
    chmod -R a+rx "$TMP" 2>/dev/null || true
    # TMPDIR=/tmp (T-3647): the close gate runs verification under a private root-only
    # TMPDIR (AEF T-4846); `nobody` cannot create runme's fallback log dir there, and the
    # failed lock mkdir then reads as "another run is already in progress".
    out=$(runuser -u nobody -- env -u RUNME_FIXTURE_ALLOW_NONROOT TMPDIR=/tmp RUNME_CRON_DIR="$TMP/cron" bash "$RUNME" 2>&1); rc=$?
    if [ "$rc" = "2" ] && echo "$out" | grep_all "needs root"; then ok "non-root run refuses with exit 2 and applies nothing"
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
if echo "$out" | grep_all "Pending decisions"; then ok "default run LISTS pending decisions"
else bad "lists pending decisions" "$out"; fi
if echo "$out" | grep_all "nothing here runs by default"; then ok "the listing states that nothing runs by default"
else bad "listing states non-execution" "$out"; fi
# T-3285: APPROVED decisions (action 3) do run; a PENDING, undecided one (T-9055) never does.
if echo "$out" | grep_all "would record T-9055"; then bad "default run must not record a pending decision" "$out"
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
if [ "$rc" = "2" ] && echo "$vout" | grep_all "verdict must be go, no-go or defer"; then ok "verdict outside go/no-go/defer => exit 2 (refused for the verdict)"
else bad "bad verdict refused" "rc=$rc: $vout"; fi
rc=0; RUNME_CRON_DIR="$TMP/cron" bash "$RUNME" --decide T-9055 >/dev/null 2>&1 || rc=$?
if [ "$rc" = "2" ]; then ok "malformed --decide (no '=') => exit 2"
else bad "malformed refused" "rc=$rc"; fi

# ---------------------------------------------------------------------------
# 14. The plugin cleanup is OPT-IN: absent from a default run, since it is a
#     recommendation the operator has not approved, not a repair.
# ---------------------------------------------------------------------------
if echo "$out" | grep_all -i "disabling purpose-mismatch"; then bad "plugin disable must be opt-in" "$out"
else ok "plugin cleanup absent from a default run"; fi

# ---------------------------------------------------------------------------
# 15. T-3276 — the REAL shape: with no approved closures and no pending decisions
#     (seams unset), a run shows no closures section and says decisions are none.
# ---------------------------------------------------------------------------
eout=$(env -u RUNME_TEST_DECISIONS RUNME_TEST_CLOSES= RUNME_TEST_APPROVED_DECISIONS= RUNME_CRON_DIR="$TMP/cron" bash "$RUNME" --dry-run 2>&1); rc=$?
if [ "$rc" = "0" ] && ! echo "$eout" | grep_all "Approved closures" && ! echo "$eout" | grep_all "Approved inception decisions" && echo "$eout" | grep_all "none pending"; then
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
   && echo "$out" | grep_all "installed and verified: $M/usrlocal/termlink (termlink 9.9.9"; then
    ok "multi-dest: always-install path + stale existing copy both installed and verified"
else bad "multi-dest install" "rc=$rc: $out"; fi
if [ ! -e "$M/dotlocal/termlink" ] && echo "$out" | grep_all "no termlink at $M/dotlocal/termlink — not creating one"; then
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
if [ "$rc" = "0" ] && [ "$rc2" = "1" ] && echo "$out2" | grep_all "FAILED  could not install $TMP/tl-new -> $M/blocker/sub/termlink" \
   && cmp -s "$TMP/tl-new" "$M/usrlocal/termlink"; then
    ok "multi-dest: one failing path => FAILED naming it, exit 1, the other path still installed"
else bad "per-path failure" "rc=$rc rc2=$rc2: $out2"; fi

# ---------------------------------------------------------------------------
# 14. T-3290 — hub restart. All through the fake systemctl / proc / runtime dir.
# ---------------------------------------------------------------------------
hub_state 200 "$H/hubbin"
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "skip    hub (pid 200) already runs the installed binary" && [ ! -e "$H/restarts" ]; then
    ok "hub: already on the installed binary => skip, no restart"
else bad "hub idempotent skip" "rc=$rc: $out"; fi
hub_state 300 "/old/termlink (deleted)"
out=$(run --dry-run); rc=$?
if echo "$out" | grep_all "would restart termlink-hub (pid 300 runs '/old/termlink (deleted)'" && [ ! -e "$H/restarts" ]; then
    ok "hub: --dry-run reports the restart and does NOT perform it"
else bad "hub dry-run" "rc=$rc: $out"; fi
hub_state 400 "/old/termlink (deleted)"
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "OK      restarted and verified: pid 400 -> 401" && [ "$(wc -l < "$H/restarts")" = "1" ]; then
    ok "hub: stale hub restarted once, new pid on the installed exe, secret+cert unchanged => OK"
else bad "hub restart" "rc=$rc: $out"; fi
hub_state 450 "/old/termlink (deleted)"
out=$(FAKE_HUB_STATUS_LAG=1 run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "OK      restarted and verified: pid 450 -> 451"; then
    ok "hub (T-3450): hub status lagging one read behind the restart is re-read, not a false FAILED"
else bad "hub status lag" "rc=$rc: $out"; fi
hub_state 460 "/old/termlink (deleted)"
out=$(FAKE_HUB_STATUS_LAG=999 run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep_all "hub-status-not-running-as-461"; then
    ok "hub (T-3450): hub status that never reads running still FAILS, named"
else bad "hub status never running" "rc=$rc: $out"; fi
# T-3587 action 5b: acknowledge flap residue with reset-failed (hub already current, so 5 skips).
hub_state 100 "$H/hubbin"; rm -f "$H/resets"
echo 3 > "$H/nrestarts"; out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "skip    termlink-hub NRestarts=3 (max 5)" && [ ! -e "$H/resets" ]; then
    ok "flap ack: NRestarts at or below the max is a skip, nothing reset"
else bad "flap ack skip" "rc=$rc: $out"; fi
echo 433 > "$H/nrestarts"; out=$(run --dry-run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "would run: systemctl reset-failed termlink-hub (NRestarts=433" && [ ! -e "$H/resets" ]; then
    ok "flap ack: --dry-run names the reset and performs none"
else bad "flap ack dry-run" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "OK      acknowledged: termlink-hub NRestarts 433 -> 0" && [ "$(wc -l < "$H/resets")" = "1" ]; then
    ok "flap ack: active unit above the max is reset once and verified (433 -> 0)"
else bad "flap ack reset" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "skip    termlink-hub NRestarts=0" && [ "$(wc -l < "$H/resets")" = "1" ]; then
    ok "flap ack: second run skips (idempotent)"
else bad "flap ack idempotent" "rc=$rc: $out"; fi
echo 433 > "$H/nrestarts"; rm -f "$H/resets"; out=$(FAKE_HUB_ACTIVE=activating run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep_all "live crash loop, not residue; not acknowledging" && [ ! -e "$H/resets" ]; then
    ok "flap ack: a unit that is not active is a live incident => FAILED, never reset"
else bad "flap ack not active" "rc=$rc: $out"; fi
out=$(FAKE_HUB_RESET_STICKY=1 run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep_all "reset-failed did not clear the counter: NRestarts=433"; then
    ok "flap ack: a reset that does not clear the counter is FAILED, not OK"
else bad "flap ack sticky" "rc=$rc: $out"; fi
echo 0 > "$H/nrestarts"; rm -f "$H/resets"
# T-3592 action 2b: hand human-owned tasks back to the agent (scratch tasks only).
printf -- '---\nid: T-9592\nstatus: captured\nowner: human\n---\n' > "$TMP/tasks/active/T-9592-fixture.md"
printf -- '---\nid: T-9593\nstatus: captured\nowner: agent\n---\n' > "$TMP/tasks/active/T-9593-fixture.md"
out=$(RUNME_TEST_HANDBACKS="T-9592 T-9593 T-9594" run --dry-run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "would hand T-9592 back" && grep -q '^owner: human' "$TMP/tasks/active/T-9592-fixture.md"; then
    ok "handback: --dry-run names the hand-back and changes nothing"
else bad "handback dry-run" "rc=$rc: $out"; fi
out=$(RUNME_TEST_HANDBACKS="T-9592 T-9593 T-9594" run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "OK      handed back and verified: T-9592 owner: agent" \
   && echo "$out" | grep_all "skip    T-9593 already owner: agent" && echo "$out" | grep_all "skip    T-9594 is no longer active" \
   && grep -q '^owner: agent' "$TMP/tasks/active/T-9592-fixture.md"; then
    ok "handback: human-owned task handed back and verified; agent-owned and missing ones skip"
else bad "handback run" "rc=$rc: $out"; fi
sed -i 's/^owner: .*/owner: human/' "$TMP/tasks/active/T-9592-fixture.md"
out=$(FAKE_HANDBACK_REFUSE=1 RUNME_TEST_HANDBACKS="T-9592" run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep_all "FAILED  T-9592 still not owner: agent"; then
    ok "handback: fw that claims success but changes nothing => FAILED, not OK"
else bad "handback refuse" "rc=$rc: $out"; fi
rm -f "$TMP/tasks/active/T-9592-fixture.md" "$TMP/tasks/active/T-9593-fixture.md"
hub_state 500 "/old/termlink (deleted)"
out=$(FAKE_HUB_RESTART_MODE=rotate run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep_all "hub.secret-CHANGED"; then
    ok "hub: secret rotated by the restart => FAILED naming it (PL-021 class)"
else bad "hub secret rotation must fail" "rc=$rc: $out"; fi
hub_state 600 "/old/termlink (deleted)"
out=$(FAKE_HUB_NEW_EXE="/old/termlink (deleted)" run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep_all "exe='/old/termlink (deleted)'"; then
    ok "hub: restarted onto the wrong binary => FAILED"
else bad "hub wrong exe must fail" "rc=$rc: $out"; fi
hub_state 700 "/old/termlink (deleted)"
out=$(FAKE_HUB_RESTART_MODE=fail run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep_all "FAILED  systemctl restart termlink-hub failed"; then
    ok "hub: systemctl restart failure => FAILED, exit 1"
else bad "hub restart failure" "rc=$rc: $out"; fi
hub_state 800 "/old/termlink (deleted)"
out=$(FAKE_HUB_ACTIVE=failed run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep_all "unit=failed"; then
    ok "hub: unit not active after restart => FAILED"
else bad "hub inactive must fail" "rc=$rc: $out"; fi
hub_state 100 "$H/hubbin"

# ---------------------------------------------------------------------------
# 15. T-3290 action 6 — remote fleet upgrade, all through fakes.
# ---------------------------------------------------------------------------
fleet_state 9.9.9
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "skip    fake-hub already serves 9.9.9" && [ ! -e "$F/deploys" ]; then
    ok "fleet: hub already at a current-code version => skip, no deploy"
else bad "fleet idempotent skip" "rc=$rc: $out"; fi
fleet_state 0.11.1
out=$(run --dry-run); rc=$?
if echo "$out" | grep_all "would stage musl 9.9.9 on fake-hub (10.0.0.1:9100, now 0.11.1)" && [ ! -e "$F/deploys" ] && [ ! -e "$F/remote-cmds" ]; then
    ok "fleet: --dry-run reports the upgrade and does NOT perform it"
else bad "fleet dry-run" "rc=$rc: $out"; fi
fleet_state 0.11.1
out=$(FAKE_UNIT=1 run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "OK      upgraded and verified: fake-hub serves 9.9.9" \
   && grep -q -- "--binary $F/musl --dst /tmp/termlink.new --probe" "$F/deploys" && ! grep -q -- "--swap-restart" "$F/deploys" \
   && grep -q "install -m 755 /tmp/termlink.new /usr/local/bin/termlink" "$F/remote-cmds" && grep -q "systemctl restart termlink-hub" "$F/remote-cmds"; then
    ok "fleet: systemd-supervised hub => stage+probe, install over the unit's binary, restart THROUGH systemd (G-070), no --swap-restart"
else bad "fleet systemd path" "rc=$rc: $out $(cat "$F/deploys" "$F/remote-cmds" 2>/dev/null)"; fi
fleet_state 0.11.1
out=$(FAKE_UNIT=0 run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "OK      upgraded and verified" && grep -q -- "--swap-restart" "$F/deploys" \
   && ! grep -q "systemctl restart" "$F/remote-cmds"; then
    ok "fleet: hub with no unit (watchdog-launched) => --swap-restart, never systemctl"
else bad "fleet watchdog path" "rc=$rc: $out $(cat "$F/deploys" 2>/dev/null)"; fi
fleet_state 0.11.1
out=$(FAKE_SWAP_CONFIRM=0 run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep_all "install/restart on fake-hub did not confirm: install: cannot create"; then
    ok "fleet: unconfirmed systemd install/restart => FAILED naming the remote error"
else bad "fleet unconfirmed swap" "rc=$rc: $out"; fi
fleet_state 0.11.1
cp "$TMP/tl-old" "$F/musl"; printf '#!/usr/bin/env bash\nexit 1\n' > "$F/musl"
out=$(run); rc=$?
cp "$TMP/tl-new" "$F/musl"
if [ "$rc" = "1" ] && echo "$out" | grep_all "musl binary $F/musl does not run — nothing shipped" && [ ! -e "$F/deploys" ]; then
    ok "fleet: musl binary that does not run => FAILED, nothing shipped"
else bad "fleet broken musl" "rc=$rc: $out"; fi
fleet_state 0.11.1
out=$(FAKE_DEPLOY_MODE=fail run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep_all "FAILED  staging on fake-hub failed" && [ ! -e "$F/remote-cmds" ]; then
    ok "fleet: staging/probe failure => FAILED, no swap attempted"
else bad "fleet staging failure" "rc=$rc: $out"; fi
fleet_state 0.11.1
out=$(FAKE_DEPLOY_MODE=auth run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep_all "status='auth-mismatch'"; then
    ok "fleet: hub up but auth broken after the swap (secret rotated) => FAILED"
else bad "fleet auth regression must fail" "rc=$rc: $out"; fi
fleet_state 0.11.1
out=$(FAKE_TOFU_RC=1 run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep_all "tofu verify failed"; then
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
if [ "$rc" = "0" ] && [ "$(calls)" = "1" ] && echo "$out" | grep_all "cached  host build is current"; then
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
if echo "$out" | grep_all "targets 2 zombie session(s) — listed before any is touched" && echo "$out" | grep_all "pid 501  z-one" \
   && echo "$out" | grep_all "would SIGTERM these 2" && [ ! -e "$Z/killed" ]; then
    ok "zombie: --dry-run lists every target and signals nothing"
else bad "zombie dry-run" "rc=$rc: $out"; fi
out=$(FAKE_ZOMBIES="501:z-one 502:z-two" run); rc=$?
lst=$(echo "$out" | grep -n "pid 502  z-two" | cut -d: -f1); okl=$(echo "$out" | grep -n "OK      reaped and verified: 2 session" | cut -d: -f1)
if [ "$rc" = "0" ] && [ -n "$lst" ] && [ -n "$okl" ] && [ "$lst" -lt "$okl" ] && [ "$(wc -l < "$Z/killed")" = "2" ]; then
    ok "zombie: targets listed before acting, both SIGTERMed and verified gone"
else bad "zombie reap" "rc=$rc: $out"; fi
if ! echo "$out" | grep_all "No such file or directory"; then
    ok "zombie: an exited session's re-check prints no shell error into the log (T-3301)"
else bad "zombie re-check stderr noise" "$(echo "$out" | grep -m2 'No such file')"; fi
zombie_reset; zombie_proc 601 z-real
mkdir -p "$Z/proc/602"; printf 'vim\0notes.txt\0' > "$Z/proc/602/cmdline"   # pid 602 was reused
out=$(FAKE_ZOMBIES="601:z-real 602:z-gone" run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "pid 602 is no longer register 'z-gone' — not signalled" \
   && ! grep -qx 602 "$Z/killed" && grep -qx 601 "$Z/killed" && [ -d "$Z/proc/602" ]; then
    ok "zombie: a reused pid (cmdline no longer that session) is skipped, never signalled"
else bad "zombie pid-reuse guard" "rc=$rc: $out $(cat "$Z/killed" 2>/dev/null)"; fi
zombie_reset; zombie_proc 701 z-stuck
out=$(FAKE_ZOMBIES="701:z-stuck" FAKE_STUCK=701 run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep_all "still running after SIGTERM (not escalated to SIGKILL): 701(z-stuck)" \
   && [ "$(wc -l < "$Z/killed")" = "1" ]; then
    ok "zombie: a session that ignores SIGTERM => FAILED naming it, signalled once, no SIGKILL"
else bad "zombie stuck" "rc=$rc: $out"; fi
zombie_reset
out=$(FAKE_ZOMBIES="" run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "skip    no zombie sessions" && [ ! -e "$Z/killed" ]; then
    ok "zombie: no zombies => skip (idempotent)"
else bad "zombie idempotent" "rc=$rc: $out"; fi
out=$(RUNME_ZOMBIE_DETECT=false run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep_all "zombie detector could not run — nothing reaped"; then
    ok "zombie: detector failure => FAILED, nothing reaped (fail-closed)"
else bad "zombie detector failure" "rc=$rc: $out"; fi

# ---------------------------------------------------------------------------
# 18. T-3299 — hub unit install, through the fake systemctl / proc.
# ---------------------------------------------------------------------------
hub_state 900 "$H/hubbin"; printf 'unit-v2\n' > "$H/unit.src"
out=$(run --dry-run); rc=$?
if echo "$out" | grep_all "would install $H/unit.src -> $H/unit.dest, daemon-reload, restart termlink-hub" && grep -q unit-v1 "$H/unit.dest" && [ ! -e "$H/restarts" ]; then
    ok "hub unit: --dry-run reports the install and changes nothing"
else bad "hub unit dry-run" "rc=$rc: $out"; fi
hub_state 910 "$H/hubbin"
out=$(FAKE_HUB_MOUNTINFO="36 25 0:1 / $H/rwpool rw,relatime - btrfs /dev/x rw" run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "OK      unit installed and verified: termlink-hub active (pid 911); $H/rwpool read-write" \
   && cmp -s "$H/unit.src" "$H/unit.dest"; then
    ok "hub unit: differing unit installed, hub restarted, pool verified read-write in the hub's namespace"
else bad "hub unit install" "rc=$rc: $out"; fi
hub_state 920 "$H/hubbin"; printf 'unit-v3\n' > "$H/unit.src"
out=$(FAKE_HUB_MOUNTINFO="36 25 0:1 / $H/rwpool ro,relatime - btrfs /dev/x ro" run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep_all "rwpool-not-writable-in-hub-namespace"; then
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
if [ "$rc" = "2" ] && echo "$out" | grep_all "FIXTURE run reached the REAL default for RUNME_HUB_UNIT_DEST" && [ "$before" = "$after" ]; then
    ok "fixture mode: a missing seam aborts the run (exit 2) and the real unit file is untouched"
else bad "fixture guard" "rc=$rc: $(echo "$out" | tail -3)"; fi

# ---------------------------------------------------------------------------
# 20. T-3303 — identity env merged into Claude local settings, scratch file only.
# ---------------------------------------------------------------------------
CL="$TMP/claude-local.json"
printf '{"permissions":{"allow":["Bash(ls:*)"]},"enableAllProjectMcpServers":true,"env":{"KEEP":"1"}}\n' > "$CL"
out=$(run --dry-run); rc=$?
if echo "$out" | grep_all "would add env.TERMLINK_AGENT_ID=claude-termlink" && ! grep -q TERMLINK_AGENT_ID "$CL"; then
    ok "identity env: --dry-run reports and changes nothing"
else bad "identity env dry-run" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "OK      set and verified: $CL" && python3 -c "
import json,sys; d=json.load(open('$CL'))
assert d['env']=={'KEEP':'1','TERMLINK_AGENT_ID':'claude-termlink'}, d
assert d['permissions']=={'allow':['Bash(ls:*)']} and d['enableAllProjectMcpServers'] is True"; then
    ok "identity env: key added, existing env and all other keys preserved"
else bad "identity env merge" "rc=$rc: $out $(cat "$CL")"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "skip    $CL already sets TERMLINK_AGENT_ID=claude-termlink"; then
    ok "identity env: second run skips (idempotent)"
else bad "identity env idempotent" "rc=$rc: $out"; fi

# ---------------------------------------------------------------------------
# 21. T-2988 — bound health:ring20-fedprobe (fake termlink, scratch state only).
# ---------------------------------------------------------------------------
# State at the first run's end (messages:100, 104 records) -> latest-per-cv-key.
echo "messages 100 104" > "$FAKE_FEDPROBE_STATE"
out=$(run --dry-run); rc=$?
if echo "$out" | grep_all "would set health:ring20-fedprobe retention messages -> latest-per-cv-key" && grep -q "^messages 100 104$" "$FAKE_FEDPROBE_STATE"; then
    ok "fedprobe: --dry-run reports and changes nothing"
else bad "fedprobe dry-run" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "OK      health:ring20-fedprobe now keeps the latest record per cv_key; swept to 1 records (verified)" && grep -q "^latest_per_cv_key 0 1$" "$FAKE_FEDPROBE_STATE"; then
    ok "fedprobe: latest-per-cv-key set, swept to one record, verified"
else bad "fedprobe bound" "rc=$rc: $out $(cat "$FAKE_FEDPROBE_STATE")"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "skip    health:ring20-fedprobe already keeps the latest record per cv_key"; then
    ok "fedprobe: second run skips (idempotent)"
else bad "fedprobe idempotent" "rc=$rc: $out"; fi
echo "forever 0 2777" > "$FAKE_FEDPROBE_STATE"
out=$(FAKE_FEDPROBE_NOSWEEP=1 run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep_all "FAILED  health:ring20-fedprobe: after set-retention + sweep it reads 'latest_per_cv_key 0 2777'"; then
    ok "fedprobe: a sweep that does not take effect is FAILED, not OK"
else bad "fedprobe no-sweep" "rc=$rc: $out"; fi
out=$(FAKE_FEDPROBE_MISSING=1 run); rc=$?
if echo "$out" | grep_all "skip    health:ring20-fedprobe not found on the local hub"; then
    ok "fedprobe: missing topic is a skip"
else bad "fedprobe missing" "rc=$rc: $out"; fi
echo "latest_per_cv_key 0 1" > "$FAKE_FEDPROBE_STATE"

# ---------------------------------------------------------------------------
# 22. T-3325 — restart stale notify-sidecars (fake detector/kill/supervisor only).
# ---------------------------------------------------------------------------
STALE2='{"stale_count":2,"current_count":0,"stale":[{"pid":111,"agent_id":"a1"},{"pid":222,"agent_id":"a2"}]}'
echo "$STALE2" > "$FAKE_SIDECAR_STATE"; : > "$FAKE_SIDECAR_KILLED"
out=$(run --dry-run); rc=$?
if echo "$out" | grep_all "would restart 2 stale sidecar(s): a1 a2" && [ ! -s "$FAKE_SIDECAR_KILLED" ]; then
    ok "sidecars: --dry-run reports and signals nothing"
else bad "sidecars dry-run" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "OK      restarted 2 sidecar(s) onto current code: a1 a2 (verified: 0 stale, 2 current)" \
   && [ "$(tr '\n' ' ' < "$FAKE_SIDECAR_KILLED")" = "111 222 " ]; then
    ok "sidecars: stale pids signalled, supervisor restarts them, verified"
else bad "sidecars restart" "rc=$rc: $out killed=$(cat "$FAKE_SIDECAR_KILLED")"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "skip    all 2 notify-sidecar(s) already run the current code"; then
    ok "sidecars: second run skips (idempotent)"
else bad "sidecars idempotent" "rc=$rc: $out"; fi
echo "$STALE2" > "$FAKE_SIDECAR_STATE"
out=$(FAKE_SIDECAR_NOFIX=1 run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep_all "FAILED  after restart: stale=2"; then
    ok "sidecars: a restart that leaves them stale is FAILED, not OK"
else bad "sidecars no-fix" "rc=$rc: $out"; fi
echo "not json" > "$FAKE_SIDECAR_STATE"
out=$(run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep_all "FAILED  check-stale-sidecar-code gave no readable result"; then
    ok "sidecars: unreadable detector output is FAILED (fail-closed), never a skip"
else bad "sidecars unreadable" "rc=$rc: $out"; fi
echo '{"stale_count":0,"current_count":3,"stale":[]}' > "$FAKE_SIDECAR_STATE"

# ---------------------------------------------------------------------------
# 23. T-3336 — schedule the hourly vector reindex (fake fw cron install, temp file only).
# ---------------------------------------------------------------------------
: > "$RUNME_AUDIT_CRON_FILE"
out=$(run --dry-run); rc=$?
if echo "$out" | grep_all "would run: fw cron install" && [ ! -s "$RUNME_AUDIT_CRON_FILE" ]; then
    ok "reindex: --dry-run reports and installs nothing"
else bad "reindex dry-run" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "OK      hourly vector reindex installed and verified" && grep -q 'index reindex' "$RUNME_AUDIT_CRON_FILE"; then
    ok "reindex: installed and verified on disk"
else bad "reindex install" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "skip    .* already schedules the hourly vector reindex"; then
    ok "reindex: second run skips (idempotent)"
else bad "reindex idempotent" "rc=$rc: $out"; fi
: > "$RUNME_AUDIT_CRON_FILE"
out=$(FAKE_CRON_NOINSTALL=1 run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep_all "FAILED  fw cron install ran but"; then
    ok "reindex: an install that leaves no line is FAILED, not OK"
else bad "reindex no-install" "rc=$rc: $out"; fi
cron_both

# 25b. T-3554 — deploy the regenerated audit crontab when source and installed differ.
SRC_SAVED="$RUNME_AUDIT_CRON_SRC"
export RUNME_AUDIT_CRON_SRC="$TMP/agentic-audit-src.crontab"
{ cat "$RUNME_AUDIT_CRON_FILE"; echo '# regenerated by fw upgrade'; } > "$RUNME_AUDIT_CRON_SRC"
out=$(run --dry-run); rc=$?
if echo "$out" | grep_all "would run: fw cron install (deploy the regenerated registry crontab" && ! cmp -s "$RUNME_AUDIT_CRON_SRC" "$RUNME_AUDIT_CRON_FILE"; then
    ok "audit crontab: --dry-run reports and installs nothing"
else bad "audit crontab dry-run" "rc=$rc: $out"; fi
out=$(FAKE_CRON_NOINSTALL=1 run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep_all "FAILED  fw cron install ran but .* still differs"; then
    ok "audit crontab: an install that leaves it different is FAILED, not OK"
else bad "audit crontab no-install" "rc=$rc: $out"; fi
out=$(FAKE_CRON_COPY_SRC=1 run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "OK      registry crontab deployed and verified identical" && cmp -s "$RUNME_AUDIT_CRON_SRC" "$RUNME_AUDIT_CRON_FILE"; then
    ok "audit crontab: deployed and verified identical on disk"
else bad "audit crontab install" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "skip    .* already equals the registry source"; then
    ok "audit crontab: second run skips (idempotent)"
else bad "audit crontab idempotent" "rc=$rc: $out"; fi
export RUNME_AUDIT_CRON_SRC="$SRC_SAVED"
cron_both

# 25c. T-3568 — restart a stale Watchtower onto current web/ code; skip a current one.
: > "$TMP_WT_STALE"
out=$(run --dry-run); rc=$?
if echo "$out" | grep_all "would run: fw watchtower restart" && [ -e "$TMP_WT_STALE" ]; then
    ok "watchtower: --dry-run reports and restarts nothing"
else bad "watchtower dry-run" "rc=$rc: $out"; fi
out=$(FAKE_WT_RESTART_NOOP=1 run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep_all "FAILED  fw watchtower restart ran but Watchtower is still stale"; then
    ok "watchtower: a restart that leaves it stale is FAILED, not OK"
else bad "watchtower restart-noop" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "OK      Watchtower restarted onto the current web/ code" && [ ! -e "$TMP_WT_STALE" ]; then
    ok "watchtower: stale -> restarted and verified current"
else bad "watchtower restart" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "skip    Watchtower already serves the current web/ code"; then
    ok "watchtower: second run skips (idempotent)"
else bad "watchtower idempotent" "rc=$rc: $out"; fi

# 25. T-3370 — install the cron jobs AEF 1.8.3 added (sidecar sweep), temp file only.
echo '20 * * * * root cd "/x" && "/x/fw" index reindex' > "$RUNME_AUDIT_CRON_FILE"
out=$(run --dry-run); rc=$?
if echo "$out" | grep_all "would run: fw cron install (adds the 5-minute 'fw sidecar sweep'" && ! grep -q 'sidecar sweep' "$RUNME_AUDIT_CRON_FILE"; then
    ok "sidecar sweep: --dry-run reports and installs nothing"
else bad "sweep dry-run" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "OK      sidecar ack-ledger sweep installed and verified" && grep -q 'sidecar sweep' "$RUNME_AUDIT_CRON_FILE"; then
    ok "sidecar sweep: installed and verified on disk"
else bad "sweep install" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "skip    .* already schedules the sidecar ack-ledger sweep"; then
    ok "sidecar sweep: second run skips (idempotent)"
else bad "sweep idempotent" "rc=$rc: $out"; fi
echo '20 * * * * root cd "/x" && "/x/fw" index reindex' > "$RUNME_AUDIT_CRON_FILE"
out=$(FAKE_CRON_NOINSTALL=1 run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep_all "FAILED  fw cron install ran but .* has no 'sidecar sweep' line"; then
    ok "sidecar sweep: an install that leaves no line is FAILED, not OK"
else bad "sweep no-install" "rc=$rc: $out"; fi
cron_both

# 24. T-3343 — stop the stray hub (fake termlink, fake liveness, fake /proc only).
out=$(run); rc=$?
if echo "$out" | grep_all "skip    no live hub recorded in $TMP/stray"; then
    ok "stray hub: no pidfile in the stray dir -> skip"
else bad "stray none" "rc=$rc: $out"; fi
echo 4242 > "$TMP/stray/hub.pid"; echo alive > "$FAKE_STRAY_STATE"
out=$(run --dry-run); rc=$?
if echo "$out" | grep_all "would stop stray hub pid 4242" && grep -q alive "$FAKE_STRAY_STATE"; then
    ok "stray hub: --dry-run reports and stops nothing"
else bad "stray dry-run" "rc=$rc: $out"; fi
out=$(FAKE_STRAY_NOGUARD=1 run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep_all "FAILED  installed termlink lacks the T-3340 guard" && grep -q alive "$FAKE_STRAY_STATE"; then
    ok "stray hub: refuses while an installed termlink lacks the guard"
else bad "stray noguard" "rc=$rc: $out"; fi
out=$(RUNME_STRAY_RESCUE_REPORT="$TMP/no-such-report.md" run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep_all "FAILED  rescue report" && grep -q alive "$FAKE_STRAY_STATE"; then
    ok "stray hub: refuses before the mail is rescued"
else bad "stray noreport" "rc=$rc: $out"; fi
printf 'bash\0sleep\0' > "$TMP/proc/4242/cmdline"
out=$(run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep_all "is not a termlink hub process" && grep -q alive "$FAKE_STRAY_STATE"; then
    ok "stray hub: a reused pid that is not a termlink hub is never stopped"
else bad "stray foreign pid" "rc=$rc: $out"; fi
printf 'termlink\0hub\0start\0' > "$TMP/proc/4242/cmdline"
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "OK      stray hub pid 4242 stopped; a client without TERMLINK_RUNTIME_DIR now resolves the canonical hub pid 906"; then
    ok "stray hub: stopped and the canonical hub verified"
else bad "stray stop" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "skip    no live hub recorded"; then
    ok "stray hub: second run skips (idempotent)"
else bad "stray idempotent" "rc=$rc: $out"; fi
echo alive > "$FAKE_STRAY_STATE"
out=$(FAKE_STRAY_NOSTOP=1 run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep_all "still alive 15 s after 'hub stop'"; then
    ok "stray hub: a hub that will not stop is FAILED, never killed hard"
else bad "stray nostop" "rc=$rc: $out"; fi
rm -f "$TMP/stray/hub.pid"; echo alive > "$FAKE_STRAY_STATE"

# 26. T-3351 — record the operator's own Human tick (T-3349), temp ledger only.
: > "$RUNME_HUMAN_TICKS_LEDGER"
out=$(run --dry-run); rc=$?
if echo "$out" | grep_all "would record that you ticked T-3349 Human AC#1" && [ ! -s "$RUNME_HUMAN_TICKS_LEDGER" ]; then
    ok "operator tick: --dry-run reports and records nothing"
else bad "tick dry-run" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "OK      recorded that you ticked T-3349 Human AC#1 (verified" \
   && grep -q '"via": "operator-ack"' "$RUNME_HUMAN_TICKS_LEDGER"; then
    ok "operator tick: recorded and verified in the ledger"
else bad "tick record" "rc=$rc: $out"; fi
# T-3393: every listed tick is recorded, one row per task, not just the first.
if echo "$out" | grep_all "OK      recorded that you ticked T-3351 Human AC#1" \
   && echo "$out" | grep_all "OK      recorded that you ticked T-3388 Human AC#1" \
   && echo "$out" | grep_all "OK      recorded that you ticked T-3291 Human AC#1" \
   && echo "$out" | grep_all "OK      recorded that you ticked T-3302 Human AC#1" \
   && echo "$out" | grep_all "OK      recorded that you ticked T-3006 Human AC#1" \
   && echo "$out" | grep_all "OK      recorded that you ticked T-3200 Human AC#1" \
   && [ "$(grep -c '"via": "operator-ack"' "$RUNME_HUMAN_TICKS_LEDGER")" = "7" ]; then
    ok "operator tick: all seven listed ticks recorded, one ledger row each"
else bad "tick record all" "rows=$(grep -c operator-ack "$RUNME_HUMAN_TICKS_LEDGER"): $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "skip    T-3349 Human AC#1: your tick is already recorded"; then
    ok "operator tick: second run skips (idempotent)"
else bad "tick idempotent" "rc=$rc: $out"; fi
: > "$RUNME_HUMAN_TICKS_LEDGER"
out=$(FAKE_TICK_NOWRITE=1 run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep_all "FAILED  human_ac_ticks ack T-3349 --ac 1 ran but"; then
    ok "operator tick: an ack that records nothing is FAILED, not OK"
else bad "tick nowrite" "rc=$rc: $out"; fi
ticks_recorded

# 27. T-3416 — current Claude Code for testdev (fake binary and installer only).
rm -f "$RUNME_TESTDEV_CLAUDE"
out=$(run --dry-run); rc=$?
if echo "$out" | grep_all "would install Claude Code for testdev" && [ ! -e "$RUNME_TESTDEV_CLAUDE" ]; then
    ok "testdev claude: --dry-run reports and installs nothing"
else bad "testdev dry-run" "rc=$rc: $out"; fi
fake_claude 2.1.27
out=$(FAKE_CLAUDE_NEW=2.1.300 run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "OK      testdev now has Claude Code 2.1.300" && "$RUNME_TESTDEV_CLAUDE" --version | grep_all 2.1.300; then
    ok "testdev claude: an old version is replaced and verified by running it"
else bad "testdev install" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "skip    testdev already has Claude Code 2.1.300"; then
    ok "testdev claude: second run skips (idempotent)"
else bad "testdev idempotent" "rc=$rc: $out"; fi
fake_claude 2.1.27
out=$(FAKE_CLAUDE_NEW= run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep_all "FAILED  the installer ran but .* reports '2.1.27', need >= 2.1.296"; then
    ok "testdev claude: an installer that leaves the old version is FAILED, not OK"
else bad "testdev old stays" "rc=$rc: $out"; fi
rm -f "$RUNME_TESTDEV_CLAUDE"
out=$(FAKE_CLAUDE_NEW= run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep_all "FAILED  the installer ran but .* reports 'nothing'"; then
    ok "testdev claude: an installer that leaves no binary is FAILED, not OK"
else bad "testdev none" "rc=$rc: $out"; fi
fake_claude 2.1.296

# 28. T-3421 — claude-fw on PATH becomes the router (fake files only).
cfw_old "$TMP/cfw-a"; rm -f "$TMP/cfw-b"; rm -rf "$RUNME_CLAUDE_FW_BACKUP_DIR"
out=$(RUNME_CLAUDE_FW_TARGETS="$TMP/cfw-a $TMP/cfw-b" run --dry-run); rc=$?
if echo "$out" | grep_all "would replace $TMP/cfw-a" && grep -q "old fixed copy" "$TMP/cfw-a" && [ ! -e "$TMP/cfw-b" ]; then
    ok "claude-fw router: --dry-run reports and changes nothing"
else bad "cfw dry-run" "rc=$rc: $out"; fi
out=$(RUNME_CLAUDE_FW_TARGETS="$TMP/cfw-a $TMP/cfw-b" run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "OK      $TMP/cfw-a is now the claude-fw router" && cmp -s "$TMP/cfw-a" "$RUNME_CLAUDE_FW_ROUTER" \
   && ls "$RUNME_CLAUDE_FW_BACKUP_DIR" | grep_all cfw-a && grep -q "old fixed copy" "$RUNME_CLAUDE_FW_BACKUP_DIR"/*cfw-a*; then
    ok "claude-fw router: a fixed copy is replaced, verified on disk, and backed up"
else bad "cfw replace" "rc=$rc: $out"; fi
if echo "$out" | grep_all "skip    $TMP/cfw-b does not exist here" && [ ! -e "$TMP/cfw-b" ]; then
    ok "claude-fw router: an absent path is left alone, never created"
else bad "cfw absent" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "skip    $TMP/cfw-a is already the claude-fw router"; then
    ok "claude-fw router: second run skips (idempotent)"
else bad "cfw idempotent" "rc=$rc: $out"; fi
cfw_old "$TMP/cfw-a"
out=$(RUNME_CLAUDE_FW_ROUTER="$TMP/no-such-router" run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep_all "FAILED  the router .* is missing" && grep -q "old fixed copy" "$TMP/cfw-a"; then
    ok "claude-fw router: a missing router is FAILED and nothing is touched"
else bad "cfw no router" "rc=$rc: $out"; fi
cfw_router "$TMP/cfw-a"

# T-3435 (P3): sidecar supervision install — systemd must RESOLVE the template,
# a host without systemd skips the unit and says cron supervises.
rm -f "$SCU/units/termlink-sidecar@.service"; touch "$SCU/broken"
out=$(run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep_all "FAILED  .*termlink-sidecar@.service installed but systemd does not resolve"; then
    ok "sidecar unit: a template systemd does not resolve is FAILED, not OK"
else bad "sidecar unit unresolved" "rc=$rc: $out"; fi
rm -f "$SCU/broken" "$SCU/units/termlink-sidecar@.service"
printf '#!/usr/bin/env bash\nexit 1\n' > "$SCU/systemctl-fail"; chmod +x "$SCU/systemctl-fail"
out=$(RUNME_SIDECAR_SYSTEMCTL="$SCU/systemctl-fail" run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep_all "FAILED  installed .* but daemon-reload failed"; then
    ok "sidecar unit: a failed daemon-reload is FAILED"
else bad "sidecar unit daemon-reload" "rc=$rc: $out"; fi
rm -f "$SCU/units/termlink-sidecar@.service"
out=$(RUNME_SIDECAR_SYSTEMD_RUN="$SCU/no-systemd" run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "no systemd on this host: the cron backstop is the sidecar supervisor" \
   && [ ! -e "$SCU/units/termlink-sidecar@.service" ]; then
    ok "sidecar unit: a host without systemd skips the unit and names cron as the supervisor"
else bad "sidecar no-systemd" "rc=$rc: $out"; fi
run >/dev/null 2>&1

# T-3438 (P7): W1 — readiness every run, switch-on only on a recorded go.
: > "$SCU/w1-calls"; out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "W1 is ready; waiting for the operator's go" && grep -qx -- "--check" "$SCU/w1-calls" && ! grep -q -- "--apply" "$SCU/w1-calls"; then
    ok "W1: without a recorded go only the read-only check runs"
else bad "W1 default" "rc=$rc: $out"; fi
: > "$SCU/w1-calls"; out=$(RUNME_W1_GO="2026-10-13|fixture" run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "OK      W1 pair provisioned and running" && grep -qx -- "--apply" "$SCU/w1-calls"; then
    ok "W1: with a recorded go it provisions and reports OK"
else bad "W1 go" "rc=$rc: $out"; fi
out=$(RUNME_W1_GO="2026-10-13|fixture" FAKE_W1_APPLY_RC=1 run); rc=$?
if [ "$rc" != "0" ] && echo "$out" | grep_all "FAILED  W1 provisioning"; then
    ok "W1: a failed provisioning is FAILED, not OK"
else bad "W1 apply fail" "rc=$rc: $out"; fi

# T-3438: the kill switch runs alone, needs a reason, reads each flip back.
: > "$SCU/w1-calls"; out=$(run --sidecar-delivery off --why "fixture drill"); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "OK      delivery off on every listed sidecar" \
   && grep -q "sidecar admin delivery off --why fixture drill --all --list $SCU/admin.list" "$SCU/w1-calls" \
   && ! echo "$out" | grep_all "1. Cron installs"; then
    ok "kill switch: flips every listed sidecar and runs no other action"
else bad "kill switch" "rc=$rc: $out"; fi
out=$(run --sidecar-delivery off); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep_all -- "--why"; then ok "kill switch: refuses without a reason"
else bad "kill switch no why" "rc=$rc: $out"; fi
out=$(FAKE_KS_RC=1 run --sidecar-delivery off --why x); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep_all "FAILED  at least one sidecar did not confirm"; then ok "kill switch: an unconfirmed flip is FAILED"
else bad "kill switch unconfirmed" "rc=$rc: $out"; fi
out=$(RUNME_SIDECAR_ADMIN_LIST="$SCU/none.list" run --sidecar-delivery on --why x); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep_all "no sidecars listed"; then ok "kill switch: an empty list switches nothing and says so"
else bad "kill switch empty list" "rc=$rc: $out"; fi


# ---------------------------------------------------------------------------
# 23. T-3443 — cap leftover forever project inboxes (fake termlink, scratch state only).
# ---------------------------------------------------------------------------
echo "forever 0 1042 1041 969" > "$FAKE_INBOX_STATE"; echo 1040 > "$TMP/inbox.marker"
out=$(run --dry-run); rc=$?
if echo "$out" | grep_all "would set inbox:t/010 retention forever -> messages 1000 and sweep offsets <= 41" && grep -q "^forever 0 1042 1041 969$" "$FAKE_INBOX_STATE"; then
    ok "inbox: --dry-run reports the cut and changes nothing"
else bad "inbox dry-run" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "OK      inbox:t/010 now keeps the newest 1000; swept to 1000 records, every reader was past offset 41 (verified)" && grep -q "^messages 1000 1000 " "$FAKE_INBOX_STATE"; then
    ok "inbox: readers past the cut -> capped, swept, verified"
else bad "inbox cap" "rc=$rc: $out $(cat "$FAKE_INBOX_STATE")"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "skip    inbox:t/010 already keeps the newest 1000"; then ok "inbox: second run skips (idempotent)"
else bad "inbox idempotent" "rc=$rc: $out"; fi
echo "forever 0 1500 1499 300" > "$FAKE_INBOX_STATE"
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "HELD    inbox:t/010: refused — a reader is behind the cut point 499 (hub receipt at 300)" && echo "$out" | grep_all " 1 held, 0 failed" && grep -q "^forever 0 1500 1499 300$" "$FAKE_INBOX_STATE"; then
    ok "inbox: a hub reader behind the cut is HELD (rc 0, T-3606), nothing changed"
else bad "inbox receipt behind" "rc=$rc: $out"; fi
echo "forever 0 1500 1499 1499" > "$FAKE_INBOX_STATE"; echo 200 > "$TMP/inbox.marker"
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "HELD .*alert marker at 200" && grep -q "^forever 0 1500 " "$FAKE_INBOX_STATE"; then
    ok "inbox: our own alert marker behind the cut is HELD"
else bad "inbox marker behind" "rc=$rc: $out"; fi
echo "forever 0 1500 1499 none" > "$FAKE_INBOX_STATE"
out=$(RUNME_INBOX_TOPICS="inbox:t/aef|" run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "HELD    inbox:t/aef: refused — no reader position is known" && grep -q "^forever 0 1500 " "$FAKE_INBOX_STATE"; then
    ok "inbox: no known reader position is HELD, never assumed read"
else bad "inbox no readers" "rc=$rc: $out"; fi
echo "forever 0 1500 1499 1499" > "$FAKE_INBOX_STATE"; printf 'x' > "$TMP/inbox.marker"
out=$(run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep_all "FAILED  inbox:t/010: alert marker unreadable" && grep -q "^forever 0 1500 " "$FAKE_INBOX_STATE"; then
    ok "inbox: an unreadable alert marker is FAILED (could not look), not HELD"
else bad "inbox marker unreadable" "rc=$rc: $out"; fi
echo "forever 0 1500 1499 1499" > "$FAKE_INBOX_STATE"; echo 1499 > "$TMP/inbox.marker"
out=$(FAKE_INBOX_NOSWEEP=1 run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep_all "FAILED  inbox:t/010: after set-retention + sweep it reads 'messages 1000 1500'"; then
    ok "inbox: a sweep that does not take effect is FAILED, not OK"
else bad "inbox no-sweep" "rc=$rc: $out"; fi
echo "forever 0 182 181 none" > "$FAKE_INBOX_STATE"
out=$(RUNME_INBOX_TOPICS="inbox:t/small|" run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "OK      inbox:t/small now keeps the newest 1000; 182 records, nothing deleted (verified)"; then
    ok "inbox: under the cap with no known reader -> policy set, nothing deleted, said so"
else bad "inbox small" "rc=$rc: $out"; fi
echo "messages 1000 1000 4999 4999" > "$FAKE_INBOX_STATE"
out=$(FAKE_INBOX_MISSING=1 run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep_all "FAILED  inbox:t/010: cannot read it on the local hub"; then ok "inbox: an unreadable topic is FAILED, never a skip"
else bad "inbox missing" "rc=$rc: $out"; fi
echo "messages 1000 1000 4999 4999" > "$FAKE_INBOX_STATE"; echo 4999 > "$TMP/inbox.marker"


# ---------------------------------------------------------------------------
# 24. T-3426 — the W1 kill-switch drill runs once and records a pass (fake drill only).
# ---------------------------------------------------------------------------
rm -f "$RUNME_W1_DRILL_DONE" "$FAKE_DRILL_RAN"
out=$(run --dry-run); rc=$?
if echo "$out" | grep_all "would run the W1 kill-switch drill" && [ ! -f "$FAKE_DRILL_RAN" ]; then ok "drill: --dry-run runs nothing"
else bad "drill dry-run" "rc=$rc: $out"; fi
out=$(FAKE_DRILL_RC=1 run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep_all "FAILED  W1 kill-switch drill rc=1" && [ ! -s "$RUNME_W1_DRILL_DONE" ]; then ok "drill: a failed drill is FAILED and records no pass"
else bad "drill fail" "rc=$rc: $out"; fi
out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "OK      W1 kill-switch drill passed" && grep -q "8 passed, 0 failed" "$RUNME_W1_DRILL_DONE"; then ok "drill: a pass is recorded"
else bad "drill pass" "rc=$rc: $out"; fi
: > "$FAKE_DRILL_RAN"; out=$(run); rc=$?
if [ "$rc" = "0" ] && echo "$out" | grep_all "skip    the W1 kill-switch drill already passed" && [ ! -s "$FAKE_DRILL_RAN" ]; then ok "drill: once passed it never runs again (idempotent)"
else bad "drill idempotent" "rc=$rc: $out"; fi

# ---------------------------------------------------------------------------
# 25. T-3440 5c — attach driver scoring specs (scratch arc + fake attach only).
# ---------------------------------------------------------------------------
arcs_specced 0
out=$(run --dry-run); rc=$?
if echo "$out" | grep_all "would attach .*AS_FAILURE_OBSERVABILITY" && ! grep -q "scoring:" "$RUNME_ARCS_DIR/arc-substrate-fitness.yaml"; then ok "specs: --dry-run attaches nothing"
else bad "specs dry-run" "rc=$rc: $out"; fi
out=$(FAKE_ATTACH_LIE=1 run); rc=$?
if [ "$rc" = "1" ] && echo "$out" | grep_all "attach reported success but no valid scoring block is stored"; then ok "specs: a claimed attach that stored nothing is FAILED"
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
if echo "$s21" | grep_all "skip    fake-hub already sweeps every 3600s" && ! grep -q "t3311" "$F/remote-cmds" 2>/dev/null; then
    ok "sweeper: already on => skip, nothing sent to the hub"
else bad "sweeper skip" "$s21"; fi
sweep_case 0
out=$(run --dry-run); s21=$(echo "$out" | sed -n '/^.*21\. Retention sweeper/,/^.*Verification/p')
if echo "$out" | grep_all "21. Retention sweeper on for the .122 hub — THE .122 HUB RESTARTS" \
   && echo "$s21" | grep_all "would add TERMLINK_SWEEP_INTERVAL_SECS=3600 .* and RESTART that hub" \
   && ! grep -q "t3311" "$F/remote-cmds" 2>/dev/null; then
    ok "sweeper: heading says the hub restarts; --dry-run sends nothing"
else bad "sweeper dry-run" "$s21"; fi
sweep_case 0
out=$(run); s21=$(echo "$out" | sed -n '/^.*21\. Retention sweeper/,/^.*Verification/p')
if echo "$s21" | grep_all "OK      fake-hub restarted and sweeps every 3600s" \
   && grep -q "termlink-hub.service.d/t3311-sweep.conf" "$F/remote-cmds" && grep -q "daemon-reload" "$F/remote-cmds" \
   && grep -q "systemctl restart termlink-hub" "$F/remote-cmds"; then
    ok "sweeper: drop-in written, daemon-reload, restart through the unit, verified 3600"
else bad "sweeper apply" "$s21 // $(cat "$F/remote-cmds" 2>/dev/null)"; fi
sweep_case 0
out=$(FAKE_UNIT=0 run); rc=$?; s21=$(echo "$out" | sed -n '/^.*21\. Retention sweeper/,/^.*Verification/p')
if [ "$rc" = "1" ] && echo "$s21" | grep_all "FAILED  fake-hub has no termlink-hub.service — nothing changed"; then
    ok "sweeper: no unit => FAILED, nothing changed"
else bad "sweeper no unit" "rc=$rc: $s21"; fi
sweep_case 0
out=$(FAKE_SWEEP_APPLIES=0 run); rc=$?; s21=$(echo "$out" | sed -n '/^.*21\. Retention sweeper/,/^.*Verification/p')
if [ "$rc" = "1" ] && echo "$s21" | grep_all "FAILED  fake-hub after restart: interval='0' (want 3600)"; then
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
if echo "$x" | grep_all "skip    pack already in .* verified; the /var/tmp drop is gone"; then ok "pack: already imported, drop gone => skip"
else bad "pack skip" "$x"; fi
pack_reset; cp "$TMP/pack-good.tgz" "$RUNME_PACK_SRC"
out=$(run --dry-run); x=$(s22 "$out")
if echo "$x" | grep_all "\[DRY\]   would unpack" && [ -e "$RUNME_PACK_SRC" ] && [ ! -d "$RUNME_PACK_DEST" ]; then ok "pack: --dry-run changes nothing"
else bad "pack dry-run" "$x"; fi
out=$(run); x=$(s22 "$out")
if echo "$x" | grep_all "OK      pack unpacked into" && [ ! -e "$RUNME_PACK_SRC" ] && [ -f "$RUNME_PACK_DEST/pack/plan.md" ]; then
    ok "pack: unpacked, verified, drop deleted"
else bad "pack apply" "$x"; fi
if [ -z "$(find "$RUNME_PACK_DEST" -perm /077 -print -quit)" ]; then ok "pack: unpacked tree is owner-only on disk (tar does not leave it 755)"
else bad "pack mode" "$(find "$RUNME_PACK_DEST" -perm /077 | head -3)"; fi
chmod -R go+rX "$RUNME_PACK_DEST"
out=$(run); rc=$?; x=$(s22 "$out")
if [ "$rc" = "0" ] && echo "$x" | grep_all "OK      .* was group/other-readable; now owner-only" && [ -z "$(find "$RUNME_PACK_DEST" -perm /077 -print -quit)" ]; then
    ok "pack: imported but readable => tightened to owner-only"
else bad "pack tighten" "rc=$rc: $x"; fi
pack_reset; printf 'not the pack' > "$RUNME_PACK_SRC"
out=$(run); rc=$?; x=$(s22 "$out")
if [ "$rc" = "1" ] && echo "$x" | grep_all "FAILED  .* sha256 .* does not match" && [ -e "$RUNME_PACK_SRC" ] && [ ! -d "$RUNME_PACK_DEST" ]; then
    ok "pack: wrong sha256 => FAILED, nothing copied, drop kept"
else bad "pack wrong sha" "rc=$rc: $x"; fi
pack_reset; mkdir -p "$TMP/packbad/pack"; cp "$TMP/packsrc/pack/"* "$TMP/packbad/pack/"; printf 'tampered\n' > "$TMP/packbad/pack/plan.md"
tar -czf "$RUNME_PACK_SRC" -C "$TMP/packbad" pack
out=$(RUNME_PACK_SHA="$(sha256sum "$RUNME_PACK_SRC" | cut -d' ' -f1)" run); rc=$?; x=$(s22 "$out")
if [ "$rc" = "1" ] && echo "$x" | grep_all "FAILED  unpacked files do not match the SHA256SUMS inside" && [ -e "$RUNME_PACK_SRC" ]; then
    ok "pack: contents not matching SHA256SUMS => FAILED, drop kept"
else bad "pack tampered" "rc=$rc: $x"; fi
pack_reset; cp "$TMP/pack-good.tgz" "$RUNME_PACK_SRC"; mkdir -p "$TMP/plainrepo"; git -C "$TMP/plainrepo" init -q 2>/dev/null
out=$(RUNME_PACK_GITROOT="$TMP/plainrepo" RUNME_PACK_DEST="$TMP/plainrepo/pack" run); rc=$?; x=$(s22 "$out")
if [ "$rc" = "1" ] && echo "$x" | grep_all "FAILED  .* is not gitignored" && [ ! -d "$TMP/plainrepo/pack" ] && [ -e "$RUNME_PACK_SRC" ]; then
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
if echo "$x" | grep_all "would add the background-class line: .*heartbeat.crontab" && ! grep -q TERMLINK_WORKER_CLASS "$TMP/cron/termlink-heartbeat"; then
    ok "1b: --dry-run names the env-only drift and writes nothing"
else bad "1b dry-run" "$x"; fi
out=$(run); rc=$?; x=$(s1b "$out")
if [ "$rc" = "0" ] && echo "$x" | grep_all "OK      1 installed crontab(s) now carry TERMLINK_WORKER_CLASS=background" \
   && cmp -s "$REPO_ROOT/.context/cron/heartbeat.crontab" "$TMP/cron/termlink-heartbeat"; then
    ok "1b: an installed copy differing ONLY by the class line is reinstalled and verified identical"
else bad "1b resync" "rc=$rc: $x"; fi
if echo "$x" | grep_all "left alone .*termlink-presence-sweep" && cmp -s "$TMP/presence.before" "$TMP/cron/termlink-presence-sweep"; then
    ok "1b: an installed copy with any other drift is named and left untouched"
else bad "1b other drift" "$x"; fi
rm -f "$TMP/cron/termlink-presence-sweep"
out=$(run); x=$(s1b "$out")
if echo "$x" | grep_all "skip    every installed crontab already carries"; then ok "1b: second run skips (idempotent)"
else bad "1b idempotent" "$x"; fi
rm -f "$TMP/cron/termlink-heartbeat"

# ---------------------------------------------------------------------------
# T-3615 action 22b — scoped .107 tokens on the fleet hubs, through fakes.
# ---------------------------------------------------------------------------
s22b() { echo "$1" | sed -n '/^.*22b\. Scoped .107 tokens/,/^.*22c\. /p'; }
TOKF="$TR/.termlink/secrets/107-hub.token"; TOKSTATE="$F/tok-state/fake-hub.state"
mints() { [ -f "$F/tok-mints" ] && wc -l < "$F/tok-mints" || echo 0; }
tok_reset; rm -f "$TMP/cron/termlink-fleet-token-remint"; cp "$TR/.termlink/hubs.toml" "$TMP/hubs.before"
out=$(run --dry-run); x=$(s22b "$out")
if echo "$x" | grep_all "\[DRY\]   fake-hub: would mint an Interact token on .107 (no verified token on record)" && [ ! -e "$TOKF" ] \
   && [ "$(mints)" = 0 ] && cmp -s "$TMP/hubs.before" "$TR/.termlink/hubs.toml" && [ ! -e "$TOKSTATE" ]; then
    ok "22b: --dry-run probes, names the mint, mints and writes nothing"
else bad "22b dry-run" "$x"; fi
out=$(run); rc=$?; x=$(s22b "$out")
if [ "$rc" = "0" ] && echo "$x" | grep_all "OK      fake-hub: Interact token installed and proven" \
   && [ "$(stat -c %a "$TOKF")" = 600 ] && [ "$(sha256sum < "$TOKF" | cut -d' ' -f1)" = "$(sed -n 's/^token_sha256=//p' "$TOKSTATE")" ] \
   && grep -q "^token_file = \"$TOKF\"$" "$TR/.termlink/hubs.toml" && grep -q '^secret_file = ".*/107-hub.hex"$' "$TR/.termlink/hubs.toml" \
   && grep -q '^secret_file = "/nonexistent/local.hex"$' "$TR/.termlink/hubs.toml" \
   && ! grep -qE "^(secret_file|secret) *=" "$TR/.termlink/token-only-home/.termlink/hubs.toml" && grep -q "^token_file = " "$TR/.termlink/token-only-home/.termlink/hubs.toml"; then
    ok "22b: mints, installs (0600, sha on record), adds token_file and KEEPS the secret_file line; the token-only home holds no secret"
else bad "22b install" "rc=$rc: $x"; fi
if grep -qx "list token $TR/.termlink/token-only-home" "$F/tok-auth" && grep -qx "post token $TR/.termlink/token-only-home" "$F/tok-auth" \
   && ! grep -q "secret" "$F/tok-auth"; then
    ok "22b: the proof authenticated with the TOKEN (list + accepted post), never the secret"
else bad "22b token proof" "$(cat "$F/tok-auth" 2>/dev/null)"; fi
if ! echo "$out" | grep_all -F "$(cat "$TOKF")" && ! echo "$out" | grep_all -F "fixture-not-a-secret"; then
    ok "22b: neither the token nor the secret appears in runme's output"
else bad "22b leak" "token or secret printed"; fi
if echo "$x" | grep_all "note    fake-hub signs its posts as 0123456789abcdef" && cmp -s "$REPO_ROOT/.context/cron/fleet-token-remint.crontab" "$TMP/cron/termlink-fleet-token-remint" \
   && grep -q "fleet-token-probe" "$F/tok-local"; then
    ok "22b: names the host's signing fingerprint, installs the re-mint cron, creates the scratch topic on .107"
else bad "22b note/cron" "$x"; fi
n=$(mints); out=$(run); rc=$?; x=$(s22b "$out")
if [ "$rc" = "0" ] && echo "$x" | grep_all "skip    fake-hub: verified token on record, installed unchanged" && [ "$(mints)" = "$n" ]; then
    ok "22b: second run skips (idempotent), mints nothing"
else bad "22b idempotent" "rc=$rc: $x"; fi
old_sha=$(sha256sum < "$TOKF"); sed -i "s/^expires_at=.*/expires_at=$(( $(date +%s) + 3600 ))/" "$TOKSTATE"
out=$(run); rc=$?; x=$(s22b "$out")
if [ "$rc" = "0" ] && echo "$x" | grep_all "OK      fake-hub: Interact token installed" && [ "$(mints)" = $((n + 1)) ] && [ "$(sha256sum < "$TOKF")" != "$old_sha" ]; then
    ok "22b: a token with under 3 days left is re-minted and replaced"
else bad "22b renew" "rc=$rc: $x"; fi
echo "tampered" > "$TOKF"; chmod 600 "$TOKF"; out=$(run); rc=$?; x=$(s22b "$out")
if [ "$rc" = "0" ] && echo "$x" | grep_all "OK      fake-hub: Interact token installed" && [ "$(sha256sum < "$TOKF" | cut -d' ' -f1)" = "$(sed -n 's/^token_sha256=//p' "$TOKSTATE")" ]; then
    ok "22b: an installed token that differs from the record is replaced"
else bad "22b tamper" "rc=$rc: $x"; fi
cp "$TOKSTATE" "$TMP/state.before"; rm -f "$TOKSTATE"
out=$(FAKE_TOK_POST_FAIL=1 run); rc=$?; x=$(s22b "$out")
if [ "$rc" = "1" ] && echo "$x" | grep_all "FAILED  fake-hub: token authenticated but a post to fleet-token-probe on .107 was not accepted" && [ ! -e "$TOKSTATE" ]; then
    ok "22b: a post that is not accepted => FAILED, nothing recorded"
else bad "22b post fail" "rc=$rc: $x"; fi
out=$(FAKE_RR_AUTH_RC=1 run); rc=$?; x=$(s22b "$out")
if [ "$rc" = "1" ] && echo "$x" | grep_all "FAILED  fake-hub: channel list --hub 192.168.10.107:9100 with the token-only config did not authenticate" && [ ! -e "$TOKSTATE" ]; then
    ok "22b: token-only config does not authenticate => FAILED, nothing recorded"
else bad "22b auth fail" "rc=$rc: $x"; fi
n=$(mints); out=$(FAKE_TOK_DOWN=1 run); rc=$?; x=$(s22b "$out")
if [ "$rc" = "1" ] && echo "$x" | grep_all "FAILED  fake-hub: no answer over remote exec" && [ "$(mints)" = "$n" ]; then
    ok "22b: unreachable host => FAILED, nothing minted"
else bad "22b down" "rc=$rc: $x"; fi
out=$(FAKE_MINT_FAIL=1 run); rc=$?; x=$(s22b "$out")
if [ "$rc" = "1" ] && echo "$x" | grep_all "FAILED  fake-hub: minting on .107 failed" && [ ! -e "$TOKSTATE" ]; then
    ok "22b: the mint fails => FAILED, nothing written"
else bad "22b mint fail" "rc=$rc: $x"; fi
out=$(FAKE_MINT_SCOPE=execute run); rc=$?; x=$(s22b "$out")
if [ "$rc" = "1" ] && echo "$x" | grep_all "FAILED  fake-hub: the mint did not produce an Interact token" && [ ! -e "$TOKSTATE" ]; then
    ok "22b: a mint that is not Interact is refused, nothing written"
else bad "22b scope" "rc=$rc: $x"; fi
printf '\n[hubs.other-107]\naddress = "192.168.10.107:9100"\nsecret_file = "/x"\n' >> "$TR/.termlink/hubs.toml"
out=$(run); rc=$?; x=$(s22b "$out")
if [ "$rc" = "1" ] && echo "$x" | grep_all "FAILED  fake-hub has 2 profiles at 192.168.10.107:9100" && [ ! -e "$TOKSTATE" ]; then
    ok "22b: two profiles at .107 (which one the CLI uses is undefined) => FAILED, nothing written"
else bad "22b ambiguous" "rc=$rc: $x"; fi
tok_reset; out=$(run); rc=$?
if [ "$rc" = "0" ] && [ -e "$TOKSTATE" ]; then ok "22b: restored to the installed state for later cases"; else bad "22b restore" "rc=$rc: $(s22b "$out")"; fi

# ---------------------------------------------------------------------------
# T-3756 action 22c — the non-root design user's own .107 token, through fakes.
# ---------------------------------------------------------------------------
s22c() { echo "$1" | sed -n '/^.*22c\. /,/^.*22d\. /p'; }
LTOK="$LH/.termlink/secrets/107-local.token"; LSTATE="$F/tok-state/local-$LTU.state"; LT="local:$LTU"
lt_reset; cp "$LH/.termlink/hubs.toml" "$TMP/lt-hubs.before"; cp "$LH/.termlink/secrets/hub.secret" "$TMP/lt-secret.before"
n=$(mints); out=$(run --dry-run); x=$(s22c "$out")
if echo "$x" | grep_all -F "[DRY]   $LT: would mint an Interact token on .107 (no verified token on record)" && [ ! -e "$LTOK" ] \
   && [ "$(mints)" = "$n" ] && cmp -s "$TMP/lt-hubs.before" "$LH/.termlink/hubs.toml" && [ ! -e "$LSTATE" ]; then
    ok "22c: --dry-run probes as the user, names the mint, mints and writes nothing"
else bad "22c dry-run" "$x"; fi
rm -f "$F/tok-auth"; out=$(run); rc=$?; x=$(s22c "$out")
if [ "$rc" = "0" ] && echo "$x" | grep_all -F "OK      $LT: Interact token installed and proven" && echo "$x" | grep_all -F "$LTU's own config authenticates too" \
   && [ "$(stat -c %a:%U "$LTOK")" = "600:$LTU" ] && [ "$(sha256sum < "$LTOK" | cut -d' ' -f1)" = "$(sed -n 's/^token_sha256=//p' "$LSTATE")" ] \
   && [ "$(head -c "$(stat -c %s "$TMP/lt-hubs.before")" "$LH/.termlink/hubs.toml" | sha256sum)" = "$(sha256sum < "$TMP/lt-hubs.before")" ] \
   && python3 -c 'import re,sys; t=open(sys.argv[1]).read(); s=[x for x in re.split(r"(?m)^\[",t) if "address = \"127.0.0.1:9100\"" in x]; sys.exit(0 if len(s)==1 and ("token_file = \"%s\"" % sys.argv[2]) in s[0] and not re.search(r"(?m)^(secret|secret_file)\s*=", s[0]) else 1)' "$LH/.termlink/hubs.toml" "$LTOK" \
   && cmp -s "$TMP/lt-secret.before" "$LH/.termlink/secrets/hub.secret" \
   && ! grep -qE "^(secret_file|secret) *=" "$LH/.termlink/token-only-home/.termlink/hubs.toml" && grep -q '^address = "127.0.0.1:9100"$' "$LH/.termlink/token-only-home/.termlink/hubs.toml"; then
    ok "22c: mints, installs 600:user (sha on record), adds a 127.0.0.1:9100 token_file profile; the user's other profiles and secret copy are byte-identical"
else bad "22c install" "rc=$rc: $x"; fi
if grep -qx "list token $LH/.termlink/token-only-home" "$F/tok-auth" && grep -qx "post token $LH/.termlink/token-only-home" "$F/tok-auth" \
   && grep -qx "list token $LH" "$F/tok-auth"; then
    ok "22c: proven with the TOKEN over TCP (token-only list + accepted post) and with the user's own config"
else bad "22c token proof" "$(cat "$F/tok-auth" 2>/dev/null)"; fi
if [ -s "$F/lt-argv" ] && ! grep -qF "$(cat "$LTOK")" "$F/lt-argv" && ! echo "$out" | grep_all -F "$(cat "$LTOK")" && grep -qx "argv: $LTU $LH" "$F/lt-argv"; then
    ok "22c: runs as the user (runuser argv = user + home only); the token is in no argv and not in runme's output"
else bad "22c leak" "$(cat "$F/lt-argv" 2>/dev/null)"; fi
if echo "$x" | grep_all -F "note    $LTU's own PATH runs /home/u/.local/bin/termlink 0.11.931, which predates token_file" \
   && grep -q -- '--cron --local-user dimitri-mint-dev ' "$REPO_ROOT/.context/cron/fleet-token-remint.crontab"; then
    ok "22c: names the user's old PATH termlink as unable to use token_file; the daily cron carries the renewal line"
else bad "22c note/cron" "$x"; fi
n=$(mints); out=$(run); rc=$?; x=$(s22c "$out")
if [ "$rc" = "0" ] && echo "$x" | grep_all -F "skip    $LT: verified token on record, installed unchanged" && [ "$(mints)" = "$n" ]; then
    ok "22c: second run skips (idempotent), mints nothing"
else bad "22c idempotent" "rc=$rc: $x"; fi
old_sha=$(sha256sum < "$LTOK"); sed -i "s/^expires_at=.*/expires_at=$(( $(date +%s) + 3600 ))/" "$LSTATE"
out=$(run); rc=$?; x=$(s22c "$out")
if [ "$rc" = "0" ] && echo "$x" | grep_all -F "OK      $LT: Interact token installed" && [ "$(mints)" = $((n + 1)) ] && [ "$(sha256sum < "$LTOK")" != "$old_sha" ]; then
    ok "22c: a token with under 3 days left is re-minted and replaced"
else bad "22c renew" "rc=$rc: $x"; fi
# Negative controls: each must be FAILED with rc 1, never OK or skip.
rm -f "$LSTATE"; out=$(FAKE_RR_AUTH_RC=1 run); rc=$?; x=$(s22c "$out")
if [ "$rc" = "1" ] && echo "$x" | grep_all -F "FAILED  $LT: channel list --hub 127.0.0.1:9100 with the token-only config did not authenticate" && [ ! -e "$LSTATE" ] \
   && ! echo "$x" | grep_all -E "^  (OK|skip) +$LT"; then
    ok "22c control: the round trip as the user does not authenticate => FAILED rc 1, nothing recorded"
else bad "22c auth fail" "rc=$rc: $x"; fi
n=$(mints); out=$(FAKE_LT_DOWN=1 run); rc=$?; x=$(s22c "$out")
if [ "$rc" = "1" ] && echo "$x" | grep_all -F "FAILED  $LT: no answer over runuser as $LTU" && [ "$(mints)" = "$n" ]; then
    ok "22c control: runuser gives no answer => FAILED, nothing minted"
else bad "22c down" "rc=$rc: $x"; fi
out=$(RUNME_LOCALTOK_HOME="" RUNME_LOCALTOK_USER=no-such-user-t3756 run); rc=$?; x=$(s22c "$out")
if [ "$rc" = "1" ] && echo "$x" | grep_all -F "FAILED  local:no-such-user-t3756: no such user on this host"; then
    ok "22c control: an unknown user => FAILED"
else bad "22c no user" "rc=$rc: $x"; fi
# A 127.0.0.1:9100 profile whose (stale) secret_file wins over the token: the token is proven and
# recorded, but the user's own config does not authenticate => FAILED, and a re-run does not re-mint.
lt_reset; printf '\n[hubs.local]\naddress = "127.0.0.1:9100"\nsecret_file = "%s"\n' "$LH/.termlink/secrets/hub.secret" >> "$LH/.termlink/hubs.toml"
out=$(FAKE_SECRET_STALE=1 run); rc=$?; x=$(s22c "$out")
if [ "$rc" = "1" ] && echo "$x" | grep_all -F "FAILED  $LT: token installed, proven with the token-only config and recorded, but $LTU's own config" \
   && [ -e "$LSTATE" ] && grep -q "^secret_file = \"$LH/.termlink/secrets/hub.secret\"$" "$LH/.termlink/hubs.toml" && grep -q "^token_file = \"$LTOK\"$" "$LH/.termlink/hubs.toml"; then
    ok "22c control: a winning stale secret_file in the user's profile => FAILED (token kept + recorded, secret line untouched)"
else bad "22c stale secret" "rc=$rc: $x"; fi
n=$(mints); out=$(FAKE_SECRET_STALE=1 run); rc=$?; x=$(s22c "$out")
if [ "$rc" = "1" ] && echo "$x" | grep_all -F "FAILED  $LT: token on record and verified, but $LTU's own config" && [ "$(mints)" = "$n" ]; then
    ok "22c control: re-run with the stale secret still FAILED, and re-mints nothing"
else bad "22c stale secret rerun" "rc=$rc: $x"; fi
# The cron renews the local user: quiet when healthy; re-mints when due; the first install is runme's.
ltcron() {
    FLEET_TOKEN_MINT="$F/tok-mint" FLEET_TOKEN_LOCAL_TL="$F/tok-ltl" FLEET_TOKEN_RUNTIME_DIR="$F/tok-rt" FLEET_TOKEN_STATE_DIR="$F/tok-state" \
    FLEET_TOKEN_LOCAL_HOME="$LH" FLEET_TOKEN_LOCAL_RUN="$F/lt-run" FLEET_TOKEN_USER_TL="$F/rr-tl" \
    FLEET_TOKEN_HEARTBEAT="$F/lt-hb" FLEET_TOKEN_NOTE="$F/tok-note" bash "$REPO_ROOT/scripts/fleet-token-remint.sh" --cron --local-user "$LTU" 2>&1
}
lt_reset; n=$(mints); o=$(ltcron); rc=$?
if [ "$rc" = 0 ] && [ -z "$o" ] && [ "$(mints)" = "$n" ] && [ ! -e "$LTOK" ]; then
    ok "22c cron: no token on record => renews nothing (the first install is runme's), silent"
else bad "22c cron first" "rc=$rc: $o"; fi
out=$(run); rc=$?; sed -i "s/^expires_at=.*/expires_at=$(( $(date +%s) + 7200 ))/" "$LSTATE"; old_sha=$(sha256sum < "$LTOK"); n=$(mints); o=$(ltcron); rc2=$?
if [ "$rc" = 0 ] && [ "$rc2" = 0 ] && [ -z "$o" ] && [ "$(mints)" = $((n + 1)) ] && [ "$(sha256sum < "$LTOK")" != "$old_sha" ] && [ -e "$F/lt-hb" ]; then
    ok "22c cron: a due local token is re-minted silently (empty findings log), heartbeat touched"
else bad "22c cron renew" "rc=$rc/$rc2: $o"; fi

# The same script as the .107 cron runs it: quiet when healthy, loud when not.
cronrun() {
    FLEET_TOKEN_HUBS=fake-hub FLEET_TOKEN_REMOTE="$F/remote" FLEET_TOKEN_MINT="$F/tok-mint" FLEET_TOKEN_LOCAL_TL="$F/tok-ltl" \
    FLEET_TOKEN_RUNTIME_DIR="$F/tok-rt" FLEET_TOKEN_STATE_DIR="$F/tok-state" FLEET_TOKEN_REMOTE_TOKEN="$TOKF" \
    FLEET_TOKEN_REMOTE_HOME="$TR/.termlink/token-only-home" FLEET_TOKEN_REMOTE_HUBS="$TR/.termlink/hubs.toml" \
    FLEET_TOKEN_REMOTE_ID="$TR/.termlink" FLEET_TOKEN_REMOTE_TL="$F/rr-tl" FLEET_TOKEN_REMOTE_OWNER="$(id -un)" \
    FLEET_TOKEN_HEARTBEAT="$F/tok-hb" FLEET_TOKEN_NOTE="$F/tok-note" bash "$REPO_ROOT/scripts/fleet-token-remint.sh" --cron "$@" 2>&1
}
printf '#!/usr/bin/env bash\necho "$1|$2" >> "$FAKE_FLEET_DIR/tok-notes"\n' > "$F/tok-note"; chmod +x "$F/tok-note"
rm -f "$F/tok-hb" "$F/tok-notes"; n=$(mints); o=$(cronrun); rc=$?
if [ "$rc" = 0 ] && [ -z "$o" ] && [ -e "$F/tok-hb" ] && [ "$(mints)" = "$n" ] && [ ! -e "$F/tok-notes" ]; then
    ok "remint cron: healthy and not due => no output (empty findings log), heartbeat touched, nothing minted"
else bad "remint cron healthy" "rc=$rc: $o"; fi
sed -i "s/^expires_at=.*/expires_at=$(( $(date +%s) + 7200 ))/" "$TOKSTATE"
o=$(FAKE_TOK_POST_FAIL=1 cronrun); rc=$?
if [ "$rc" = 1 ] && echo "$o" | grep_all -E '^[0-9T:-]+Z FAILED  fake-hub: token authenticated but a post' && grep -q '^urgent-token-expiring|T-3615: 1 fleet token re-mint' "$F/tok-notes"; then
    ok "remint cron: a failed re-mint is one timestamped FAILED line + an urgent needs-attention note (token expires within a day)"
else bad "remint cron fail" "rc=$rc: $o / $(cat "$F/tok-notes" 2>/dev/null)"; fi
sed -i "s/^expires_at=.*/expires_at=$(( $(date +%s) + 200000 ))/" "$TOKSTATE"; rm -f "$F/tok-notes"
o=$(FAKE_TOK_DOWN=1 cronrun --force); rc=$?
if [ "$rc" = 1 ] && grep -q '^token-remint|' "$F/tok-notes"; then
    ok "remint cron: a failure with days left is noted as token-remint (not urgent)"
else bad "remint cron kind" "rc=$rc: $o"; fi
o=$(cronrun); rc=$?
if [ "$rc" = 0 ] && [ -z "$o" ] && [ "$(sha256sum < "$TOKF" | cut -d' ' -f1)" = "$(sed -n 's/^token_sha256=//p' "$TOKSTATE")" ]; then
    ok "remint cron: renews a due token silently (under 3 days left) and records it"
else bad "remint cron renew" "rc=$rc: $o"; fi
mv "$TOKSTATE" "$TMP/state.keep"; n=$(mints); o=$(cronrun); rc=$?
if [ "$rc" = 0 ] && [ -z "$o" ] && [ "$(mints)" = "$n" ]; then
    ok "remint cron: a host with no token on record is left to runme (the cron never does a first install)"
else bad "remint cron first install" "rc=$rc: $o"; fi
mv "$TMP/state.keep" "$TOKSTATE"
if grep -q '2>> .context/working/.fleet-token-remint-canary.log.stderr$' "$REPO_ROOT/.context/cron/fleet-token-remint.crontab" \
   && grep -q -- '--cron >> .context/working/.fleet-token-remint-canary.log ' "$REPO_ROOT/.context/cron/fleet-token-remint.crontab"; then
    ok "remint cron: findings and stderr go to separate sinks (T-2685), named for /canaries discovery"
else bad "remint crontab sinks"; fi

# ---------------------------------------------------------------------------
# T-3758 action 22d — the design user's stale termlink copies moved aside, through fakes.
# T-3762: every stale copy AHEAD of the current one on the user's PATH, not just ~/.local/bin.
# ---------------------------------------------------------------------------
s22d() { echo "$1" | sed -n '/^.*22d\. /,/^.*22e\. /p'; }
SOLD="$SH/.local/bin/termlink"; SBAK="$SOLD.bak-0.11.931"
SCARGO="$SH/.cargo/bin/termlink"; SCBAK="$SCARGO.bak-0.11.931"
stl_reset; orig_sha=$(sha256sum < "$SOLD"); out=$(run --dry-run); x=$(s22d "$out")
if echo "$x" | grep_all -F "[DRY]   would move $SOLD (0.11.931, 750:$LTU:" && [ -f "$SOLD" ] && [ ! -e "$SBAK" ] && [ ! -e "$F/stl-auth" ]; then
    ok "22d: --dry-run names the move, moves nothing, runs no round trip"
else bad "22d dry-run" "$x"; fi
out=$(run); rc=$?; x=$(s22d "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all -F "OK      moved $SOLD to $SBAK (750:$LTU:" && echo "$x" | grep_all -F "PATH runs $SSYS/termlink 0.12.77" \
   && [ ! -e "$SOLD" ] && [ -f "$SBAK" ] && [ "$(sha256sum < "$SBAK")" = "$orig_sha" ] && [ "$(stat -c %a:%U "$SBAK")" = "750:$LTU" ]; then
    ok "22d: moves the 0.11.931 copy to termlink.bak-0.11.931 (same sha256, mode 750 and owner kept); the user's PATH runs the current one"
else bad "22d move" "rc=$rc: $x"; fi
if grep -qx "list $SH channel list --hub 127.0.0.1:9100" "$F/stl-auth" && [ -s "$F/stl-argv" ] && ! grep -vqx "argv: $LTU $SH" "$F/stl-argv"; then
    ok "22d: every step runs as the user (runuser argv = user + home only), and the bare termlink round trip used 127.0.0.1:9100"
else bad "22d as user" "$(cat "$F/stl-auth" "$F/stl-argv" 2>/dev/null)"; fi
bak_before=$(sha256sum < "$SBAK"); out=$(run); rc=$?; x=$(s22d "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all -F "skip    no stale termlink ahead of the current one on $LTU's PATH (backups kept: $SBAK)" \
   && [ "$(sha256sum < "$SBAK")" = "$bak_before" ] && [ ! -e "$SOLD" ]; then
    ok "22d: second run skips (idempotent), names the kept backup, backup unchanged"
else bad "22d idempotent" "rc=$rc: $x"; fi
# T-3762: the field case — a second 0.11.931 in ~/.cargo/bin. Both go, in one OK line, both kept.
stl_reset; mkdir -p "$SH/.cargo/bin"; stl_old 0.11.931 "$SCARGO"; chmod 755 "$SCARGO"; c_sha=$(sha256sum < "$SCARGO")
out=$(run --dry-run); x=$(s22d "$out")
if echo "$x" | grep_all -F "[DRY]   would move $SOLD (0.11.931" && echo "$x" | grep_all -F "[DRY]   would move $SCARGO (0.11.931, 755:$LTU:" \
   && [ -f "$SOLD" ] && [ -f "$SCARGO" ] && [ ! -e "$SBAK" ] && [ ! -e "$SCBAK" ]; then
    ok "22d: --dry-run names both stale copies, moves neither"
else bad "22d dry-run two" "$x"; fi
out=$(run); rc=$?; x=$(s22d "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all -F "OK      moved $SOLD to $SBAK (750:$LTU:" && echo "$x" | grep_all -F ", moved $SCARGO to $SCBAK (755:$LTU:" \
   && [ ! -e "$SOLD" ] && [ ! -e "$SCARGO" ] && [ "$(sha256sum < "$SBAK")" = "$orig_sha" ] && [ "$(sha256sum < "$SCBAK")" = "$c_sha" ] \
   && [ "$(stat -c %a:%U "$SCBAK")" = "755:$LTU" ] && echo "$x" | grep_all -F "PATH runs $SSYS/termlink 0.12.77"; then
    ok "22d: two stale copies (~/.local/bin and ~/.cargo/bin) are both moved to termlink.bak-0.11.931 in their own folder (sha256 and mode kept); PATH runs the current one"
else bad "22d two stale" "rc=$rc: $x"; fi
out=$(run); rc=$?; x=$(s22d "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all -F "skip    no stale termlink ahead of the current one on $LTU's PATH (backups kept: $SBAK, $SCBAK)"; then
    ok "22d: after two moves a re-run skips and lists both backups"
else bad "22d two stale rerun" "rc=$rc: $x"; fi
# A copy that IS the current build stops the walk: it and everything after it stay untouched.
stl_reset; mkdir -p "$SH/.cargo/bin" "$SH/after"; cp -p "$SSYS/termlink" "$SCARGO"; chmod 700 "$SCARGO"; cur_copy=$(sha256sum < "$SCARGO")
stl_old 0.11.931 "$SH/after/termlink"; after_sha=$(sha256sum < "$SH/after/termlink")
out=$(run); rc=$?; x=$(s22d "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all -F "OK      moved $SOLD to $SBAK" && ! echo "$x" | grep_all -F "moved $SCARGO" \
   && [ "$(sha256sum < "$SCARGO")" = "$cur_copy" ] && [ "$(stat -c %a "$SCARGO")" = 700 ] && ! compgen -G "$SH/.cargo/bin/*bak*" >/dev/null \
   && [ "$(sha256sum < "$SH/after/termlink")" = "$after_sha" ] && [ ! -e "$SH/after/termlink.bak-0.11.931" ] && echo "$x" | grep_all -F "PATH runs $SCARGO 0.12.77"; then
    ok "22d: a copy that IS the current build (~/.cargo/bin) is never touched, nor the stale copy after it on PATH; the one ahead is moved"
else bad "22d current build" "rc=$rc: $x"; fi
# An identical backup left by an earlier attempt may be replaced: same bytes, nothing lost.
stl_reset; cp -p "$SOLD" "$SBAK"; out=$(run); rc=$?; x=$(s22d "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all -F "OK      moved $SOLD to $SBAK" && [ ! -e "$SOLD" ] && [ "$(sha256sum < "$SBAK")" = "$orig_sha" ]; then
    ok "22d: an existing byte-identical backup => moved, still one copy of the same bytes"
else bad "22d identical backup" "rc=$rc: $x"; fi
# An older copy at another version is stale too: it gets its own version in the backup name.
stl_reset; stl_old 0.11.900 "$SOLD"; s_before=$(sha256sum < "$SOLD"); out=$(run); rc=$?; x=$(s22d "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all -F "OK      moved $SOLD to $SOLD.bak-0.11.900" && [ "$(sha256sum < "$SOLD.bak-0.11.900")" = "$s_before" ] && [ ! -e "$SOLD" ]; then
    ok "22d: an older copy at another version (0.11.900) is moved to termlink.bak-0.11.900"
else bad "22d older version" "rc=$rc: $x"; fi
# Negative controls: each FAILED with rc 1, never OK or skip.
stl_reset; mkdir -p "$SH/.cargo/bin"; stl_old 0.13.0 "$SCARGO"; n_sha=$(sha256sum < "$SCARGO"); out=$(run); rc=$?; x=$(s22d "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all -F "FAILED  $SCARGO is version 0.13.0, newer than the current 0.12.77 but not it" \
   && echo "$x" | grep_all -F "note    nothing was moved" && [ "$(sha256sum < "$SCARGO")" = "$n_sha" ] && [ "$(sha256sum < "$SOLD")" = "$orig_sha" ] \
   && [ ! -e "$SBAK" ] && [ ! -e "$SCARGO.bak-0.13.0" ] && [ ! -e "$F/stl-auth" ] && ! echo "$x" | grep_all -E '^  (OK|skip) '; then
    ok "22d control: a newer-but-not-current copy is refused and reported => FAILED rc 1; nothing moved, not even the stale copy beside it"
else bad "22d newer" "rc=$rc: $x"; fi
stl_reset; echo "someone else's file" > "$SBAK"; cp "$SBAK" "$TMP/stl-bak.before"; out=$(run); rc=$?; x=$(s22d "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all -F "FAILED  $SBAK already exists with different content; refusing to overwrite it" \
   && cmp -s "$SBAK" "$TMP/stl-bak.before" && [ "$(sha256sum < "$SOLD")" = "$orig_sha" ] && ! echo "$x" | grep_all -E '^  (OK|skip) '; then
    ok "22d control: backup already exists with different content => FAILED rc 1, both files byte-identical"
else bad "22d backup differs" "rc=$rc: $x"; fi
stl_reset; mkdir -p "$SH/.cargo/bin"; printf '#!/usr/bin/env bash\necho no version here\n' > "$SCARGO"; chmod 755 "$SCARGO"; out=$(run); rc=$?; x=$(s22d "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all -F "FAILED  $SCARGO gave no version; left untouched" && [ -f "$SOLD" ] && [ ! -e "$SBAK" ]; then
    ok "22d control: an unversioned copy ahead of the current one => FAILED rc 1, nothing moved"
else bad "22d unversioned" "rc=$rc: $x"; fi
stl_reset; mkdir -p "$SH/.cargo/bin" "$SH/elsewhere"; stl_old 0.11.931 "$SH/elsewhere/tl"; ln -s "$SH/elsewhere/tl" "$SCARGO"; out=$(run); rc=$?; x=$(s22d "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all -F "FAILED  $SCARGO is a link (version 0.11.931), not a regular file; left untouched" && [ -L "$SCARGO" ] && [ -f "$SOLD" ]; then
    ok "22d control: a stale symlink ahead of the current one => FAILED rc 1, left untouched"
else bad "22d symlink" "rc=$rc: $x"; fi
stl_reset; mkdir -p "$SH/.cargo/bin"; stl_old 0.11.931 "$SCARGO"; out=$(FAKE_STL_NOSYS=1 run); rc=$?; x=$(s22d "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all -F "FAILED  $LTU's PATH still resolves termlink to nothing ?, not the current 0.12.77" \
   && echo "$x" | grep_all -F "undo: rename $SBAK back to termlink; rename $SCBAK back to termlink" && [ -f "$SBAK" ] && [ -f "$SCBAK" ] \
   && ! echo "$x" | grep_all -E '^  (OK|skip) '; then
    ok "22d control: no current termlink on PATH after the move => FAILED rc 1, names the undo for every moved copy"
else bad "22d nothing current" "rc=$rc: $x"; fi
out=$(FAKE_STL_NOSYS=1 run); rc=$?; x=$(s22d "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all -F "FAILED  $LTU's PATH still resolves termlink to nothing" && ! echo "$x" | grep_all -E '^  (OK|skip) '; then
    ok "22d control: re-run with still no current termlink on PATH => still FAILED (never skip)"
else bad "22d nothing current rerun" "rc=$rc: $x"; fi
stl_reset; out=$(FAKE_STL_AUTH_RC=1 run); rc=$?; x=$(s22d "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all -F "FAILED  $LTU's termlink is the current 0.12.77 ($SSYS/termlink), but 'termlink channel list --hub 127.0.0.1:9100' as $LTU failed (rc 1)" \
   && ! echo "$x" | grep_all -E '^  (OK|skip) '; then
    ok "22d control: the round trip as the user fails => FAILED rc 1"
else bad "22d round trip" "rc=$rc: $x"; fi
stl_reset; out=$(FAKE_STL_DOWN=1 run); rc=$?; x=$(s22d "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all -F "FAILED  $LTU: no answer over runuser as $LTU; nothing changed" && [ -f "$SOLD" ] && [ ! -e "$SBAK" ]; then
    ok "22d control: runuser gives no answer => FAILED rc 1, nothing moved"
else bad "22d down" "rc=$rc: $x"; fi
stl_reset; out=$(RUNME_STALETL_HOME="" RUNME_LOCALTOK_USER=no-such-user-t3758 run); rc=$?; x=$(s22d "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all -F "FAILED  no-such-user-t3758: no such user on this host"; then
    ok "22d control: an unknown user => FAILED"
else bad "22d no user" "rc=$rc: $x"; fi
# Restore the installed state for later cases (moved and verified).
stl_reset; out=$(run); rc=$?
if [ "$rc" = 0 ] && [ -f "$SBAK" ] && [ ! -e "$SOLD" ]; then ok "22d: restored to the moved state for later cases"; else bad "22d restore" "rc=$rc: $(s22d "$out")"; fi

# ---------------------------------------------------------------------------
# T-3781 action 22e — the design user's hub link, through fakes (no real mint, user or hub).
# ---------------------------------------------------------------------------
s22e() { echo "$1" | sed -n '/^.*22e\. /,/^.*23\. Daily telemetry rollup/p'; }
hlm() { [ -f "$F/hl-mints" ] && wc -l < "$F/hl-mints" || echo 0; }
HLT="$HLH/.termlink/hub-register.token"; HLA="$HLH/.termlink/hub-register.addr"; HLST="$HLS/$LTU.state"; HL="hub-link:$LTU"
hl_skip_state; rm -f "$F/hl-mints"   # counts below are this block's own
out=$(run); rc=$?; x=$(s22e "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all -F "skip    $HL: verified link on record, installed unchanged" && [ "$(hlm)" = 0 ]; then
    ok "22e: a verified, unchanged, not-yet-due link is skipped and nothing is minted"
else bad "22e skip" "rc=$rc: $x"; fi
rm -rf "$HLH" "$HLS" "$F/hl-argv" "$F/hl-registers"; mkdir -p "$HLH"
out=$(run --dry-run); x=$(s22e "$out")
if echo "$x" | grep_all -F "[DRY]   $HL: would mint a registration claim (principal $LTU, prefix $LTU-, 7 days)" && [ ! -e "$HLT" ] && [ "$(hlm)" = 0 ]; then
    ok "22e --dry-run: says what it would mint, writes and mints nothing"
else bad "22e dry-run" "$x"; fi
out=$(run); rc=$?; x=$(s22e "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all -F "OK      $HL: registration claim (prefix $LTU-) installed 600:$LTU with address 127.0.0.1:9100 and proven" \
   && [ "$(stat -c %a:%U "$HLT")" = "600:$LTU" ] && [ "$(stat -c %a "$HLA")" = 600 ] && [ "$(cat "$HLA")" = "127.0.0.1:9100" ] \
   && [ "$(sha256sum < "$HLT" | cut -d' ' -f1)" = "$(sed -n 's/^token_sha256=//p' "$HLST")" ] \
   && grep -qF -- "--register-principal $LTU --register-prefix $LTU- --ttl 604800" "$F/hl-mints" && grep -qx "register $HLH" "$F/hl-registers"; then
    ok "22e: mints a 7-day claim for the user, installs token + address 600:user, proves it with a plain register as the user, records the sha"
else bad "22e install" "rc=$rc: $x"; fi
if [ -s "$F/hl-argv" ] && ! grep -qF "$(cat "$HLT")" "$F/hl-argv" && ! echo "$out" | grep_all -F "$(cat "$HLT")" && grep -qx "argv: $LTU $HLH" "$F/hl-argv"; then
    ok "22e: runs as the user (argv = user + home only); the token is in no argv and not in runme's output"
else bad "22e leak" "$(cat "$F/hl-argv" 2>/dev/null)"; fi
n=$(hlm); out=$(run); rc=$?; x=$(s22e "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all -F "skip    $HL: verified link on record" && [ "$(hlm)" = "$n" ]; then
    ok "22e: second run skips (idempotent), mints nothing"
else bad "22e idempotent" "rc=$rc: $x"; fi
old=$(sha256sum < "$HLT"); sed -i "s/^expires_at=.*/expires_at=$(( $(date +%s) + 3600 ))/" "$HLST"
out=$(run); rc=$?; x=$(s22e "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all -F "OK      $HL: registration claim" && [ "$(hlm)" = $((n + 1)) ] && [ "$(sha256sum < "$HLT")" != "$old" ]; then
    ok "22e: a link with under 3 days left is re-minted and replaced"
else bad "22e renew" "rc=$rc: $x"; fi
if grep -qE -- '^[0-9]+ [0-9]+ \* \* \* root cd /opt/termlink && bash scripts/hub-link-provision.sh --cron --user dimitri-mint-dev ' "$REPO_ROOT/.context/cron/fleet-token-remint.crontab"; then
    ok "22e: the daily cron (installed by 22b) carries the renewal line"
else bad "22e cron" "no hub-link-provision line"; fi
out=$(HUBLINK_HOME="$HLH" HUBLINK_RUN="$F/hl-run" HUBLINK_MINT="$F/hl-mint" HUBLINK_STATE_DIR="$F/hl-nostate" HUBLINK_TL="$F/hl-tl" \
      bash "$REPO_ROOT/scripts/hub-link-provision.sh" --cron --user "$LTU" 2>&1); rc=$?
if [ "$rc" = 0 ] && echo "$out" | grep_all -F "skip    $HL: no link on record; the first install is runme.sh action 22e" && [ "$(hlm)" = $((n + 1)) ]; then
    ok "22e cron: with no link on record the cron installs nothing (the first install is runme's)"
else bad "22e cron first" "rc=$rc: $out"; fi
# Negative controls: each FAILED with rc 1, nothing recorded.
rm -f "$HLST"; n=$(hlm); out=$(FAKE_HL_OWNER=someone-else run); rc=$?; x=$(s22e "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all -F "FAILED  $HL: a plain 'termlink register' as $LTU did not come up linked with owner $LTU" && [ ! -e "$HLST" ] \
   && ! echo "$x" | grep_all -E "^  (OK|skip) +$HL"; then
    ok "22e control: the register proof shows another owner => FAILED rc 1, nothing recorded"
else bad "22e owner" "rc=$rc: $x"; fi
n=$(hlm); out=$(FAKE_HL_DOWN=1 run); rc=$?; x=$(s22e "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all -F "FAILED  $HL: no answer over runuser as $LTU" && [ "$(hlm)" = "$n" ]; then
    ok "22e control: runuser gives no answer => FAILED, nothing minted"
else bad "22e down" "rc=$rc: $x"; fi
out=$(FAKE_HL_MINT_FAIL=1 run); rc=$?; x=$(s22e "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all -F "FAILED  $HL: minting on .107 failed" && [ ! -e "$HLST" ]; then
    ok "22e control: the mint fails => FAILED, nothing written"
else bad "22e mint fail" "rc=$rc: $x"; fi
out=$(RUNME_HUBLINK_HOME="" RUNME_HUBLINK_USER=no-such-user-t3781 run); rc=$?; x=$(s22e "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all -F "FAILED  hub-link:no-such-user-t3781: no such user on this host"; then
    ok "22e control: an unknown user => FAILED"
else bad "22e no user" "rc=$rc: $x"; fi
hl_skip_state   # later cases see the skip state again

# ---------------------------------------------------------------------------
# T-3615 action 23 — the daily telemetry rollup on the fleet hubs, through fakes.
# ---------------------------------------------------------------------------
s23() { echo "$1" | sed -n '/^.*23\. Daily telemetry rollup/,/^.*Verification/p'; }
rr_reset() { rm -rf "$F/rr-root" "$F/rollup-cmds"; rm -f "$RUNME_REMOTE_ROLLUP_CRON"; }
rr_reset
out=$(RUNME_ROLLUP_CREDENTIAL_DECISION="" run); rc=$?; x=$(s23 "$out")
if [ "$rc" = "0" ] && echo "$x" | grep_all "HELD    no credential decision recorded (T-3615 Q3" \
   && [ ! -e "$F/rr-root" ] && [ ! -e "$RUNME_REMOTE_ROLLUP_CRON" ] && [ ! -s "$F/rollup-cmds" ] \
   && echo "$out" | grep_all -E " 1 held, 0 failed"; then
    ok "rollup: default (no credential decision recorded) is HELD, rc 0, nothing probed or written (T-3615 Q3)"
else bad "rollup held default" "rc=$rc: $x"; fi
out=$(run --dry-run); x=$(s23 "$out")
if echo "$x" | grep_all "would install the rollup on fake-hub" && [ ! -e "$F/rr-root" ] && [ ! -e "$RUNME_REMOTE_ROLLUP_CRON" ]; then
    ok "rollup: --dry-run probes, reports the install, writes nothing"
else bad "rollup dry-run" "$x"; fi
out=$(run); rc=$?; x=$(s23 "$out")
if [ "$rc" = "0" ] && echo "$x" | grep_all "OK      fake-hub: rollup installed and verified" \
   && cmp -s "$REPO_ROOT/scripts/telemetry-rollup.sh" "$F/rr-root/scripts/telemetry-rollup.sh" \
   && grep -q '^TELEMETRY_HUB=192.168.10.107:9100$' "$RUNME_REMOTE_ROLLUP_CRON" && grep -q '^TELEMETRY_HOST=fake-hub$' "$RUNME_REMOTE_ROLLUP_CRON" \
   && grep -q '^TERMLINK_WORKER_CLASS=background$' "$RUNME_REMOTE_ROLLUP_CRON" \
   && grep -q "^30 2 \* \* \* root cd $F/rr-root && bash scripts/telemetry-rollup.sh --catch-up 7 " "$RUNME_REMOTE_ROLLUP_CRON"; then
    ok "rollup: installs script + cron file (posts to .107, host label, background class, --catch-up 7) and verifies both plus a remote dry run"
else bad "rollup install" "rc=$rc: $x"; fi
: > "$F/tok-auth"
if grep -q "^TELEMETRY_TERMLINK=$F/rr-root/bin/termlink-107-token$" "$RUNME_REMOTE_ROLLUP_CRON" && [ -x "$F/rr-root/bin/termlink-107-token" ] \
   && HOME="$TR" "$F/rr-root/bin/termlink-107-token" channel post telemetry:daily --payload '{}' --hub 192.168.10.107:9100 >/dev/null \
   && grep -qx "post token $TR/.termlink/token-only-home" "$F/tok-auth" && ! grep -q "post secret" "$F/tok-auth"; then
    ok "rollup: the cron's termlink is the token wrapper: a --hub post uses the token-only config even though the host's profile still has the secret (Q3=A)"
else bad "rollup token wrapper" "$(cat "$F/tok-auth" 2>/dev/null)"; fi
: > "$F/rollup-cmds"
out=$(run); rc=$?; x=$(s23 "$out")
if [ "$rc" = "0" ] && echo "$x" | grep_all "skip    fake-hub already runs the current rollup daily" && ! grep -q RUNME-ROLLUP-INSTALL "$F/rollup-cmds" 2>/dev/null; then
    ok "rollup: second run skips (sha256 identical), no install command sent"
else bad "rollup idempotent" "rc=$rc: $x"; fi
echo "# stale" >> "$F/rr-root/scripts/telemetry-rollup.sh"; : > "$F/rollup-cmds"
out=$(run); rc=$?; x=$(s23 "$out")
if [ "$rc" = "0" ] && echo "$x" | grep_all "OK      fake-hub: rollup installed" && cmp -s "$REPO_ROOT/scripts/telemetry-rollup.sh" "$F/rr-root/scripts/telemetry-rollup.sh"; then
    ok "rollup: a drifted remote script is reinstalled"
else bad "rollup drift" "rc=$rc: $x"; fi
rr_reset
out=$(FAKE_RR_AUTH_RC=1 run); rc=$?; x=$(s23 "$out")
if [ "$rc" = "1" ] && echo "$x" | grep_all "FAILED  fake-hub cannot authenticate to .107 (192.168.10.107:9100) with the token-only config" && echo "$x" | grep_all "action 22b installs it" \
   && [ ! -e "$F/rr-root" ] && [ ! -e "$RUNME_REMOTE_ROLLUP_CRON" ]; then
    ok "rollup: no verified token for .107 => FAILED naming action 22b, nothing written"
else bad "rollup no auth" "rc=$rc: $x"; fi
out=$(FAKE_RR_DOWN=1 run); rc=$?; x=$(s23 "$out")
if [ "$rc" = "1" ] && echo "$x" | grep_all "FAILED  fake-hub: no answer over remote exec" && [ ! -e "$F/rr-root" ]; then
    ok "rollup: unreachable hub => FAILED, nothing written"
else bad "rollup down" "rc=$rc: $x"; fi
out=$(RUNME_REMOTE_TL="$F/no-such-termlink" run); rc=$?; x=$(s23 "$out")
if [ "$rc" = "1" ] && echo "$x" | grep_all "FAILED  fake-hub lacks python3 or termlink" && [ ! -e "$F/rr-root" ]; then
    ok "rollup: termlink missing on the remote => FAILED, nothing written"
else bad "rollup no termlink" "rc=$rc: $x"; fi
printf '#!/bin/sh\nexit 1\n' > "$F/bad-src.sh"; rr_reset
out=$(RUNME_ROLLUP_SRC="$F/bad-src.sh" run); rc=$?; x=$(s23 "$out")
if [ "$rc" = "1" ] && echo "$x" | grep_all "FAILED  fake-hub after install: .*dry run failed"; then
    ok "rollup: files in place but the installed rollup's dry run fails => FAILED (verified, not assumed)"
else bad "rollup verify dry run" "rc=$rc: $x"; fi
rr_reset
mkdir -p "$F/rr-root/scripts"; out=$(run); rc=$?
if [ "$rc" = "0" ]; then ok "rollup: restored to installed state for later cases"; else bad "rollup restore" "rc=$rc: $(s23 "$out")"; fi

# ---------------------------------------------------------------------------
# T-3615 action 24 — remove .107's full secret from a host whose token path is proven.
# ---------------------------------------------------------------------------
s24() { echo "$1" | sed -n '/^.*24\. Remove .107.s full secret/,/^.*Verification/p'; }
SECF="$TR/.termlink/secrets/107-hub.hex"
earlier() { sed -i "s/^verified_at=.*/verified_at=$(( $(date +%s) - 100 ))/" "$TOKSTATE"; }
r24() { RUNME_SECRET_REMOVAL_HUBS=fake-hub run "$@"; }
tok_reset; run >/dev/null; earlier; cp "$TR/.termlink/hubs.toml" "$TMP/hubs.before"
out=$(r24); rc=$?; x=$(s24 "$out")
if [ "$rc" = "0" ] && echo "$x" | grep_all "HELD    fake-hub: no removal decision recorded" && [ -e "$SECF" ] && cmp -s "$TMP/hubs.before" "$TR/.termlink/hubs.toml"; then
    ok "24: default (no per-host decision) is HELD, rc 0, the secret stays"
else bad "24 held default" "rc=$rc: $x"; fi
export RUNME_SECRET_REMOVAL_DECISION_fake_hub="fixture: host confirmed"
rm -f "$TOKSTATE"; out=$(r24); rc=$?; x=$(s24 "$out")
if [ "$rc" = "0" ] && echo "$x" | grep_all "HELD    fake-hub: decided (fixture: host confirmed), but its token path was not verified on an earlier run" && [ -e "$SECF" ]; then
    ok "24: decided, but the token was proven only in THIS run => HELD, the secret stays"
else bad "24 same run" "rc=$rc: $x"; fi
# greenfield ruling B: an "after-remint" decision also waits for the first automatic re-mint.
export RUNME_SECRET_REMOVAL_DECISION_fake_hub="after-remint: fixture"
earlier; first0="$(sed -n 's/^first_verified_at=//p' "$TOKSTATE")"
out=$(r24); rc=$?; x=$(s24 "$out")
if [ "$rc" = "0" ] && grep -qx 'mints=1' "$TOKSTATE" && echo "$x" | grep_all "HELD    fake-hub: decided (after-remint: fixture), waits for the first automatic re-mint to verify (verified mints on record: 1)" && [ -e "$SECF" ]; then
    ok "24: after-remint decision with only the first install on record (mints=1) => HELD, the secret stays"
else bad "24 after-remint held" "rc=$rc: $(grep mints "$TOKSTATE") $x"; fi
sed -i "s/^expires_at=.*/expires_at=$(( $(date +%s) + 60 ))/" "$TOKSTATE"; out=$(r24); x=$(s24 "$out")
if grep -qx 'mints=2' "$TOKSTATE" && grep -qx "first_verified_at=$first0" "$TOKSTATE" && echo "$x" | grep_all "HELD    fake-hub: decided (after-remint: fixture), but its token path was not verified on an earlier run"; then
    ok "24: a re-mint counts (mints=2, first_verified_at kept); proven in THIS run => still HELD until the next run"
else bad "24 remint counted" "$(cat "$TOKSTATE") / $x"; fi
earlier; out=$(r24 --dry-run); x=$(s24 "$out")
if echo "$x" | grep_all "\[DRY\]   would remove secret_file from fake-hub's .107 profile .*(decision: after-remint: fixture)" && [ -e "$SECF" ]; then
    ok "24: after the re-mint verified on an earlier run, the after-remint decision proceeds (dry-run names it)"
else bad "24 after-remint proceeds" "$x"; fi
export RUNME_SECRET_REMOVAL_DECISION_fake_hub="fixture: host confirmed"
earlier; out=$(r24 --dry-run); x=$(s24 "$out")
if echo "$x" | grep_all "\[DRY\]   would remove secret_file from fake-hub's .107 profile and move $SECF" && [ -e "$SECF" ] && grep -q '^secret_file = ".*/107-hub.hex"' "$TR/.termlink/hubs.toml"; then
    ok "24: --dry-run names the removal and changes nothing"
else bad "24 dry-run" "$x"; fi
echo "secret_file = \"$SECF\"" > "$F/tok-refs/sidecar.env"
out=$(r24); rc=$?; x=$(s24 "$out")
if [ "$rc" = "1" ] && echo "$x" | grep_all "FAILED  fake-hub: the secret file .* is still named elsewhere (profiles: none; files: $F/tok-refs/sidecar.env)" && [ -e "$SECF" ]; then
    ok "24: another file on the host still names the secret file => FAILED, nothing changed"
else bad "24 refs" "rc=$rc: $x"; fi
rm -f "$F/tok-refs/sidecar.env"
cp "$TR/.termlink/hubs.toml" "$TMP/hubs.pre"; : > "$F/tok-auth"
out=$(FAKE_TOK_POST_FAIL=1 r24); rc=$?; x=$(s24 "$out")
if [ "$rc" = "1" ] && echo "$x" | grep_all "FAILED  fake-hub: authenticated, but a post to fleet-token-probe was not accepted; put hubs.toml and .* back" \
   && [ -e "$SECF" ] && [ ! -e "$SECF.t3615-retired" ] && cmp -s "$TMP/hubs.pre" "$TR/.termlink/hubs.toml"; then
    ok "24: verification fails after the removal => FAILED and hubs.toml + the secret file are put back"
else bad "24 rollback" "rc=$rc: $x"; fi
: > "$F/tok-auth"; out=$(r24); rc=$?; x=$(s24 "$out")
if [ "$rc" = "0" ] && echo "$x" | grep_all "OK      fake-hub: .107's full secret removed from its profile" \
   && [ ! -e "$SECF" ] && [ "$(stat -c %a "$SECF.t3615-retired")" = 600 ] && [ -e "$TR/.termlink/hubs.toml.t3615-pre-removal" ] \
   && ! grep -q '107-hub.hex' "$TR/.termlink/hubs.toml" && grep -q '^secret_file = "/nonexistent/local.hex"$' "$TR/.termlink/hubs.toml" \
   && grep -q "^token_file = \"$TOKF\"$" "$TR/.termlink/hubs.toml" && grep -qx "post token $TR" "$F/tok-auth"; then
    ok "24: removes only the .107 secret_file line, moves the file to a 0600 .t3615-retired copy, and mail authenticates with the token through the REAL config"
else bad "24 remove" "rc=$rc: $x / $(cat "$F/tok-auth")"; fi
out=$(r24); rc=$?; x=$(s24 "$out")
if [ "$rc" = "0" ] && echo "$x" | grep_all "skip    fake-hub: .107's secret already removed from its profile; mail still authenticates"; then
    ok "24: second run re-verifies and skips (idempotent)"
else bad "24 idempotent" "rc=$rc: $x"; fi
out=$(r24); x=$(s22b "$out")
if echo "$x" | grep_all "skip    fake-hub: verified token on record"; then
    ok "24: action 22b still recognises the host after the removal (token_file line intact)"
else bad "24 then 22b" "$x"; fi
tok_reset; run >/dev/null; earlier
sed -i 's|^secret_file = ".*/107-hub.hex"$|secret = "00"|' "$TR/.termlink/hubs.toml"
out=$(r24); rc=$?; x=$(s24 "$out")
if [ "$rc" = "1" ] && echo "$x" | grep_all 'FAILED  fake-hub: the .107 profile holds an inline secret' && grep -q '^secret = "00"$' "$TR/.termlink/hubs.toml"; then
    ok "24: an inline secret is refused (left for a hand edit), nothing changed"
else bad "24 inline" "rc=$rc: $x"; fi
unset RUNME_SECRET_REMOVAL_DECISION_fake_hub
tok_reset; out=$(run); rc=$?
if [ "$rc" = "0" ]; then ok "24: restored for later cases"; else bad "24 restore" "rc=$rc"; fi

# ---------------------------------------------------------------------------
# T-3618 action 25 — remove 150's retired ntfy cron lines, nothing else.
# ---------------------------------------------------------------------------
s25() { echo "$1" | sed -n '/^.*25\. Remove 150-skills-manager/,/^.*Verification/p'; }
seed_ct() { cat > "$CT" <<'EOC'
0 3 * * * /opt/150-skills-manager/deploy/ntfy/backup.sh >> /opt/150-skills-manager/.context/working/ntfy-backup.log 2>&1
0 6 * * * /opt/x/update.sh
*/5 * * * * /opt/150-skills-manager/deploy/ntfy/health-check.sh 2>&1
# ntfy backup (T-111)
# ntfy health check (T-111)
# other job
EOC
rm -f "$TMP"/crontab.bak.T-476*; }
want_ct=$'0 6 * * * /opt/x/update.sh\n# other job'
seed_ct; out=$(run --dry-run); x=$(s25 "$out")
if echo "$x" | grep_all '\[DRY\]   would remove 4 ntfy line(s)' && grep -q ntfy "$CT"; then
    ok "25: --dry-run names the 4 lines and changes nothing"
else bad "25 dry-run" "$x"; fi
out=$(run); rc=$?; x=$(s25 "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all 'OK      removed 4 ntfy line(s) from root.s crontab; every other line verified unchanged' \
   && [ "$(cat "$CT")" = "$want_ct" ] && [ "$(stat -c %a "$TMP/crontab.bak.T-476")" = 600 ] && grep -q 'health-check.sh' "$TMP/crontab.bak.T-476"; then
    ok "25: removes exactly the 2 ntfy jobs + 2 comments, keeps every other line in order, 0600 backup holds the original"
else bad "25 remove" "rc=$rc: $x / $(cat "$CT")"; fi
out=$(run); x=$(s25 "$out")
if echo "$x" | grep_all 'skip    no ntfy line from 150-skills-manager'; then ok "25: second run skips (idempotent)"; else bad "25 idempotent" "$x"; fi
seed_ct; out=$(FAKE_CT_REFUSE=1 run); rc=$?; x=$(s25 "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all 'FAILED  crontab refused the edited table; restored' && [ "$(grep -c ntfy "$CT")" = 4 ]; then
    ok "25: crontab refuses the edit => FAILED, the original is restored"
else bad "25 refuse" "rc=$rc: $x"; fi
seed_ct; out=$(FAKE_CT_DRIFT=1 run); rc=$?; x=$(s25 "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all 'FAILED  the installed crontab is not the old one minus the 4 ntfy line(s); restored' && [ "$(grep -c ntfy "$CT")" = 4 ]; then
    ok "25: installed table differs from old-minus-ntfy => FAILED, the original is restored"
else bad "25 drift" "rc=$rc: $x"; fi
printf '0 6 * * * /opt/x/update.sh\n# other job\n' > "$CT"; rm -f "$TMP"/crontab.bak.T-476*

# ---------------------------------------------------------------------------
# T-3657 action 26 — drop the waiting messages T-3656 proved harmless.
# ---------------------------------------------------------------------------
s26() { echo "$1" | sed -n '/^.*26\. Drop the/,/^.*Verification/p'; }
seed_w() { printf 'aaaa-1111\nbbbb-2222\nzzzz-9999\n' > "$FAKE_WAIT_IDS"; : > "$FAKE_DROP_LOG"; }
seed_w; out=$(CLAUDECODE= run --dry-run); x=$(s26 "$out")
if echo "$x" | grep_all '\[DRY\]   would drop 2 of 3 listed waiting messages' && [ "$(wc -l < "$FAKE_WAIT_IDS")" = 3 ] && [ ! -s "$FAKE_DROP_LOG" ]; then
    ok "26: --dry-run counts the 2 still waiting of 3 listed and drops nothing"
else bad "26 dry-run" "$x"; fi
seed_w; out=$(CLAUDECODE=1 run); rc=$?; x=$(s26 "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all "FAILED  2 messages left to drop, but this shell is an agent's (CLAUDECODE=1)" \
   && [ ! -s "$FAKE_DROP_LOG" ] && [ "$(wc -l < "$FAKE_WAIT_IDS")" = 3 ]; then
    ok "26: under CLAUDECODE=1 => FAILED with the run-it-yourself line; no drop attempted, no --i-am-human forged"
else bad "26 agent shell" "rc=$rc: $x"; fi
seed_w; out=$(CLAUDECODE= run); rc=$?; x=$(s26 "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all 'OK      dropped 2 waiting messages with their evidence' \
   && [ "$(cat "$FAKE_WAIT_IDS")" = "zzzz-9999" ] && grep -qxF 'bbbb-2222|T-3656: answered: reply "ok" posted 2026-10-06' "$FAKE_DROP_LOG" \
   && grep -qxF 'aaaa-1111|T-3656: e2e residue: selftest conv x' "$FAKE_DROP_LOG" && ! grep -q FORGED "$FAKE_DROP_LOG" && ! grep -q cccc "$FAKE_DROP_LOG"; then
    ok "26: drops exactly the listed ids still waiting, each with its evidence as --reason; unlisted id untouched"
else bad "26 drop" "rc=$rc: $x / $(cat "$FAKE_DROP_LOG")"; fi
out=$(CLAUDECODE= run); x=$(s26 "$out")
if echo "$x" | grep_all 'skip    none of the 3 listed messages is still waiting'; then ok "26: second run skips (idempotent)"; else bad "26 idempotent" "$x"; fi
seed_w; out=$(CLAUDECODE= FAKE_DROP_REFUSE=bbbb-2222 run); rc=$?; x=$(s26 "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all 'FAILED  1 of 2 listed messages are still waiting after the drop (first: bbbb-2222)'; then
    ok "26: a drop that does not take => FAILED by the on-disk re-read, naming the id"
else bad "26 refuse" "rc=$rc: $x"; fi
seed_w; out=$(CLAUDECODE= FAKE_WAIT_UNREADABLE=1 run); rc=$?; x=$(s26 "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all "FAILED  could not read 'fw sidecar waiting --json'; nothing dropped" && [ ! -s "$FAKE_DROP_LOG" ]; then
    ok "26: unreadable waiting list => FAILED, nothing dropped (never a skip)"
else bad "26 unreadable" "rc=$rc: $x"; fi
: > "$FAKE_WAIT_IDS"; : > "$FAKE_DROP_LOG"

# ---------------------------------------------------------------------------
# T-3664 action 27 — close .107's Docker API on tcp 2375 (operator ruling A).
# ---------------------------------------------------------------------------
s27() { echo "$1" | sed -n '/^.*27\. Close .107.s Docker API/,/^.*Verification/p'; }
DK_OPEN='{
    "hosts": ["fd://", "tcp://0.0.0.0:2375"],
    "insecure-registries": ["192.168.10.201:5000"],
    "runtimes": {
        "nvidia": {
            "args": [],
            "path": "nvidia-container-runtime"
        }
    }
}'
DK_CLOSED='{
    "hosts": ["fd://"],
    "insecure-registries": ["192.168.10.201:5000"],
    "runtimes": {
        "nvidia": {
            "args": [],
            "path": "nvidia-container-runtime"
        }
    }
}'
seed_dk() { printf '%s\n' "$DK_OPEN" > "$DK/daemon.json"; chmod 640 "$DK/daemon.json"; : > "$DK/listen"
            printf 'watchtower\nsprechloop-dev\nopen-webui\n' > "$DK/names"; cp "$DK/names" "$DK/names.before"
            rm -f "$DK/restarts" "$DK"/daemon.json.bak-*; }
seed_dk; out=$(run --dry-run); x=$(s27 "$out")
if echo "$x" | grep_all '\[DRY\]   would remove tcp://0.0.0.0:2375 from .*daemon.json hosts .* the 3 running container(s) come back' \
   && [ "$(cat "$DK/daemon.json")" = "$DK_OPEN" ] && [ ! -e "$DK/restarts" ] && ! ls "$DK" | grep_all bak-; then
    ok "27: --dry-run names the tcp host and the 3 containers, changes nothing, no restart"
else bad "27 dry-run" "$x"; fi
seed_dk; out=$(run); rc=$?; x=$(s27 "$out"); bk=$(ls "$DK"/daemon.json.bak-* 2>/dev/null | head -1)
if [ "$rc" = 0 ] && echo "$x" | grep_all 'OK      :2375 closed (dockerd on its local socket only), docker active, .docker ps. works, all 3 container(s) running again' \
   && [ "$(cat "$DK/daemon.json")" = "$DK_CLOSED" ] && [ -n "$bk" ] && [ "$(cat "$bk")" = "$DK_OPEN" ] \
   && [ "$(stat -c %a "$bk")" = 640 ] && [ "$(stat -c %a "$DK/daemon.json")" = 640 ] && [ "$(wc -l < "$DK/restarts")" = 1 ]; then
    ok "27: open -> closed: only the tcp host removed (rest byte-for-byte), mode kept, backup holds the original, one restart, all containers back"
else bad "27 close" "rc=$rc: $x / $(cat "$DK/daemon.json")"; fi
out=$(run); rc=$?; x=$(s27 "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all 'skip    .*daemon.json has no tcp host and nothing listens on :2375 (already closed)' && [ "$(wc -l < "$DK/restarts")" = 1 ]; then
    ok "27: second run skips (idempotent: no rewrite, no restart)"
else bad "27 idempotent" "rc=$rc: $x"; fi
seed_dk; out=$(FAKE_DK_PORT_STAYS=1 run); rc=$?; x=$(s27 "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all 'FAILED  something still listens on :2375 after the restart. Roll back: cp -p .*daemon.json.bak-.* && systemctl restart docker'; then
    ok "27: restart leaves the port open => FAILED, rc 1, prints the rollback with the backup path"
else bad "27 port stays" "rc=$rc: $x"; fi
seed_dk; out=$(FAKE_DK_LOST=sprechloop-dev run); rc=$?; x=$(s27 "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all 'FAILED  :2375 is closed, but 1 of 3 container(s) did not come back: sprechloop-dev\. .*roll back: cp -p'; then
    ok "27: a container that does not come back => FAILED, named, with the rollback"
else bad "27 lost container" "rc=$rc: $x"; fi
seed_dk; out=$(FAKE_DK_EXECARGS="-H fd:// --containerd=/run/containerd/containerd.sock" run); rc=$?; x=$(s27 "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all 'FAILED  docker.s ExecStart passes -H/--host' && [ "$(cat "$DK/daemon.json")" = "$DK_OPEN" ] && [ ! -e "$DK/restarts" ]; then
    ok "27: ExecStart with -H (no clearing drop-in) => refused before any change, no restart"
else bad "27 -H" "rc=$rc: $x"; fi
seed_dk; out=$(FAKE_DK_INACTIVE=1 run); rc=$?; x=$(s27 "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all 'FAILED  docker is not active after the restart. Roll back:'; then
    ok "27: docker not active after restart => FAILED with rollback"
else bad "27 inactive" "rc=$rc: $x"; fi
seed_dk; out=$(FAKE_DK_PS_BROKEN=1 run); rc=$?; x=$(s27 "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all "FAILED  'docker ps' does not work before the change; nothing changed" && [ "$(cat "$DK/daemon.json")" = "$DK_OPEN" ] && [ ! -e "$DK/restarts" ]; then
    ok "27: docker ps broken beforehand => FAILED, nothing changed"
else bad "27 ps broken" "rc=$rc: $x"; fi
seed_dk; printf '{\n    "hosts": ["fd://"]\n}\n' > "$DK/daemon.json"; out=$(run); rc=$?; x=$(s27 "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all 'FAILED  .*has no tcp host, yet something listens on :2375; not restarting docker blind' && [ ! -e "$DK/restarts" ]; then
    ok "27: no tcp host but port open => FAILED, no blind restart"
else bad "27 unexplained listener" "rc=$rc: $x"; fi
printf '{\n    "hosts": ["fd://"]\n}\n' > "$DK/daemon.json"; rm -f "$DK/listen" "$DK/restarts" "$DK"/daemon.json.bak-*

# ---------------------------------------------------------------------------
# T-3685 action 28 — a long-lived Claude token for testdev (operator ruling A).
# ---------------------------------------------------------------------------
echo "T-3685: 28 testdev's long-lived Claude token"
s28() { echo "$1" | sed -n '/^.*28\. A long-lived Claude token/,/^.*Verification/p'; }
TDF="$TD/home/.config/claude/oauth-token"
expire_td() { printf 'sk-ant-oat01-EXPIRED\n' > "$TDF"; chmod 600 "$TDF"; rm -f "$TD/home/.profile" "$TD/tty-out"; }
out=$(run --dry-run); x=$(s28 "$out")
if echo "$x" | grep_all "skip    testdev's stored token answers"; then ok "28: a working token is skipped"
else bad "28 skip" "$x"; fi
expire_td; out=$(run --dry-run); x=$(s28 "$out")
if echo "$x" | grep_all "\[DRY\]   would run 'claude setup-token' as testdev" && [ "$(cat "$TDF")" = "sk-ant-oat01-EXPIRED" ] && [ ! -e "$TD/home/.profile" ]; then
    ok "28: --dry-run with an expired token describes the step and changes nothing"
else bad "28 dry-run" "$x"; fi
expire_td; out=$(RUNME_TESTDEV_TTY_IN="$TD/no-such-tty" run); rc=$?; x=$(s28 "$out")
if echo "$x" | grep_all "HELD    no terminal to ask you on" && [ "$(cat "$TDF")" = "sk-ant-oat01-EXPIRED" ]; then
    ok "28: no terminal => HELD, nothing stored"
else bad "28 no tty" "rc=$rc: $x"; fi
expire_td; printf 'hello there\n' > "$TD/tty-in"; out=$(run); rc=$?; x=$(s28 "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all "FAILED  that does not look like a setup-token" && [ "$(cat "$TDF")" = "sk-ant-oat01-EXPIRED" ] && ! grep_all 'T-3685: long-lived Claude token' "$TD/home/.profile"; then
    ok "28: a paste that is not a setup-token => FAILED, nothing stored"
else bad "28 bad paste" "rc=$rc: $x"; fi
expire_td; : > "$TD/tty-in"; out=$(run); rc=$?; x=$(s28 "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all "FAILED  no token pasted; nothing stored"; then ok "28: an empty paste => FAILED"
else bad "28 empty paste" "rc=$rc: $x"; fi
expire_td; printf '  %s  \n' "$TD_GOOD" > "$TD/tty-in"; out=$(run); rc=$?; x=$(s28 "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all "OK      testdev has its own long-lived token" && [ "$(cat "$TDF")" = "$TD_GOOD" ] \
   && [ "$(stat -c %a "$TDF")" = 600 ] && [ "$(stat -c %a "$(dirname "$TDF")")" = 700 ] \
   && grep_all -c 'T-3685: long-lived Claude token for testdev' "$TD/home/.profile" \
   && grep_all 'export CLAUDE_CODE_OAUTH_TOKEN=' "$TD/home/.profile" && grep_all 'fake setup-token: open' "$TD/tty-out" \
   && [ -z "$(ls -A "$TD/rec" 2>/dev/null)" ]; then
    ok "28: a good paste (whitespace trimmed) is stored 600 in a 700 dir, exported from .profile, verified OK; setup-token talked to the terminal"
else bad "28 good paste" "rc=$rc: $x"; fi
# Absence check that could have looked (T-3144): the log dir must exist and hold this run's log.
if [ -d "$RUNME_LOG_DIR" ] && ls "$RUNME_LOG_DIR"/runme-*.log >/dev/null 2>&1 \
   && ! echo "$out" | grep_all -F "$TD_GOOD" && ! grep -rlF "$TD_GOOD" "$RUNME_LOG_DIR" >/dev/null 2>&1; then
    ok "28: the token appears nowhere in the run's output or the run logs"
else bad "28 token leak" "the token appeared in stdout or in $RUNME_LOG_DIR (or that dir holds no log)"; fi
printf '%s\n' "$TD_GOOD" > "$TD/tty-in"; printf 'sk-ant-oat01-EXPIRED\n' > "$TDF"; out=$(run); x=$(s28 "$out")
if [ "$(grep -c 'T-3685: long-lived Claude token for testdev' "$TD/home/.profile")" = 1 ]; then
    ok "28: a second store does not add the .profile line twice"
else bad "28 profile twice" "$(cat "$TD/home/.profile")"; fi
# Run of 2026-11-11: token fine, but /etc/environment's root-owned temp dir broke testdev's claude.
printf '%s\n' "$TD_GOOD" > "$TDF"; chmod 600 "$TDF"; rm -f "$TD/tty-in" "$TD/tty-out" "$TD/home/.profile"
out=$(run); rc=$?; x=$(s28 "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all "OK      testdev's Claude temp dir is now its own" && grep_all -F "$TD_TMPLINE" "$TD/home/.profile" \
   && ! echo "$x" | grep_all 'HELD\|Paste the token'; then
    ok "28: a good stored token behind the root-owned temp dir => temp dir line added, OK, no new paste asked"
else bad "28 tmp dir" "rc=$rc: $x"; fi
printf '%s\n' "$TD_TMPLINE" > "$TD/home/.profile"
printf 'sk-ant-oat01-EXPIRED\n' > "$TDF"; printf '%s\n' "sk-ant-oat01-STILLBAD" > "$TD/tty-in"; out=$(run); x=$(s28 "$out")
if echo "$x" | grep_all 'FAILED  token stored, but a prompt as testdev did not answer OK (it said: Failed to authenticate' && ! echo "$x" | grep_all 'STILLBAD'; then
    ok "28: a failed check names what testdev's claude said, with any token masked"
else bad "28 failed check detail" "$x"; fi
# Runs of 2026-11-11: setup-token printed the token wrapped over three lines (ANSI around it);
# pasting kept only the first line (43 chars, 401). Now it is taken from the recording, joined.
expire_td; printf '%s\n' "$TD_TMPLINE" > "$TD/home/.profile"; : > "$TD/tty-in"
printf '\033[1mYour OAuth token (valid for 1 year):\033[0m\r\n\r\n  \033[32m%s\r\n  %s\r\n  %s\033[0m\r\n\r\nStore this token securely.\r\n' \
    "${TD_GOOD:0:16}" "${TD_GOOD:16:12}" "${TD_GOOD:28}" > "$TD/setup-out-wrapped"
out=$(FAKE_TD_SETUP_OUT="$TD/setup-out-wrapped" run); rc=$?; x=$(s28 "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all "OK      testdev has its own long-lived token" && [ "$(cat "$TDF")" = "$TD_GOOD" ] \
   && ! grep_all 'was not picked up' "$TD/tty-out" && [ -z "$(ls -A "$TD/rec" 2>/dev/null)" ] && ! echo "$out" | grep_all -F "${TD_GOOD:16:12}"; then
    ok "28: a token wrapped over 3 lines with colour codes is picked up whole from the recording, no paste asked, recording deleted, not in output"
else bad "28 wrapped token" "rc=$rc: $x / tty: $(cat "$TD/tty-out" 2>/dev/null | tail -2)"; fi
expire_td; printf '%s\n' "$TD_TMPLINE" > "$TD/home/.profile"; printf '%s\n%s\n' "${TD_GOOD:0:20}" "${TD_GOOD:20}" > "$TD/tty-in"
out=$(run); rc=$?; x=$(s28 "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all "OK      testdev has its own long-lived token" && [ "$(cat "$TDF")" = "$TD_GOOD" ] && grep_all 'was not picked up' "$TD/tty-out"; then
    ok "28: no token in the recording => paste asked; a paste over 2 lines is joined whole"
else bad "28 two-line paste" "rc=$rc: $x"; fi
printf '%s\n' "$TD_GOOD" > "$TDF"; chmod 600 "$TDF"; rm -f "$TD/tty-in" "$TD/tty-out"
printf '%s\nexport CLAUDE_CODE_TMPDIR="$HOME/.cache/claude-tmp"\n' "$TD_TMPLINE" > "$TD/home/.profile"

# ---- T-3647: action 4b restarts W1 sidecars running older code -------------------
echo "T-3647: 4b W1 sidecars onto the installed binary"
s4b() { echo "$1" | sed -n '/^.*4b\. W1 sidecars onto the installed binary/,/^.*17\. arc-011 W1/p'; }
: > "$W1R/termlink-new"; : > "$W1R/termlink-old"
cat > "$W1R/systemctl" <<'EOS'
#!/usr/bin/env bash
D="${FAKE_W1_DIR:?}"; u="${@: -1}"; f="$D/pid.$u"
case "$1" in
  show) cat "$f" 2>/dev/null || echo 0 ;;
  is-active) echo active ;;
  restart) echo "$u" >> "$D/restarts"; n=$(( $(cat "$f") + 100 )); echo "$n" > "$f"
           mkdir -p "$D/proc/$n"
           if [ "${FAKE_W1_STALE_AFTER:-0}" = 1 ]; then ln -sf "$D/termlink-old" "$D/proc/$n/exe"; else ln -sf "$D/termlink-new" "$D/proc/$n/exe"; fi ;;
esac
EOS
chmod +x "$W1R/systemctl"
w1_unit() { echo "$2" > "$W1R/pid.$1"; mkdir -p "$W1R/proc/$2"; ln -sf "$W1R/$3" "$W1R/proc/$2/exe"; }
export FAKE_W1_DIR="$W1R"
rm -f "$W1R/restarts"; w1_unit u-main 11 termlink-old; w1_unit u-alt 12 termlink-new
out=$(RUNME_W1_UNITS="u-main u-alt" run); x=$(s4b "$out")
if echo "$x" | grep_all 'OK      u-main restarted and verified: active, pid 111 runs the installed binary' \
   && echo "$x" | grep_all 'skip    u-alt already runs the installed binary' && [ "$(cat "$W1R/restarts")" = u-main ]; then
    ok "T-3647: stale unit restarted and verified, current unit skipped (only one restart)"
else bad "T-3647 restart" "$x"; fi
out=$(RUNME_W1_UNITS="u-main u-alt" run); x=$(s4b "$out")
if echo "$x" | grep_all 'skip    u-main already runs' && [ "$(wc -l < "$W1R/restarts")" = 1 ]; then
    ok "T-3647: second run skips both (idempotent)"
else bad "T-3647 idempotent" "$x"; fi
rm -f "$W1R/restarts"; w1_unit u-main 21 termlink-old
out=$(FAKE_W1_STALE_AFTER=1 RUNME_W1_UNITS="u-main" run); rc=$?; x=$(s4b "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all 'FAILED  u-main after restart: not active or not on the installed binary'; then
    ok "T-3647: restart that leaves old code => FAILED, rc 1"
else bad "T-3647 still-stale" "rc=$rc: $x"; fi
rm -f "$W1R/pid.u-none"; out=$(RUNME_W1_UNITS="u-none" run); x=$(s4b "$out")
if echo "$x" | grep_all 'skip    u-none not running'; then ok "T-3647: unit not running => skip, nothing restarted"
else bad "T-3647 absent" "$x"; fi
rm -f "$W1R"/pid.* "$W1R/restarts"

# ---- T-3651: action 4c lists or re-pins the W1 main, audited ----------------------
echo "T-3651: 4c W1 pinned main re-pin"
s4c() { echo "$1" | sed -n '/^.*4c\. W1 pinned main/,/^.*4d\. W1 sidecars know/p'; }
# The fake live sidecar: `restart` (fake systemctl above) re-reads nothing, so the live pin is
# kept in live.<agent> and refreshed from the config here on restart unless FAKE_W1_LIVE_STALE=1.
cat > "$W1R/status-tl" <<'EOS'
#!/usr/bin/env bash
D="${FAKE_W1_DIR:?}"; a=$(basename "${TERMLINK_SIDECAR_STATUS_SOCKET:?}" .status.sock)
[ -f "$D/live.$a" ] || { echo '{"ok":false,"error":"no socket"}'; exit 1; }
printf '{"ok":true,"main":{"pinned":"%s","liveness":{"state":"alive"}}}\n' "$(cat "$D/live.$a")"
EOS
chmod +x "$W1R/status-tl"
mv "$W1R/systemctl" "$W1R/systemctl.4b"
cat > "$W1R/systemctl" <<'EOS'
#!/usr/bin/env bash
D="${FAKE_W1_DIR:?}"; u="${@: -1}"
"$D/systemctl.4b" "$@"; rc=$?
if [ "$1" = restart ] && [ "${FAKE_W1_LIVE_STALE:-0}" != 1 ]; then
    python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["book"]["main"])' "$D/conf/$u.json" > "$D/live.$u" 2>/dev/null
fi
exit $rc
EOS
chmod +x "$W1R/systemctl"
w1_conf() {  # <agent> <peer> <main>
    printf '{\n  "agent": "%s",\n  "book": {\n    "main": "%s",\n    "peers": {\n      "%s": "%s.w1"\n    }\n  }\n}' "$1" "$3" "$2" "$2" > "$W1R/conf/$1.json"
    chmod 0644 "$W1R/conf/$1.json"; echo "$3" > "$W1R/live.$1"
}
w1_reset4c() { rm -f "$W1R"/restarts "$W1R"/repins.jsonl "$W1R"/conf/*; w1_conf u-main u-alt u-main; w1_conf u-alt u-main u-main
    w1_unit u-main 31 termlink-new; w1_unit u-alt 32 termlink-new; }
w1_reset4c
out=$(RUNME_W1_UNITS="u-main u-alt" run); x=$(s4c "$out")
if echo "$x" | grep_all 'list    u-main: main pinned to u-main (no re-pin recorded' && echo "$x" | grep_all 'list    u-alt: main pinned to u-main' \
   && [ ! -e "$W1R/restarts" ] && [ ! -e "$W1R/repins.jsonl" ] && ! ls "$W1R/conf" | grep_all bak; then
    ok "T-3651: empty W1_REPIN_MAIN => list-only, nothing restarted, written or audited"
else bad "T-3651 list-only" "$x"; fi
out=$(RUNME_W1_REPIN_MAIN="u-alt|operator 2026-11-02, T-3651 fixture" RUNME_W1_UNITS="u-main u-alt" run); rc=$?; x=$(s4c "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all 'OK      u-main: main re-pinned u-main -> u-alt; u-main restarted (pid 131) and status reports u-alt' \
   && echo "$x" | grep_all 'OK      u-alt: main re-pinned u-main -> u-alt' \
   && [ "$(w1_m=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["book"]["main"])' "$W1R/conf/u-alt.json"); echo "$w1_m")" = u-alt ] \
   && [ "$(ls "$W1R/conf" | grep -c '^u-main.json.bak-')" = 1 ] && [ "$(stat -c %a "$W1R/conf/u-main.json")" = 644 ] \
   && [ "$(wc -l < "$W1R/repins.jsonl")" = 1 ] \
   && python3 -c 'import json,sys; e=json.loads(open(sys.argv[1]).readline()); sys.exit(0 if e["to"]=="u-alt" and e["from"]=="u-main=u-main,u-alt=u-main" and e["recorded"].startswith("operator 2026-11-02") and e["outcome"]=="ok" and e["ts"] else 1)' "$W1R/repins.jsonl"; then
    ok "T-3651: recorded re-pin => both configs backed up + rewritten, restarted, status verified, one audit line"
else bad "T-3651 re-pin" "rc=$rc: $x"; fi
out=$(RUNME_W1_REPIN_MAIN="u-alt|operator 2026-11-02, T-3651 fixture" RUNME_W1_UNITS="u-main u-alt" run); rc=$?; x=$(s4c "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all 'skip    u-main: main already pinned to u-alt (config and live status)' \
   && echo "$x" | grep_all 'skip    u-alt: main already pinned' && [ "$(wc -l < "$W1R/restarts")" = 2 ] && [ "$(wc -l < "$W1R/repins.jsonl")" = 1 ]; then
    ok "T-3651: second run with the same recorded pin skips (idempotent, no restart, no audit line)"
else bad "T-3651 idempotent" "rc=$rc: $x"; fi
out=$(RUNME_W1_REPIN_MAIN="u-alt|x" RUNME_W1_UNITS="u-main u-alt" run --dry-run); x=$(s4c "$out")
if echo "$x" | grep_all 'skip    u-main: main already pinned'; then ok "T-3651: dry-run reads, changes nothing"; else bad "T-3651 dry" "$x"; fi
w1_reset4c
out=$(FAKE_W1_LIVE_STALE=1 RUNME_W1_REPIN_MAIN="u-alt|operator, fixture" RUNME_W1_UNITS="u-main" run); rc=$?; x=$(s4c "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all 'FAILED  u-main: after restart status reports main=u-main, not u-alt' \
   && python3 -c 'import json,sys; sys.exit(0 if json.loads(open(sys.argv[1]).readline())["outcome"]=="failed" else 1)' "$W1R/repins.jsonl"; then
    ok "T-3651: status not reporting the new pin => FAILED, rc 1, audit line says failed"
else bad "T-3651 verify-miss" "rc=$rc: $x"; fi
w1_reset4c
out=$(RUNME_W1_REPIN_MAIN="u-ghost|operator, fixture" RUNME_W1_UNITS="u-main u-alt" run); rc=$?; x=$(s4c "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all 're-pin refused: u-ghost is neither u-main nor a peer' && [ ! -e "$W1R/restarts" ] \
   && ! ls "$W1R/conf" | grep_all bak && [ ! -e "$W1R/repins.jsonl" ]; then
    ok "T-3651: pin to an agent outside the book => refused before any change"
else bad "T-3651 unknown agent" "rc=$rc: $x"; fi
out=$(RUNME_W1_REPIN_MAIN="u-alt" RUNME_W1_UNITS="u-main u-alt" run); rc=$?; x=$(s4c "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all 'names no recorder' && [ ! -e "$W1R/restarts" ]; then
    ok "T-3651: re-pin with no recorder => refused, nothing changed"
else bad "T-3651 no recorder" "rc=$rc: $x"; fi
rm -f "$W1R"/pid.* "$W1R"/live.* "$W1R/restarts" "$W1R/repins.jsonl" "$W1R"/conf/*

# ---- T-3654: action 4d gives the W1 sidecars their project (main.lease readable) ----
echo "T-3654: 4d W1 sidecars know their project"
s4d() { echo "$1" | sed -n '/^.*4d\. W1 sidecars know their project/,/^.*5\. /p'; }
# Fake live sidecar: lease.<agent> holds "state|project|why"; status reports it as main.lease.
cat > "$W1R/status-tl" <<'EOS'
#!/usr/bin/env bash
D="${FAKE_W1_DIR:?}"; a=$(basename "${TERMLINK_SIDECAR_STATUS_SOCKET:?}" .status.sock)
[ -f "$D/lease.$a" ] || { echo '{"ok":false,"error":"no socket"}'; exit 1; }
IFS='|' read -r st pr wy < "$D/lease.$a"
python3 -c 'import json,sys; s,p,w=sys.argv[1:4]; l={"state":s}; p and l.update(project=p); w and l.update(why=w); print(json.dumps({"ok":True,"main":{"pinned":"","lease":l}}))' "$st" "$pr" "$wy"
EOS
chmod +x "$W1R/status-tl"
mv "$W1R/systemctl" "$W1R/systemctl.4c"
cat > "$W1R/systemctl" <<'EOS'
#!/usr/bin/env bash
# On restart the fake sidecar re-reads its env file: with the project line it reads the lease.
D="${FAKE_W1_DIR:?}"; u="${@: -1}"
"$D/systemctl.4c" "$@"; rc=$?
if [ "$1" = restart ]; then
    p=$(sed -n 's/^TERMLINK_MAIN_PROJECT=//p' "$D/conf/$u.env" 2>/dev/null)
    if [ -n "$p" ]; then echo "${FAKE_W1_LEASE_STATE:-vacant}|$p|${FAKE_W1_LEASE_WHY:-}" > "$D/lease.$u"
    else echo "unknown||no main_project configured for this sidecar" > "$D/lease.$u"; fi
fi
exit $rc
EOS
chmod +x "$W1R/systemctl"
w1_env() {  # <unit> [extra line]: a provisioned env file without (or with) a project line
    printf 'PROJECT_DIR=/x\nAGENT=%s\nHOME=/root\n' "$1" > "$W1R/conf/$1.env"
    [ -z "${2:-}" ] || echo "$2" >> "$W1R/conf/$1.env"
    chmod 0640 "$W1R/conf/$1.env"
}
w1_reset4d() { rm -f "$W1R"/restarts "$W1R"/lease.* "$W1R"/conf/*; w1_unit u-main 41 termlink-new; w1_unit u-alt 42 termlink-new
    w1_env u-main; w1_env u-alt
    echo "unknown||no main_project configured for this sidecar" > "$W1R/lease.u-main"; cp "$W1R/lease.u-main" "$W1R/lease.u-alt"; }
w1_reset4d
out=$(RUNME_W1_UNITS="u-main u-alt" run --dry-run); x=$(s4d "$out")
if echo "$x" | grep_all '\[DRY\]   would set TERMLINK_MAIN_PROJECT=010-termlink in .*/u-main.env and restart u-main' \
   && ! grep -q TERMLINK_MAIN_PROJECT "$W1R/conf/u-main.env" && [ ! -e "$W1R/restarts" ]; then
    ok "T-3654: dry-run names the change, writes and restarts nothing"
else bad "T-3654 dry" "$x"; fi
out=$(RUNME_W1_UNITS="u-main u-alt" run); rc=$?; x=$(s4d "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all 'OK      u-main: TERMLINK_MAIN_PROJECT=010-termlink; u-main restarted (pid 141); main.lease.state=vacant' \
   && echo "$x" | grep_all 'OK      u-alt: TERMLINK_MAIN_PROJECT=010-termlink' \
   && [ "$(grep -cx 'TERMLINK_MAIN_PROJECT=010-termlink' "$W1R/conf/u-main.env")" = 1 ] && [ "$(tail -1 "$W1R/conf/u-main.env")" = "TERMLINK_MAIN_PROJECT=010-termlink" ] \
   && [ "$(stat -c %a "$W1R/conf/u-main.env")" = 640 ] && [ "$(ls "$W1R/conf" | grep -c '^u-main.env.bak-')" = 1 ] \
   && [ "$(wc -l < "$W1R/restarts")" = 2 ]; then
    ok "T-3654: env without project => backed up, line appended (mode kept), restarted, lease state verified"
else bad "T-3654 set" "rc=$rc: $x"; fi
out=$(RUNME_W1_UNITS="u-main u-alt" run); rc=$?; x=$(s4d "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all 'skip    u-main: project 010-termlink set and status reports main.lease.state=vacant' \
   && [ "$(wc -l < "$W1R/restarts")" = 2 ] && [ "$(ls "$W1R/conf" | grep -c '^u-main.env.bak-')" = 1 ]; then
    ok "T-3654: second run skips (idempotent: no write, no restart)"
else bad "T-3654 idempotent" "rc=$rc: $x"; fi
w1_reset4d; w1_env u-main "TERMLINK_MAIN_PROJECT=010-termlink"
out=$(RUNME_W1_UNITS="u-main" run); rc=$?; x=$(s4d "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all 'OK      u-main: TERMLINK_MAIN_PROJECT=010-termlink; u-main restarted' \
   && ! ls "$W1R/conf" | grep_all bak && [ "$(wc -l < "$W1R/restarts")" = 1 ]; then
    ok "T-3654: env right but running sidecar predates it => restart only, no rewrite"
else bad "T-3654 restart-only" "rc=$rc: $x"; fi
w1_reset4d; w1_env u-main "TERMLINK_MAIN_PROJECT=010-termlink"; echo "unknown|010-termlink|hub unreachable" > "$W1R/lease.u-main"
out=$(RUNME_W1_UNITS="u-main" run); rc=$?; x=$(s4d "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all 'FAILED  u-main: project 010-termlink set but main.lease.state=unknown (hub unreachable); not restarted' \
   && [ ! -e "$W1R/restarts" ]; then
    ok "T-3654: project set but lease unknown for another reason => FAILED, rc 1, no restart"
else bad "T-3654 hub-down" "rc=$rc: $x"; fi
w1_reset4d
out=$(FAKE_W1_LEASE_STATE=unknown FAKE_W1_LEASE_WHY="not read yet" RUNME_W1_UNITS="u-main" run); rc=$?; x=$(s4d "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all 'FAILED  u-main: after restart main.lease.state=unknown (not read yet), not held or vacant'; then
    ok "T-3654: lease still unknown after restart => FAILED, rc 1"
else bad "T-3654 verify-miss" "rc=$rc: $x"; fi
w1_reset4d; w1_env u-main "TERMLINK_MAIN_PROJECT=other-project"
out=$(FAKE_W1_LEASE_STATE=held RUNME_W1_UNITS="u-main" run); rc=$?; x=$(s4d "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all 'main.lease.state=held' && ! grep -q other-project "$W1R/conf/u-main.env" \
   && [ "$(grep -c '^TERMLINK_MAIN_PROJECT=' "$W1R/conf/u-main.env")" = 1 ]; then
    ok "T-3654: a different project line is replaced, never duplicated"
else bad "T-3654 replace" "rc=$rc: $x"; fi
w1_reset4d; rm -f "$W1R/conf/u-alt.env"
out=$(RUNME_W1_UNITS="u-alt" run); x=$(s4d "$out")
if echo "$x" | grep_all 'skip    u-alt: no settings at' && [ ! -e "$W1R/restarts" ]; then ok "T-3654: no env file => skip, nothing restarted"
else bad "T-3654 absent" "$x"; fi
rm -f "$W1R"/pid.* "$W1R"/live.* "$W1R"/lease.* "$W1R/restarts" "$W1R"/conf/*

# ---- T-3698: action 4e installs a named release, switches the W1 units, rolls back ----
echo "T-3698: 4e release install / upgrade / rollback / restart (arc-011 P-8)"
s4e() { echo "$1" | sed -n '/^.*4e\. termlink release on \.107/,/^.*5a\. /p'; }
REL="$RUNME_REL_ROOT"; RST="$RUNME_REL_STATE"
# A local "release" per tag, shaped like release.yml's: artifacts + checksums.txt (sha256sum *).
rel_publish() {  # <tag>
    local d="$TMP/rel-pub/$1" a
    mkdir -p "$d"
    for a in termlink-linux-x86_64 termlink-linux-x86_64-static termlink-linux-aarch64; do
        printf '#!/usr/bin/env bash\n# %s\n[ "$1" = --version ] && { echo "termlink %s"; exit 0; }\nexit 1\n' "$a" "${1#v}" > "$d/$a"
    done
    ( cd "$d" && sha256sum termlink-* > checksums.txt )
}
rel_publish v9.9.1; rel_publish v9.9.2
# Fake systemctl: restart re-reads SIDECAR_BIN from the unit's env file (as systemd does) and
# the new process runs what it resolves to (or old code with FAKE_REL_STALE_AFTER=1).
# FAKE_REL_LOSE=1 drops one STORED message on restart, so the after-count is short.
mv "$W1R/systemctl" "$W1R/systemctl.4d"
cat > "$W1R/systemctl" <<'EOS'
#!/usr/bin/env bash
D="${FAKE_W1_DIR:?}"; u="${@: -1}"; f="$D/pid.$u"
case "$1" in
  show) cat "$f" 2>/dev/null || echo 0 ;;
  is-active) echo active ;;
  restart) echo "$u" >> "$D/restarts"; n=$(( $(cat "$f") + 100 )); echo "$n" > "$f"; mkdir -p "$D/proc/$n"
           b=$(sed -n 's/^SIDECAR_BIN=//p' "$D/conf/$u.env")
           if [ "${FAKE_REL_STALE_AFTER:-0}" = 1 ]; then ln -sf "$D/termlink-old" "$D/proc/$n/exe"; else ln -sf "$(readlink -f "$b")" "$D/proc/$n/exe"; fi
           [ "${FAKE_REL_LOSE:-0}" = 1 ] && [ -s "$D/kept.$u" ] && sed -i '$d' "$D/kept.$u" ;;
esac
exit 0
EOS
chmod +x "$W1R/systemctl"
# Fake sidecar client: status computes `binary` like tools.rs binary_check (sha of the running exe
# against <exe>/../../share/termlink/checksums.txt); admin drain/undrain keep kept.<agent> and write
# the manifest into the store. FAKE_REL_DRAIN=nocount|mismatch, FAKE_REL_STATUS=<verdict> are controls.
cat > "$W1R/status-tl" <<'EOS'
#!/usr/bin/env bash
D="${FAKE_W1_DIR:?}"
if [ "$1 $2" = "sidecar status" ]; then
    a=$(basename "${TERMLINK_SIDECAR_STATUS_SOCKET:?}" .status.sock)
    pid=$(cat "$D/pid.$a" 2>/dev/null) || { echo '{"ok":false,"error":"no socket"}'; exit 1; }
    exe=$(readlink -f "$D/proc/$pid/exe"); sha=$(sha256sum "$exe" | cut -d' ' -f1)
    rec="$(dirname "$(dirname "$exe")")/share/termlink/checksums.txt"
    if [ ! -r "$rec" ]; then v=no-record; elif awk -v s="$sha" '$1==s{f=1} END{exit !f}' "$rec"; then v=yes; else v=no; fi
    v="${FAKE_REL_STATUS:-$v}"
    printf '{"ok":true,"main":{"pinned":"","lease":{"state":"vacant","project":"010-termlink"}},"binary":{"path":"%s","sha256":"%s","record":"%s","verified":"%s"}}\n' "$exe" "$sha" "$rec" "$v"
    exit 0
fi
sub="$3"; shift 3; sock=""
while [ $# -gt 0 ]; do [ "$1" = --socket ] && sock="$2"; shift; done
a=$(basename "$sock" .admin.sock); st="$D/store/$a"; mkdir -p "$st"
echo "$sub" >> "$D/drains.$a"
case "$sub" in
  drain)
    touch "$st/receive.drained"; n=$(grep -c . "$D/kept.$a" 2>/dev/null); n=${n:-0}
    case "${FAKE_REL_DRAIN:-}" in
      nocount)  echo '{"ok":true,"drained":true}'; exit 0 ;;
      mismatch) printf '{"ok":false,"drained":true,"listed":%d,"kept":%d,"unreadable":["c1/x: digest"],"code":"count-mismatch"}\n' $((n+1)) "$n"; exit 1 ;;
    esac
    python3 -c 'import json,sys
k=[dict(zip(("conv","msg_id","from","seq"),l.split())) for l in open(sys.argv[1]) if l.strip()] if len(sys.argv)>2 and sys.argv[2]=="1" else []
json.dump({"at_ms":1,"why":"x","by":"admin","listed":len(k),"kept":k,"unreadable":[]},open(sys.argv[3],"w"))' "$D/kept.$a" "$([ -s "$D/kept.$a" ] && echo 1 || echo 0)" "$st/state.drain-manifest"
    printf '{"ok":true,"drained":true,"listed":%d,"kept":%d,"unreadable":[],"manifest":"state.drain-manifest"}\n' "$n" "$n" ;;
  undrain) rm -f "$st/receive.drained"; echo '{"ok":true,"drained":false}' ;;
esac
EOS
chmod +x "$W1R/status-tl"
rel_env() {  # <unit>: a provisioned env file on the dev binary
    printf 'AGENT=%s\nSIDECAR_CONFIG=/x/%s.json\nSIDECAR_BIN=%s\nTERMLINK_SIDECAR_STORE=%s\nTERMLINK_MAIN_PROJECT=010-termlink\n' \
        "$1" "$1" "$W1R/termlink-new" "$W1R/store/$1" > "$W1R/conf/$1.env"
    chmod 0640 "$W1R/conf/$1.env"; mkdir -p "$W1R/store/$1"
}
rel_reset() { rm -rf "$REL" "$RST" "$W1R"/restarts "$W1R"/drains.* "$W1R"/store "$W1R"/kept.* "$W1R"/conf/*
    w1_unit u-main 51 termlink-new; w1_unit u-alt 52 termlink-new; rel_env u-main; rel_env u-alt
    printf 'c1 m-0001 peer-a 3\nc2 m-0002 peer-b 7\n' > "$W1R/kept.u-main"; : > "$W1R/kept.u-alt"; }
rel_drained() { ls "$W1R"/store/*/receive.drained 2>/dev/null | grep -c .; }
rel_ok_dir() {  # <tag>: installed binary's sha256 is in that tag's checksums.txt
    awk -v s="$(sha256sum "$REL/$1/bin/termlink" | cut -d' ' -f1)" '$1==s{f=1} END{exit !f}' "$REL/$1/share/termlink/checksums.txt"; }
U2="u-main u-alt"
rel_reset

out=$(RUNME_W1_UNITS="$U2" run); rc=$?; x=$(s4e "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all 'list    no release named (REL_TAG is empty' && [ ! -e "$REL" ] && [ ! -e "$W1R/restarts" ]; then
    ok "T-3698: no release named => list-only, nothing installed or restarted"
else bad "T-3698 list-only" "rc=$rc: $x"; fi

out=$(RUNME_REL_TAG=v9.9.1 RUNME_W1_UNITS="$U2" run --dry-run); x=$(s4e "$out")
if echo "$x" | grep_all '\[DRY\]   would install release v9.9.1' && [ ! -e "$REL/v9.9.1" ] && [ ! -e "$W1R/restarts" ] && [ "$(rel_drained)" = 0 ]; then
    ok "T-3698: dry-run names the install, writes, drains and restarts nothing"
else bad "T-3698 dry" "$x"; fi

out=$(RUNME_REL_TAG=v9.9.1 RUNME_W1_UNITS="$U2" run); rc=$?; x=$(s4e "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all 'OK      installed release v9.9.1 into .*/v9.9.1 and verified' \
   && echo "$x" | grep_all 'OK      current -> v9.9.1 (previous: none)' \
   && echo "$x" | grep_all 'OK      u-main: runs v9.9.1 (pid 151), status binary verified against .*/v9.9.1/share/termlink/checksums.txt; 2 STORED kept and listed' \
   && echo "$x" | grep_all 'kept    c1/m-0001 from peer-a (seq 3)' && echo "$x" | grep_all 'kept    c2/m-0002 from peer-b (seq 7)' \
   && echo "$x" | grep_all 'OK      u-alt: runs v9.9.1 (pid 152)' && rel_ok_dir v9.9.1 \
   && [ "$(readlink "$REL/current")" = v9.9.1 ] && grep -qx "SIDECAR_BIN=$REL/current/bin/termlink" "$W1R/conf/u-main.env" \
   && [ "$(stat -c %a "$W1R/conf/u-main.env")" = 640 ] && [ "$(ls "$W1R/conf" | grep -c '^u-main.env.bak-')" = 1 ] \
   && [ "$(wc -l < "$W1R/restarts")" = 2 ] && [ "$(rel_drained)" = 0 ] && ! ls "$RST"/pending.* >/dev/null 2>&1 \
   && ls "$RST"/*-u-main-before.json "$RST"/*-u-main-after.json >/dev/null 2>&1; then
    ok "T-3698: install => real install.sh verifies checksums, current set, units drained, moved, verified in status, recounted, undrained"
else bad "T-3698 install" "rc=$rc: $x"; fi

out=$(RUNME_REL_TAG=v9.9.1 RUNME_W1_UNITS="$U2" run); rc=$?; x=$(s4e "$out"); y=$(s4b "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all 'skip    v9.9.1 already installed and verified' && echo "$x" | grep_all 'skip    current already -> v9.9.1' \
   && echo "$x" | grep_all 'skip    u-main already runs v9.9.1 (pid 151); status binary verified against' \
   && echo "$y" | grep_all 'skip    u-main already runs the installed binary' && [ "$(wc -l < "$W1R/restarts")" = 2 ] && [ "$(rel_drained)" = 0 ]; then
    ok "T-3698: second run skips (idempotent: no download, drain or restart; 4b sees the release as the installed binary)"
else bad "T-3698 idempotent" "rc=$rc: $x"; fi

out=$(RUNME_REL_TAG=v9.9.2 RUNME_W1_UNITS="$U2" run); rc=$?; x=$(s4e "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all 'OK      current -> v9.9.2 (previous: v9.9.1)' \
   && echo "$x" | grep_all 'OK      u-main: runs v9.9.2 (pid 251), status binary verified against .*/v9.9.2/share/termlink/checksums.txt; 2 STORED kept' \
   && echo "$x" | grep_all 'note    rollback = set REL_TAG to v9.9.1' && [ "$(readlink "$REL/previous")" = v9.9.1 ] \
   && [ "$(wc -l < "$W1R/restarts")" = 4 ] && [ "$(rel_drained)" = 0 ]; then
    ok "T-3698: upgrade => v9.9.2 current, v9.9.1 kept as previous, units moved and verified"
else bad "T-3698 upgrade" "rc=$rc: $x"; fi

out=$(RUNME_REL_BASE_URL="file://$TMP/no-such-release" RUNME_REL_TAG=v9.9.1 RUNME_W1_UNITS="$U2" run); rc=$?; x=$(s4e "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all 'skip    v9.9.1 already installed' && echo "$x" | grep_all 'OK      current -> v9.9.1 (previous: v9.9.2)' \
   && echo "$x" | grep_all 'OK      u-main: runs v9.9.1 (pid 351), .*2 STORED kept and listed before and after' \
   && echo "$x" | grep_all 'kept    c1/m-0001 from peer-a' && [ "$(readlink "$REL/previous")" = v9.9.2 ] && [ "$(rel_drained)" = 0 ]; then
    ok "T-3698: rollback => previous release, no download, STORED messages kept, counted and listed"
else bad "T-3698 rollback" "rc=$rc: $x"; fi

# E-81.5 control: a drain without the count compare (and one with a mismatch) is refused.
for mode in nocount mismatch; do
    n0=$(wc -l < "$W1R/restarts")
    out=$(FAKE_REL_DRAIN=$mode RUNME_REL_TAG=v9.9.2 RUNME_W1_UNITS="$U2" run); rc=$?; x=$(s4e "$out")
    case $mode in nocount) why='the drain reply has no count compare (listed/kept missing)' ;; mismatch) why='3 listed, 2 kept intact, 1 unreadable' ;; esac
    if [ "$rc" = 1 ] && echo "$x" | grep_all "FAILED  refused: u-main drain: $why. Nothing switched" \
       && [ "$(readlink "$REL/current")" = v9.9.1 ] && [ "$(wc -l < "$W1R/restarts")" = "$n0" ] \
       && [ "$(rel_drained)" = 0 ] && ! ls "$RST"/pending.* >/dev/null 2>&1; then
        ok "T-3698: drain $mode => refused, undrained, nothing switched or restarted (E-81.5 control)"
    else bad "T-3698 drain $mode" "rc=$rc: $x"; fi
done

# E-100 control: status `binary` not "yes" fails even with nothing to change.
out=$(FAKE_REL_STATUS=no RUNME_REL_TAG=v9.9.1 RUNME_W1_UNITS="$U2" run); rc=$?; x=$(s4e "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all 'FAILED  u-main runs v9.9.1 but status binary verified=no'; then
    ok "T-3698: status binary not verified => FAILED, rc 1 (checked every run)"
else bad "T-3698 status no" "rc=$rc: $x"; fi

# Restart: a unit found on old code is put back on the release (4b does it, 4e then verifies).
w1_unit u-alt 77 termlink-old
out=$(RUNME_REL_TAG=v9.9.1 RUNME_W1_UNITS="$U2" run); rc=$?; x=$(s4e "$out"); y=$(s4b "$out")
if [ "$rc" = 0 ] && echo "$y" | grep_all 'OK      u-alt restarted and verified: active, pid 177 runs the installed binary' \
   && echo "$x" | grep_all 'skip    u-alt already runs v9.9.1 (pid 177); status binary verified'; then
    ok "T-3698: restart => a unit on old code goes back onto the selected release and is verified"
else bad "T-3698 restart" "rc=$rc: $x $y"; fi

# Tamper: the installed copy no longer matches checksums.txt => flagged, reinstalled, FAILED once.
printf '# modified\n' >> "$REL/v9.9.1/bin/termlink"
out=$(RUNME_REL_TAG=v9.9.1 RUNME_W1_UNITS="$U2" run); rc=$?; x=$(s4e "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all 'FLAGGED v9.9.1 at .* does not verify: the sha256 of' \
   && echo "$x" | grep_all 'FAILED  v9.9.1 had been modified on disk; reinstalled and verified now' && rel_ok_dir v9.9.1; then
    ok "T-3698: modified release binary => flagged, reinstalled from the artifact, rc 1"
else bad "T-3698 tamper" "rc=$rc: $x"; fi
out=$(RUNME_REL_TAG=v9.9.1 RUNME_W1_UNITS="$U2" run); rc=$?
if [ "$rc" = 0 ]; then ok "T-3698: run after the reinstall is clean"; else bad "T-3698 after tamper" "rc=$rc: $(s4e "$out")"; fi

# A message lost across the move: the count compare fails, the unit stays drained (mail on the hub).
out=$(FAKE_REL_LOSE=1 RUNME_REL_TAG=v9.9.2 RUNME_W1_UNITS="$U2" run); rc=$?; x=$(s4e "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all 'FAILED  u-main: count compare: 2 STORED kept before, 1 after on v9.9.2; left drained' \
   && [ -e "$W1R/store/u-main/receive.drained" ] && [ -e "$RST/pending.u-main" ] && echo "$x" | grep_all 'OK      u-alt: runs v9.9.2'; then
    ok "T-3698: count short after the move => FAILED, left drained with its pending record"
else bad "T-3698 lost" "rc=$rc: $x"; fi
# Release unknown to the publisher: FAILED, nothing switched.
out=$(RUNME_REL_TAG=v9.9.9 RUNME_W1_UNITS="$U2" run); rc=$?; x=$(s4e "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all 'FAILED  release v9.9.9 did not install' && [ "$(readlink "$REL/current")" = v9.9.2 ]; then
    ok "T-3698: a release that cannot be fetched => FAILED, current unchanged"
else bad "T-3698 missing release" "rc=$rc: $x"; fi
rel_reset; rm -rf "$REL" "$RST" "$W1R"/pid.* "$W1R"/conf/* "$W1R"/kept.* "$W1R/restarts"

# ---------------------------------------------------------------------------
# T-3778 action 11a: vendored framework upgrade to v1.8.16-be.49 (scratch repos only).
# ---------------------------------------------------------------------------
s11a() { echo "$1" | sed -n '/11a\. Upgrade the vendored framework/,/11b\. Deploy/p'; }
AP="$AE/proj"
aefu_head() { git -C "$AP" rev-parse HEAD; }
aefu_untouched() {  # <head-before>: fresh project unchanged — VERSION, CLAUDE.md, HEAD, no stray files
    [ "$(cat "$AP/.agentic-framework/VERSION")" = "1.8.15+1.g${AEFU_A:0:9}" ] && [ "$(aefu_head)" = "$1" ] \
    && grep -qx 'project rule' "$AP/CLAUDE.md" && [ ! -e "$AP/.agentic-framework/lib/new-upstream.py" ] \
    && [ ! -e "$AP/.agentic-framework/lib/half-written.py" ] && [ -z "$(git -C "$AP" status --porcelain)" ]
}

out=$(run); rc=$?; x=$(s11a "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all "skip    vendored framework 1.8.15+2.g${AEFU_B:0:9} is at or past v1.8.16-be.49, merges in place, HOST 0.0.0.0, Watchtower answers"; then
    ok "T-3778: at be.49, merged, pushed, HOST set, Watchtower answering => skip"
else bad "T-3778 done skip" "rc=$rc: $x"; fi

aefu_state fresh; h0=$(aefu_head)
out=$(run --dry-run); rc=$?; x=$(s11a "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all "\[DRY\]   would upgrade 1.8.15+1.g${AEFU_A:0:9} -> v1.8.16-be.49 (${AEFU_B:0:9}, 1 commits)" \
   && aefu_untouched "$h0" && [ -z "$(ls "$AE/work")" ]; then
    ok "T-3778: --dry-run => says what it would do, clones and changes nothing"
else bad "T-3778 dry-run" "rc=$rc: $x"; fi

out=$(run); rc=$?; x=$(s11a "$out")
park="$AP/.context/working/vendor-upstream-v1816-be49/scripts/tool.sh.upstream"
if [ "$rc" = 0 ] && echo "$x" | grep_all "OK      upgraded to 1.8.15+2.g${AEFU_B:0:9}; CLAUDE.md project part and 1 preserved files byte-identical; 1 merged files installed" \
   && echo "$x" | grep_all 'OK      HOST 0.0.0.0 set in .framework.yaml' && echo "$x" | grep_all 'OK      pushed to origin' \
   && echo "$x" | grep_all 'OK      Watchtower restarted and answering at http://fixture.invalid:3003' \
   && echo "$x" | grep_all 'OK      fw doctor: no new FAIL' \
   && cmp -s "$AP/.agentic-framework/lib/keep.sh" "$AE/merges/lib/keep.sh.merged" && grep -qx 'HOST: 0.0.0.0' "$AP/.framework.yaml" \
   && git -C "$AP" log -1 --format=%s | grep_all '^T-3778: fw upgrade -> v1.8.16-be.49' \
   && [ "$(git -C "$AE/origin.git" rev-parse main)" = "$(aefu_head)" ] && [ -f "$park" ] && [ ! -e "$AP/scripts/tool.sh.upstream" ] \
   && git -C "$AP" ls-files --error-unmatch .agentic-framework/lib/new-upstream.py >/dev/null 2>&1 && [ -z "$(git -C "$AP" status --porcelain)" ]; then
    ok "T-3778: upgrade => at be.49, merge installed, HOST set, committed T-3778, pushed, .upstream parked, Watchtower answers, doctor clean"
else bad "T-3778 upgrade" "rc=$rc: $x"; fi

h1=$(aefu_head); out=$(run); rc=$?; x=$(s11a "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all 'skip    vendored framework' && [ "$(aefu_head)" = "$h1" ]; then
    ok "T-3778: second run skips (idempotent: no clone, no commit)"
else bad "T-3778 idempotent" "rc=$rc: $x"; fi

rm -f "$AE/wtup"; out=$(run); rc=$?; x=$(s11a "$out")
if [ "$rc" = 0 ] && echo "$x" | grep_all 'OK      Watchtower restarted and answering' && [ -e "$AE/wtup" ]; then
    ok "T-3778: Watchtower down after the upgrade => restarted and verified on the LAN URL"
else bad "T-3778 wt repair" "rc=$rc: $x"; fi

rm -f "$AE/wtup"; out=$(FAKE_AEFU_WT_DEAD=1 run); rc=$?; x=$(s11a "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all 'FAILED  Watchtower restarted but http://fixture.invalid:3003 does not answer'; then
    ok "T-3778: Watchtower not answering after a restart => FAILED, rc 1 (control)"
else bad "T-3778 wt dead" "rc=$rc: $x"; fi
: > "$AE/wtup"

sed -i '/^HOST:/d' "$AP/.framework.yaml"; out=$(FAKE_AEFU_NOHOST=1 run); rc=$?; x=$(s11a "$out")
if [ "$rc" = 1 ] && echo "$x" | grep_all "FAILED  fw config set HOST 0.0.0.0 ran but .framework.yaml has no 'HOST: 0.0.0.0'"; then
    ok "T-3778: HOST not written => FAILED, rc 1 (control)"
else bad "T-3778 nohost" "rc=$rc: $x"; fi

# Each control on a fresh project: refused or rolled back, rc 1, the project left as it was.
for c in clobber oldver keep uprc doctor; do
    aefu_state fresh; h0=$(aefu_head)
    case $c in
        clobber) out=$(FAKE_AEFU_CLOBBER=1 run); rc=$?; why="CLAUDE.md above '## Core Principle' changed (T-2015)" ;;
        oldver)  out=$(FAKE_AEFU_OLDVER=1 run); rc=$?; why="after the upgrade VERSION reads '1.8.15+1.g${AEFU_A:0:9}', not v1.8.16-be.49 or later" ;;
        keep)    out=$(FAKE_AEFU_TOUCH_KEEP=1 run); rc=$?; why="preserved lib/keep.sh was not kept byte-identical (T-4870)" ;;
        uprc)    out=$(FAKE_AEFU_UP_RC=3 run); rc=$?; why="fw upgrade exited 3" ;;
        doctor)  out=$(FAKE_AEFU_DOCTOR_FAIL=1 run); rc=$?; why="fw doctor has FAIL lines it did not have before the upgrade" ;;
    esac
    x=$(s11a "$out")
    if [ "$c" = doctor ]; then
        if [ "$rc" = 1 ] && echo "$x" | grep_all "FAILED  $why" && echo "$x" | grep_all 'FAIL  something new' \
           && git -C "$AP" log -1 --format=%s | grep_all '^T-3778: fw upgrade'; then
            ok "T-3778: a new fw doctor FAIL after the upgrade => FAILED and named, rc 1 (control)"
        else bad "T-3778 doctor" "rc=$rc: $x"; fi
    elif [ "$rc" = 1 ] && echo "$x" | grep_all "FAILED  $why" && echo "$x" | grep_all 'rolled back to HEAD' && aefu_untouched "$h0"; then
        ok "T-3778: $c => FAILED, every changed path rolled back, rc 1 (control)"
    else bad "T-3778 $c" "rc=$rc: $x"; fi
done

for c in div side dirty stale notag; do
    aefu_state fresh; h0=$(aefu_head); envs=()
    case $c in
        div)   envs=(FAKE_AEFU_DIV_DIRTY=1); why="check-vendor-divergence.sh is not clean" ;;
        side)  aefu_state fresh "$AEFU_S"; h0=$(aefu_head); why="vendored commit ${AEFU_S:0:9} is not an ancestor of v1.8.16-be.49" ;;
        dirty) echo "local edit" >> "$AP/.agentic-framework/lib/keep.sh"; why="uncommitted changes under .agentic-framework" ;;
        stale) echo "another local fix" >> "$AP/.agentic-framework/lib/keep.sh"; aefu_g -C "$AP" commit -qam "T-1: later fix"; h0=$(aefu_head)
               why="vendored lib/keep.sh is not the copy the stored merge was made from (re-merge needed)" ;;
        notag) git -C "$AE/src" tag -d v1.8.16-be.49 >/dev/null; why="v1.8.16-be.49 is not in $AE/src" ;;
    esac
    out=$(env "${envs[@]}" RUNME_CRON_DIR="$TMP/cron" bash "$RUNME" 2>&1); rc=$?; x=$(s11a "$out")
    [ "$c" = notag ] && git -C "$AE/src" tag v1.8.16-be.49 "$AEFU_B"
    if [ "$rc" = 1 ] && echo "$x" | grep_all "FAILED  .*$why" && [ "$(aefu_head)" = "$h0" ] && [ -z "$(ls "$AE/work")" ] \
       && [ ! -e "$AP/.agentic-framework/lib/new-upstream.py" ]; then
        ok "T-3778: $c => refused before changing anything, rc 1 (control)"
    else bad "T-3778 refuse $c" "rc=$rc: $x"; fi
done
aefu_state done

echo ""
echo "----------------------------------------"
printf 'T-3052 fixtures: %d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" = "0" ] || exit 1
exit 0
