#!/usr/bin/env bash
set -euo pipefail

# Run apm-proxy locally with a self-signed cert for testing.
# Expects apm-server to be running on port 3000.

docker run --rm -p 80:80 -p 443:443 \
  -e DOMAIN=localhost \
  -e TLS_MODE=self-signed \
  -e UPSTREAM=http://host.docker.internal:3000 \
  apm-proxy
