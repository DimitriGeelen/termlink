#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
#
# T-3167 — the missing half of arc-008's success condition.
#
# arc-008's headline mechanic requires every remaining audit warning to be
# "fixed OR explicitly acknowledged with a cited reason". `fw audit` has no
# acknowledgement mechanism: the only silencing available is a per-check global
# kill switch (FW_RETIRE_WHEN_ADVISORY=0 is the one example in audit.sh), which
# removes the finding AND the record — the opposite of acknowledging it.
#
# So an operator reading "Warn 8" cannot tell an examined-and-explained warning
# from one nobody has looked at. Measured today: three of the eight ARE
# acknowledged in substance — root cause identified, filed upstream at
# framework:pickup offset 179/178, reason printed in the audit's own Mitigation
# line — and they land in the same count as the unexamined ones. That is how a
# warning list gets re-reported unread ten runs in a row (T-2818 alarm fatigue),
# which is the failure arc-008 exists to end.
#
# This repo already solved this problem twelve times on the other side of the
# guard layer: every static check carries a git-tracked
# .context/checks/<name>-allowlist whose entries are counted and reported but do
# not fire, each with a cited reason (T-2681, T-2483, T-2680). The audit — the
# oldest and most-read surface — never got one. This is that ledger, built as a
# LOCAL post-processor because audit.sh is vendored (G-062) and a local patch to
# it is deleted by the next re-vendor.
#
# Three classes:
#   acknowledged  — WARN matches a ledger entry. Counted and reported, never fires.
#   unexamined    — WARN matches no entry. FIRES (exit 1). The arc's real backlog.
#   stale-entry   — ledger entry matches no current WARN. Reported; fires only
#                   under --strict. This class exists because the ledger can rot
#                   the same way tasks do: T-3117 and T-3009 both carried hard
#                   numbers that the next audit run contradicted and neither
#                   noticed. A ledger with no rot detection would repeat that.
#
# A FAIL is NEVER acknowledgeable. The arc's bar is "zero failures"; silencing a
# failure is not within the mechanism's remit, so any FAIL exits 1 regardless of
# what the ledger says.
#
# Ledger format — one entry per line:
#   <drift-stable pattern>  # <cited reason>
# The pattern is a case-sensitive SUBSTRING of the audit's `check` text. Use the
# check's NAME, not its numbers: check strings embed volatile counts ("93 ... of
# 176", "297/535 cards"), and a pattern containing them silently stops matching
# the moment the count moves — the same drift that made T-3117's premise expire.
# Same precision/recall trade as the sibling checks' fn-name-based signatures.
#
# Exit: 0 = every WARN acknowledged and no FAIL · 1 = unexamined WARN, any FAIL,
#       or (with --strict) a stale entry · 2 = tooling. Fail-closed throughout.
set -uo pipefail

AUDIT_PATH=""
LEDGER_PATH="${FW_AUDIT_ACK_LEDGER:-.context/checks/audit-warning-allowlist}"
AUDIT_DIR="${FW_AUDIT_ACK_DIR:-.context/audits}"
JSON=0; QUIET=0; STRICT=0

while [ $# -gt 0 ]; do
    case "$1" in
        --audit)   AUDIT_PATH="${2:-}"; shift 2 ;;
        --ledger)  LEDGER_PATH="${2:-}"; shift 2 ;;
        --dir)     AUDIT_DIR="${2:-}"; shift 2 ;;
        --json)    JSON=1; shift ;;
        --quiet)   QUIET=1; shift ;;
        --strict)  STRICT=1; shift ;;
        --no-heartbeat) shift ;;   # accepted for guard-layer parity; writes none anyway
        -h|--help)
            sed -n '2,50p' "$0" | sed 's/^# \{0,1\}//'
            exit 0 ;;
        *) echo "check-audit-warning-acknowledgement: unknown argument: $1" >&2; exit 2 ;;
    esac
done

command -v python3 >/dev/null 2>&1 || {
    echo "check-audit-warning-acknowledgement: python3 not found — cannot parse the audit" >&2
    exit 2
}

