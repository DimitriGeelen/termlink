---
id: T-1150
name: "Fix watchtower episodic YAML parser — tolerate frontmatter + multi-doc + markdown body formats"
description: >
  Fix watchtower episodic YAML parser — tolerate frontmatter + multi-doc + markdown body formats

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-19T23:51:15Z
last_update: 2026-04-19T23:58:23Z
date_finished: 2026-04-19T23:58:23Z
---

# T-1150: Fix watchtower episodic YAML parser — tolerate frontmatter + multi-doc + markdown body formats

## Context

Watchtower's episodic-memory reader (`search_utils.py::aggregate_tags`, `shared.py::get_episodic_tags`) uses `yaml.safe_load` on every `.context/episodic/T-*.yaml`. 24 legacy episodic files fail to parse and spam `watchtower.log` with `Failed to parse episodic file ...` warnings — dropping their tags from the tag cloud and breaking episodic search for those tasks. Two root formats:
- Jekyll-style frontmatter (T-121/155/162/166/167/170/171/172): `---\n<yaml>\n---\n<more yaml>` — triggers ComposerError "expected a single document in the stream".
- Mixed YAML + markdown body (T-836/894-905/920/924-929/935): first lines are `key:` but body contains `*word*` or `"unterminated` that YAML mistakes for an alias/scalar — triggers ScannerError.

Fix: a tolerant parser helper that extracts only the leading valid YAML block and returns it as a dict.

## Acceptance Criteria

### Agent
- [x] Add `load_episodic_yaml(path)` helper that returns a dict for all 1170 episodic files (0 broken remaining)
- [x] Replace `yaml.safe_load(fh)` call sites in `search_utils.py::aggregate_tags` and `shared.py::get_episodic_tags` with the helper
- [x] After fix, Python-scan all episodics and confirm 0 parse failures
- [x] Watchtower process logs no new `Failed to parse episodic file` warnings after restart

## Verification

python3 -c "import os,pathlib,sys; os.environ['PROJECT_ROOT']=str(pathlib.Path('.').resolve()); sys.path.insert(0,'.agentic-framework'); from web.search_utils import load_episodic_yaml; bad=[f.name for f in pathlib.Path('.context/episodic').glob('T-*.yaml') if not isinstance(load_episodic_yaml(f), dict)]; print(f'broken: {len(bad)}'); sys.exit(1 if bad else 0)"

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-04-19T23:51:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1150-fix-watchtower-episodic-yaml-parser--tol.md
- **Context:** Initial task creation

### 2026-04-19T23:58:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
