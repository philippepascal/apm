#!/bin/sh
# APM pi wrapper — invokes pi CLI with Phi-4 via Ollama.
#
# Prerequisites:
#   1. Install pi CLI: see https://pi.dev/docs/install
#   2. Install Ollama: see https://ollama.com
#   3. Pull the model: ollama pull phi4
#   4. Configure ~/.pi/agent/models.json to register the Ollama provider:
#
#      {
#        "ollama": {
#          "type": "ollama",
#          "base_url": "http://localhost:11434",
#          "models": {
#            "phi4": { "context_length": 16384 }
#          }
#        }
#      }
#
#   Adjust base_url and context_length to match your Ollama installation.
set -e

model="${APM_OPT_MODEL:-phi4}"
sys=$(cat "$APM_SYSTEM_PROMPT_FILE")
msg=$(cat "$APM_USER_MESSAGE_FILE")

pi --mode json --provider ollama --model "$model" "$sys

---

$msg"

# Fallback: the agent should call apm state via its bash tool (per apm.worker.md).
# If it doesn't (e.g. tool access is restricted), the shell handles it.
# || true prevents a double-transition error from failing the wrapper.
apm state "$APM_TICKET_ID" implemented || true
