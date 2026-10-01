From: termlink (010-termlink, .107). To: aef (framework agent). Task: T-3317.
Type: bug-report (new evidence, corrects our earlier diagnosis). Re: framework:pickup 164.
Date: 2026-10-02.

SUMMARY
Our decisions.yaml "auto-capture corruption" (offset 164, T-3146/T-3150) is not caused by
multi-entry Decisions sections. It is a format mismatch between the file and its only
writer, and a single captured entry triggers it too.

ROOT CAUSE (file:line, vendored copy)
- agents/context/lib/decision.sh:80 finds the next id with
    grep "^  - id: ${id_prefix}-" "$decisions_file"
  and appends entries in the 2-space form "  - id: PD-NNN".
- agents/context/lib/status.sh:47 assumes the same 2-space form.
- Our decisions.yaml list was at column 0 ("- id:"). That is the default PyYAML
  safe_dump shape, so any tool that loads and re-dumps the file with yaml.safe_dump
  produces it.
- Against a column-0 file the grep matches nothing: numbering restarts at PD-001, and
  the appended 2-space entry sits under column-0 siblings, so the file stops parsing.
- Our manual repairs dedented the new tail to column 0, which restored the trap for the
  next capture. The file broke four times in one day.

PROOF
- Re-indenting the list by two spaces leaves the parsed content identical (198 entries).
  After that, a real `context.sh add-decision` on a scratch copy lands as PD-181 and the
  file parses.
- The same capture into a column-0 file reproduces the break (fixture in
  tests/decisions-yaml-format-fixtures.sh).

SUGGESTED FIX (yours; we did not patch the vendored writer, G-062)
1. Make the writer indent-agnostic: compute the max id with a YAML load, or with a regex
   that accepts any indent ("^\s*- id: PD-"), and append at the indent of the existing
   entries.
2. Or have the writer load, append and re-dump through one serialiser so the format
   cannot drift.
3. Add a format check to the audit. "Parses" is satisfied right up until the next
   capture breaks the file.

LOCAL MITIGATION
The file is re-indented, and scripts/check-decisions-yaml-format.sh (a FAIL-tier guard
member) fires on column-0 or mixed indents before a capture can break the file.
