---
id: T-1206
name: "T-1103 upstream mirror — apply Watchtower /fleet page in framework repo"
description: >
  T-1103 built the Watchtower /fleet page in the termlink vendored copy only — never mirrored to upstream. The 2026-04-23 vendor refresh (T-915) wiped fleet.py + fleet.html because they don't exist upstream. Files restored from git and registered in .local-patches, but the durable fix is to land them upstream so the next vendor refresh preserves them automatically.
status: started-work
workflow_type: build
owner: human
horizon: now
tags: [framework, upstream-mirror, vendor-loss-recovery]
components: []
related_tasks: [T-1103, T-1114, T-1115, T-1116, T-1184, T-915]
created: 2026-04-23T18:55:00Z
last_update: 2026-04-23T18:55:00Z
date_finished: null
---

# T-1206: T-1103 upstream mirror — apply Watchtower /fleet page in framework repo

## Context

Discovered during T-915 vendor refresh on 2026-04-23: `fw vendor --source /opt/999-Agentic-Engineering-Framework` deleted `web/blueprints/fleet.py` (488 lines) and `web/templates/fleet.html` (107 lines) from the vendored copy because they only exist in termlink's local copy — never mirrored upstream.

**Symptom:** `/fleet` route returned HTTP 404 immediately after vendor; restored from `git checkout HEAD~1` and re-registered fleet_bp in `web/blueprints/__init__.py`. Restoration committed in `aa1bc066`. Files registered in `.agentic-framework/.local-patches` so future vendor refreshes skip them.

**Why this is recurring class:** Same pattern as T-1175 (Rust detector) → mirrored as T-1176, T-1187 (pl007-scanner) → mirrored as T-1188, T-1189 (hook-enable) → mirrored as T-1190. Local vendored fixes need an upstream mirror task or they decay on the next vendor sync.

**T-559 boundary:** Agent sessions rooted in `/opt/termlink` cannot edit `/opt/999-AEF` directly. Use TermLink dispatch with `--workdir` (Channel 1 pattern from T-1192) or human-direct work.

## Acceptance Criteria

### Agent
- [x] T-1103 termlink-side closure verified (fleet.py + fleet.html in `.agentic-framework/web/`)
- [x] Files registered in `.local-patches` to prevent future vendor wipe
- [x] This pickup task records the artifacts + apply plan self-contained

### Human
- [ ] [RUBBER-STAMP] Mirror fleet.py + fleet.html to framework repo
  **Steps:**
  1. From a session rooted at `/opt/999-Agentic-Engineering-Framework` (NOT /opt/termlink — T-559 boundary):
     ```
     cp /opt/termlink/.agentic-framework/web/blueprints/fleet.py /opt/999-Agentic-Engineering-Framework/web/blueprints/fleet.py
     cp /opt/termlink/.agentic-framework/web/templates/fleet.html /opt/999-Agentic-Engineering-Framework/web/templates/fleet.html
     ```
  2. Apply the 2-line patch to upstream `web/blueprints/__init__.py`:
     - Add `from web.blueprints.fleet import bp as fleet_bp` near the other imports
     - Add `fleet_bp` to the registration tuple
  3. Verify: `python3 -c "import ast; ast.parse(open('web/blueprints/fleet.py').read())"` → clean
  4. Smoke-test in a consumer project that has a fleet config: open `/fleet` page, expect HTTP 200
  5. Commit with message referencing T-1103 + T-1206 + this vendor incident
  6. Push per framework's own push policy
  **Expected:** Upstream framework contains fleet.py + fleet.html + __init__.py registration. Next consumer vendor refresh preserves them. Eligible to remove the 3 fleet entries from termlink's `.local-patches` after that.
  **If not:** File divergence as new task in framework repo.

## Verification

test -f /opt/termlink/.agentic-framework/web/blueprints/fleet.py
test -f /opt/termlink/.agentic-framework/web/templates/fleet.html
grep -q "fleet_bp" /opt/termlink/.agentic-framework/web/blueprints/__init__.py
grep -q "fleet.py" /opt/termlink/.agentic-framework/.local-patches
curl -sf http://localhost:3100/fleet -o /dev/null

## Decisions

### 2026-04-23 — restore approach

- **Chose:** Restore from `git checkout HEAD~1` rather than re-vendor with `--exclude` flags
- **Why:** vendor doesn't have a per-file exclude option exposed in CLI; restore is one-shot and reversible
- **Rejected:** Patching upstream framework first then re-vendoring — would have left /fleet broken for the duration

## Updates

### 2026-04-23T18:55:00Z — task-created [agent-direct]
- **Action:** Created task to track upstream mirror of T-1103 fleet page
- **Trigger:** T-915 vendor refresh wiped fleet.py + fleet.html
- **Recovery:** Files restored in commit aa1bc066; registered in .local-patches in subsequent commit
