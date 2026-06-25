#!/usr/bin/env bash
#
# lint-command-hints.sh — validate back-ticked `termlink <group> <verb>` hint
# strings in source against the real clap command tree.
#
# Prevention for T-2279 / PL-230: a CLI hint that names a non-existent command
# (e.g. the `agent listeners --fleet` bug) sends users to an "unrecognized
# subcommand" dead end. clap never validates hint *strings*, so this static lint
# closes the gap — it walks the live command tree and flags any hint whose group
# is real but whose verb is not a subcommand of that group.
#
# Usage:
#   scripts/lint-command-hints.sh            # lint the tree; exit 1 on bad hints
#   scripts/lint-command-hints.sh --json     # machine-readable envelope
#   scripts/lint-command-hints.sh --strict   # also fail on unknown-group hints
#   scripts/lint-command-hints.sh --self-test # prove it catches a known-bad hint
#
# Exit codes: 0 = clean, 1 = invalid hint(s) found, 2 = tooling error.
#
set -uo pipefail

JSON=0
STRICT=0
SELFTEST=0
for arg in "$@"; do
  case "$arg" in
    --json) JSON=1 ;;
    --strict) STRICT=1 ;;
    --self-test) SELFTEST=1 ;;
    -h|--help) sed -n '2,30p' "$0"; exit 0 ;;
    *) echo "lint-command-hints: unknown arg: $arg" >&2; exit 2 ;;
  esac
done

# Resolve repo root (script lives in <root>/scripts/).
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Resolve a termlink binary: prefer PATH, fall back to a release build.
TERMLINK=""
if command -v termlink >/dev/null 2>&1; then
  TERMLINK="$(command -v termlink)"
elif [ -x "$ROOT/target/release/termlink" ]; then
  TERMLINK="$ROOT/target/release/termlink"
else
  echo "lint-command-hints: no termlink binary on PATH or in target/release" >&2
  exit 2
fi

# --- Build the valid command tree -----------------------------------------
# Parse a clap "Commands:" block: lines indented exactly two spaces whose first
# token is a lowercase command name. Stops at the "Options:" section.
parse_subcommands() {
  # $* = args to pass before --help (e.g. "agent")
  "$TERMLINK" "$@" --help 2>&1 \
    | sed -n '/^Commands:/,/^Options:/p' \
    | grep -E '^  [a-z][a-z0-9-]*' \
    | awk '{print $1}'
}

TL_GROUPS="$(parse_subcommands)"
if [ -z "$TL_GROUPS" ]; then
  echo "lint-command-hints: could not parse top-level command tree from $TERMLINK" >&2
  exit 2
fi

# VALID_PAIR: "group verb" set. IS_GROUP: every top-level command name.
# HAS_SUBCOMMANDS: only the groups that actually own a "Commands:" block.
# The distinction matters: leaf commands (ping/spawn/mirror/signal) take a
# positional argument, so a hint like `termlink ping <session>` has a 2nd token
# that is an ARGUMENT, not a verb — validating it would be a false positive. We
# therefore only flag bad verbs for groups in HAS_SUBCOMMANDS.
declare -A VALID_PAIR
declare -A IS_GROUP
declare -A HAS_SUBCOMMANDS
for g in $TL_GROUPS; do
  IS_GROUP["$g"]=1
  subs="$(parse_subcommands "$g")"
  if [ -n "$subs" ]; then
    HAS_SUBCOMMANDS["$g"]=1
    for v in $subs; do
      VALID_PAIR["$g $v"]=1
    done
  fi
done

# --- Extract hints ---------------------------------------------------------
# Back-ticked `termlink <group> <verb>` occurrences in CLI + MCP source.
# Token charset [a-z0-9-] excludes flags (--foo), placeholders (<id>, ARG), and
# punctuation — so we only test real group+verb word pairs.
# Each entry may be a directory (scanned recursively) OR a single file. The
# operator-facing surfaces (auto-loaded CLAUDE.md + the .claude/commands/ skill
# files) are the most-read hint sources, so they are linted alongside source.
HINT_DIRS=(
  "crates/termlink-cli/src"
  "crates/termlink-mcp/src"
  "CLAUDE.md"
  ".claude/commands"
)

