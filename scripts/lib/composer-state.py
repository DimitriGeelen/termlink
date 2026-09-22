#!/usr/bin/env python3
"""T-3079 — is the Claude Code composer EMPTY? Decided from a RAW PTY tail.

WHY RAW BYTES AND NOT --strip-ansi
----------------------------------
Claude Code renders a *suggested follow-up* inside the composer as DIM text
(SGR 2 ... SGR 22). Once ANSI is stripped, that suggestion is byte-identical to a
line a human typed, and the two demand opposite answers:

  * a suggestion is NOT input. Typing replaces it, so injecting is safe.
  * real pending text means an injected line is APPENDED to it, so the composer
    holds two concatenated messages and submits neither cleanly — the T-2396 loss
    this rail exists to prevent.

WHY ROW TRACKING AND NOT "EVERYTHING AFTER THE PROMPT GLYPH"
-----------------------------------------------------------
The PTY is a stream of cursor-addressed redraws, so byte order is NOT screen
order. A first version of this file judged "everything after the last ❯", and on
a live session that swallowed the version/update status line:

    ESC[23C ESC[19B ESC[38;5;246m current: 2.1.267 · stable: 2.1.267 … Checking for update

That text is grey but NOT dim, and it is painted at row 19 while the composer
sits at row 21 — so it counted as pending input and the classifier deferred
FOREVER on an idle REPL. Safe, but useless: it is the same "never READY" bug
T-3079 exists to fix, reintroduced one layer down.

So we track the cursor row well enough to ask the only question that matters:
what was painted LAST on the composer's own row?

CONTRACT
--------
stdin : raw PTY tail (NOT stripped)
exit 0: a composer prompt is present and is effectively EMPTY  -> safe to inject
exit 1: no composer prompt, or it carries real pending text     -> defer
exit 2: usage error

FAIL-SAFE BY CONSTRUCTION: anything left on the composer row that is neither
whitespace nor dim counts as pending, so an unrecognised render defers rather
than authorising an inject. A missed wake costs one cron cycle; a blind inject
costs the message.
"""
import re
import sys

PROMPT = "❯"
NBSP = " "
ELLIPSIS = "…"

# CSI sequences. The private-parameter prefix ([?>=<]) matters: an earlier
# version matched only [0-9;] and therefore did NOT consume things like
# ESC[?25h, so those bytes fell through as composer TEXT and every state read as
# "not empty". Only sequences with NO prefix are interpreted for cursor motion.
CSI = re.compile(r"\x1b\[([?>=<]?)([0-9;]*)([@-~])")
# OSC (window title etc.) — dropped whole; it carries text that is not on screen.
OSC = re.compile(r"\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)")
# Two-character escapes (charset selection, keypad mode, ...).
ESC2 = re.compile(r"\x1b[()#][A-Za-z0-9]|\x1b[=>ONM78]")
CTRL = re.compile(r"[\x00-\x08\x0b\x0c\x0e-\x1f\x7f]")


def _int(s, default=1):
    try:
        return int(s)
    except (TypeError, ValueError):
        return default


def composer_is_empty(raw):
    """True iff a composer prompt is present and its ROW holds nothing but
    whitespace and/or dim (suggested) text after the last repaint."""
    raw = OSC.sub("", raw)
    raw = ESC2.sub("", raw)

    row = 1
    dim = False
    # segments: (row, dim, text) in stream order
    segments = []
    pos = 0
    for m in CSI.finditer(raw):
        text = raw[pos:m.start()]
        if text:
            # \r returns to column 0 but does not change the row; \n advances.
            for piece in text.split("\n"):
                if piece:
                    segments.append((row, dim, piece))
                row += 1
            row -= 1
        prefix, params, final = m.group(1), m.group(2), m.group(3)
        first = params.split(";")[0] if params else ""
        if prefix:
            pos = m.end()          # private-mode sequence: consumed, not interpreted
            continue
        if final == "m":
            for p in (params.split(";") if params else ["0"]):
                p = p or "0"
                if p == "2":
                    dim = True
                elif p in ("0", "22"):
                    dim = False
        elif final == "H" or final == "f":
            parts = params.split(";") if params else []
            row = _int(parts[0], 1) if parts and parts[0] else 1
        elif final == "A":
            row -= _int(first)
        elif final == "B":
            row += _int(first)
        elif final == "d":
            row = _int(first)
        pos = m.end()
    tail = raw[pos:]
    if tail:
        for piece in tail.split("\n"):
            if piece:
                segments.append((row, dim, piece))
            row += 1

    # Locate the LAST segment that paints the prompt glyph; its row is the
    # composer row, and only repaints of that row afterwards are composer input.
    last_prompt = None
    for idx, (r, _d, text) in enumerate(segments):
        if PROMPT in text:
            last_prompt = (idx, r, text)
    if last_prompt is None:
        return False
    idx, comp_row, text = last_prompt

    # Anything on the prompt's own segment after the glyph counts too.
    residue = [text.split(PROMPT)[-1]]
    for r, d, t in segments[idx + 1:]:
        if r == comp_row and not d:
            residue.append(t)

    joined = CTRL.sub("", "".join(residue))
    joined = joined.replace(NBSP, "").replace(ELLIPSIS, "")
    return joined.strip() == ""


def main():
    if len(sys.argv) > 1:
        sys.stderr.write("usage: composer-state.py < raw-pty-tail\n")
        return 2
    data = sys.stdin.buffer.read().decode("utf-8", "replace")
    return 0 if composer_is_empty(data) else 1


if __name__ == "__main__":
    sys.exit(main())
