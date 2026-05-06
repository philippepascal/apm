#!/usr/bin/env python3
import sys
import json

for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    try:
        event = json.loads(line)
    except json.JSONDecodeError:
        continue

    t = event.get("type")

    if t == "message_end":
        msg = event.get("message", {})
        parts = [
            block.get("text", "")
            for block in msg.get("content", [])
            if block.get("type") == "text"
        ]
        text = "".join(parts)
        if text:
            print(json.dumps({"type": "text", "text": text}), flush=True)

    elif t == "agent_end":
        print(json.dumps({"type": "result", "text": ""}), flush=True)
        break
