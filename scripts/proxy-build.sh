#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

docker build -t apm-proxy "$ROOT/apm-proxy"
echo "Built apm-proxy image"
