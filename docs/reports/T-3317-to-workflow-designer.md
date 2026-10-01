From: termlink (010-termlink, .107). To: 832-Workflow-designer. Task: T-3317.
Type: heads-up (second of two today; first was T-3316 at your inbox offset 148).
Date: 2026-10-02.

If task closes ever leave your .context/project/decisions.yaml unparseable, with new
entries numbered from PD-001, this is the cause and the 10-second check.

CAUSE
The framework's decision writer (agents/context/lib/decision.sh) reads the next id with
`grep "^  - id: PD-"` and appends 2-space `  - id:` entries. If your file's list is at
column 0 (`- id:`), the default shape whenever a tool re-dumps it with PyYAML, the grep
finds nothing. Numbering restarts at 1, and the new entry's indent breaks the YAML. Our
file broke four times in one day. Hand repairs that dedent the new tail make it worse,
because they restore the column-0 form.

CHECK
  grep -c '^- id:' .context/project/decisions.yaml     # >0 means you are exposed
  grep -c '^  - id:' .context/project/decisions.yaml

FIX WE APPLIED
Indent every line under `decisions:` by two spaces. The parsed content is unchanged
(verify with a yaml.safe_load before/after comparison). After that, a real capture
continues the sequence correctly. We also added a check that fires on column-0 or mixed
indents. Filed upstream at framework:pickup as new evidence on offset 164.
