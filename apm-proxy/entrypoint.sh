#!/bin/bash
set -e

# 1. Apply defaults
UPSTREAM="${UPSTREAM:-http://host.docker.internal:3000}"
TLS_MODE="${TLS_MODE:-letsencrypt}"

if [ -z "$DOMAIN" ]; then
    echo "ERROR: DOMAIN environment variable is required" >&2
    exit 1
fi

# 2. Render nginx config from template
mkdir -p /etc/nginx/conf.d
envsubst '${DOMAIN} ${UPSTREAM}' < /etc/nginx/nginx.conf.template > /etc/nginx/conf.d/default.conf

# 3. Create webroot directory for certbot
mkdir -p /var/www/certbot

CERT_DIR="/etc/letsencrypt/live/${DOMAIN}"

if [ "$TLS_MODE" = "self-signed" ]; then
    # 4. Generate self-signed cert
    mkdir -p "$CERT_DIR"
    openssl req -x509 -nodes -newkey rsa:2048 -days 365 \
        -keyout "$CERT_DIR/privkey.pem" \
        -out "$CERT_DIR/fullchain.pem" \
        -subj "/CN=${DOMAIN}"

elif [ "$TLS_MODE" = "letsencrypt" ]; then
    if [ -z "$EMAIL" ]; then
        echo "ERROR: EMAIL environment variable is required for TLS_MODE=letsencrypt" >&2
        exit 1
    fi

    # 5. Check for a real Let's Encrypt cert (cert.pem is written by certbot; absent when only a bootstrap self-signed exists)
    if [ -f "$CERT_DIR/cert.pem" ]; then
        # Real LE cert already present — start nginx in background, skip issuance
        nginx -g 'daemon on;'
    else
        # No real cert — generate temp self-signed, start nginx, obtain LE cert
        mkdir -p "$CERT_DIR"
        openssl req -x509 -nodes -newkey rsa:2048 -days 1 \
            -keyout "$CERT_DIR/privkey.pem" \
            -out "$CERT_DIR/fullchain.pem" \
            -subj "/CN=${DOMAIN}"

        nginx -g 'daemon on;'

        certbot certonly --webroot \
            --webroot-path /var/www/certbot \
            --non-interactive --agree-tos \
            --email "$EMAIL" \
            -d "$DOMAIN"

        nginx -s reload
    fi

    # Background renewal loop every 12 hours
    (
        while true; do
            sleep 43200
            certbot renew --webroot --webroot-path /var/www/certbot --non-interactive --quiet
            nginx -s reload
        done
    ) &

else
    echo "ERROR: Unknown TLS_MODE '${TLS_MODE}'. Use 'letsencrypt' or 'self-signed'." >&2
    exit 1
fi

# 6. Run nginx in foreground
exec nginx -g 'daemon off;'
