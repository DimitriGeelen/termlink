#!/usr/bin/env bash
# arc-011 PH-1 demonstration (architecture note section 8.1; R-69: the demo is the test).
#
#   sudo bash scripts/arc011-ph1-demo.sh [--report FILE] [--no-probes] [--keep]
#   bash scripts/arc011-ph1-demo.sh --scripted
#
# Default (B14, exits X1, X3 and X4 of step 3 section 5.9): the two-agent acceptance.
#   1. Builds the hub, the sidecar and the acceptance test.
#   2. Runs `acceptance_two_real_agents` (crates/termlink-sidecar/tests/acceptance.rs) as
#      the unprivileged account `testdev` (E-7), under `strace -f`: a throwaway estate
#      with its own hub, keys and two sidecars; two real Claude Code sessions (testdev's
#      own login, /home/testdev/.local/bin/claude) talk through them, BV-1..BV-7.
#   3. X4: lists every path opened, created, renamed or removed and every socket
#      connected outside the estate, and fails on any production path (root's HOME,
#      /var/lib/termlink, /tmp/termlink-0, /run/user/0, the repository outside target/,
#      another account's HOME, the production hub port 9100).
#      The run also shows the OQ-2 ruling E live: a third agent outside B's roster
#      knocks, is held, the operator stand-in admits it on the admin socket, and B gets
#      it at its next tool call (T-3418).
#   4. X3: runs the tests of the 17 floor items (step 3 section 5.7 plus SI-18 with the
#      allow-list and SI-23), one by one, as root (two of them need root).
#   5. Writes the report (default docs/reports/arc011-ph1-acceptance.md) and, beside it,
#      the run's evidence file (<report>.evidence.json).
# Needs root: to run the test as testdev, to strace it, and for the root-only probes.
# The working dir /tmp/arc011-acc-<time> is deleted at the end unless --keep.
#
# --scripted: `hooks_scripted_demo` only (B10): scripted hook input, no real agents.
#
# Env: ARC011_AGENT_USER (default testdev), ARC011_CLAUDE (default
# /home/<user>/.local/bin/claude), ARC011_MODEL (default claude-sonnet-5-5).
#
# Exit codes: 0 demo passed, 1 demo failed, 2 usage or tooling.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT="$ROOT/docs/reports/arc011-ph1-acceptance.md"
SCRIPTED=0
PROBES=1
KEEP=0
while [ $# -gt 0 ]; do
  case "$1" in
    --scripted) SCRIPTED=1; shift ;;
    --report) REPORT="${2:?--report needs a file}"; shift 2 ;;
    --no-probes) PROBES=0; shift ;;
    --keep) KEEP=1; shift ;;
    -h|--help) sed -n '2,31p' "$0"; exit 0 ;;
    *) echo "usage: $0 [--scripted] [--report FILE] [--no-probes] [--keep]" >&2; exit 2 ;;
  esac
done

command -v cargo >/dev/null 2>&1 || { echo "arc011-ph1-demo: cargo not found" >&2; exit 2; }
cd "$ROOT" || exit 2
# The estate starts a throwaway hub from the workspace's own termlink build
# (newest of target/debug and target/release, or $TERMLINK_BIN); build one if none.
if [ -z "${TERMLINK_BIN:-}" ] && [ ! -x target/debug/termlink ] && [ ! -x target/release/termlink ]; then
  cargo build -q -p termlink --bin termlink || { echo "arc011-ph1-demo: cannot build termlink" >&2; exit 2; }
fi

if [ "$SCRIPTED" = 1 ]; then
  if cargo test -q -p termlink-sidecar --test hooks_tests hooks_scripted_demo -- --exact --nocapture; then
    echo "arc011-ph1-demo --scripted: PASS"
    exit 0
  fi
  echo "arc011-ph1-demo --scripted: FAIL" >&2
  exit 1
fi

# ---- the two-agent acceptance ---------------------------------------------------------
die() { echo "arc011-ph1-demo: $*" >&2; exit 2; }
[ "$(id -u)" = 0 ] || die "needs root (runs the agents as the unprivileged account under strace)"
AGENT_USER="${ARC011_AGENT_USER:-testdev}"
id "$AGENT_USER" >/dev/null 2>&1 || die "no account $AGENT_USER (OQ-3)"
AGENT_HOME="$(getent passwd "$AGENT_USER" | cut -d: -f6)"
CLAUDE="${ARC011_CLAUDE:-$AGENT_HOME/.local/bin/claude}"
runuser -u "$AGENT_USER" -- test -x "$CLAUDE" || die "$CLAUDE is not executable by $AGENT_USER"
command -v strace >/dev/null 2>&1 || die "strace not found (the X4 path audit needs it)"
command -v python3 >/dev/null 2>&1 || die "python3 not found (renders the report)"

