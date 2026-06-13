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
#   Check 4: termlink binary freshness vs project root VERSION (T-2181;
#            T-2226 feature-aware: WARN only when crates/ changed since the
#            installed binary, not on doc/task-only VERSION-number drift)
#            → Catches "I deployed but the binary is older than the
#              source tree's documented features" footgun. CLAUDE.md
#              catalog promises flags like `--only-stuck` (T-2076) and
#              subcommands like `fleet governor-status` (T-2062) that
#              the operator's stale binary refuses with "unknown flag" /
#              "unrecognized subcommand". Substrate still works for
#              primitives the binary has — WARN, not FAIL.
#
#   Check 5: local hub binary freshness via field-presence probe (T-2184)
#            → Symmetric to Check 4: catches stale HUB (vs Check 4's stale
#              CLIENT). When operator runs `cargo install termlink` but
#              never restarts the running hub, the new file on disk
#              replaces the inode but the process keeps serving from the
#              old in-memory binary (/proc/<pid>/exe shows
#              "...(deleted)"). The CLI loyally renders fields the hub's
#              older binary doesn't emit as "n/a", and the operator infers
#              a missing-feature gap when the actual gap is a missing
#              restart. Probe: `hub status --governor --json` MUST contain
#              `rate_buckets_evicted_total` (T-2139 field). Absence ⇒ WARN
#              with restart remediation. Origin: T-2183 PL-209
#              misdiagnosis loop, ~30min wasted investigation.
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
HEARTBEAT=1
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
  4. termlink binary version >= project root VERSION (T-2181) — catches
     stale-binary footgun where catalog promises flags the binary lacks
  5. local hub serves T-2139 field (`rate_buckets_evicted_total`) — catches
     stale-HUB footgun where a `(deleted)` in-memory binary keeps serving
     pre-T-2139 envelopes (T-2184, symmetric to Check 4)

Options:
  --json           Emit a machine-readable envelope instead of human-format output
  --quiet          Empty-log canary mode: exit 0 with NO output on PASS; print full
                   output (and exit non-zero) only on WARN/FAIL. Use in cron jobs
                   where an empty log file is the healthy state.
                   Composes with --json (envelope only on WARN/FAIL).
  --no-heartbeat   Suppress the T-1723 heartbeat-file touch. Used by the meta-canary
                   (check-canary-aliveness.sh with CANARY_PROBE_CMD) so the
                   freshness probe doesn't side-effect the very signal it watches.
  --help           Print this help and exit 0

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
        --no-heartbeat) HEARTBEAT=0; shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "substrate-preflight.sh: unknown flag: $1" >&2; exit 2 ;;
    esac
done

# T-2175 (mirror of T-1723) heartbeat: prove this canary ran, even on FAIL
# cycles. scripts/check-canary-aliveness.sh stats this file's mtime; if stale,
# the canary itself is broken (cron didn't load, script crashed, etc.).
# Placed BEFORE the --quiet buffer redirect so a buffer setup failure can't
# silently swallow the heartbeat. --no-heartbeat suppresses the touch so the
# meta-canary can probe the substrate state without side-effecting the very
# signal it's checking.
HEARTBEAT_FILE="${HEARTBEAT_FILE:-.context/working/.substrate-preflight-canary.heartbeat}"
if [ "$HEARTBEAT" = 1 ]; then
    mkdir -p "$(dirname "$HEARTBEAT_FILE")" 2>/dev/null || true
    touch -- "$HEARTBEAT_FILE" 2>/dev/null || true
fi

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

# ---- Check 4: termlink binary freshness (T-2181) -----------------------

# Compare two SemVer-ish `major.minor.patch` strings. Returns 0 (true) if
# $1 < $2, 1 otherwise. Empty / malformed components compare as 0.
version_lt() {
    local a="$1" b="$2"
    local IFS=.
    # shellcheck disable=SC2206
    local av=($a) bv=($b)
    local i
    for i in 0 1 2; do
        local ai="${av[$i]:-0}" bi="${bv[$i]:-0}"
        # Strip non-digits (defensive).
        ai="${ai//[^0-9]/}"; bi="${bi//[^0-9]/}"
        ai="${ai:-0}"; bi="${bi:-0}"
        if [ "$ai" -lt "$bi" ]; then return 0; fi
        if [ "$ai" -gt "$bi" ]; then return 1; fi
    done
    return 1
}

