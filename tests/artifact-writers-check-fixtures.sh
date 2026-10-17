#!/usr/bin/env bash
# Fixtures for scripts/check-artifact-writers.sh (T-3432, FX-5 ruling C′). Hermetic: scratch crates.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SUT="$ROOT/scripts/check-artifact-writers.sh"
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
PASS=0; FAIL=0
ok() { PASS=$((PASS + 1)); echo "ok   $*"; }
bad() { FAIL=$((FAIL + 1)); echo "FAIL $*"; }
check() { if eval "$2"; then ok "$1"; else bad "$1"; fi; }

C="$T/crate/src"; mkdir -p "$C"
cat > "$C/lib.rs" <<'EOF'
async fn upload_artifact_chunked(c: &Client) {
    c.call(control::method::ARTIFACT_PUT, p).await;
}
pub async fn send_artifact_via_client(c: &Client) {
    upload_artifact_chunked(c).await;
}
fn cli_send() {
    send_artifact_via_client(&c);
}
// a comment naming send_artifact_via_client( and "artifact.put" is not a writer
#[cfg(test)]
mod tests {
    fn t() { send_artifact_via_client(&c); let m = "artifact.put"; }
}
EOF
cat > "$T/allow" <<EOF
$C/lib.rs::upload_artifact_chunked::artifact-put-rpc  # the emitter
$C/lib.rs::send_artifact_via_client::upload-chunked   # shared path
$C/lib.rs::cli_send::send-artifact                    # today's writer
EOF
run() { bash "$SUT" --root "$C" --allowlist "$T/allow" "$@" 2>&1; }

out=$(run); rc=$?
check "1 every site acknowledged: clean, rc 0, counts them" '[ $rc = 0 ] && grep -q "clean — 3 artifact-store write site(s), all 3 acknowledged" <<< "$out"'
check "2 comment lines and test code are not scanned" '! grep -q "tests\|comment" <<< "$out"'

printf 'fn new_feature() {\n    send_artifact_via_client(&c);\n}\n' > "$C/new.rs"
out=$(run); rc=$?
check "3 a NEW caller of the send path fires, named by fn and rule" '[ $rc = 1 ] && grep -q "new.rs:2  send-artifact  (in fn new_feature)" <<< "$out"'
rm "$C/new.rs"

printf 'fn sneaky() {\n    c.call("artifact.put", p);\n}\n' > "$C/raw.rs"
out=$(run); rc=$?
check "4 a raw artifact.put emitter outside the shared path fires" '[ $rc = 1 ] && grep -q "raw.rs:2  artifact-put-rpc  (in fn sneaky)" <<< "$out"'
rm "$C/raw.rs"

sed 's/fn cli_send()/fn cli_send_renamed()/' "$C/lib.rs" > "$T/x" && cp "$T/x" "$C/lib.rs"
out=$(run); rc=$?
check "5 renaming an allowlisted writer re-fires it (signature is fn-based)" '[ $rc = 1 ] && grep -q "in fn cli_send_renamed" <<< "$out"'
sed -i 's/fn cli_send_renamed()/fn cli_send()/' "$C/lib.rs"

out=$(run --json); rc=$?
check "6 --json is valid and ok:true on the clean tree" 'python3 -c "import json,sys; d=json.loads(sys.argv[1]); sys.exit(0 if d[\"ok\"] and d[\"acknowledged\"]==3 else 1)" "$out"'

bash "$SUT" --root "$T/nope" --allowlist "$T/allow" >/dev/null 2>&1
check "7 no source files: tooling rc 2, never a vacuous clean" '[ $? = 2 ]'

out=$(bash "$SUT" --root "$C" --allowlist "$T/missing" 2>&1); rc=$?
check "8 an absent allowlist acknowledges nothing (all three fire)" '[ $rc = 1 ] && [ "$(grep -c "  crate\|/lib.rs:" <<< "$out")" -ge 3 ]'

# Mutant: if the send-artifact rule is removed, a new caller would pass silently.
sed 's/print NR "\\t" fn "\\tsend-artifact"/next/' "$SUT" > "$T/mutant.sh"
printf 'fn new_feature() {\n    send_artifact_via_client(&c);\n}\n' > "$C/new.rs"
bash "$T/mutant.sh" --root "$C" --allowlist "$T/allow" >/dev/null 2>&1; mrc=$?
rm "$C/new.rs"
check "9 mutant without the send-artifact rule misses the new caller (case 3 is load-bearing)" '[ $mrc = 0 ]'

out=$(bash "$SUT" 2>&1); rc=$?
check "10 the real tree is clean with the tracked allowlist" '[ $rc = 0 ]'

echo "---"
echo "artifact-writers check fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" = 0 ]
