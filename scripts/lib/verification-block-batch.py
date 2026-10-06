#!/usr/bin/env python3
"""Batch form of the framework's `extract_verification_block` (T-3282, SQ-21 option 3).

WHY. `scripts/check-verification-heading-shadow.sh` called the framework's
`extract_verification_block` once per task file. That pipeline is
`sed | sed | tail | python3 lib/comment_strip.py | grep`, so a ~3000-file corpus
started Python ~3000 times. It was the guard layer's long pole: ~29% of its wall
time (T-3090), 226 s once and a 300 s timeout once on the contended .107 host.

WHAT IS AND IS NOT REIMPLEMENTED.
  * The COMMENT rule is NOT reimplemented: `strip_html_comment_lines` is imported
    from the framework's own `lib/comment_strip.py`, the single shared
    implementation T-2954 created so that no copy could drift.
  * The SELECTION around it (`sed -n '/^## Verification/,/^## /p' | sed '$d' |
    tail -n +2`) and the final `grep -vE '^\\s*$|^\\s*#|^\\s*```'` ARE
    re-expressed here, because they are shell, not an importable function. That
    is the one place drift could enter. It is guarded twice: by a full-corpus
    equivalence proof (`--prove`), and at run time by the check itself, which
    cross-checks every firing file plus a random sample against the REAL
    `extract_verification_block` and exits 2 on any mismatch.

Semantics reproduced exactly, quirks included:
  * GNU sed ranges REPEAT: after a range closes on a `^## ` line, a later
    `^## Verification` opens a new one. The closing line is not re-tested as a
    start.
  * `sed '$d'` drops the last line of the COMBINED output, and `tail -n +2` the
    first. If no closing heading exists, the range runs to EOF and the file's own
    last line is the one dropped.
  * A file the real pipeline's `python3` cannot decode as UTF-8 yields an EMPTY
    block there (the traceback goes to /dev/null), so it yields empty here too.

Usage:
  verification-block-batch.py extract <framework_root> <outdir>
      NUL-separated file paths on stdin. Writes <outdir>/<index> for every file
      whose extracted block is non-empty (index = position on stdin) and prints
      nothing else. Exit 0 ok, 2 tooling (comment_strip unloadable).
"""
from __future__ import annotations

import importlib.util
import os
import re
import sys

START = re.compile(r"^## Verification")
# awk's /^## Verification[[:space:]]*$/ — [[:space:]] is the ASCII set in C/UTF-8 locales.
START_EXACT = re.compile(r"^## Verification[ \t\n\r\f\v]*$")
END = re.compile(r"^## ")
# grep -E '^\s*$|^\s*#|^\s*```' with GNU \s ([[:space:]]); ASCII set, as in the C/UTF-8
# locales CI and this host use. The corpus proof pins it.
DROP = re.compile(r"^[ \t\n\r\f\v]*$|^[ \t\n\r\f\v]*#|^[ \t\n\r\f\v]*```")


def load_strip(framework_root: str):
    path = os.path.join(framework_root, "lib", "comment_strip.py")
    spec = importlib.util.spec_from_file_location("fw_comment_strip", path)
    if spec is None or spec.loader is None:
        raise ImportError(path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod.strip_html_comment_lines


def sed_lines(data: str) -> list[str]:
    parts = data.split("\n")
    if data.endswith("\n"):
        parts = parts[:-1]
    return parts


def extract(raw: bytes, strip) -> str:
    try:
        data = raw.decode("utf-8")
    except UnicodeDecodeError:
        return ""
    # T-3370: AEF 1.8.3 (its T-3134) replaced the sed range with an anchored,
    # first-match-only awk selection in lib/verification-port.sh:
    #     /^## Verification[[:space:]]*$/ { if (!seen) { seen=1; inblk=1 }; next }
    #     inblk && /^## / { inblk=0 }
    #     inblk { print }
    # Re-expressed line for line. Note the `next`: a LATER exact heading is skipped
    # and does NOT close an open block, and a prefix like `## Verification Notes` no
    # longer opens one. The old sed-range emulation (repeating ranges, `sed '$d'`,
    # `tail -n +2`) is gone; the runtime cross-check and the corpus proof pin this.
    sel: list[str] = []
    seen = inblk = False
    for line in sed_lines(data):
        if START_EXACT.match(line):
            if not seen:
                seen = inblk = True
            continue
        if inblk and END.match(line):
            inblk = False
        if inblk:
            sel.append(line)
    if not sel:
        return ""
    stripped = strip("".join(l + "\n" for l in sel))
    kept = [l for l in sed_lines(stripped) if not DROP.match(l)]
    return "".join(l + "\n" for l in kept)


def main(argv: list[str]) -> int:
    if len(argv) != 4 or argv[1] != "extract":
        sys.stderr.write(__doc__ or "")
        return 2
    fw_root, outdir = argv[2], argv[3]
    try:
        strip = load_strip(fw_root)
    except Exception as e:  # noqa: BLE001 — any load failure is tooling
        sys.stderr.write(f"verification-block-batch: cannot load comment_strip: {e}\n")
        return 2
    paths = [p for p in sys.stdin.buffer.read().split(b"\0") if p]
    for i, p in enumerate(paths):
        try:
            with open(p, "rb") as fh:
                raw = fh.read()
        except OSError:
            continue    # the real pipeline's sed errors to /dev/null -> empty
        blk = extract(raw, strip)
        if blk:
            with open(os.path.join(outdir, str(i)), "w", encoding="utf-8", errors="surrogateescape") as out:
                out.write(blk)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