# T-2226 — Distinguish version-number drift from genuine CLI-feature drift.
# VERSION is git-derived (patch = commits-since-tag), so it increments on EVERY
# commit — including doc/task/script commits that never touch the binary. The
# naive "binary_version < VERSION" test therefore false-WARNs after any such
# commit even though the installed binary's feature set is current (PL-219
# alert-fatigue class: a check that cries wolf trains operators to ignore it,
# masking a REAL stale binary). This returns 0 (true) ONLY when it can PROVE
# crates/ (the CLI source) is unchanged between the installed binary and HEAD.
# ANY uncertainty returns 1 (keep WARN) — the check must never silence a
# genuinely stale binary, only the proven-false-positive case.
crates_unchanged_since_binary() {
    local bin_ver="$1" repo_ver="$2"
    local IFS=.
    # shellcheck disable=SC2206
    local bv=($bin_ver) rv=($repo_ver)
    # Same release line only: a major/minor bump is a real boundary — keep WARN.
    [ "${bv[0]:-x}" = "${rv[0]:-y}" ] || return 1
    [ "${bv[1]:-x}" = "${rv[1]:-y}" ] || return 1
    local bp="${bv[2]:-}" rp="${rv[2]:-}"
    bp="${bp//[^0-9]/}"; rp="${rp//[^0-9]/}"
    [ -n "$bp" ] && [ -n "$rp" ] || return 1
    local delta=$(( rp - bp ))
    [ "$delta" -gt 0 ] || return 1
    # Need a git repo with enough history to look back $delta commits.
    git rev-parse --verify -q "HEAD~${delta}" >/dev/null 2>&1 || return 1
    # Any crates/ commit in (binary, HEAD] → genuine feature drift → keep WARN.
    local touched
    touched=$(git log --oneline "HEAD~${delta}..HEAD" -- crates/ 2>/dev/null | head -n1)
    [ -z "$touched" ]
}

check_binary_freshness() {
    local version_file="VERSION"
    if [ ! -r "$version_file" ]; then
        # No VERSION in cwd (foreign tree) — skip. No signal vs noise.
        return
    fi
    local repo_version
    repo_version=$(head -n1 "$version_file" 2>/dev/null | tr -d '[:space:]')
    if [ -z "$repo_version" ]; then
        return
    fi

    # Resolve termlink binary version.
    if ! command -v termlink >/dev/null 2>&1; then
        emit_check "binary" "medium" "warn" \
            "termlink not on PATH — catalog flags won't resolve" \
            "cargo build --release && install -m 755 target/release/termlink ~/.cargo/bin/"
        return
    fi
    local binary_version
    binary_version=$(termlink --version 2>/dev/null | awk '{print $NF}')
    if [ -z "$binary_version" ]; then
        emit_check "binary" "medium" "warn" \
            "termlink --version returned no parseable version" \
            "Reinstall: cargo build --release && install -m 755 target/release/termlink ~/.cargo/bin/"
        return
    fi

    if version_lt "$binary_version" "$repo_version"; then
        if crates_unchanged_since_binary "$binary_version" "$repo_version"; then
            # T-2226: version number drifted but no crates/ change since the
            # installed binary — features are current, rebuild not required.
            emit_check "binary" "medium" "pass" \
                "termlink $binary_version < VERSION $repo_version but no crates/ change since binary — version drift only, rebuild not required (T-2226)"
        else
            emit_check "binary" "medium" "warn" \
                "termlink $binary_version older than project VERSION $repo_version — catalog features may surface as 'unknown flag' / 'unrecognized subcommand'" \
                "cargo build --release && install -m 755 target/release/termlink ~/.cargo/bin/"
        fi
    else
        emit_check "binary" "medium" "pass" \
            "termlink $binary_version >= project VERSION $repo_version"
    fi
}

# T-2184: Check 5 — local hub binary freshness.
# Symmetric to check_binary_freshness (Check 4, T-2181) but probes the
# RUNNING hub via JSON-RPC field presence rather than the on-disk binary
# version. Catches the "I rebuilt the binary but never restarted the hub"
# failure mode where /proc/<pid>/exe shows "...(deleted)" and the live
# process keeps serving an in-memory binary that predates T-2139's
# rate_buckets_evicted_total field. The CLI loyally renders absent
# fields as "n/a", which is how PL-209 spent 30+ minutes chasing a
# phantom telemetry gap.
#
# Probe: `termlink hub status --governor --json` MUST contain the literal
# string `rate_buckets_evicted_total` somewhere in its output. Field
# presence (any numeric value including 0) ⇒ PASS; absence ⇒ WARN with
# `restart hub` remediation. Graceful degradation: hub not running, CLI
# missing, or non-zero exit ⇒ SKIP (different failure modes, Check 1/4
# cover them).
check_hub_binary_freshness() {
    # If termlink isn't installed, Check 4 already warned — don't double-emit.
    if ! command -v termlink >/dev/null 2>&1; then
        return
    fi
    # Probe live hub. Bounded by timeout — if hub is unresponsive,
    # SKIP rather than block /preflight on a wedged hub.
    local probe_output probe_rc
    probe_output=$(timeout 5 termlink hub status --governor --json 2>/dev/null)
    probe_rc=$?
    if [ "$probe_rc" -ne 0 ] || [ -z "$probe_output" ]; then
        # Hub down, RPC failed, timeout, or empty body — out of scope.
        # Don't false-positive a stale-binary classification.
        return
    fi
    # L-387 capture-first SIGPIPE safety: capture full body, then grep.
    if echo "$probe_output" | grep -q '"rate_buckets_evicted_total"'; then
        emit_check "hub-binary" "medium" "pass" \
            "local hub serves T-2139 rate_buckets_evicted_total field — fresh binary"
    else
        emit_check "hub-binary" "medium" "warn" \
            "local hub omits rate_buckets_evicted_total (pre-T-2139) — likely (deleted)-on-disk binary still in memory" \
            "Restart hub to pick up new binary (verify runtime_dir persists secret/cert per Check 1 first). Inspect: ls -la /proc/\$(pgrep -f 'termlink hub start' | head -1)/exe"
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
check_binary_freshness
check_hub_binary_freshness

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
