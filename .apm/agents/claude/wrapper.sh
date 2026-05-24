#!/bin/sh
# APM default wrapper — invokes the claude binary with standard APM arguments.
# Edit this file to customise agent invocation for this project (binary path,
# extra flags, model pinning, etc.).
set -e

sys=$(cat "$APM_SYSTEM_PROMPT_FILE")
msg=$(cat "$APM_USER_MESSAGE_FILE")

set --
[ -n "$APM_MODEL" ] && set -- "$@" --model "$APM_MODEL"
[ "$APM_SKIP_PERMISSIONS" = "1" ] && set -- "$@" --dangerously-skip-permissions

exec claude \
  --print \
  --output-format stream-json \
  --verbose \
  --disable-slash-commands \
  "$@" \
  --system-prompt "$sys" \
  "$msg"
