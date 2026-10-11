#!/usr/bin/env bash
# guard-layer: source --help
# check-test-artifact-secrets.sh (arc-011 PH-0 6.2.1.a (6), SI-25, fixture FC-17)
#
# WHY: a test run writes logs, status output, captured frames and records. If a hub secret,
# private key, token or the FC-17 secret canary appears in any of them, the test run itself
# leaked it. SI-25: no secret or key material in logs, status output, frames or records.
#
# WHAT: scan every file (binary included: frames) under DIR for
#   (1) the vendored secret-scan catalogue (.secret-scan-patterns, same file the pre-commit
#       hook uses; not re-invented here),
#   (2) a hub-secret shape: secret/token key followed by 64 hex chars,
#   (3) literal canary values (--canary V, repeatable, or $TERMLINK_TEST_SECRET_CANARY),
#   (4) literal values read from secret files (--secret-file F, repeatable; e.g. a test hub's
#       hub.secret): the file's content must appear nowhere in the artifacts.
# Findings print file:line and the rule name only, NEVER the matched value.
#
# SCOPE: a clean result means none of the above shapes/values were found in DIR. It does not
# prove a secret in another shape, or outside DIR, is absent.
#
# Exit 0 clean / 1 found / 2 tooling (fail-closed: missing or empty DIR, unreadable catalogue,
# empty/short canary or secret value are never "clean").
# Test seam: SECRET_ARTIFACT_MUTANT_NO_CANARY=1 disables literal matching (fixture mutant).
set -u

usage() {
  cat <<'U'
usage: check-test-artifact-secrets.sh DIR [--canary VALUE]... [--secret-file FILE]...
                                          [--patterns FILE] [--json] [--quiet] [--help]
exit: 0 clean, 1 secret material found, 2 tooling error (fail-closed)
env:  TERMLINK_TEST_SECRET_CANARY  extra canary literal (FC-17)
U
}

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/.." && pwd)"
dir=""; patterns=""; json=0; quiet=0
canaries=(); sfiles=()
[ -n "${TERMLINK_TEST_SECRET_CANARY:-}" ] && canaries+=("$TERMLINK_TEST_SECRET_CANARY")

while [ $# -gt 0 ]; do
  case "$1" in
    --help|-h) usage; exit 0 ;;
    --canary) [ $# -ge 2 ] || { echo "error: --canary needs a value" >&2; exit 2; }; canaries+=("$2"); shift ;;
    --secret-file) [ $# -ge 2 ] || { echo "error: --secret-file needs a path" >&2; exit 2; }; sfiles+=("$2"); shift ;;
    --patterns) [ $# -ge 2 ] || { echo "error: --patterns needs a path" >&2; exit 2; }; patterns="$2"; shift ;;
    --json) json=1 ;;
    --quiet) quiet=1 ;;
    -*) echo "error: unknown option $1" >&2; usage >&2; exit 2 ;;
    *) [ -z "$dir" ] && dir="$1" || { echo "error: extra argument $1" >&2; exit 2; } ;;
  esac
  shift
done

[ -n "$dir" ] || { usage >&2; exit 2; }
[ -d "$dir" ] || { echo "error: not a directory: $dir" >&2; exit 2; }
if [ -z "$(find "$dir" -type f -print -quit 2>/dev/null)" ]; then
  echo "error: no files under $dir (an empty artifact set is not a clean one)" >&2; exit 2
fi
if [ -z "$patterns" ]; then
  for c in "$ROOT/.secret-scan-patterns" "$ROOT/.agentic-framework/.secret-scan-patterns"; do
    [ -f "$c" ] && { patterns="$c"; break; }
  done
fi
[ -n "$patterns" ] && [ -r "$patterns" ] || { echo "error: secret-scan pattern catalogue not found" >&2; exit 2; }

findings=()   # "file:line<TAB>rule"
scan() {      # scan <mode:E|F> <rule> <needle>
  local mode="$1" rule="$2" needle="$3" out rc
  out=$(grep -rna"$mode" -e "$needle" -- "$dir" 2>/dev/null); rc=$?
  [ "$rc" -le 1 ] || { echo "error: grep failed (rc=$rc) on rule $rule" >&2; exit 2; }
  out=$(printf '%s\n' "$out" | cut -d: -f1,2 | grep -v '^$')
  [ -n "$out" ] && while IFS= read -r l; do findings+=("$l"$'\t'"$rule"); done <<<"$out"
  return 0
}

# (1) catalogue
while IFS=$'\t' read -r name re; do
  case "$name" in ''|'#'*) continue ;; esac
  [ -n "${re:-}" ] || continue
  scan E "catalogue:$name" "$re"
done < <(tr -d '\r' < "$patterns")

# (2) hub-secret shape
scan E "hub-secret-shape" '(secret|token)[A-Za-z_-]*["'"'"']?[[:space:]]*[:=][[:space:]]*["'"'"']?[0-9a-fA-F]{64}([^0-9a-fA-F]|$)'

# (3)(4) literals
if [ "${SECRET_ARTIFACT_MUTANT_NO_CANARY:-0}" != "1" ]; then
  for c in "${canaries[@]+"${canaries[@]}"}"; do
    [ "${#c}" -ge 8 ] || { echo "error: canary shorter than 8 chars would match noise" >&2; exit 2; }
    scan F "canary" "$c"
  done
  for f in "${sfiles[@]+"${sfiles[@]}"}"; do
    [ -r "$f" ] || { echo "error: unreadable secret file: $f" >&2; exit 2; }
    v=$(tr -d '\r\n' < "$f")
    [ "${#v}" -ge 16 ] || { echo "error: secret file $f holds fewer than 16 chars" >&2; exit 2; }
    scan F "secret-file:$(basename "$f")" "$v"
  done
fi

n=${#findings[@]}
if [ "$json" = 1 ]; then
  printf '{"ok":%s,"count":%d,"findings":[' "$([ "$n" = 0 ] && echo true || echo false)" "$n"
  i=0; for f in "${findings[@]+"${findings[@]}"}"; do
    [ $i -gt 0 ] && printf ','
    printf '{"at":"%s","rule":"%s"}' "${f%%$'\t'*}" "${f#*$'\t'}"; i=$((i+1))
  done; printf ']}\n'
fi
if [ "$n" -gt 0 ]; then
  if [ "$json" = 0 ]; then
    echo "SECRET MATERIAL in test artifacts ($n finding(s), values withheld):"
    printf '%s\n' "${findings[@]}" | sort -u | sed 's/\t/  rule=/'
  fi
  exit 1
fi
[ "$quiet" = 1 ] || [ "$json" = 1 ] || echo "clean: no secret material found under $dir (catalogue + hub-secret shape + ${#canaries[@]} canary + ${#sfiles[@]} secret file(s); scope: this directory only)"
exit 0
