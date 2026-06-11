#!/usr/bin/env bash
# T-2169 — substrate systemd template static-verify regression smoke.
#
# Protects the production-systemd surface shipped by T-2165 (orchestrator
# template) and T-2167 (worker template). A single accidental edit to
# either .service file can silently break a future operator's
# `systemctl enable --now` install — bad `%i` specifier, missing
# `EnvironmentFile=`, dropped `Restart=on-failure`, or a
# `TERMLINK_SUBSTRATE_SCRIPT=` path that no longer exists on disk.
#
# This smoke catches those classes BEFORE the templates land on a host.
# It is STATIC by design:
#   - No `systemctl enable`, no daemon-reload, no service install
#   - No termlink RPC, no hub auth, no state mutation
#   - Pure file-content checks + script-existence + script-+x + optional
#     `systemd-analyze verify`
#
# Pattern mirrors scripts/substrate-smoke.sh (T-2151): PASS/FAIL per
# stage, exit 0/1/2, optional --json.
#
# Exit codes:
#   0   both templates valid — every stage PASSed
#   1   any stage FAILed — failing stage + error on stderr
#   2   usage / missing input (no templates/, --help)
#
# Usage:
#   substrate-systemd-smoke.sh [--json] [--help]
#
# Pair with: scripts/substrate-smoke.sh (T-2151) — that one proves the
# end-to-end runtime pattern; this one proves the systemd install path.

set -u

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TEMPLATE_DIR="${REPO_ROOT}/systemd-templates"

ORCH_TPL="${TEMPLATE_DIR}/termlink-substrate-orchestrator@.service"
WORKER_TPL="${TEMPLATE_DIR}/termlink-substrate-worker@.service"

JSON_MODE=0

STAGES_PASSED=()
STAGES_FAILED=()
ERRORS_OUT=""

usage() {
    cat <<'EOF'
Usage: substrate-systemd-smoke.sh [--json] [--help]

Static-verifies the T-2165 orchestrator + T-2167 worker systemd
templates without installing or enabling anything. Safe to run anywhere.

Stages:
  1. Template files exist
  2. Required sections + directives present per template
  3. TERMLINK_SUBSTRATE_SCRIPT references an existing +x script
  4. TERMLINK_RUNTIME_DIR is NOT /tmp (PL-021 prevention)
  5. %i hard-wiring (--orchestrator-id %i / --worker-id %i)
  6. Worker TERMLINK_SW_CMD required-env guard
  7. systemd-analyze verify (skipped if analyzer absent)

Options:
  --json    Emit a machine-readable envelope
              {ok, stages_passed, stages_failed, errors}
  --help    Show this help and exit

Exit codes:
  0  every stage PASSed
  1  one or more stages FAILed
  2  usage / missing dependency
EOF
}

# ── helpers ────────────────────────────────────────────────────────────

stage_pass() {
    STAGES_PASSED+=("$1")
    [ "$JSON_MODE" -eq 1 ] || echo "PASS  $1"
}

stage_fail() {
    local stage="$1"; shift
    STAGES_FAILED+=("$stage")
    local msg="$*"
    ERRORS_OUT="${ERRORS_OUT}${stage}: ${msg}"$'\n'
    [ "$JSON_MODE" -eq 1 ] || {
        echo "FAIL  $stage" >&2
        echo "      $msg" >&2
    }
}

# Look up a single-key=value Environment= line.
# Templates carry: `Environment=KEY=VAL` (one pair per line).
read_env() {
    local file="$1" key="$2"
    grep -E "^Environment=${key}=" "$file" 2>/dev/null \
        | head -n 1 \
        | sed -E "s/^Environment=${key}=//"
}

# ── parse args ─────────────────────────────────────────────────────────

while [ $# -gt 0 ]; do
    case "$1" in
        --json) JSON_MODE=1; shift ;;
        --help|-h) usage; exit 0 ;;
        *) echo "substrate-systemd-smoke.sh: unknown arg: $1" >&2; usage >&2; exit 2 ;;
    esac
done

# ── stage 1: templates exist ───────────────────────────────────────────

if [ -f "$ORCH_TPL" ]; then
    stage_pass "templates.orchestrator.exists"