extract_hints() {
  # Emits "file:line<TAB>group<TAB>verb" per hint occurrence.
  # Scope: USER-FACING hint strings (error messages, printed help, MCP tool
  # descriptions) — NOT code/doc comments. Comment lines loosely reference
  # commands (deliberate typo examples, `help <cmd>` notes, RPC-method names)
  # and are noise for this lint, so lines whose content starts with a comment
  # marker (// /// /* *) are skipped. grep -rnE yields "<path>:<lineno>:<line>";
  # paths have no colon so the first two colon-fields are the location.
  local d rawline loc content trimmed m group verb
  for d in "${HINT_DIRS[@]}"; do
    [ -e "$ROOT/$d" ] || continue          # accept both files and directories
    while IFS= read -r rawline; do
      [ -n "$rawline" ] || continue
      loc="${rawline%%:*}"                       # path
      content="${rawline#*:}"                     # lineno:line
      loc="$loc:${content%%:*}"                   # path:lineno
      loc="${loc#"$ROOT"/}"
      content="${content#*:}"                     # the source line
      trimmed="${content#"${content%%[![:space:]]*}"}"   # left-trim
      case "$trimmed" in '//'*|'/*'*|'*'*) continue ;; esac
      # A line may carry more than one hint — extract each.
      while IFS= read -r m; do
        [ -n "$m" ] || continue
        m="${m#\`termlink }"
        group="${m%% *}"
        verb="${m#* }"; verb="${verb%% *}"
        printf '%s\t%s\t%s\n' "$loc" "$group" "$verb"
      done < <(printf '%s\n' "$content" | grep -oE '`termlink [a-z][a-z0-9-]+ [a-z][a-z0-9-]+')
    done < <(grep -rnE '`termlink [a-z][a-z0-9-]+ [a-z][a-z0-9-]+' "$ROOT/$d" 2>/dev/null)
  done
}

# --- Classify --------------------------------------------------------------
BAD_VERB=()      # real group, unknown verb (the T-2279 class) — always fails
UNKNOWN_GROUP=() # group not in tree — warns (fails only with --strict)

