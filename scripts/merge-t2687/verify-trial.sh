#!/usr/bin/env bash
# Verify the RESOLVED merge tree: fixture suites + the checks this branch added.
# Run in the scratch worktree; the real branch is untouched.
set -uo pipefail
cd /root/.claude/jobs/d638a35c/tmp/merge-trial || exit 2
pass=0; fail=0
for f in tests/*fixtures.sh; do
    [ -f "$f" ] || continue
    if timeout 300 bash "$f" >/tmp/.vt.out 2>&1; then
        printf 'PASS  %s\n' "$(basename "$f")"; pass=$((pass+1))
    else
        printf 'FAIL  %s\n' "$(basename "$f")"
        grep -iE 'FAIL' /tmp/.vt.out | head -3 | sed 's/^/        /'
        fail=$((fail+1))
    fi
done
echo "----"
echo "fixture suites: $((pass+fail))  passed: $pass  failed: $fail"
echo
echo "=== YAML sanity on the unioned registers ==="
python3 - <<'PY'
import yaml
for p, k in ((".context/project/learnings.yaml", "learnings"),
             (".context/project/decisions.yaml", "decisions"),
             (".context/project/metrics-history.yaml", None)):
    try:
        d = yaml.safe_load(open(p))
        n = len(d.get(k) or []) if (k and isinstance(d, dict)) else "n/a"
        print("  %-42s parses, %s records" % (p, n))
    except Exception as e:
        print("  %-42s FAILS: %s" % (p, str(e).split("\n")[0][:50]))
PY