if [ -z "${TERMLINK_BIN:-}" ]; then
  for b in target/debug/termlink target/release/termlink; do
    if [ -x "$b" ] && { [ -z "${TERMLINK_BIN:-}" ] || [ "$b" -nt "$TERMLINK_BIN" ]; }; then TERMLINK_BIN="$ROOT/$b"; fi
  done
fi
echo "arc011-ph1-demo: building the sidecar and the acceptance test..."
ACC_BIN="$(cargo test -q -p termlink-sidecar --test acceptance --no-run --message-format=json 2>/dev/null | python3 -c '
import json, sys
for l in sys.stdin:
    try: m = json.loads(l)
    except ValueError: continue
    if m.get("reason") == "compiler-artifact" and m.get("target", {}).get("name") == "acceptance" and m.get("executable"):
        print(m["executable"])')"
[ -n "$ACC_BIN" ] && [ -x "$ACC_BIN" ] || die "cannot build the acceptance test"

W="/tmp/arc011-acc-$(date -u +%Y%m%dT%H%M%SZ)"
mkdir -p "$W" || die "cannot create $W"
chown "$AGENT_USER:" "$W"
cleanup() { [ "$KEEP" = 1 ] || rm -rf "$W"; }
trap cleanup EXIT
STARTED="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

echo "arc011-ph1-demo: acceptance as $AGENT_USER under strace (two real Claude sessions, ~1-3 min)..."
# Run from the working dir, so no relative path of the run resolves into the repository.
cd "$W" || die "cannot enter $W"
strace -f -qq -s 512 -o "$W/strace.log" \
  -e trace=open,openat,openat2,creat,execve,execveat,rename,renameat,renameat2,unlink,unlinkat,mkdir,mkdirat,symlink,symlinkat,link,linkat,truncate,chmod,fchmodat,chown,lchown,fchownat,connect,clone,clone3,fork,vfork \
  runuser -u "$AGENT_USER" -- /usr/bin/env -i \
    HOME="$AGENT_HOME" USER="$AGENT_USER" LOGNAME="$AGENT_USER" PATH=/usr/bin:/bin LANG=C.UTF-8 RUST_BACKTRACE=1 \
    TERMLINK_BIN="$TERMLINK_BIN" ARC011_EVIDENCE="$W/evidence.json" ARC011_CLAUDE="$CLAUDE" \
    ARC011_MODEL="${ARC011_MODEL:-claude-sonnet-5-5}" \
    "$ACC_BIN" --ignored --exact acceptance_two_real_agents --nocapture --test-threads=1 >"$W/acceptance.log" 2>&1
ACC_RC=$?
cd "$ROOT" || die "cannot return to $ROOT"
echo "arc011-ph1-demo: acceptance rc=$ACC_RC"
tail -5 "$W/acceptance.log"