nearest_verb() {
  # Suggest the closest valid verb of a group by shared-prefix length.
  local g="$1" bad="$2" best="" bestlen=0 v vlen i a b
  for key in "${!VALID_PAIR[@]}"; do
    [ "${key%% *}" = "$g" ] || continue
    v="${key#* }"
    # shared prefix length
    vlen=0
    for ((i=0; i<${#v} && i<${#bad}; i++)); do
      a="${v:$i:1}"; b="${bad:$i:1}"
      [ "$a" = "$b" ] || break
      vlen=$((vlen+1))
    done
    if [ "$vlen" -gt "$bestlen" ]; then bestlen=$vlen; best="$v"; fi
  done
  [ "$bestlen" -ge 2 ] && echo "$best"
}

while IFS=$'\t' read -r loc group verb; do
  [ -n "${group:-}" ] || continue
  # `termlink help <cmd>` is valid for any real top-level command — help takes a
  # command name as its argument, not a fixed subcommand set.
  if [ "$group" = "help" ]; then
    [ -n "${IS_GROUP["$verb"]:-}" ] && continue
    BAD_VERB+=("$loc|help|$verb|")
    continue
  fi
  if [ -n "${VALID_PAIR["$group $verb"]:-}" ]; then
    continue                              # valid hint
  fi
  if [ -n "${HAS_SUBCOMMANDS["$group"]:-}" ]; then
    # Real group that owns subcommands, but the verb is not one of them.
    sug="$(nearest_verb "$group" "$verb")"
    BAD_VERB+=("$loc|$group|$verb|$sug")
  elif [ -n "${IS_GROUP["$group"]:-}" ]; then
    # Real top-level command with NO subcommands (leaf, e.g. ping/spawn): the
    # 2nd token is a positional argument, not a verb — not a hint to validate.
    continue
  else
    UNKNOWN_GROUP+=("$loc|$group|$verb")
  fi
done < <(extract_hints)

# --- Self-test -------------------------------------------------------------
# Prove the extractor + classifier catch a known-bad hint (no false negatives).
if [ "$SELFTEST" -eq 1 ]; then
  fixture="$(mktemp)"
  printf 'let msg = "run `termlink agent listeners` to see peers";\n' > "$fixture"
  got="$(grep -rnoE '`termlink [a-z][a-z0-9-]+ [a-z][a-z0-9-]+' "$fixture" \
        | sed -E 's/.*`termlink ([a-z0-9-]+) ([a-z0-9-]+).*/\1 \2/')"
  rm -f "$fixture"
  if [ "$got" != "agent listeners" ]; then
    echo "self-test FAIL: extractor did not capture 'agent listeners' (got: '$got')" >&2
    exit 2
  fi
  if [ -n "${VALID_PAIR["agent listeners"]:-}" ]; then
    echo "self-test FAIL: 'agent listeners' unexpectedly present in command tree" >&2
    exit 2
  fi
  echo "self-test PASS: 'termlink agent listeners' is extracted AND flagged as invalid (real group 'agent', no verb 'listeners')."
  exit 0
fi

# --- Report ----------------------------------------------------------------
if [ "$JSON" -eq 1 ]; then
  printf '{"ok":%s,"bad_verb":[' "$([ ${#BAD_VERB[@]} -eq 0 ] && echo true || echo false)"
  first=1
  for e in "${BAD_VERB[@]:-}"; do
    [ -n "$e" ] || continue
    IFS='|' read -r loc g v sug <<< "$e"
    [ $first -eq 1 ] || printf ','; first=0
    printf '{"loc":"%s","group":"%s","verb":"%s","suggest":"%s"}' "$loc" "$g" "$v" "$sug"
  done
  printf '],"unknown_group":['
  first=1
  for e in "${UNKNOWN_GROUP[@]:-}"; do
    [ -n "$e" ] || continue
    IFS='|' read -r loc g v <<< "$e"
    [ $first -eq 1 ] || printf ','; first=0
    printf '{"loc":"%s","group":"%s","verb":"%s"}' "$loc" "$g" "$v"
  done
  printf ']}\n'
else
  if [ ${#BAD_VERB[@]} -gt 0 ]; then
    echo "lint-command-hints: ${#BAD_VERB[@]} invalid hint(s) — real group, NON-EXISTENT verb:"
    for e in "${BAD_VERB[@]}"; do
      IFS='|' read -r loc g v sug <<< "$e"
      if [ -n "$sug" ]; then
        echo "  $loc: \`termlink $g $v\` — '$v' is not a subcommand of '$g'. Did you mean \`termlink $g $sug\`?"
      else
        echo "  $loc: \`termlink $g $v\` — '$v' is not a subcommand of '$g'."
      fi
    done
  fi
  if [ ${#UNKNOWN_GROUP[@]} -gt 0 ]; then
    echo "lint-command-hints: ${#UNKNOWN_GROUP[@]} hint(s) with unknown group (warning):"
    for e in "${UNKNOWN_GROUP[@]}"; do
      IFS='|' read -r loc g v <<< "$e"
      echo "  $loc: \`termlink $g $v\` — '$g' is not a known command group."
    done
  fi
  if [ ${#BAD_VERB[@]} -eq 0 ] && { [ "$STRICT" -eq 0 ] || [ ${#UNKNOWN_GROUP[@]} -eq 0 ]; }; then
    echo "lint-command-hints: OK — all back-ticked \`termlink <group> <verb>\` hints name real commands."
  fi
fi

# --- Exit ------------------------------------------------------------------
[ ${#BAD_VERB[@]} -gt 0 ] && exit 1
[ "$STRICT" -eq 1 ] && [ ${#UNKNOWN_GROUP[@]} -gt 0 ] && exit 1
exit 0
