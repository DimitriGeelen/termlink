#!/usr/bin/env bash
# T-2154 — substrate pre-flight health check.
#
# Catches deployment-time misconfigs that cause silent substrate failures:
#
#   Check 1: TERMLINK_RUNTIME_DIR on volatile /tmp
#            → PL-021: hub regenerates secret + TLS cert every reboot,
#              every client sees auth-mismatch + TOFU drift, fleet wedges.
#              Both mechanisms detected: tmpfs mount AND systemd-tmpfiles
#              D-rule wipe (the rule that looks innocent in `mount` output
#              but still nukes /tmp on boot — T-1294 ring20-management).
#
#   Check 2: ~/.termlink/hubs.toml present + has [hubs.*] sections
#            → Without it, every heal path (T-1054/T-1055/T-1291) fails;
#              fleet verbs (fleet doctor, fleet verify, fleet history)
#              all return "no profiles".
#
#   Check 3: ~/.termlink/be-reachable.state freshness
#            → If pid is dead, pickup loops, agent contact, and DM
#              receipts all look healthy at registration time but the
#              listener is gone. Catches the "I forgot to /be-reachable
#              again after reboot" footgun.
#
# Read-only, no network, no auth, no state mutation. Safe in any context.
#
# Exit codes:
#   0    All PASS
#   1    WARN — medium-severity issue (operator may proceed but should fix)
#   2    FAIL — high-severity issue (substrate will silently misbehave)
#
# Usage:
#   substrate-preflight.sh [--json]
#
# JSON envelope:
#   {ok, exit_code, checks: [{name, severity, status, message, remediation?}],
#    summary: {pass, warn, fail}}

set -u

JSON=0
QUIET=0
EXIT_RC=0
PASS_COUNT=0
WARN_COUNT=0
FAIL_COUNT=0
declare -a CHECK_JSON_ROWS=()

usage() {
    cat <<'EOF'
Usage: substrate-preflight.sh [--json] [--help]

Run pre-flight diagnostics that catch deployment-time misconfigs which
cause silent substrate failures.

Checks:
  1. TERMLINK_RUNTIME_DIR not on volatile /tmp (PL-021)
  2. ~/.termlink/hubs.toml present and non-empty
  3. ~/.termlink/be-reachable.state pid alive (if file exists)

Options:
  --json     Emit a machine-readable envelope instead of human-format output
  --quiet    Empty-log canary mode: exit 0 with NO output on PASS; print full
             output (and exit non-zero) only on WARN/FAIL. Use in cron jobs
             where an empty log file is the healthy state.
             Composes with --json (envelope only on WARN/FAIL).
  --help     Print this help and exit 0

Exit codes:
  0   All checks PASS
  1   At least one WARN, no FAIL
  2   At least one FAIL — substrate will silently misbehave

Examples:
  # Default: human-readable pre-flight before starting a hub
  substrate-preflight.sh

  # Pipe-friendly form for CI / automation
  substrate-preflight.sh --json | jq '.summary'

  # Cron canary form — log only grows when something is wrong:
  substrate-preflight.sh --quiet >> ~/.substrate-preflight-canary.log 2>&1

See: docs/operations/substrate-getting-started.md
     PL-021 (hub rotation under volatile runtime_dir)
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --json) JSON=1; shift ;;
        --quiet) QUIET=1; shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "substrate-preflight.sh: unknown flag: $1" >&2; exit 2 ;;
    esac
done

# --quiet buffers all stdout to a temp file; on exit, only emit if non-zero.
if [ "$QUIET" -eq 1 ]; then
    QUIET_BUF=$(mktemp)
    trap 'rm -f "$QUIET_BUF"' EXIT
    exec 3>&1
    exec >"$QUIET_BUF"
fi

# ---- Reporting helpers --------------------------------------------------

# json_escape <string> — minimal JSON string escape (\, ", and newline)
json_escape() {
    local s="$1"
    s="${s//\\/\\\\}"
    s="${s//\"/\\\"}"
    s="${s//$'\n'/\\n}"
    printf '%s' "$s"
}

