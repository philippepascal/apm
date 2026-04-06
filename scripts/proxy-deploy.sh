#!/usr/bin/env bash
set -euo pipefail

red()  { printf '\033[0;31m%s\033[0m\n' "$*"; }
bold() { printf '\033[1m%s\033[0m\n' "$*"; }

abort() { red "ERROR: $*" >&2; exit 1; }

[[ -n "${DOMAIN:-}" ]] || abort "DOMAIN is required (e.g. DOMAIN=apm.example.com)"
[[ -n "${EMAIL:-}"  ]] || abort "EMAIL is required for Let's Encrypt (e.g. EMAIL=you@example.com)"

UPSTREAM="${UPSTREAM:-http://host.docker.internal:3000}"

bold "Deploying apm-proxy"
echo "  Domain:   $DOMAIN"
echo "  Email:    $EMAIL"
echo "  Upstream: $UPSTREAM"
echo

docker run -d \
  --name apm-proxy \
  --restart unless-stopped \
  -p 80:80 -p 443:443 \
  -e DOMAIN="$DOMAIN" \
  -e EMAIL="$EMAIL" \
  -e TLS_MODE=letsencrypt \
  -e UPSTREAM="$UPSTREAM" \
  -v apm-letsencrypt:/etc/letsencrypt \
  apm-proxy

echo
bold "apm-proxy running at https://$DOMAIN"
echo "  Certs stored in docker volume: apm-letsencrypt"
echo "  Logs: docker logs apm-proxy"
echo "  Stop: docker stop apm-proxy && docker rm apm-proxy"
