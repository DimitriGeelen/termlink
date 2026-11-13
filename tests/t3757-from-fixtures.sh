#!/usr/bin/env bash
# Fixture suite for T-3757 step 1 (arc-011, follow-on to P-14 / B-4, 06a AL-32): every
# founding-verb caller declares its identity as `params.from`.
# Prints one PASS line per check (or FAIL). Exit 0 only when all passed; 1 on a failure;
# 2 when the tests could not run.
#
#   bash tests/t3757-from-fixtures.sh            # the working tree
#   bash tests/t3757-from-fixtures.sh mutants    # every mutant must be KILLED
#
# Mutants run on a `git clone --shared` of HEAD under .context/working/t3757/src (gitignored),
# built with --manifest-path; the shared working tree is never touched (other workers build
# crates/ concurrently). A mutant whose pattern is not found, or that does not compile, is
# TOOLING (exit 2): "could not look" is never reported as killed.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 2
export TMPDIR="${TMPDIR:-/var/tmp}" CARGO_TARGET_TMPDIR="${CARGO_TARGET_TMPDIR:-/var/tmp}"
unset PROJECT_ROOT
mode="${1:-fixtures}"
work=.context/working/t3757
mkdir -p "$work" || exit 2

SES_UNIT=(-p termlink-session --lib founding_log::tests::t3757_)
CLI_E2E=(-p termlink --test t3757_declared_from)
MCP_E2E=(-p termlink-mcp --test t3757_mcp_from)
SC_UNIT=(-p termlink-sidecar --lib doorbell::tests::t3757_)

run_tests() {
    local out="$1"; shift
    cargo test "$@" >"$out" 2>&1
    grep -q "^test result:" "$out" || return 2
    return 0
}

# Static sweep: every non-test line in the CLI commands and the MCP tools that names a
# founding-verb method next to an RPC call must pass its params through
# `with_declared_from` (within the call's own lines). Prints offenders; rc 1 if any.
static_sweep() {
    local root="${1:-.}"
    python3 -I - "$root/crates/termlink-cli/src/commands" "$root/crates/termlink-mcp/src/tools.rs" <<'PY'
import os, re, sys
files = []
for a in sys.argv[1:]:
    if os.path.isdir(a):
        files += [os.path.join(a, f) for f in sorted(os.listdir(a)) if f.endswith(".rs")]
    else:
        files.append(a)
lit = re.compile(r'"command\.(inject|execute|exec)"')
bad, seen = [], 0
for f in files:
    lines = open(f).read().split("\n")
    # Skip top-level `#[cfg(test)] mod … { … }` bodies: tests build their own raw requests.
    in_test, i = False, -1
    while i + 1 < len(lines):
        i += 1
        l = lines[i]
        if l == "#[cfg(test)]" and i + 1 < len(lines) and lines[i + 1].startswith("mod "):
            in_test = True
            continue
        if in_test:
            if l == "}":
                in_test = False
            continue
        if not lit.search(l) or l.strip().startswith("//"):
            continue
        win = "\n".join(lines[max(0, i - 3): i + 4])
        if "call" not in win:
            continue  # a match arm / scope table, not a call
        seen += 1
        if "with_declared_from" not in "\n".join(lines[max(0, i - 2): i + 3]):
            bad.append(f"{f}:{i+1}: {l.strip()}")
print(f"call sites: {seen}")
for b in bad:
    print("  unstamped:", b)
sys.exit(1 if bad or seen == 0 else 0)
PY
}

if [ "$mode" = fixtures ]; then
    out="$work/fx.out"; : >"$out"
    for args in SES_UNIT CLI_E2E MCP_E2E SC_UNIT; do
        declare -n a="$args"
        run_tests "$out.part" "${a[@]}" || { echo "TOOLING: cannot run cargo test ${a[*]}"; tail -20 "$out.part"; exit 2; }
        cat "$out.part" >>"$out"
    done
    rc=0
    check() {
        local label="$1"; shift
        local ok=1 t
        for t in "$@"; do
            if grep -q "^test \(.*::\)\?${t} \.\.\. ok" "$out"; then continue; fi
            if grep -q "^test \(.*::\)\?${t} \.\.\. FAILED" "$out"; then echo "  failed: $t"; else echo "  missing: $t"; fi
            ok=0
        done
        if [ "$ok" = 1 ]; then echo "PASS $label"; else echo "FAIL $label"; rc=1; fi
    }
    echo "== T-3757 step 1: founding-verb callers declare 'from'"
    check "1a resolver order (TERMLINK_FROM > TERMLINK_AGENT_ID > user: > uid:, never empty); an explicit from is never overwritten; unstamped call refused by owner-only (control)" \
        t3757_declared_from_order t3757_stamp_keeps_explicit_and_skips_non_objects t3757_stamped_call_is_logged_with_its_from_and_passes_owner_only
    check "1b CLI exec / inject / pty inject rows name the caller; no identity still names the user; owner accepted, other caller refused (control)" \
        t3757_exec_declares_termlink_from t3757_pty_inject_and_inject_declare_agent_id t3757_no_identity_still_names_the_user t3757_owner_only_owner_accepted_other_refused
    check "1c MCP termlink_exec / termlink_inject rows name the caller" \
        t3757_mcp_exec_and_inject_declare_from
    check "1d sidecar doorbell command.notify declares the agent it serves" \
        t3757_notify_declares_the_agent_as_from
    if static_sweep >"$work/sweep.out" 2>&1; then
        echo "PASS 1e every CLI/MCP founding-verb call site stamps from ($(head -1 "$work/sweep.out"))"
    else
        echo "FAIL 1e static sweep"; cat "$work/sweep.out"; rc=1
    fi
    [ "$rc" = 0 ] && echo "ALL PASS (T-3757 step 1)"
    exit "$rc"
