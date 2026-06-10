#!/usr/bin/env bash
# lint-doc-fenced-bash.sh — PL-206 mitigation layer (c).
#
# Validates every `termlink <verb> [<subverb>]` invocation in fenced
# ```bash (and ```sh / ```shell) code blocks across
# docs/operations/*.md, .claude/commands/*.md, and CLAUDE.md against
# the LIVE CLI's verb tree, built from `termlink --help` plus per-verb
# `termlink <verb> --help` output.
#
# Companion to layer (b) (scripts/lint-doc-cli-references.sh), which
# catches three KNOWN drift patterns. Layer (c) catches the broader
# generic class: any invocation referencing a top-verb or subverb that
# no longer exists in the live CLI.
#
# Drift class (PL-206): markdown code blocks are inert text — never
# executed, never type-checked. When the CLI evolves (verb rename,
# subverb removal, subsystem refactor), recipe docs lapse silently
# and first failure surfaces at the user's copy-paste time.
#
# Usage:
#   bash scripts/lint-doc-fenced-bash.sh              # scan and report
#   bash scripts/lint-doc-fenced-bash.sh --help       # show help
#   bash scripts/lint-doc-fenced-bash.sh --json       # JSON envelope
#
# Env vars:
#   TERMLINK_BIN   Path to termlink binary (default: from PATH)
#
# Exit codes: 0 clean, 1 drift found, 2 arg/setup error
#
# References: PL-206 (.context/project/learnings.yaml), T-2133 (this),
# T-2132 (layer b), T-2129 (PL-206 captured).

set -u

