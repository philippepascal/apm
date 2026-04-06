# apm-proxy (optional / legacy)

`apm-proxy` is an nginx-based reverse proxy that terminates TLS for `apm-server`.

It is no longer necessary for production deployments. `apm-server` now has
built-in TLS support via `--tls` (Let's Encrypt / ACME), `--tls=self-signed`,
or `--tls-cert / --tls-key` (custom certificate). See `apm-server --help`.

This directory is kept for users who prefer the Docker-based nginx approach or
need features such as HTTP-to-HTTPS redirect on port 80.
