#!/usr/bin/env bash
# Fixture suite for B-11 (arc-011 plan unit P-6, T-3696): per-agent X25519 encryption keys in the register,
# the TLBLOB wrapper (format version 2) and the hub's artifact writer check. Runs the hermetic tests that carry
# each listed E-n and prints one PASS line per E-n (or FAIL), and a DEFERRED line, with its reason, for each part
# of a listed E-n that belongs to a later unit. Exit 0 only when every E-n in scope passed; 1 on a failure; 2 when
# the tests could not run.
#
#   bash tests/b11-enckey-fixtures.sh            # the E-n
#   bash tests/b11-enckey-fixtures.sh --mutants  # FP 3.1.b: every mutant must be KILLED
#
# The mutant run works on a clean clone of HEAD (`git clone --shared`, never a worktree, never cd'd into: cargo
# runs with --manifest-path) at $B11_MUT_DIR/src (default .context/working/t3696/src, gitignored), applies one
# source mutation at a time, runs the tests that must catch it, and requires them to FAIL. A mutation that no
# longer applies to the source is a tooling error (2), never a pass.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 2
unset PROJECT_ROOT # T-3755: cargo builds must not see the framework's PROJECT_ROOT
export TMPDIR="${TMPDIR:-/var/tmp}" CARGO_TARGET_TMPDIR="${CARGO_TARGET_TMPDIR:-/var/tmp}"

