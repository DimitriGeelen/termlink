---
id: T-139
name: "query.output --strip-ansi option"
description: >
  Add optional ANSI escape sequence stripping to the query.output RPC handler
  and CLI output command. Makes scrollback output clean text for automated parsing.

status: captured
workflow_type: build
owner: agent
horizon: next
tags: [protocol, scrollback, ansi]
components: []
related_tasks: [T-136, T-137]
created: 2026-03-14T17:07:00Z
last_update: 2026-03-14T17:07:00Z
date_finished: null
---

# T-139: query.output --strip-ansi option

## Context

T-136 spike found that scrollback contains raw ANSI escape sequences which
break JSON parsers and make pattern matching harder. Add a `strip_ansi` param
to the `query.output` RPC handler that strips escape codes server-side.

## Acceptance Criteria

### Agent
- [ ] `query.output` RPC accepts optional `strip_ansi: true` param
- [ ] When set, output has ANSI escape sequences removed before returning
- [ ] `termlink output <session> --strip-ansi` CLI flag
- [ ] Backward compatible — default behavior unchanged
- [ ] Tests for ANSI stripping (at least 2)
- [ ] All existing tests pass

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace 2>&1 | grep -q "test result: ok"

## Updates

### 2026-03-14T17:07:00Z — task-created
- Enhancement from T-136 spike findings (ANSI in scrollback breaks json.load)
