#!/usr/bin/env bash
# T-3544 — fixtures for scripts/fabric-command-cards.py (slash-command fabric cards from what a command RUNS).
# Hermetic: a temp repo root with its own .claude/commands, scripts/ and .fabric/components.
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
GEN="$ROOT/scripts/fabric-command-cards.py"
W="$(mktemp -d)"; trap 'rm -rf "$W"' EXIT
pass=0; fail=0
ok()  { echo "ok   $1"; pass=$((pass+1)); }
bad() { echo "FAIL $1"; fail=$((fail+1)); }

setup() {
    rm -rf "$1"; mkdir -p "$1/.claude/commands" "$1/scripts" "$1/.fabric/components" "$1/crates/termlink-cli/src"
    touch "$1/scripts/runs.sh" "$1/scripts/mentioned.sh" "$1/crates/termlink-cli/src/main.rs"
    cat > "$1/.claude/commands/demo.md" <<'EOF'
# /demo — a fixture command

Background: see scripts/mentioned.sh for history (prose, not run).

```bash
bash scripts/runs.sh --json
termlink channel claims-summary --all
```

Then `fw task update T-1 --status started-work` and `scripts/missing.sh` (absent: ignored).
EOF
}

R="$W/r1"; setup "$R"
python3 "$GEN" --root "$R" > "$W/out" 2>&1 && [ -f "$R/.fabric/components/claude-commands-demo.yaml" ] \
    && ok "1 a card is written per command file" || bad "1 write: $(cat "$W/out")"
python3 - "$R/.fabric/components/claude-commands-demo.yaml" > "$W/chk" 2>&1 <<'EOF'
import sys, yaml
c = yaml.safe_load(open(sys.argv[1]))
targets = [d["target"] for d in c["depends_on"]]
assert targets == ["scripts/runs.sh", "crates/termlink-cli/src/main.rs"], targets
cli = {i["tool"]: i["commands"] for i in c["interfaces"] if i.get("type") == "cli"}
assert cli == {"termlink": ["channel claims-summary"], "fw": ["task update"]}, cli
assert c["type"] == "slash-command" and c["purpose"].startswith("/demo"), c
EOF
[ $? -eq 0 ] && ok "2 depends_on holds the script it runs and the termlink CLI; prose mentions and absent files are not dependencies" \
    || bad "2 card content: $(cat "$W/chk")"

python3 "$GEN" --root "$R" --check > "$W/out" 2>&1 && ok "3 --check is clean right after a refresh" || bad "3 check: $(cat "$W/out")"

printf '\n```\nbash scripts/mentioned.sh\n```\n' >> "$R/.claude/commands/demo.md"
python3 "$GEN" --root "$R" --check > "$W/out" 2>&1; rc=$?
[ "$rc" = 1 ] && grep -q "depends_on differs" "$W/out" && ok "4 --check fires when a command starts running a new script" || bad "4 drift rc=$rc: $(cat "$W/out")"

cat > "$R/.claude/commands/new.md" <<'EOF'
# /new — added later
EOF
python3 "$GEN" --root "$R" --check > "$W/out" 2>&1; rc=$?
[ "$rc" = 1 ] && grep -q "new.md: no card" "$W/out" && ok "5 --check fires on a command file with no card" || bad "5 no card rc=$rc: $(cat "$W/out")"

rm -rf "$R/.claude/commands"; python3 "$GEN" --root "$R" --check > "$W/out" 2>&1; rc=$?
[ "$rc" = 2 ] && ok "6 an unreadable commands dir is a tooling error, never clean" || bad "6 rc=$rc"

# Mutant: count every mention, not only code. Case 2 must then fail (mentioned.sh becomes a dependency).
M="$W/mut"; mkdir -p "$M"
sed 's/    for span in code_spans(text):/    for span in text.splitlines():/' "$GEN" > "$M/gen.py"
grep -q "for span in text.splitlines():" "$M/gen.py" || bad "M1 mutant did not apply"
R2="$W/r2"; setup "$R2"; python3 "$M/gen.py" --root "$R2" > /dev/null 2>&1
grep -q "scripts/mentioned.sh" "$R2/.fabric/components/claude-commands-demo.yaml" \
    && ok "M1 mutant counting prose makes a mention a dependency (so case 2 is load-bearing)" || bad "M1 mutant not caught"

echo "---"
echo "fabric-command-cards fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