# The 17 floor items (architecture note 9.4): item | test target | test name.
PROBE_LIST="$W/probes.list"
cat >"$PROBE_LIST" <<'EOF'
SI-3|fc_matrix|fc_1_hub_refuses_a_post_whose_signature_does_not_verify
SI-3|receive_hub|receive_hub_forged_or_misaddressed_messages_are_refused_never_stored
SI-3|lib|wire::wire_tests::wire_shared_key_refused
SI-4|receive_hub|receive_hub_same_id_with_another_body_is_a_conflict_and_never_stored
SI-4|store_tests|store_conflict_recorded_first_record_untouched
SI-6|deliver_tests|deliver_handed_over_reaches_the_sender_as_a_signed_stage
SI-6|receive_hub|receive_hub_forged_or_stray_stage_records_are_ignored_and_shown
SI-7|deliver_tests|frame_layout_is_fixed_and_bodies_come_back_byte_for_byte
SI-7|deliver_tests|frame_a_body_containing_the_boundary_forces_a_new_one
SI-7|deliver_tests|frame_header_fields_cannot_inject_lines
SI-8|deliver_tests|frame_hostile_bodies_stay_inside_the_boundary
SI-8|policy_tests|policy_reads_only_sender_and_priority
SI-8|policy_tests|policy_unknown_sender_is_held_the_operator_decides_and_the_sender_hears_back
SI-14|api_tests|api_other_uid_refused_on_every_scope
SI-14|api_tests|api_real_other_uid_cannot_reach_the_socket
SI-14|api_tests|api_agent_sockets_cannot_reach_admin
SI-14|api_tests|api_status_scope_cannot_send
SI-15|store_tests|store_kill_at_every_step_never_reported_and_missing
SI-15|store_tests|store_corrupt_record_is_quarantined_on_read_and_never_returned
SI-15|fc_matrix|fc_2_a_full_disk_never_reports_stored_and_the_message_waits
SI-15|estate_kit|estate_sidecar_binary_refuses_outside_estate_exit_2
SI-16|receive_hub|receive_hub_second_sidecar_on_the_same_store_is_refused
SI-16|store_tests|store_second_open_refused_then_allowed_after_drop
SI-17|deliver_tests|deliver_handed_over_only_on_a_harness_record_never_on_echoes
SI-17|deliver_tests|deliver_evidence_reader_matches_the_real_b1_fixture
SI-17|deliver_tests|deliver_no_completeness_point_never_redelivers_on_a_timeout
SI-18|policy_tests|policy_priority_is_clamped_and_trust_is_computed_never_read
SI-18|policy_tests|policy_urgent_from_an_allow_listed_sender_is_kept_and_answered
SI-18|policy_tests|policy_unknown_sender_is_held_the_operator_decides_and_the_sender_hears_back
SI-22|store_tests|store_chain_break_detected_for_edit_insert_delete
SI-22|api_tests|api_status_reports_chain_head_and_first_break
SI-22|receive_hub|receive_hub_concurrent_writers_never_fork_a_chain
SI-23|policy_tests|policy_delivery_switch_logs_who_when_why_and_is_read_on_every_call
SI-23|deliver_tests|deliver_switch_off_and_withdrawn_allow_list_offer_nothing
SI-25|fc_matrix|fc_17_the_canary_appears_in_nothing_the_estate_produced
SI-25|estate_kit|estate_secret_scan_finds_planted_canary_only
SI-26|lib|wire::wire_tests::wire_wrong_instance_refused
SI-26|receive_hub|receive_hub_forged_or_misaddressed_messages_are_refused_never_stored
SI-30|deliver_tests|deliver_concurrent_pulls_never_hand_out_a_message_twice
SI-30|deliver_tests|deliver_an_open_claim_survives_a_restart_and_blocks_a_second_frame
SI-30|deliver_tests|deliver_unreadable_transcript_is_uncertain_and_keeps_the_attempt_open
SI-30|deliver_tests|deliver_orphan_claims_are_closed_at_start_and_open_attempts_are_not
FX-2|receive_hub|receive_hub_wrong_hub_id_reads_nothing_and_says_so
FX-2|send_hub|send_hub_wrong_hub_id_is_refused_and_nothing_is_queued_or_posted
FX-7|record_print|record_print_every_pre_handover_state_uses_only_accepted_vocabulary
FX-7|api_tests|api_status_never_says_delivered_without_handed_over
FX-7|hooks_tests|hooks_sender_sees_waiting_for_recipient_then_stuck_never_delivered
EOF
: >"$W/probes.tsv"
if [ "$PROBES" = 1 ]; then
  echo "arc011-ph1-demo: floor probes (one test at a time, as root)..."
  cargo test -q -p termlink-sidecar --no-run >/dev/null 2>&1 || die "cannot build the sidecar tests"
  n=0
  while IFS='|' read -r item target name; do
    [ -n "$item" ] || continue
    n=$((n+1))
    if [ "$target" = lib ]; then sel=(--lib); else sel=(--test "$target"); fi
    log="$W/probe-$n.log"
    t0=$(date +%s)
    timeout 1200 cargo test -q -p termlink-sidecar "${sel[@]}" -- --exact "$name" --nocapture --test-threads=1 >"$log" 2>&1
    rc=$?
    secs=$(( $(date +%s) - t0 ))
    if [ "$rc" = 0 ] && grep -q '^test result: ok. 1 passed' "$log"; then
      if grep -q 'SKIPPED' "$log"; then res=SKIPPED; else res=pass; fi
    elif [ "$rc" = 0 ]; then
      res="NOT RUN (filter matched no test)"
    else
      res="FAIL (rc $rc)"
    fi
    printf '%s\t%s\t%s\t%s\t%s\n' "$item" "$target" "$name" "$res" "$secs" >>"$W/probes.tsv"
    printf '  %-6s %-50s %s\n' "$item" "$name" "$res"
  done <"$PROBE_LIST"
fi

mkdir -p "$(dirname "$REPORT")"
EVIDENCE_OUT="${REPORT%.md}.evidence.json"
python3 "$ROOT/scripts/lib/arc011-acceptance-report.py" \
  --evidence "$W/evidence.json" --strace "$W/strace.log" --probes "$W/probes.tsv" \
  --acceptance-log "$W/acceptance.log" --acceptance-rc "$ACC_RC" --probes-run "$PROBES" \
  --agent-user "$AGENT_USER" --agent-home "$AGENT_HOME" --repo "$ROOT" \
  --started "$STARTED" --head "$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)" \
  --out "$REPORT" --evidence-out "$EVIDENCE_OUT"
RC=$?
echo "arc011-ph1-demo: report $REPORT (rc=$RC)"
[ "$KEEP" = 1 ] && echo "arc011-ph1-demo: working files kept in $W"
exit "$RC"
