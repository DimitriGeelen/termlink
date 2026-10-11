#!/usr/bin/env python3
"""arc-011 PH-1 B1: assert the PostToolUse transcript fixture has the record shape
the sidecar finalizer (B9) will rely on. Hermetic: reads only the committed fixture.
Exit 0 ok / 1 shape violated / 2 tooling (fixture missing or unreadable)."""
import json, os, sys

FIX = os.environ.get("ARC011_FIXTURE", os.path.join(os.path.dirname(os.path.abspath(__file__)), "fixtures", "transcript-posttooluse.jsonl"))
try:
    recs = [json.loads(l) for l in open(FIX) if l.strip()]
except (OSError, ValueError) as e:
    print(f"tooling: cannot read fixture {FIX}: {e}", file=sys.stderr); sys.exit(2)
if not recs:
    print("tooling: empty fixture", file=sys.stderr); sys.exit(2)

def fail(m): print("FAIL:", m); sys.exit(1)

idx = [i for i, r in enumerate(recs) if r.get("type") == "attachment" and r["attachment"].get("type") == "hook_additional_context"]
if len(idx) != 1: fail(f"expected exactly one hook_additional_context record, found {len(idx)}")
i = idx[0]; a = recs[i]["attachment"]
if a.get("hookEvent") != "PostToolUse": fail("hookEvent is not PostToolUse")
if not str(a.get("hookName", "")).startswith("PostToolUse:"): fail("hookName shape")
if not (isinstance(a.get("content"), list) and a["content"] and "nonce=" in a["content"][0]): fail("content[0] lacks the header/nonce line")
tid = a.get("toolUseID")
if not tid: fail("no toolUseID")
res = [j for j, r in enumerate(recs) if r.get("type") == "user" and isinstance(r["message"]["content"], list)
       and any(c.get("type") == "tool_result" and c.get("tool_use_id") == tid for c in r["message"]["content"])]
if not res or res[0] > i: fail("attachment must follow the tool_result of the same toolUseID")
if not any(r.get("type") == "assistant" for r in recs[i + 1:]): fail("no assistant record after the attachment (context not consumed in-turn)")
# FC-11: the nonce must NOT count as evidence anywhere else in this fixture except the harness-written attachments
nonce = a["content"][0].split("nonce=")[1].split()[0].split("\\n")[0]
for j, r in enumerate(recs):
    if j != i and r.get("type") != "attachment" and nonce in json.dumps(r): fail(f"nonce leaked into a {r.get('type')} record")
print("ok: PostToolUse additionalContext lands as attachment/hook_additional_context after its tool_result")
