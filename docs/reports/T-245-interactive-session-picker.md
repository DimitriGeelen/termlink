# T-245: Interactive Session Picker for attach/mirror/stream

**Decision:** GO (2026-03-23)
**Rationale:** Straightforward UX improvement with shared utility function.

## Problem

~15 interactive commands (attach, mirror, stream, ping, status, watch, topics, output, interact, inject, kv, events, wait, remote ping, remote status) require a target session argument. When no target is given and stdin is a TTY, the command fails with a usage error instead of helping the user pick a session.

## Design

When target-requiring commands run without a target and stdin is TTY:
1. List sessions (numbered)
2. Auto-select if only 1 session exists
3. Prompt user to choose if 2+ sessions exist

Shared utility function in a common module, works for both local and remote (`--hub`) sessions. Applies to all commands that require a session target.
