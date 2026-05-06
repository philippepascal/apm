#!/usr/bin/env python3
import json
import os
import pathlib
import subprocess
import sys
import urllib.request

APM_SYSTEM_PROMPT_FILE = os.environ["APM_SYSTEM_PROMPT_FILE"]
APM_USER_MESSAGE_FILE = os.environ["APM_USER_MESSAGE_FILE"]
APM_TICKET_ID = os.environ["APM_TICKET_ID"]
APM_BIN = os.environ["APM_BIN"]

TOOLS = [
    {
        "type": "function",
        "function": {
            "name": "bash",
            "description": "Execute a shell command and return stdout+stderr.",
            "parameters": {
                "type": "object",
                "properties": {"command": {"type": "string"}},
                "required": ["command"],
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "read_file",
            "description": "Read and return the contents of a file.",
            "parameters": {
                "type": "object",
                "properties": {"path": {"type": "string"}},
                "required": ["path"],
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "write_file",
            "description": "Write content to a file, creating parent directories as needed.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "content": {"type": "string"},
                },
                "required": ["path", "content"],
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "str_replace",
            "description": "Replace the first occurrence of old_str with new_str in a file.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "old_str": {"type": "string"},
                    "new_str": {"type": "string"},
                },
                "required": ["path", "old_str", "new_str"],
            },
        },
    },
]


def run_tool(name, args):
    if name == "bash":
        result = subprocess.run(
            args["command"], shell=True, capture_output=True, text=True
        )
        output = result.stdout + result.stderr
        return output[:4000]
    elif name == "read_file":
        return pathlib.Path(args["path"]).read_text()
    elif name == "write_file":
        p = pathlib.Path(args["path"])
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(args["content"])
        return "ok"
    elif name == "str_replace":
        p = pathlib.Path(args["path"])
        text = p.read_text()
        p.write_text(text.replace(args["old_str"], args["new_str"], 1))
        return "ok"
    else:
        return f"unknown tool: {name}"


sys_prompt = pathlib.Path(APM_SYSTEM_PROMPT_FILE).read_text()
user_msg = pathlib.Path(APM_USER_MESSAGE_FILE).read_text()

history = [
    {"role": "system", "content": sys_prompt},
    {"role": "user", "content": user_msg},
]

final_text = ""

while True:
    payload = json.dumps(
        {"model": "phi4", "tools": TOOLS, "messages": history, "stream": False}
    ).encode()
    req = urllib.request.Request(
        "http://localhost:11434/v1/chat/completions",
        data=payload,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(req) as resp:
        data = json.loads(resp.read())

    choice = data["choices"][0]
    assistant_msg = choice["message"]

    if choice.get("finish_reason") == "tool_calls":
        history.append(assistant_msg)
        for tc in assistant_msg.get("tool_calls", []):
            tool_name = tc["function"]["name"]
            tool_args = json.loads(tc["function"]["arguments"])
            result = run_tool(tool_name, tool_args)
            history.append(
                {
                    "role": "tool",
                    "tool_call_id": tc["id"],
                    "content": result,
                }
            )
    else:
        final_text = assistant_msg.get("content", "")
        break

print(json.dumps({"type": "result", "text": final_text}))
sys.stdout.flush()

subprocess.run([APM_BIN, "state", APM_TICKET_ID, "implemented"], check=True)
