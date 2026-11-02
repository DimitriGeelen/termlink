# T-3658 — /triage compact layout: design (via `/design`)

Session: `/design start "compact, navigable /triage"`. This was a non-interactive worker, so the operator's
request (T-3658 Context and ACs) stood in for the dialogue. EXPLORE → DECIDE → PLAN are below; VALIDATE is
at the end and was filled in after the screenshots.

## Problem (measured)

1. One card per task: title, summary, evidence, details, proposal and six full-size buttons. That is about
   220 px of height. At 1920x1080 roughly 3 cards fit on screen (`.context/visual/T-3649/cards-top@1920.png`).
2. The batch has 287 rows in six groups (C 122, D2 49, B 47, A3 30, A2 23, D1 16). The group comes from the
   `group X` token in `details`, because no row sets `group`.
3. There is no keyboard path. Every ruling is a mouse trip, and after a click you have to scroll to the next card.
4. Progress is a single number at the top that scrolls away.

Constraints that do not change: `batch.json` in, `answers.jsonl` out, the page never writes a task file, the proposal
is highlighted and never pre-selected, and nothing is added outside the three T-3649 files (the vendor-divergence
registration stays "new files only"). The page loads through `hx-boost`, so any document-level listener must be
idempotent across `#content` swaps.

## EXPLORE — options

### Option A: Dense rows, details inline on demand (one grid row per task)
1. **Approach:** a CSS-grid row `[id] [title · one-line summary] [proposal chip] [6 small buttons]`. Evidence and
   details sit in a hidden second line that opens with `o`/Enter or by clicking the title.
2. **Pros:** about 40 px per row, so more than 15 tasks per 1080p screen. There is one place to look and the server
   rendering stays the same.
3. **Cons:** the summary is truncated, so the operator has to expand a row to read it in full.
4. **Effort:** low. It needs a template and CSS change and about 80 lines of JS.

### Option B: Master/detail split (list left, full card right)
1. **Pros:** you always see the full context for the selected task.
2. **Cons:** at 1024 px the split is cramped and needs a responsive fallback. It is also a second layout to maintain.
   And it hides the buttons from the list, so each ruling costs two clicks with the mouse.
3. **Effort:** medium.

### Option C: One-at-a-time "flashcard" mode
1. **Pros:** fastest when you use only the keyboard.
2. **Cons:** you cannot scan, and 287 rows become 287 screens. It also gives the worst overview, which is the
   opposite of the request ("easier to navigate").
3. **Effort:** medium.

### Recommendation
Option A, with the keyboard model from C. The rows stay scannable and the keys give C's speed.

## DECIDE — D-UI-01: Compact grid rows + keyboard cursor + sticky progress bar

| Field | Value |
|-------|-------|
| **Status** | APPROVED (operator brief T-3658) |
| **Decided** | 2026-11-02 |
| **Choice** | Option A + keyboard cursor |
| **Rationale** | It shows the most tasks per screen and keeps the same data path. It also works at 1024 px, because the summary column takes up the slack. |
| **Alternatives** | B was rejected for cramped 1024 px and two-click rulings. C was rejected because it loses the overview. |

### Details of the decision
1. **Row:** `article.triage-card#card-T-N` is kept for the tests and for deep links. The grid columns are
   id (6rem), text (1fr: bold title, then the summary in muted text on the same line with ellipsis), and
   buttons (auto).
2. **Buttons:** six compact buttons labelled `1 Prioritize` … `6 More`. The digit is a `<kbd>` hint. The proposal
   gets the existing dashed outline plus a small "proposed" dot. Nothing gets `triage-chosen` until the operator clicks.
3. **Expand:** a `.triage-more` line holds the evidence, the details, the full summary and the task link. It is hidden
   by default and toggled by `o`, Enter, or a click on the title text. It has a `aria-expanded` state.
4. **Sticky bar** (`position: sticky; top: 0`): the batch id, an overall progress bar `<progress>` with the text
   `answered/in-scope`, group pills each showing `name a/t` with a thin per-group bar, the show toggle, and a
   one-line key legend. The links stay server-side query links, so filtering by group still works without JS.
5. **Keys** (ignored while typing in an input or while the palette/overlay is open, and never with Ctrl or Meta):
   1. `j` / `↓` next row, `k` / `↑` previous row
   2. `1`–`6` choose (button order)
   3. `o` / Enter expand or collapse
   4. `n` next unanswered
   5. `[` / `]` previous or next group
   The letter scheme p/k/a/d/c/m was rejected because its `k` collides with j/k navigation, so digits are the only
   choice keys. The legend says so.
6. **After an answer:** the row is marked answered (dimmed, chosen button filled) and stays in place, so a
   mis-click can be corrected. The cursor moves to the next unanswered row below and scrolls it into view
   (`block: nearest`). The progress text, the bar and the current group's pill counts update in place.
7. **hx-boost safety:** one document-level listener, installed once (`window.__triageKeysInstalled`). It looks up
   `#triage-list` at keypress time and does nothing on other pages.

## PLAN

| # | Task | Depends | Complexity |
|---|------|---------|------------|
| 1 | Template: sticky bar, grid rows, expand line, legend | – | Low |
| 2 | JS: cursor, keys, answer POST (same endpoint and CSRF header), progress update, focus next unanswered | 1 | Medium |
| 3 | Blueprint: give each group pill the current group's counts (already computed) and nothing else | – | Low |
| 4 | Tests: legend and keys documented, sticky bar and progress present, details hidden by default, still nothing pre-selected, still 6 buttons per row | 1 | Low |
| 5 | Visual: a throwaway Watchtower on :5099 with the real batch (read-only) and a temp answers file; screenshots at 1024/1440/1920 | 1–2 | Low |

### Risks
1. **Risk:** a global `j`/`k` steals keys from other pages after an hx-boost navigation. **Mitigation:** the handler
   returns early when `#triage-list` is missing.
2. **Risk:** six buttons do not fit at 1024 px. **Mitigation:** the text column is `minmax(0,1fr)` and the button
   labels shorten under 1200 px (the digit plus a short word).

## VALIDATE

**Status: VALIDATED** (2026-11-02). This was checked on a throwaway Watchtower on :5099 against the real batch (287 rows, read-only copy) with a
temp answers file. The real `answers.jsonl` stayed at 12 lines.

| Assumption | Expected | Actual | Match |
|---|---|---|---|
| Rows per screen at 1920x1080 | 6 or more | about 15 (`rows@1920.png`) | yes |
| 1024 px fits six buttons | no wrap | short labels, one line (`rows@1024.png`) | yes |
| Keys | j moves the cursor, o expands, 2 answers | j and o shown in screenshots; `2` recorded `keep` for T-1415, the cursor moved to the next unanswered row, and the count and D1 pill went from 0 to 1 | yes |
| Sticky bar | stays at top | bar top = 0 after a 3000 px scroll (`scrolled-sticky@1024.png`) | yes |
| Nothing pre-selected | no `aria-pressed=true` before a click | pinned by tests | yes |

Found and fixed during VALIDATE: Pico styles `nav` as space-between, so the group pills spread over the full width.
The pill row is now a `div role=navigation`.
