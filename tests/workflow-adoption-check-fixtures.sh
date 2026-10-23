#!/usr/bin/env bash
# T-3544 — fixtures for scripts/check-workflow-adoption.sh. Hermetic: a temp repo root with its own
# manifest, cards and designer projects dir (WORKFLOW_ADOPTION_ROOT / WORKFLOW_ADOPTION_PROJECTS).
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CHK="$ROOT/scripts/check-workflow-adoption.sh"
W="$(mktemp -d)"; trap 'rm -rf "$W"' EXIT
pass=0; fail=0
ok()  { echo "ok   $1"; pass=$((pass+1)); }
bad() { echo "FAIL $1"; fail=$((fail+1)); }

step() { printf '<bpmn:%s id="%s" name="n"><bpmn:extensionElements><aef:uid value="%s"/></bpmn:extensionElements></bpmn:%s>\n' "$1" "$2" "$3" "$1"; }
card() { # <root> <wf> <uid> <version>
    cat > "$1/.fabric/components/workflow-$2-$3.yaml" <<EOF
id: workflow/$2/$3
type: workflow-step
workflow: $2
aef_uid: $3
location: .agentic-framework/.context/designer/projects/$2/v$4.bpmn
EOF
}
setup() { # a clean tree: wf1 adopted at v1 with two steps (a task and a catch event), both carded
    R="$W/$1"; rm -rf "$R"; P="$R/.agentic-framework/.context/designer/projects"
    mkdir -p "$P/wf1" "$R/.fabric/components"
    { echo '<bpmn:definitions xmlns:bpmn="b" xmlns:aef="a">'; step serviceTask n1 s_one; step intermediateCatchEvent n2 s_wait
      echo '<bpmn:exclusiveGateway id="g" name="g"><bpmn:extensionElements><aef:uid value="gw"/></bpmn:extensionElements></bpmn:exclusiveGateway>'
      echo '</bpmn:definitions>'; } > "$P/wf1/v1.bpmn"
    card "$R" wf1 s_one 1; card "$R" wf1 s_wait 1
    printf 'workflows:\n  wf1:\n    status: adopted\n    version: 1\n' > "$R/.fabric/workflow-adoption.yaml"
    printf 'patterns:\n  - glob: "scripts/**/*.sh"\n  - glob: ".agentic-framework/.context/designer/projects/wf1/v1.bpmn"\n' > "$R/.fabric/watch-patterns.yaml"
}
run() { WORKFLOW_ADOPTION_ROOT="$R" WORKFLOW_ADOPTION_PROJECTS="$P" bash "${CHECK:-$CHK}" > "$W/out" 2>&1; echo $?; }

setup c1; [ "$(run)" = 0 ] && grep -q "0 failure" "$W/out" && ok "1 a matching tree is clean (a gateway is not a step)" || bad "1 clean: $(cat "$W/out")"

setup c2; mkdir -p "$P/wf2"; : > "$P/wf2/v1.bpmn"
[ "$(run)" = 1 ] && grep -q "wf2: designer project not in" "$W/out" && ok "2 an unlisted project folder fails" || bad "2: $(cat "$W/out")"

setup c3; rm "$P/wf1/v1.bpmn"; : > "$P/wf1/v2.bpmn"
[ "$(run)" = 1 ] && grep -q "adopted v1 but .* is gone" "$W/out" && ok "3 an adopted version whose diagram is gone fails" || bad "3: $(cat "$W/out")"

setup c4; rm "$R/.fabric/components/workflow-wf1-s_wait.yaml"
[ "$(run)" = 1 ] && grep -q "step s_wait in v1 has no card" "$W/out" && ok "4 a diagram step with no card fails" || bad "4: $(cat "$W/out")"

setup c5; card "$R" wf1 s_gone 1
[ "$(run)" = 1 ] && grep -q "card for s_gone names a step that is not in v1" "$W/out" && ok "5 a card for a step the diagram no longer has fails" || bad "5: $(cat "$W/out")"

setup c6; card "$R" wf1 s_one 2; cp "$P/wf1/v1.bpmn" "$P/wf1/v2.bpmn"
[ "$(run)" = 1 ] && grep -q "points at .*v2.bpmn, not the adopted v1" "$W/out" && ok "6 a card on another version than the adopted one fails" || bad "6: $(cat "$W/out")"

setup c7; cp "$P/wf1/v1.bpmn" "$P/wf1/v2.bpmn"
[ "$(run)" = 0 ] && grep -q "WARN wf1: v2 exists, adopted v1" "$W/out" && ok "7 a newer version only warns" || bad "7: $(cat "$W/out")"

setup c8; printf 'workflows:\n  wf1:\n    status: not-adopted\n' > "$R/.fabric/workflow-adoption.yaml"
[ "$(run)" = 1 ] && grep -q "not-adopted needs a reason" "$W/out" && ok "8 not-adopted without a reason fails" || bad "8: $(cat "$W/out")"

setup c9; rm "$R/.fabric/workflow-adoption.yaml"
[ "$(run)" = 2 ] && ok "9 a missing manifest is a tooling error, never clean" || bad "9: $(cat "$W/out")"

setup c11; printf 'patterns:\n  - glob: "scripts/**/*.sh"\n' > "$R/.fabric/watch-patterns.yaml"
[ "$(run)" = 1 ] && grep -q "adopted v1 is not in watch-patterns.yaml" "$W/out" && ok "11 an adopted diagram missing from watch-patterns fails" || bad "11: $(cat "$W/out")"

setup c12; printf '  - glob: ".agentic-framework/.context/designer/projects/wf1/v2.bpmn"\n' >> "$R/.fabric/watch-patterns.yaml"
[ "$(run)" = 1 ] && grep -q "watches v2, which the manifest does not adopt" "$W/out" && ok "12 a watched version the manifest does not adopt fails" || bad "12: $(cat "$W/out")"

# The check's notion of a step must be the adoption tool's, character for character.
tool_re="$(grep -o "r'<bpmn:(serviceTask[^']*'" "$ROOT/scripts/fabric-register-workflow.sh" | head -1)"
chk_re="$(grep -o "r'<bpmn:(serviceTask[^']*'" "$CHK" | head -1)"
[ -n "$tool_re" ] && [ "$tool_re" = "$chk_re" ] && ok "10 the step kinds are the adoption tool's (same regex)" || bad "10 regex drift: tool=[$tool_re] check=[$chk_re]"

# Mutant: drop the card-vs-diagram comparison. Case 4 must then pass silently, so it is load-bearing.
sed 's/    for miss in sorted(want - have):/    for miss in []:/' "$CHK" > "$W/mut.sh"
grep -q "for miss in \[\]:" "$W/mut.sh" || bad "M1 mutant did not apply"
setup m1; rm "$R/.fabric/components/workflow-wf1-s_wait.yaml"
[ "$(CHECK="$W/mut.sh" run)" = 0 ] && ok "M1 mutant without the comparison misses an uncarded step (case 4 is load-bearing)" || bad "M1 mutant still fired"

echo "---"
echo "workflow-adoption check fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