# Newest DATE-STAMPED audit. The dir also holds non-dated files
# (upgrades.yaml, orchestrator-*.yaml) which are not audit runs and carry no
# `findings` list; a plain `sort | tail -1` picks upgrades.yaml because `u` sorts
# after any digit. Restricting to the YYYY-MM-DD shape makes the selection mean
# what it says. Found by running it: the first draft failed CLOSED on
# upgrades.yaml (exit 2, "no parseable findings") rather than reporting a vacuous
# clean, which is the contract working.
if [ -z "$AUDIT_PATH" ]; then
    if [ ! -d "$AUDIT_DIR" ]; then
        echo "check-audit-warning-acknowledgement: audit dir not found: $AUDIT_DIR" >&2
        exit 2
    fi
    AUDIT_PATH="$(find "$AUDIT_DIR" -maxdepth 1 -type f \
        -regex '.*/[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]\.yaml' 2>/dev/null \
        | sort | tail -1)"
fi
if [ -z "$AUDIT_PATH" ] || [ ! -f "$AUDIT_PATH" ]; then
    echo "check-audit-warning-acknowledgement: no audit file to read (looked in ${AUDIT_DIR})" >&2
    exit 2
fi

AUDIT_PATH="$AUDIT_PATH" LEDGER_PATH="$LEDGER_PATH" JSON="$JSON" QUIET="$QUIET" STRICT="$STRICT" \
python3 <<'PYEOF'
import json, os, sys

audit_path  = os.environ["AUDIT_PATH"]
ledger_path = os.environ["LEDGER_PATH"]
as_json     = os.environ["JSON"] == "1"
quiet       = os.environ["QUIET"] == "1"
strict      = os.environ["STRICT"] == "1"

def tooling(msg):
    if as_json:
        print(json.dumps({"ok": False, "verdict": "tooling", "error": msg}))
    else:
        print("check-audit-warning-acknowledgement: %s" % msg, file=sys.stderr)
    sys.exit(2)

try:
    import yaml
except ImportError:
    tooling("PyYAML not available — cannot parse the audit (fail-closed, never a vacuous clean)")

try:
    with open(audit_path) as fh:
        doc = yaml.safe_load(fh)
except Exception as e:
    tooling("audit did not parse: %s: %s" % (audit_path, e))

if not isinstance(doc, dict):
    tooling("audit is not a mapping: %s" % audit_path)

# T-3168: the audit's SECTION SCOPE is load-bearing and was missing here.
# `fw audit` runs per-section, and the pre-push hook runs `--sections structure`
# only. Measured 2026-09-26: the structure section emits 8 warnings; a FULL run
# emitted 39 before timing out at 560s. So a ledger reporting "8 total — 5
# acknowledged" over a structure-only file reads as "the audit is nearly clean"
# while ~4/5 of the warning surface was never in the file. That is the exact
# T-2680 scope-misread this checker was built to end, and the first version of
# this script had it. Scope is now reported on every path, and an audit whose
# sections are partial or unstated is caveated rather than silently trusted.
sections_raw = doc.get("sections")
sections = str(sections_raw).strip() if sections_raw is not None else ""
KNOWN_FULL = {"all", "*", "full"}
if not sections:
    scope_state = "unknown"
elif sections.lower() in KNOWN_FULL:
    scope_state = "full"
elif "," in sections or " " in sections:
    # several named sections — still not provably everything, so treat as partial
    scope_state = "partial"
else:
    scope_state = "partial"
partial_scope = scope_state != "full"

findings = doc.get("findings")
if not isinstance(findings, list) or not findings:
    # Zero parsed findings is a parse failure, not a clean bill. An audit always
    # emits PASS lines, so an empty list means the shape changed under us
    # (T-2747's zero-tools lesson).
    tooling("audit carries no parseable findings — shape may have changed: %s" % audit_path)

warns, fails = [], []
for f in findings:
    if not isinstance(f, dict):
        continue
    lvl = str(f.get("level", "")).upper()
    txt = str(f.get("check", "")).strip()
    if not txt:
        continue
    if lvl == "WARN":
        warns.append({"check": txt, "mitigation": str(f.get("mitigation", "") or "")})
    elif lvl == "FAIL":
        fails.append({"check": txt})

# Ledger. ABSENT ledger acknowledges nothing rather than excusing everything —
# the same choice T-3145's notexec allowlist makes, pinned by fixture there.
entries = []
ledger_exists = os.path.isfile(ledger_path)
if ledger_exists:
    try:
        with open(ledger_path) as fh:
            for lineno, raw in enumerate(fh, 1):
                line = raw.rstrip("\n")
                if not line.strip() or line.lstrip().startswith("#"):
                    continue
                if "#" not in line:
                    tooling("ledger line %d has no '# reason' — an acknowledgement "
                            "without a cited reason is not an acknowledgement: %s"
                            % (lineno, line.strip()))
                pat, reason = line.split("#", 1)
                pat, reason = pat.strip(), reason.strip()
                if not pat:
                    tooling("ledger line %d has an empty pattern" % lineno)
                if not reason:
                    tooling("ledger line %d has an empty reason" % lineno)
                entries.append({"pattern": pat, "reason": reason, "line": lineno})
    except SystemExit:
        raise
    except Exception as e:
        tooling("ledger unreadable: %s: %s" % (ledger_path, e))

