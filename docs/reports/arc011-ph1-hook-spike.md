# arc-011 PH-1 B1 spike: PostToolUse hook output in the transcript

Task T-3401. Answers assumption **A-1** of `docs/design/arc-011-ph1-architecture.md` section 6.3.

## Result

**A-1 holds.** Claude Code 2.1.296 (model `claude-haiku-5-5`, `claude -p`, 2026-10-11) writes the `additionalContext` that a `PostToolUse` command hook prints into the session transcript as a harness-written record, immediately after the tool's result and before the model's next turn.

## Method

Disposable directory `/tmp/arc011-spike.XXXXXX` (project dir empty, hook script and settings file beside it), no project settings (`--setting-sources ""`), hook given by `--settings`. The hook reads stdin, prints
`{"hookSpecificOutput":{"hookEventName":"PostToolUse","additionalContext":"ARC011-HEADER nonce=SPIKE-NONCE-1\n<<<BOUNDARY-abc>>>\nuntrusted peer text\n<<<BOUNDARY-abc>>>"}}` and exits 0. Prompt: run `echo hi` with Bash, reply `done`. Transcript read from `~/.claude/projects/<encoded cwd>/<session>.jsonl`. Nothing under `~/.termlink`, the production hub or `/var/lib/termlink` was touched.

## Record shape found

Order in the transcript (fixture `tests/fixtures/transcript-posttooluse.jsonl`, filtered to the relevant record types):

1. `type: user` the prompt.
2. `type: assistant`, content `tool_use` (id `toolu_...`).
3. `type: user`, content `tool_result` for that `tool_use_id`.
4. `type: attachment`, `attachment.type: hook_success`: `hookName "PostToolUse:Bash"`, `toolUseID`, `hookEvent "PostToolUse"`, `stdout` (the raw hook JSON), `exitCode`, `command`, `durationMs`.
5. **`type: attachment`, `attachment.type: hook_additional_context`**: `content` is a **list of strings** (`["ARC011-HEADER nonce=...\n<<<BOUNDARY...>>>..."]`), plus `hookName "PostToolUse:Bash"`, `toolUseID`, `hookEvent "PostToolUse"`. The record also carries `rendered: [{"content": "<system-reminder>\nPostToolUse:Bash hook additional context: <content>\n</system-reminder>"}]`, `renderedRole: "system"`, `parentUuid` pointing at the `hook_success` record, plus the usual `uuid`, `timestamp`, `sessionId`, `cwd`, `version`.
6. `type: assistant` the next model turn (the context is delivered inside the running turn).

## Consequences for later tasks

- **B9 finalizer:** evidence is a record with `type == "attachment"` and `attachment.type == "hook_additional_context"` whose `content[0]` (a string, in a list) contains the header line with the nonce. `hook_success.stdout` also contains the nonce, but it is the hook's own echo, not delivery evidence; the finalizer must key on `hook_additional_context`. The `toolUseID` ties it to the tool call it followed.
- **FC-11:** in this fixture the nonce appears only in attachment records (`hook_success`, `hook_additional_context`), never in user, assistant or tool_result records; `tests/arc011-fixture-shape.py` asserts that.
- **Completeness point (CR-17):** the attachment is followed by assistant records and a trailing `last-prompt`/`cost-state` pair in the full transcript; B9 should define the completeness point from the real file, not from this reduced fixture.
- Not covered by this spike (left to B10): `Stop` and `SessionStart` hook records, the no-output (hook killed) case, interactive (non-`-p`) sessions, and subagent (`isSidechain`) turns.

## Fixture and test

- `tests/fixtures/transcript-posttooluse.jsonl`: 7 records, filtered from the real transcript (user, assistant and the two hook attachments only; environment, git status, skill listings and account fields removed; `requestId`, `advisorModel`, `wireToolInputs` dropped).
- `tests/arc011-fixture-shape.py`: hermetic shape test (exit 0 ok, 1 violated, 2 tooling; fixture overridable with `ARC011_FIXTURE`). A mutant that removes the `hook_additional_context` line makes it fail with exit 1.
- The architecture note names `cargo test -p termlink-sidecar fixture_shape`. The crate is created by B2 and did not exist when this spike ran, so the check is a standalone script; B9 should port the same assertions into the crate against this fixture.