if [ "${1:-}" = "--mutants" ]; then
    root="${B11_MUT_DIR:-$PWD/.context/working/t3696}"
    src="$root/src"
    export CARGO_TARGET_DIR="${B11_MUT_TARGET:-/var/tmp/b11-mutants/target}"
    mkdir -p "$root" || exit 2
    rm -rf "$src" && git clone -q --shared --no-checkout . "$src" && git -C "$src" checkout -q --detach "$(git rev-parse HEAD)" \
        || { echo "TOOLING: clone of HEAD failed"; exit 2; }
    rc=0
    # mutant <id> <file> <test-cmd> <python-literal-old> <python-literal-new>
    mutant() {
        local id="$1" file="$2" cmd="$3" old="$4" new="$5"
        git -C "$src" checkout -q -- . && git -C "$src" clean -qfd || { echo "TOOLING $id: reset of the clone failed"; return 2; }
        if ! OLD="$old" NEW="$new" F="$src/$file" python3 -I -c '
import os,sys
p=os.environ["F"]; s=open(p).read(); o=os.environ["OLD"]
if s.count(o)!=1: sys.exit(3)
open(p,"w").write(s.replace(o,os.environ["NEW"],1))'; then
            echo "TOOLING $id: the mutation no longer applies to $file"; return 2
        fi
        local log="$root/$id.log"
        eval "$cmd" >"$log" 2>&1
        local trc=$?
        if grep -q "^error\[E\|^error: could not compile" "$log"; then
            echo "TOOLING $id: the mutant does not compile ($log)"; return 2
        fi
        if ! grep -q "^test result:" "$log" && ! grep -q "panicked" "$log"; then
            echo "TOOLING $id: the tests did not run ($log)"; return 2
        fi
        if [ "$trc" -ne 0 ] && grep -q "FAILED\|panicked" "$log"; then
            echo "KILLED $id"; return 0
        fi
        echo "SURVIVED $id ($log)"; return 1
    }
    K=crates/termlink-sidecar/src/enckey.rs
    D=crates/termlink-sidecar/src/daemon.rs
    H=crates/termlink-hub/src/artifact.rs
    B=crates/termlink-bus/src/artifact_store.rs
    P=crates/termlink-protocol/src/tlblob.rs
    MP="--manifest-path $src/Cargo.toml"
    SKU="cargo test $MP -p termlink-sidecar --lib -- enckey::tests::"
    SKW="cargo test $MP -p termlink-sidecar --test b11_enckey_wiring"
    HUB="cargo test $MP -p termlink-hub --lib -- artifact::tests::"
    PROTO="cargo test $MP -p termlink-protocol --lib -- tlblob::"
    run() { mutant "$@"; local r=$?; [ $r -gt $rc ] && rc=$r; return 0; }
    # E-48.5 control: no start-up comparison.
    run M1-no-startup-comparison "$K" "$SKU" \
        '    if have.as_bytes() != &want {' \
        '    if false && have.as_bytes() != &want {'
    run M1w-no-startup-comparison-wired "$K" "$SKW" \
        '    if have.as_bytes() != &want {' \
        '    if false && have.as_bytes() != &want {'
    # E-168: the register's owner, mode and presence.
    run M2-no-owner-check "$K" "$SKW" \
        '    if md.uid() != owner_uid {' \
        '    if false && md.uid() != owner_uid {'
    run M3-group-writable-accepted "$K" "$SKU" \
        '    if md.permissions().mode() & 0o022 != 0 {' \
        '    if md.permissions().mode() & 0o002 != 0 {'
    run M4-missing-register-tolerated "$K" "$SKU" \
        '    let want = read_register(reg, owner_uid)?;' \
        '    let Ok(want) = read_register(reg, owner_uid) else { return Ok(None) };'
    # E-144: production never mints a key.
    run M5-production-mints "$K" "$SKU" \
        '            if production {
                return Err(format!(
                    "encryption key {} is missing' \
        '            if false && production {
                return Err(format!(
                    "encryption key {} is missing'
    # E-144 wiring: the daemon must run the check with the configured register.
    run M6-daemon-skips-the-check "$D" "$SKW" \
        'crate::enckey::startup(store.dir(), cfg.enc_key_register.as_deref(), production, reg_owner)?' \
        'crate::enckey::startup(store.dir(), None, production, reg_owner)?'
    # E-47.5 control: the arc-011 writer path uploads plaintext.
    run M7-wrapper-writer-uploads-plaintext "$H" "$HUB" \
        'Some(_) => store.put_streaming_validated(staging_id, offset, chunk, is_final, expected_sha256, &tlblob::check_header),' \
        'Some(_) => store.put_streaming(staging_id, offset, chunk, is_final, expected_sha256),'
    run M8-store-ignores-the-validator "$B" "$HUB" \
        '        if let Some(check) = validate
            && let Err(reason) = check(&bytes)' \
        '        if let Some(check) = validate.filter(|_| false)
            && let Err(reason) = check(&bytes)'
    run M9-header-skips-magic "$P" "$HUB" \
        '    if &obj[..6] != MAGIC {' \
        '    if false && &obj[..6] != MAGIC {'
    run M10-header-accepts-v1 "$P" "$PROTO" \
        '    if obj[6] != VERSION {' \
        '    if obj[6] != VERSION && obj[6] != 1 {'
    # CR-4.10 associated data: `to` verbatim must be bound.
    run M11-aad-drops-to "$K" "$SKU" \
        '        m.insert("to", json!(self.to));' \
        '        m.insert("to", json!("any"));'
    # NID: baseline writers (no `writer` parameter) are stored as today.
    run M12-baseline-writers-checked-too "$H" "$HUB" \
        '        None | Some(Value::Null) => store.put_streaming(staging_id, offset, chunk, is_final, expected_sha256),
        Some(_) =>' \
        '        Some(Value::Null) => store.put_streaming(staging_id, offset, chunk, is_final, expected_sha256),
        _ =>'
    [ "$rc" = 0 ] && echo "ALL MUTANTS KILLED"
    exit "$rc"
fi

out="$(mktemp)"
trap 'rm -f "$out" "$out".*' EXIT
cargo test -p termlink-sidecar --lib enckey >"$out.sku" 2>&1
cargo test -p termlink-sidecar --test b11_enckey_wiring >"$out.skw" 2>&1
cargo test -p termlink-hub --lib artifact::tests >"$out.hub" 2>&1
cargo test -p termlink-protocol --lib tlblob >"$out.proto" 2>&1
cargo test -p termlink-sidecar --test p7_wiring_tests >"$out.p7" 2>&1
for f in sku skw hub proto p7; do
    grep -q "^test result:" "$out.$f" || { echo "TOOLING: the $f tests did not run"; tail -20 "$out.$f"; exit 2; }
done
cat "$out.sku" "$out.skw" "$out.hub" "$out.proto" "$out.p7" >"$out"
rc=0
# check <label> <test-name>...: every named test ran and passed (a name absent from the run is a FAIL).
check() {
    local label="$1"; shift
    local ok=1 t
    for t in "$@"; do
        if ! grep -q "^test \(enckey::tests::\|artifact::tests::\|tlblob::tests::\)\?${t} \.\.\. ok" "$out"; then
            echo "  missing or failed: $t"; ok=0
        fi
    done
    if [ "$ok" = 1 ]; then echo "PASS $label"; else echo "FAIL $label"; rc=1; fi
}
echo "== P-6 (T-3696): B-11 per-agent encryption keys, TLBLOB wrapper, artifact writer check"
check "E-47 (L2: the wrapper writer's object holds no plaintext canary; an unwrapped object from a wrapper writer is refused and leaves no copy)" \
    e47_sealed_object_is_ciphertext_only_with_wrapper_header t3696_wrapper_writer_plaintext_refused_and_never_stored \
    t3696_wrapper_writer_chunked_plaintext_refused_at_finalize t3696_wrapper_writer_wrapped_object_accepted \
    header_accepts_v2_alg1 header_refuses_plaintext_short_v1_and_unknown_alg
check "E-48 (swapped register refuses start with both fingerprints; A's blob does not open with B's key; no partial plaintext)" \
    e48_swapped_register_refused_with_both_fingerprints e48_blob_to_a_does_not_open_with_b \
    e48_tamper_and_wrong_aad_fail_authenticated_decryption e144_e48_register_check_runs_at_start
check "E-144 (the register entry matches the private key; production never mints; status keys.encryption register: match)" \
    e144_register_matches_private_key_starts e144_key_file_mode_and_production_never_mints e144_e48_register_check_runs_at_start
check "E-168 (B-11 part: a register entry missing, not root-owned, group/world-writable, a link or not a key refuses start naming it)" \
    e168_bad_register_refuses_start_naming_the_entry e168_production_register_not_root_owned_refuses_start
check "NID (no register configured starts in the rollback state; baseline artifact writers and the signing-key register unchanged)" \
    e144_no_register_is_rollback_state_not_a_refusal t3696_baseline_writer_unchanged_nid put_complete_roundtrip put_chunked_5mb_roundtrip \
    a_key_that_differs_from_the_register_is_a_refused_start a_matching_register_passes_the_key_check
echo "DEFERRED E-47 L4 part (blobs sent through a throwaway hub, the whole store read with a hub token, the over-cap inline field): needs the blob send path, plan unit P-21 (B-12, T-3707), which also lists E-47"
echo "DEFERRED E-47 writer inventory '9 plus the wrapper' in scripts/check-arc011-inventory.sh: the inventory check counts states and markers only; the writer list belongs to E-150 (P-33)"
echo "DEFERRED E-168 forged QUOTA notice (B-7) and failed blob upload left in the outbox (B-12, P-21): other build items of the same E-n"
echo "DEFERRED E-144 live part (every W-agent on .107 has a matching register entry): the credential migration P-31 (T-3713)"
[ "$rc" = 0 ] && echo "ALL IN-SCOPE PASS"
exit "$rc"