emit_check() {
    # args: name severity status message [remediation]
    local name="$1" sev="$2" status="$3" msg="$4" remediation="${5:-}"

    case "$sev" in
        high)
            if [ "$status" = "fail" ]; then
                FAIL_COUNT=$((FAIL_COUNT + 1))
                [ "$EXIT_RC" -lt 2 ] && EXIT_RC=2
            elif [ "$status" = "warn" ]; then
                WARN_COUNT=$((WARN_COUNT + 1))
                [ "$EXIT_RC" -lt 1 ] && EXIT_RC=1
            else
                PASS_COUNT=$((PASS_COUNT + 1))
            fi
            ;;
        medium)
            if [ "$status" = "warn" ] || [ "$status" = "fail" ]; then
                WARN_COUNT=$((WARN_COUNT + 1))
                [ "$EXIT_RC" -lt 1 ] && EXIT_RC=1
            else
                PASS_COUNT=$((PASS_COUNT + 1))
            fi
            ;;
        *)
            PASS_COUNT=$((PASS_COUNT + 1))
            ;;
    esac

    if [ "$JSON" -eq 1 ]; then
        local row
        if [ -n "$remediation" ]; then
            row=$(printf '{"name":"%s","severity":"%s","status":"%s","message":"%s","remediation":"%s"}' \
                "$(json_escape "$name")" \
                "$(json_escape "$sev")" \
                "$(json_escape "$status")" \
                "$(json_escape "$msg")" \
                "$(json_escape "$remediation")")
        else
            row=$(printf '{"name":"%s","severity":"%s","status":"%s","message":"%s"}' \
                "$(json_escape "$name")" \
                "$(json_escape "$sev")" \
                "$(json_escape "$status")" \
                "$(json_escape "$msg")")
        fi
        CHECK_JSON_ROWS+=("$row")
    else
        local marker
        case "$status" in
            pass) marker="PASS" ;;
            warn) marker="WARN" ;;
            fail) marker="FAIL" ;;
            *)    marker="?"    ;;
        esac
        printf '[%s] %-18s %s\n' "$marker" "$name" "$msg"
        if [ -n "$remediation" ] && [ "$status" != "pass" ]; then
            printf '       remediation: %s\n' "$remediation"
        fi
    fi
}

# ---- Check 1: TERMLINK_RUNTIME_DIR volatility (PL-021) ------------------

check_runtime_dir_volatility() {
    local rd="${TERMLINK_RUNTIME_DIR:-/tmp/termlink-0}"

    case "$rd" in
        /tmp/*|/tmp|/var/tmp/*|/var/tmp)
            ;;
        *)
            emit_check "runtime_dir" "high" "pass" \
                "TERMLINK_RUNTIME_DIR=$rd (not on /tmp — persists across reboot)"
            return
            ;;
    esac

    # On /tmp — check for both volatility mechanisms.
    local volatile_reason=""
    if mount 2>/dev/null | grep -qE "^tmpfs on /tmp\b"; then
        volatile_reason="tmpfs mount"
    elif [ -r /usr/lib/tmpfiles.d/tmp.conf ] && \
         grep -qE '^[Dd][[:space:]]+/tmp([[:space:]]|$)' /usr/lib/tmpfiles.d/tmp.conf 2>/dev/null; then
        volatile_reason="systemd-tmpfiles D-rule (/usr/lib/tmpfiles.d/tmp.conf)"
    else
        # Scan /etc overrides — D-rule may live in any override file.
        local override_match=""
        for f in /etc/tmpfiles.d/*.conf; do
            [ -r "$f" ] || continue
            if grep -qE '^[Dd][[:space:]]+/tmp([[:space:]]|$)' "$f" 2>/dev/null; then
                override_match="$f"
                break
            fi
        done
        if [ -n "$override_match" ]; then
            volatile_reason="systemd-tmpfiles D-rule ($override_match)"
        fi
    fi

    if [ -n "$volatile_reason" ]; then
        emit_check "runtime_dir" "high" "fail" \
            "TERMLINK_RUNTIME_DIR=$rd on volatile /tmp ($volatile_reason) — hub will regenerate secret + TLS cert every reboot (PL-021)" \
            "export TERMLINK_RUNTIME_DIR=/var/lib/termlink before starting the hub; pre-seed with 'cp -a /tmp/termlink-0/. /var/lib/termlink/' if a working hub already exists. See CLAUDE.md Hub Auth Rotation Protocol."
    else
        emit_check "runtime_dir" "high" "warn" \
            "TERMLINK_RUNTIME_DIR=$rd on /tmp but not detected as volatile — move to /var/lib/termlink anyway (defence in depth)" \
            "export TERMLINK_RUNTIME_DIR=/var/lib/termlink"
    fi
}

# ---- Check 2: hubs.toml presence ---------------------------------------

check_hubs_toml() {
    local f="${HOME}/.termlink/hubs.toml"
    if [ ! -f "$f" ]; then
        emit_check "hubs.toml" "medium" "warn" \
            "$f missing — every fleet verb will return empty" \
            "Run 'termlink fleet profile add <name> --address <ip:port>' to declare a hub"
        return
    fi
    if ! grep -qE '^\[hubs\.' "$f" 2>/dev/null; then
        emit_check "hubs.toml" "medium" "warn" \
            "$f present but contains no [hubs.NAME] sections" \
            "Run 'termlink fleet profile add <name>' to declare at least one hub"
        return
    fi
    local hub_count
    hub_count=$(grep -cE '^\[hubs\.' "$f" 2>/dev/null || echo 0)
    emit_check "hubs.toml" "medium" "pass" "$f present ($hub_count hub(s) declared)"
}

# ---- Check 3: be-reachable.state freshness -----------------------------

check_be_reachable_state() {
    local f="${HOME}/.termlink/be-reachable.state"
    if [ ! -f "$f" ]; then
        emit_check "be-reachable" "medium" "pass" \
            "no active session (state file absent — expected if you have not run /be-reachable)"
        return
    fi
    local pid
    pid=$(grep -oE '"pid"[[:space:]]*:[[:space:]]*[0-9]+' "$f" 2>/dev/null \
          | head -n1 \
          | sed 's/.*:[[:space:]]*//')
    if [ -z "$pid" ]; then
        emit_check "be-reachable" "medium" "warn" \
            "$f present but has no readable pid field" \
            "rm $f && /be-reachable start"
        return
    fi
    if kill -0 "$pid" 2>/dev/null; then
        local agent_id
        agent_id=$(grep -oE '"agent_id"[[:space:]]*:[[:space:]]*"[^"]*"' "$f" 2>/dev/null \
                   | head -n1 \
                   | sed 's/.*"agent_id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/')
        emit_check "be-reachable" "medium" "pass" \
            "alive (agent_id=${agent_id:-unknown}, pid=$pid)"
    else
        emit_check "be-reachable" "medium" "warn" \
            "$f points at dead pid=$pid — listener is gone but state file remains; pickup loops will see stale identity" \
            "/be-reachable stop && /be-reachable start"
    fi
}

