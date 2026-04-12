# Pickup: Approvals page missing recommendation + argumentation display

**Source:** termlink (T-939)
**Type:** bug-report
**Severity:** medium
**Affects:** web/blueprints/approvals.py, web/templates/_approvals_content.html

## Problem

The `/approvals` page shows inception tasks pending GO/NO-GO decisions but **never displays the agent's recommendation or argumentation**. The `rationale_hint` variable is extracted from task files but only pre-filled into the rationale textarea — the reviewer sees task name + problem excerpt + radio buttons with no visible recommendation before deciding.

## Root Cause

Two independent issues:

### 1. UI gap (approvals template)

`_approvals_content.html` Section B (GO/NO-GO Decisions) renders:
- Task ID + name
- Problem excerpt
- Research artifact links
- Assumption counts
- Decision form (GO/NO-GO/DEFER radio + textarea)

Missing: **No visible display of the agent's recommendation or rationale.** The `rationale_hint` is only stuffed into `<textarea>` as pre-fill text.

### 2. Data gap (task template)

Many inception tasks have empty `## Recommendation` sections (HTML comment placeholders only). These predate T-974 enforcing recommendation writing. Even with a perfect UI, these show nothing. This is a process gap — the inception completion gate should block `work-completed` if `## Recommendation` is empty.

## Fix Applied Locally (termlink vendored copy)

### approvals.py changes

Added `recommendation_label` and `recommendation_text` to the `_load_pending_go_decisions()` return dict:

```python
# Parse recommendation label (GO/NO-GO/DEFER)
label_match = re.search(
    r'\*{0,2}Recommendation:?\*{0,2}\s*(GO|NO-GO|DEFER)',
    rec, re.IGNORECASE
)
recommendation_label = label_match.group(1).upper() if label_match else ""

# Extract rationale text
rationale_match = re.search(
    r'\*{0,2}Rationale:?\*{0,2}\s*(.*?)(?:\n\n|\n\*{0,2}Evidence|\Z)',
    rec, re.DOTALL | re.IGNORECASE
)
recommendation_text = rationale_match.group(1).strip() if rationale_match else ""
```

### _approvals_content.html changes

Added a `recommendation-display` div between artifact links and the decision form:

```html
<div class="recommendation-display" style="...border-left:4px solid {color}...">
    {% if t.recommendation_label %}
    <span>Agent recommends: {{ t.recommendation_label }}</span>
    <p>{{ t.recommendation_text }}</p>
    {% else %}
    <p>No recommendation yet — <a href="/inception/{{ t.task_id }}">review task file</a></p>
    {% endif %}
</div>
```

Color-coded: green for GO, red for NO-GO, amber for DEFER, grey for missing.

## Suggested Framework Actions

1. **Apply the UI fix** to `web/blueprints/approvals.py` and `web/templates/_approvals_content.html`
2. **Consider adding inception completion gate**: block `work-completed` on inception tasks if `## Recommendation` section is empty (only HTML comments)
3. **Consider adding audit check**: warn on inception tasks with `started-work` status but empty recommendation after N days
