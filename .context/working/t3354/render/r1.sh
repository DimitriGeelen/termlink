#!/usr/bin/env bash
# T-3354 render check: extract each mermaid block of the step-5 output and render it with mmdc + local chromium.
set -uo pipefail
doc=/opt/termlink/docs/design/interactive-agent-communication-05-evidence.md
d=/opt/termlink/.context/working/t3354/render
cd "$d" || exit 2
echo '{"executablePath":"/usr/bin/chromium","args":["--no-sandbox"]}' > p.json
awk -v d="$d" '/^```mermaid$/{n++; f=sprintf("%s/b%d.mmd",d,n); inb=1; next} /^```$/{if(inb){inb=0; close(f)}; next} inb{print > f}' "$doc"
rc=0
for m in b*.mmd; do
  b=${m%.mmd}
  if mmdc -p p.json -i "$m" -o "$b.svg" > "$b.log" 2>&1; then r=0; else r=$?; fi
  if grep -qi -E "syntax error|parse error" "$b.log" "$b.svg" 2>/dev/null; then e=yes; else e=no; fi
  echo "$b exit=$r errors=$e svg_bytes=$(stat -c %s "$b.svg" 2>/dev/null || echo 0)"
  [ "$r" -ne 0 ] || [ "$e" = yes ] && rc=1
done
mmdc -p p.json -i b1.mmd -o b1.png -w 1600 > /dev/null 2>&1
mmdc -p p.json -i b2.mmd -o b2.png -w 1400 > /dev/null 2>&1
exit $rc
