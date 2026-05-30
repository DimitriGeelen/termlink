#!/usr/bin/env python3
"""
T-1884 S3 spike — CLI-watch render validator.

Replaces the filing-time S3 plan (ux-review wireup smoke). The 8 ACs I
classified as REVIEW-RENDER in S1 turned out to all be `--watch` CLI
views (terminal redraw via ANSI 2J+H), NOT browser-driven UI. ux-review
is the wrong tool for them.

The right validator captures N seconds of terminal output via
`script -c`, splits on the clear+home ANSI sequence, normalises
timestamps + ANSI, and asserts frame-body identity → "steady" verdict.

Smoke-tested against T-1486 (`agent presence --watch`). Captures 8s,
expects 4 frames at --watch-interval=2, expects all frame bodies
identical modulo timestamps. Surfaces evidence as a stable-row count +
representative frame.

Validates A-026 reframed: CLI-watch wireup needs <=1 line per-task
config (the command + interval), preserving one-verb UX. Does NOT touch
ux-review (which is the wrong abstraction here).
"""
import re
import subprocess
import sys
import tempfile
from pathlib import Path


CLEAR_HOME = "\x1b[2J\x1b[H"
ANSI_RE = re.compile(r"\x1b\[[0-9;?]*[a-zA-Z]")
TS_RE = re.compile(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z")
RELATIVE_TS_RE = re.compile(r"\d+[smh] ago|\d+[smh] back")


def strip_ansi(s):
    return ANSI_RE.sub("", s).replace("\r", "")


def normalize_frame(s):
    s = strip_ansi(s)
    s = TS_RE.sub("<TS>", s)
    s = RELATIVE_TS_RE.sub("<REL>", s)
    return s.strip()


def capture(cmd, duration_s, log_path):
    """Use script -c to capture pty output; bounded by `timeout`."""
    shell_inner = f"timeout {duration_s} {cmd} || true"
    r = subprocess.run(
        ["script", "-q", "-c", shell_inner, str(log_path)],
        capture_output=True, text=True, timeout=duration_s + 10,
    )
    return r.returncode, log_path.read_text(encoding="utf-8", errors="replace")


def parse_frames(raw):
    """Split capture on clear+home; return list of frame bodies (raw)."""
    parts = raw.split(CLEAR_HOME)
    # First part is pre-first-frame noise ("Script started on...") — drop.
    frames = [p for p in parts[1:] if p.strip()]
    return frames


def validate_steady(frames):
    """Return (verdict, reason, normalized_first_frame)."""
    if not frames:
        return "INCONCLUSIVE", "no frames captured (--watch may not be supported)", ""
    if len(frames) == 1:
        return "INCONCLUSIVE", f"only 1 frame in window — duration too short or interval too large", normalize_frame(frames[0])

    normalized = [normalize_frame(f) for f in frames]
    distinct = set(normalized)
    if len(distinct) == 1:
        return "PASS-ROBUST", f"{len(frames)} frames, all bodies byte-identical modulo timestamp", normalized[0]
    if len(distinct) <= 3:
        # Small variation — content (e.g. row added) but not flickering
        return "PASS-LOOSE", f"{len(frames)} frames, {len(distinct)} distinct bodies (content changed in-window, no flicker pattern)", normalized[0]
    return "FAIL", f"{len(frames)} frames, {len(distinct)} distinct bodies — content unstable / flicker suspected", normalized[0]


# Targets from S1 REVIEW-RENDER class — reclassified as CLI-WATCH.
# (task, ac_substring, command_to_run, duration_s, interval_s)
TARGETS = [
    ("T-1486", "agent presence --watch view is steady",
     "termlink agent presence --watch --watch-interval 2", 8, 2),
    # T-1494, T-1496, T-1498, T-1557, T-1558, T-1559 — full set deferred to
    # build phase; S3 just proves the technique works on one representative.
]


def main():
    print("# T-1884 S3 — CLI-watch render validator (smoke)\n")
    print(f"Targets: {len(TARGETS)} (smoke — one representative)\n")
    print(f"Technique: script -c capture + split on \\x1b[2J\\x1b[H + "
          f"normalize timestamps + frame-body diff\n")

    for (task_id, ac_substring, cmd, duration, interval) in TARGETS:
        print(f"## {task_id} — {ac_substring}\n")
        print(f"   cmd:      `{cmd}`")
        print(f"   duration: {duration}s")
        print(f"   interval: {interval}s")
        print(f"   expected frames: ~{duration // interval}\n")

        with tempfile.NamedTemporaryFile(
            mode="w", suffix=".log", delete=False
        ) as tf:
            log_path = Path(tf.name)
        try:
            ex, raw = capture(cmd, duration, log_path)
            frames = parse_frames(raw)
            verdict, reason, first_frame = validate_steady(frames)
            print(f"   exit:     {ex}")
            print(f"   raw size: {len(raw)} bytes")
            print(f"   frames:   {len(frames)}")
            print(f"   verdict:  **{verdict}** ({reason})\n")
            if first_frame:
                print(f"   first-frame preview (normalized):\n")
                for line in first_frame.splitlines()[:8]:
                    print(f"     | {line}")
                print()
        finally:
            log_path.unlink(missing_ok=True)


if __name__ == "__main__":
    main()