else
    stage_fail "templates.orchestrator.exists" "$ORCH_TPL not found"
fi

if [ -f "$WORKER_TPL" ]; then
    stage_pass "templates.worker.exists"
else
    stage_fail "templates.worker.exists" "$WORKER_TPL not found"
fi

# If neither template exists, downstream stages are noise — bail.
if [ ${#STAGES_FAILED[@]} -gt 0 ] && [ ${#STAGES_PASSED[@]} -eq 0 ]; then
    [ "$JSON_MODE" -eq 1 ] && printf '{"ok":false,"stages_passed":[],"stages_failed":["templates.exist"],"errors":"no templates to verify"}\n'
    exit 1
fi

# ── stage 2: required sections + directives ────────────────────────────

check_sections() {
    local file="$1" label="$2"
    local missing=""
    for section in '[Unit]' '[Service]' '[Install]'; do
        # grep -F treats brackets as literals
        if ! grep -Fq "$section" "$file"; then
            missing="${missing}${section} "
        fi
    done
    if [ -n "$missing" ]; then
        stage_fail "sections.${label}" "missing sections: $missing"
        return
    fi
    stage_pass "sections.${label}"
}

check_directives() {
    local file="$1" label="$2"
    local missing=""
    # Tuple of (regex, human-name); regex matches the start of the directive line.
    local directives=(
        'Description='
        'Type=exec'
        'EnvironmentFile='
        'ExecStart='
        'Restart=on-failure'
        'RestartSec='
        'WantedBy='
    )
    for d in "${directives[@]}"; do
        if ! grep -Eq "^${d}" "$file"; then
            missing="${missing}${d} "
        fi
    done
    if [ -n "$missing" ]; then
        stage_fail "directives.${label}" "missing directives: $missing"
        return
    fi
    stage_pass "directives.${label}"
}

[ -f "$ORCH_TPL" ]   && check_sections   "$ORCH_TPL"   "orchestrator"
[ -f "$WORKER_TPL" ] && check_sections   "$WORKER_TPL" "worker"
[ -f "$ORCH_TPL" ]   && check_directives "$ORCH_TPL"   "orchestrator"
[ -f "$WORKER_TPL" ] && check_directives "$WORKER_TPL" "worker"

# ── stage 3: TERMLINK_SUBSTRATE_SCRIPT exists and is +x ────────────────

check_substrate_script() {
    local file="$1" label="$2"
    local script_path
    script_path=$(read_env "$file" "TERMLINK_SUBSTRATE_SCRIPT")
    if [ -z "$script_path" ]; then
        stage_fail "substrate_script.${label}" "Environment=TERMLINK_SUBSTRATE_SCRIPT= not set in $file"
        return
    fi
    if [ ! -f "$script_path" ]; then
        stage_fail "substrate_script.${label}" "TERMLINK_SUBSTRATE_SCRIPT=$script_path does not exist on disk"
        return
    fi
    if [ ! -x "$script_path" ]; then
        stage_fail "substrate_script.${label}" "TERMLINK_SUBSTRATE_SCRIPT=$script_path exists but is not +x (PL-208 class)"
        return
    fi
    stage_pass "substrate_script.${label}"
}

[ -f "$ORCH_TPL" ]   && check_substrate_script "$ORCH_TPL"   "orchestrator"
[ -f "$WORKER_TPL" ] && check_substrate_script "$WORKER_TPL" "worker"

# ── stage 4: TERMLINK_RUNTIME_DIR is NOT /tmp (PL-021) ─────────────────

check_runtime_dir() {
    local file="$1" label="$2"
    local runtime_dir
    runtime_dir=$(read_env "$file" "TERMLINK_RUNTIME_DIR")
    if [ -z "$runtime_dir" ]; then
        stage_fail "runtime_dir.${label}" "Environment=TERMLINK_RUNTIME_DIR= not set in $file"
        return
    fi
    case "$runtime_dir" in
        /tmp|/tmp/*)
            stage_fail "runtime_dir.${label}" "TERMLINK_RUNTIME_DIR=$runtime_dir is on /tmp (PL-021 — volatile, hub regenerates secret per reboot)"
            return
            ;;
    esac
    stage_pass "runtime_dir.${label}"
}

[ -f "$ORCH_TPL" ]   && check_runtime_dir "$ORCH_TPL"   "orchestrator"
[ -f "$WORKER_TPL" ] && check_runtime_dir "$WORKER_TPL" "worker"

# ── stage 5: %i hard-wiring ────────────────────────────────────────────

if [ -f "$ORCH_TPL" ]; then
    if grep -Eq -- '--orchestrator-id %i' "$ORCH_TPL"; then
        stage_pass "instance_specifier.orchestrator"
    else
        stage_fail "instance_specifier.orchestrator" "ExecStart does not hard-wire '--orchestrator-id %i' — systemd instance name will not become the substrate identity"
    fi
fi

if [ -f "$WORKER_TPL" ]; then
    if grep -Eq -- '--worker-id %i' "$WORKER_TPL"; then
        stage_pass "instance_specifier.worker"
    else
        stage_fail "instance_specifier.worker" "ExecStart does not hard-wire '--worker-id %i' — systemd instance name will not become the substrate identity"
    fi
fi

# ── stage 6: worker TERMLINK_SW_CMD required-env guard ─────────────────

if [ -f "$WORKER_TPL" ]; then
    # The guard should refuse start (exit 2) when TERMLINK_SW_CMD is unset.
    # The exact ExecStart line is shell-escaped with $${...} for systemd, so
    # grep for the literal substring that's stable across edits.
    if grep -q 'TERMLINK_SW_CMD' "$WORKER_TPL" \
        && grep -Eq 'TERMLINK_SW_CMD not set' "$WORKER_TPL"; then
        stage_pass "required_env_guard.worker"
    else
        stage_fail "required_env_guard.worker" "ExecStart does not guard on TERMLINK_SW_CMD — a misconfigured .env will spawn pickup with no --cmd"
    fi
fi

# ── stage 7: systemd-analyze verify (optional) ─────────────────────────

if command -v systemd-analyze >/dev/null 2>&1; then
    for tpl in "$ORCH_TPL" "$WORKER_TPL"; do
        [ -f "$tpl" ] || continue
        local_label="$(basename "$tpl" .service | sed 's/@$//')"
        # Template units skip the deep ExecStart check, but ini structure
        # is verified. Exit 0 = clean.
        out=$(systemd-analyze verify "$tpl" 2>&1) ; rc=$?
        if [ "$rc" -eq 0 ]; then
            stage_pass "analyze.${local_label}"
        else
            stage_fail "analyze.${local_label}" "systemd-analyze verify failed: ${out}"
        fi
    done
else
    [ "$JSON_MODE" -eq 1 ] || echo "SKIP  analyze.* — systemd-analyze not available on this host"
fi

# ── render ─────────────────────────────────────────────────────────────

passed_count=${#STAGES_PASSED[@]}
failed_count=${#STAGES_FAILED[@]}

if [ "$JSON_MODE" -eq 1 ]; then
    # Build JSON arrays via printf — no jq dep (so the smoke runs anywhere).
    passed_json="[]"
    if [ "$passed_count" -gt 0 ]; then
        passed_json=$(printf '"%s",' "${STAGES_PASSED[@]}" | sed 's/,$//')
        passed_json="[${passed_json}]"
    fi
    failed_json="[]"
    if [ "$failed_count" -gt 0 ]; then
        failed_json=$(printf '"%s",' "${STAGES_FAILED[@]}" | sed 's/,$//')
        failed_json="[${failed_json}]"
    fi
    # Escape errors for JSON: replace \ with \\, " with \", newlines with \n.
    errors_json=$(printf '%s' "$ERRORS_OUT" \
        | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g' \
        | awk 'BEGIN{ORS=""} {if(NR>1) print "\\n"; print}')
    ok="true"
    [ "$failed_count" -gt 0 ] && ok="false"
    printf '{"ok":%s,"stages_passed":%s,"stages_failed":%s,"errors":"%s"}\n' \
        "$ok" "$passed_json" "$failed_json" "$errors_json"
else
    echo
    echo "Summary: ${passed_count} PASS / ${failed_count} FAIL"
fi

[ "$failed_count" -eq 0 ] && exit 0
exit 1