# ---- Run all checks ----------------------------------------------------

if [ "$JSON" -eq 0 ]; then
    echo "substrate pre-flight"
    echo "===================="
fi

check_runtime_dir_volatility
check_hubs_toml
check_be_reachable_state

# ---- Summary -----------------------------------------------------------

if [ "$JSON" -eq 1 ]; then
    ok_str="true"
    [ "$EXIT_RC" -gt 0 ] && ok_str="false"
    checks_str=""
    if [ "${#CHECK_JSON_ROWS[@]}" -gt 0 ]; then
        first=1
        for row in "${CHECK_JSON_ROWS[@]}"; do
            if [ "$first" -eq 1 ]; then
                checks_str="$row"
                first=0
            else
                checks_str="$checks_str,$row"
            fi
        done
    fi
    printf '{"ok":%s,"exit_code":%d,"checks":[%s],"summary":{"pass":%d,"warn":%d,"fail":%d}}\n' \
        "$ok_str" "$EXIT_RC" "$checks_str" "$PASS_COUNT" "$WARN_COUNT" "$FAIL_COUNT"
else
    echo ""
    if [ "$EXIT_RC" -eq 0 ]; then
        printf 'Summary: %d pass, 0 warn, 0 fail — substrate-ready.\n' "$PASS_COUNT"
    elif [ "$EXIT_RC" -eq 1 ]; then
        printf 'Summary: %d pass, %d warn, %d fail — substrate will run but fix warnings when convenient.\n' \
            "$PASS_COUNT" "$WARN_COUNT" "$FAIL_COUNT"
    else
        printf 'Summary: %d pass, %d warn, %d fail — substrate will silently misbehave. FIX BEFORE DEPLOY.\n' \
            "$PASS_COUNT" "$WARN_COUNT" "$FAIL_COUNT"
    fi
fi

# --quiet replay: emit buffered output only on non-zero exit.
if [ "$QUIET" -eq 1 ]; then
    exec >&3
    exec 3>&-
    if [ "$EXIT_RC" -ne 0 ]; then
        # Prefix with timestamp so cron logs are forensically useful.
        printf '=== %s ===\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
        cat "$QUIET_BUF"
        printf -- '---\n'
    fi
fi

exit "$EXIT_RC"
