# Test-artifact secret scan (arc-011 PH-0 6.2.1.a (6), SI-25, FC-17)

SI-25: no secret or key material in the logs, status output, frames or records a test run
produces. `scripts/check-test-artifact-secrets.sh` enforces it after a run.

```bash
bash scripts/check-test-artifact-secrets.sh <artifact-dir> \
     [--canary VALUE]... [--secret-file FILE]... [--json] [--quiet]
```

Exit 0 clean, 1 secret material found, 2 tooling (missing/empty dir, no catalogue, canary
under 8 chars, secret file under 16 chars). Findings print `file:line  rule=...`, never the value.

## What it looks for
1. The vendored catalogue `.agentic-framework/.secret-scan-patterns` (the pre-commit hook's file; reused, not copied).
2. A hub-secret shape: a `secret`/`token` key followed by 64 hex characters.
3. Canary literals: `--canary V` or `$TERMLINK_TEST_SECRET_CANARY` (FC-17).
4. The content of `--secret-file` files, e.g. the test hub's `hub.secret`.
Binary files (captured frames) are scanned.

## How a test run feeds it
1. The harness collects all logs, status dumps, captured frames and records into ONE directory (`$ARTIFACT_DIR`).
2. FC-17: before the run, export a random canary (`export TERMLINK_TEST_SECRET_CANARY=FC17-$(openssl rand -hex 8)`) into the
   environment under test. A correct system never writes it out.
3. After the run: `scripts/check-test-artifact-secrets.sh "$ARTIFACT_DIR" --secret-file "$TEST_RUNTIME_DIR/hub.secret"`.
   Any exit other than 0 fails the run. Exit 2 is a failure too: an unscanned run is not a clean one.
4. Negative control (FC-17): log the environment on purpose in a scratch run; the scan must exit 1.

Scope: a clean result covers the shapes and values above in that directory only.
Fixtures: `bash tests/test-artifact-secrets-fixtures.sh` (includes a mutant that disables literal matching).