fi

if [ "$mode" != mutants ]; then echo "usage: $0 [fixtures|mutants]"; exit 2; fi

# ---- mutants ---------------------------------------------------------------
src="$work/src"
rm -rf "$src"
git clone -q --shared . "$src" || { echo "TOOLING: cannot clone HEAD"; exit 2; }
man="$src/Cargo.toml"
export CARGO_TARGET_DIR="$PWD/$work/target"
cargo test --manifest-path "$man" "${SES_UNIT[@]}" --no-run >"$work/build.out" 2>&1 \
    || { echo "TOOLING: the HEAD clone does not build"; tail -20 "$work/build.out"; exit 2; }

rc=0
# mutant <id> <file> <from> <to> <cargo test args...>   (or <file>=SWEEP for the static sweep)
mutant() {
    local id="$1" file="$2" from="$3" to="$4"; shift 4
    local f="$src/$file"
    cp "$f" "$f.orig"
    if ! python3 -I - "$f" "$from" "$to" <<'PY'
import sys
p, a, b = sys.argv[1:4]
s = open(p).read()
if s.count(a) != 1:
    sys.exit(1)
open(p, "w").write(s.replace(a, b))
PY
    then
        echo "TOOLING $id: pattern not found exactly once in $file"; mv "$f.orig" "$f"; rc=2; return
    fi
    local out="$work/$id.out"
    cargo test --manifest-path "$man" "$@" >"$out" 2>&1
    local trc=$?
    mv "$f.orig" "$f"
    if grep -q "^error\(\[E[0-9]*\]\)\?:" "$out" && ! grep -q "^test result:" "$out"; then
        echo "TOOLING $id: mutant does not compile"; [ "$rc" = 0 ] && rc=2; return
    fi
    if [ "$trc" -ne 0 ] && grep -q "FAILED" "$out"; then
        echo "KILLED $id"
    else
        echo "SURVIVED $id  <-- the tests did not notice"; rc=1
    fi
}
# sweep_mutant <id> <file> <from> <to>: the static sweep must go red on the edited clone.
sweep_mutant() {
    local id="$1" file="$2" from="$3" to="$4"
    local f="$src/$file"
    cp "$f" "$f.orig"
    python3 -I -c 'import sys; p,a,b=sys.argv[1:4]; s=open(p).read(); sys.exit(1) if s.count(a)!=1 else open(p,"w").write(s.replace(a,b))' "$f" "$from" "$to" \
        || { echo "TOOLING $id: pattern not found exactly once in $file"; mv "$f.orig" "$f"; rc=2; return; }
    if static_sweep "$src" >/dev/null 2>&1; then
        echo "SURVIVED $id  <-- the sweep did not notice"; rc=1
    else
        echo "KILLED $id"
    fi
    mv "$f.orig" "$f"
}

F=crates/termlink-session/src/founding_log.rs
W=termlink_session::founding_log::with_declared_from

mutant M1-from-env-ignored "$F" "    nonempty(FROM_ENV)
        .or_else(|| nonempty(AGENT_ID_ENV))" "    nonempty(AGENT_ID_ENV)" \
    -p termlink-session --lib founding_log::tests::t3757_declared_from_order
mutant M2-explicit-from-overwritten "$F" "        if !has {" "        if !has || true {" \
    -p termlink-session --lib founding_log::tests::t3757_stamp
mutant M3-anonymous-fallback "$F" "        .unwrap_or_else(|| format!(\"uid:{uid}\"))" "        .unwrap_or_default()" \
    -p termlink-session --lib founding_log::tests::t3757_declared_from_order
mutant M4-cli-exec-unstamped crates/termlink-cli/src/commands/session.rs "\"command.execute\", $W(params))" "\"command.execute\", params)" \
    -p termlink --test t3757_declared_from t3757_exec
mutant M5-cli-inject-unstamped crates/termlink-cli/src/commands/pty.rs "\"command.inject\", $W(params), timeout_dur)" "\"command.inject\", params, timeout_dur)" \
    -p termlink --test t3757_declared_from t3757_pty_inject
mutant M6-mcp-exec-unstamped crates/termlink-mcp/src/tools.rs "client::rpc_call(reg.socket_path(), \"command.execute\", $W(params))" "client::rpc_call(reg.socket_path(), \"command.execute\", params)" \
    -p termlink-mcp --test t3757_mcp_from
mutant M7-mcp-inject-unstamped crates/termlink-mcp/src/tools.rs "\"command.inject\", $W(serde_json::json!({\"keys\": keys})), timeout)" "\"command.inject\", serde_json::json!({\"keys\": keys}), timeout)" \
    -p termlink-mcp --test t3757_mcp_from
mutant M8-doorbell-no-from crates/termlink-sidecar/src/doorbell.rs '    json!({"ids": ids, "count": count, "from": agent})' '    let _ = agent; json!({"ids": ids, "count": count})' \
    -p termlink-sidecar --lib doorbell::tests::t3757_
sweep_mutant M9-remote-exec-unstamped crates/termlink-cli/src/commands/remote.rs "rpc_client.call(\"command.execute\", serde_json::json!(\"exec\"), $W(params))" "rpc_client.call(\"command.execute\", serde_json::json!(\"exec\"), params)"
sweep_mutant M10-mcp-remote-inject-unstamped crates/termlink-mcp/src/tools.rs ".call(\"command.inject\", req_id, $W(params))" ".call(\"command.inject\", req_id, params)"

case "$rc" in
    0) echo "ALL MUTANTS KILLED (T-3757 step 1)";;
    1) echo "MUTANT SURVIVED (T-3757 step 1)";;
    *) echo "TOOLING: some mutants could not be judged";;
esac
exit "$rc"