show_help() {
  cat <<'EOF'
lint-doc-fenced-bash.sh — PL-206 layer (c) doc fenced-bash CLI lint.

Validates every `termlink <verb> [<subverb>]` invocation in fenced
```bash / ```sh / ```shell blocks across:
  docs/operations/*.md
  .claude/commands/*.md
  CLAUDE.md

Drift caught: a verb or subverb not present in the LIVE CLI's
`termlink --help` / `termlink <verb> --help` output.

Companion to layer (b) (scripts/lint-doc-cli-references.sh), which
catches three KNOWN drift patterns. Layer (c) catches the generic
verb-existence class — strictly weaker filter, but catches renames
and removals across the full docs surface.

Flags:
  --help    Show this help and exit 0
  --json    Emit a JSON envelope instead of human-format

Env vars:
  TERMLINK_BIN   Path to termlink binary (default: from PATH)

Exit codes:
  0   Clean — no drift found
  1   One or more invocations reference an unknown verb
  2   Argument error OR termlink binary not on PATH

See: PL-206 in .context/project/learnings.yaml
     scripts/lint-doc-cli-references.sh (layer b)
EOF
}

JSON=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help) show_help; exit 0 ;;
    --json) JSON=1; shift ;;
    *) echo "lint-doc-fenced-bash.sh: unknown arg: $1" >&2; show_help >&2; exit 2 ;;
  esac
done

cd "$(dirname "$0")/.." || { echo "lint-doc-fenced-bash: cannot cd to repo root" >&2; exit 2; }

TERMLINK="${TERMLINK_BIN:-termlink}"
if ! command -v "$TERMLINK" >/dev/null 2>&1; then
  echo "lint-doc-fenced-bash: '$TERMLINK' binary not on PATH. Export TERMLINK_BIN to override." >&2
  exit 2
fi

# Build the verb tree into two temp files.
TMP_TOP="$(mktemp)"
TMP_SUB="$(mktemp)"
TMP_FINDINGS="$(mktemp)"
trap 'rm -f "$TMP_TOP" "$TMP_SUB" "$TMP_FINDINGS"' EXIT

# Top-level verbs from `termlink --help`. The clap-generated help has a
# "Commands:" header, then indented `  <verb>   <description>` lines,
# then a blank line + "Options:" section.
"$TERMLINK" --help 2>/dev/null | awk '
  /^Commands:/ {in_cmds=1; next}
  in_cmds && /^Options:/ {exit}
  in_cmds && /^[A-Z]/ {exit}
  in_cmds && /^[[:space:]]+[a-z]/ {print $1}
' | sort -u > "$TMP_TOP"

if [[ ! -s "$TMP_TOP" ]]; then
  echo "lint-doc-fenced-bash: failed to parse top-level verbs from '$TERMLINK --help'." >&2
  exit 2
fi

# Subverbs: per-top-verb `termlink <verb> --help`. If the output contains a
# "Commands:" section, it's a parent verb; extract its subverbs.
while IFS= read -r v; do
  sub_help="$("$TERMLINK" "$v" --help 2>&1 || true)"
  if printf '%s' "$sub_help" | grep -q '^Commands:'; then
    printf '%s\n' "$sub_help" | awk -v vv="$v" '
      /^Commands:/ {in_cmds=1; next}
      in_cmds && /^Options:/ {exit}
      in_cmds && /^[A-Z]/ {exit}
      in_cmds && /^[[:space:]]+[a-z]/ {print vv " " $1}
    ' >> "$TMP_SUB"
  fi
done < "$TMP_TOP"
sort -u -o "$TMP_SUB" "$TMP_SUB"

# Collect target paths.
paths=()
for p in docs/operations/*.md .claude/commands/*.md CLAUDE.md; do
  [[ -f "$p" ]] && paths+=("$p")
done
if [[ ${#paths[@]} -eq 0 ]]; then
  echo "lint-doc-fenced-bash: no target docs found (run from repo root)" >&2
  exit 2
fi

# Walk each file, track fenced-bash state, extract `termlink <v>[ <s>]`
# patterns. Findings are TAB-separated: file<TAB>line<TAB>kind<TAB>verb<TAB>raw.
for f in "${paths[@]}"; do
  awk -v file="$f" -v topfile="$TMP_TOP" -v subfile="$TMP_SUB" '
    BEGIN {
      while ((getline t < topfile) > 0) { top[t] = 1 }
      close(topfile)
      while ((getline s < subfile) > 0) {
        sub_map[s] = 1
        split(s, parts, " ")
        parent[parts[1]] = 1
      }
      close(subfile)
    }
    /^```bash$|^```sh$|^```shell$/ { in_bash=1; next }
    /^```$/ { in_bash=0; next }
    in_bash {
      line = $0
      rest = line
      # Find every "termlink <verb>[ <subverb>]" occurrence on this line.
      # Verb shape: starts with lowercase, then alphanumeric/_/- — covers
      # all CLI verbs ("dispatch-status", "fleet", "channel", etc.).
      while (match(rest, /termlink [a-z][a-zA-Z0-9_-]*( [a-z][a-zA-Z0-9_-]*)?/)) {
        full = substr(rest, RSTART, RLENGTH)
        n = split(full, parts, " ")
        verb = parts[2]
        subverb = (n >= 3) ? parts[3] : ""
        if (!(verb in top)) {
          print file "\t" NR "\tunknown-top-verb\t" verb "\t" line
        } else if (subverb != "" && (verb in parent) && !((verb " " subverb) in sub_map)) {
          print file "\t" NR "\tunknown-subverb\t" verb " " subverb "\t" line
        }
        rest = substr(rest, RSTART + RLENGTH)
      }
    }
  ' "$f" >> "$TMP_FINDINGS"
done

n_total=$(wc -l < "$TMP_FINDINGS" | tr -d ' ')
[[ -z "$n_total" ]] && n_total=0

n_top=$(wc -l < "$TMP_TOP" | tr -d ' ')
n_sub=$(wc -l < "$TMP_SUB" | tr -d ' ')

if [[ "$JSON" -eq 1 ]]; then
  esc() { printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g; s/\t/ /g'; }
  printf '{"ok":%s,"total_hits":%d,"verb_tree":{"top":%d,"sub":%d},"findings":[' \
    "$([ "$n_total" -eq 0 ] && echo true || echo false)" "$n_total" "$n_top" "$n_sub"
  first=1
  while IFS=$'\t' read -r file line kind verb raw; do
    [[ -z "${file:-}" ]] && continue
    if [[ "$first" -eq 0 ]]; then printf ','; fi
    first=0
    printf '{"file":"%s","line":%s,"kind":"%s","verb":"%s","raw":"%s"}' \
      "$(esc "$file")" "$line" "$kind" "$(esc "$verb")" "$(esc "$raw")"
  done < "$TMP_FINDINGS"
  printf '],"reference":"PL-206"}\n'
  [[ "$n_total" -eq 0 ]] && exit 0 || exit 1
fi

# Human format.
echo "lint-doc-fenced-bash.sh — PL-206 layer (c) doc fenced-bash CLI lint"
echo "  Scanned: ${#paths[@]} file(s) across docs/operations + .claude/commands + CLAUDE.md"
echo "  Verb tree: $n_top top verbs, $n_sub subverbs (from '$TERMLINK --help')"
echo

if [[ "$n_total" -eq 0 ]]; then
  echo "  Status: clean — no drift found ✓"
  echo "  PL-206 prevention layer (c) reports all fenced-bash termlink invocations resolve."
  exit 0
fi

echo "  Status: DRIFT FOUND ($n_total invocation(s) reference unknown verb(s))"
echo
while IFS=$'\t' read -r file line kind verb raw; do
  [[ -z "${file:-}" ]] && continue
  printf '  %s:%s  %s: %s\n' "$file" "$line" "$kind" "$verb"
  printf '    raw: %s\n' "$raw"
done < "$TMP_FINDINGS"
echo
echo "  Fix: rename to the correct verb (check 'termlink <parent> --help'), or remove the stale invocation."
echo "  Reference: PL-206 in .context/project/learnings.yaml"
echo "  Companion: scripts/lint-doc-cli-references.sh (layer b — known patterns)"
exit 1
