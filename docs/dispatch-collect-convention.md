# Dispatch-Collect Convention

How to spawn multiple worker agents via TermLink and collect their results without polling.

## Overview

```
Orchestrator (Claude Code session)
  │
  ├── termlink spawn worker-1 -- claude -p "do X"
  ├── termlink spawn worker-2 -- claude -p "do Y"
  └── termlink spawn worker-3 -- claude -p "do Z"
       │
       │  Workers emit events to their own bus as they work:
       │    task.progress  → percent, message
       │    task.completed → summary, status, blob_path
       │
  └── termlink event collect --topic task.completed --count 3 --timeout 300
       │
       └── Returns all 3 results in a single response (no polling)
```

## Worker Convention

### Environment

Workers receive these environment variables from the dispatch script:

| Variable | Purpose |
|----------|---------|
| `TERMLINK_PARENT_SESSION` | Orchestrator's session ID — for direct push-back |
| `TERMLINK_TASK_ID` | Task ID this work belongs to |
| `TERMLINK_WORKER_NAME` | This worker's display name |

### Events Workers MUST Emit

**On completion** — emit `task.completed` to self:

```bash
termlink event emit self task.completed \
  -p '{"task_id":"T-257","summary":"Found 3 issues","status":"ok","blob_path":"/tmp/results.md"}'
```

**On failure** — emit `task.failed` to self:

```bash
termlink event emit self task.failed \
  -p '{"task_id":"T-257","error_code":"timeout","message":"API unreachable","retryable":true}'
```

### Events Workers MAY Emit

**Progress updates** (optional, recommended for long-running work):

```bash
termlink event emit self task.progress \
  -p '{"task_id":"T-257","percent":50,"message":"Scanning tests..."}'
```

**Direct push-back** (for urgent results — bypasses collect, pushes to orchestrator's bus):

```bash
termlink event emit "$TERMLINK_PARENT_SESSION" worker.report \
  -p '{"from":"worker-1","message":"Critical finding: ..."}'
```

### Payload Schemas

These match the `task.*` topic definitions in `crates/termlink-protocol/src/events.rs`.

**task.completed:**
```json
{
  "task_id": "T-257",
  "summary": "One-line description of result",
  "status": "ok",
  "blob_path": "/path/to/detailed/output.md"
}
```

**task.progress:**
```json
{
  "task_id": "T-257",
  "percent": 75,
  "message": "Human-readable status"
}
```

**task.failed:**
```json
{
  "task_id": "T-257",
  "error_code": "timeout",
  "message": "What went wrong",
  "retryable": true
}
```

## Orchestrator Convention

### Spawning Workers

```bash
# Spawn with TermLink session registration
termlink spawn \
  --name "worker-q1" \
  --roles worker \
  --tags "task:T-257,q1" \
  --backend tmux \
  --wait \
  -- claude -p --permission-mode bypassPermissions \
    "TERMLINK_PARENT_SESSION=$MY_SESSION_ID do X, then emit task.completed"
```

Key flags:
- `--backend tmux` — provides real terminal for Claude instances
- `--wait` — blocks until session registers (prevents race conditions)
- `--permission-mode bypassPermissions` — required for non-interactive Claude

### Collecting Results

**Preferred — single blocking collect:**

```bash
# Collect all N results (blocks until N events or timeout)
termlink event collect \
  --topic task.completed \
  --count 3 \
  --interval 250 \
  --timeout 300
```

In Claude Code, run as a background Bash task:

```
Bash(run_in_background=true, timeout=300000):
  termlink event collect --topic task.completed --count 3 --interval 250
```

This consumes ~800 tokens total (1 launch + 1 notify) vs ~10K-18K for polling.

**Alternative — per-worker wait:**

```bash
# Wait for a specific worker's event
termlink event wait --topic task.completed --timeout 300 worker-1
```

Use when you need per-worker ordering or partial result processing.

### Handling Partial Failure

When some workers die before completing:

1. `collect --count N` will time out after `--timeout` seconds
2. It returns whatever events it collected before timeout
3. Parse the output to determine which workers responded
4. Use `termlink discover --tag "task:T-257"` to see which workers are still alive
5. Dead workers can be identified by cross-referencing alive sessions with expected workers

### Monitoring Progress

```bash
# Watch progress events from all workers (continuous)
termlink event collect --topic task.progress --interval 250

# Or watch a specific worker
termlink event watch --topic task.progress worker-1
```

## Token Cost Comparison

| Pattern | Tool calls | Token overhead | Latency |
|---------|-----------|---------------|---------|
| Background `collect --count N` | 2 (launch + notify) | ~800 | 250-500ms |
| N x background `wait` | 2N | ~N*400 | 250ms |
| Poll loop (5s interval, 60s avg) | ~36 | ~10K-18K | 5000ms |
| File watch loop | ~24 | ~8K-12K | 10000ms |

## Full Example: 3-Worker Research Dispatch

```bash
# 1. Record orchestrator session ID
MY_SESSION=$(/path/to/termlink list | grep "my-session" | awk '{print $1}')

# 2. Spawn workers
for q in q1 q2 q3; do
  termlink spawn \
    --name "research-$q" \
    --tags "task:T-256,$q" \
    --backend tmux \
    --wait \
    -- claude -p --permission-mode bypassPermissions \
      "Research question $q. Write findings to /tmp/$q.md. Then run: termlink event emit self task.completed -p '{\"task_id\":\"T-256\",\"summary\":\"$q done\",\"status\":\"ok\",\"blob_path\":\"/tmp/$q.md\"}'"
done

# 3. Collect results (single blocking call)
termlink event collect --topic task.completed --count 3 --interval 250

# 4. Read detailed findings from blob paths in the payloads
```