acknowledged, unexamined = [], []
matched_patterns = set()
for w in warns:
    hit = next((e for e in entries if e["pattern"] in w["check"]), None)
    if hit:
        matched_patterns.add(hit["pattern"])
        acknowledged.append({"check": w["check"], "reason": hit["reason"], "pattern": hit["pattern"]})
    else:
        unexamined.append(w)

stale = [e for e in entries if e["pattern"] not in matched_patterns]

fires = bool(unexamined) or bool(fails) or (strict and bool(stale))
rc = 1 if fires else 0

SCOPE = ("scope: classifies WARN findings in one audit file as acknowledged-with-a-reason or "
         "unexamined. It does NOT verify that a cited reason is TRUE, does not re-run the audit, "
         "and cannot acknowledge a FAIL. A green means every warning has been looked at and "
         "explained - never that the project has no warnings.")

if as_json:
    print(json.dumps({
        "ok": not fires,
        "audit": audit_path,
        "ledger": ledger_path,
        "ledger_exists": ledger_exists,
        "warn_total": len(warns),
        "acknowledged_count": len(acknowledged),
        "unexamined_count": len(unexamined),
        "fail_count": len(fails),
        "stale_entry_count": len(stale),
        "acknowledged": acknowledged,
        "unexamined": unexamined,
        "fails": fails,
        "stale_entries": stale,
        "strict": strict,
        "audit_sections": sections or None,
        "scope_state": scope_state,
        "partial_scope": partial_scope,
        "scope": SCOPE,
    }, indent=2))
    sys.exit(rc)

if quiet and not fires:
    sys.exit(0)

print("audit: %s" % audit_path)
if scope_state == "full":
    print("audit sections: %s (full run)" % sections)
elif scope_state == "partial":
    print("audit sections: %s  <-- PARTIAL RUN" % sections)
else:
    print("audit sections: (not stated in the file)  <-- SCOPE UNKNOWN")
print("ledger: %s%s" % (ledger_path, "" if ledger_exists else "  (ABSENT — acknowledges nothing)"))
print("warnings: %d total — %d acknowledged, %d unexamined"
      % (len(warns), len(acknowledged), len(unexamined)))

if fails:
    print("\nFAIL (never acknowledgeable — the arc's bar is zero failures):")
    for f in fails:
        print("  FAIL  %s" % f["check"])

if unexamined:
    print("\nUNEXAMINED — no ledger entry, nobody has explained these:")
    for w in unexamined:
        print("  WARN  %s" % w["check"])
        if w["mitigation"]:
            print("        mitigation: %s" % w["mitigation"][:160])

if acknowledged:
    print("\nacknowledged (counted and reported, non-firing):")
    for a in acknowledged:
        print("  ack   %s" % a["check"][:120])
        print("        reason: %s" % a["reason"])

if stale:
    label = "STALE LEDGER ENTRIES (firing: --strict)" if strict else "stale ledger entries (non-firing)"
    print("\n%s — pattern matches no current warning, so it is either fixed or the pattern drifted:" % label)
    for e in stale:
        print("  stale line %d: %s" % (e["line"], e["pattern"]))
        print("        reason on file: %s" % e["reason"])

if partial_scope:
    print("\nSCOPE CAVEAT — the counts above cover ONLY the section(s) this audit file "
          "recorded%s. `fw audit` runs per-section and the pre-push hook runs the "
          "structure section alone; a full run emits several times as many warnings "
          "(measured 2026-09-26: 8 for structure, 39+ for a full run). Do NOT read a "
          "low unexamined count here as \"the audit is nearly clean\" — re-run "
          "`fw audit` without a section filter to see the whole surface." %
          ("" if not sections else " (%s)" % sections))
print("\n%s" % SCOPE)
if not fires:
    print("\nverdict: every warning acknowledged with a cited reason.")
else:
    bits = []
    if fails: bits.append("%d FAIL" % len(fails))
    if unexamined: bits.append("%d unexamined warning(s)" % len(unexamined))
    if strict and stale: bits.append("%d stale ledger entry(ies)" % len(stale))
    print("\nverdict: FIRING — %s" % ", ".join(bits))
sys.exit(rc)
PYEOF
