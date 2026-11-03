#!/bin/bash
# T-3353 round 1: re-render every Mermaid block of ARC4 (section 24 method).
set -u
cd /opt/termlink
export TMPDIR=/var/tmp
D=.context/working/t3353/render/r1
mkdir -p "$D"
python3 - <<'EOF'
import re
s=open('docs/design/interactive-agent-communication-04-architecture.md').read()
for n,b in enumerate(re.findall(r'```mermaid\n(.*?)```',s,re.S),1):
    open(f'.context/working/t3353/render/r1/b{n}.mmd','w').write(b)
EOF
echo '{"executablePath":"/usr/bin/chromium","args":["--no-sandbox"]}' > "$D/p.json"
for n in 1 2 3 4 5; do
  mmdc -p "$D/p.json" -i "$D/b$n.mmd" -o "$D/b$n.svg" > "$D/b$n.log" 2>&1
  rc=$?
  echo "b$n rc=$rc size=$(stat -c %s "$D/b$n.svg" 2>/dev/null) errs=$(cat "$D/b$n.svg" "$D/b$n.log" 2>/dev/null | grep -c -i 'syntax error\|parse error')"
done
