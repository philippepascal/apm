use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::Router;
use axum::body::Body;
use axum::extract::ConnectInfo;
use hyper::Request;
use hyper::body::Incoming;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder as AutoBuilder;
use rustls::ServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tower::Service;

pub fn self_signed_config(domain: &str) -> Result<Arc<ServerConfig>> {
    let certified = rcgen::generate_simple_self_signed(vec![domain.to_string()])
        .context("generate self-signed cert")?;
    let cert_der: CertificateDer<'static> = certified.cert.der().clone();
    let key_der = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(certified.key_pair.serialize_der()));
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der], key_der)
        .context("build TLS config")?;
    Ok(Arc::new(config))
}

pub fn custom_cert_config(cert_path: &Path, key_path: &Path) -> Result<Arc<ServerConfig>> {
    let cert_bytes = std::fs::read(cert_path).context("read cert file")?;
    let key_bytes = std::fs::read(key_path).context("read key file")?;
    let certs = crate::pem::parse_certs(&cert_bytes).context("parse cert PEM")?;
    if certs.is_empty() {
        return Err(anyhow::anyhow!("no CERTIFICATE blocks found in cert file"));
    }
    let key = crate::pem::parse_private_key(&key_bytes)
        .context("parse key PEM")?
        .context("no private key found in file")?;
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .context("build TLS config")?;
    Ok(Arc::new(config))
}

pub async fn serve_tls(addr: SocketAddr, app: Router, config: Arc<ServerConfig>) {
    let acceptor = TlsAcceptor::from(config);
    let listener = TcpListener::bind(addr).await.expect("bind TLS listener");
    println!("Listening on https://{addr}");
    loop {
        let (tcp, remote_addr) = match listener.accept().await {
            Ok(pair) => pair,
            Err(e) => {
                eprintln!("accept error: {e}");
                continue;
            }
        };
        let acceptor = acceptor.clone();
        let app = app.clone();
        tokio::spawn(async move {
            let Ok(tls) = acceptor.accept(tcp).await else { return };
            let io = TokioIo::new(tls);
            let svc = hyper::service::service_fn(move |mut req: Request<Incoming>| {
                req.extensions_mut().insert(ConnectInfo(remote_addr));
                let req = req.map(Body::new);
                let mut app = app.clone();
                async move { app.call(req).await }
            });
            AutoBuilder::new(TokioExecutor::new())
                .serve_connection_with_upgrades(io, svc)
                .await
                .ok();
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn install_provider() {
        let _ = rustls::crypto::ring::default_provider().install_default();
    }

    #[test]
    fn self_signed_config_succeeds() {
        install_provider();
        let config = self_signed_config("localhost").expect("self_signed_config failed");
        drop(config);
    }

    #[test]
    fn self_signed_config_custom_domain() {
        install_provider();
        let config = self_signed_config("example.internal").expect("self_signed_config failed");
        drop(config);
    }

    #[test]
    fn custom_cert_config_roundtrip() {
        install_provider();
        let certified = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])
            .expect("rcgen failed");
        let cert_pem = certified.cert.pem();
        let key_pem = certified.key_pair.serialize_pem();

        let dir = tempdir().expect("tempdir");
        let cert_path = dir.path().join("cert.pem");
        let key_path = dir.path().join("key.pem");
        std::fs::write(&cert_path, &cert_pem).expect("write cert");
        std::fs::write(&key_path, &key_pem).expect("write key");

        let config = custom_cert_config(&cert_path, &key_path)
            .expect("custom_cert_config failed");
        drop(config);
    }

    #[test]
    fn custom_cert_config_missing_cert_file() {
        let dir = tempdir().expect("tempdir");
        let cert_path = dir.path().join("nonexistent.pem");
        let key_path = dir.path().join("nonexistent.key");
        assert!(custom_cert_config(&cert_path, &key_path).is_err());
    }

    #[test]
    fn custom_cert_config_no_certificate_blocks_errors() {
        install_provider();
        let certified = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])
            .expect("rcgen failed");
        let key_pem = certified.key_pair.serialize_pem();

        let dir = tempdir().expect("tempdir");
        let cert_path = dir.path().join("empty.pem");
        let key_path = dir.path().join("key.pem");
        std::fs::write(&cert_path, "no certificate blocks here").expect("write cert");
        std::fs::write(&key_path, &key_pem).expect("write key");

        assert!(custom_cert_config(&cert_path, &key_path).is_err());
    }

    #[test]
    fn custom_cert_config_no_private_key_block_errors() {
        install_provider();
        let certified = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])
            .expect("rcgen failed");
        let cert_pem = certified.cert.pem();

        let dir = tempdir().expect("tempdir");
        let cert_path = dir.path().join("cert.pem");
        let key_path = dir.path().join("empty.pem");
        std::fs::write(&cert_path, &cert_pem).expect("write cert");
        std::fs::write(&key_path, "no private key here").expect("write key");

        assert!(custom_cert_config(&cert_path, &key_path).is_err());
    }
}

pub async fn serve_acme(
    addr: SocketAddr,
    app: Router,
    domain: String,
    email: String,
    cache_dir: std::path::PathBuf,
) {
    use rustls_acme::AcmeConfig;
    use rustls_acme::caches::DirCache;
    use tokio_stream::StreamExt;
    use tokio_util::compat::{FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt};

    #[allow(deprecated)]
    let mut state = AcmeConfig::new(vec![domain.as_str()])
        .contact_push(format!("mailto:{email}"))
        .cache(DirCache::new(cache_dir))
        .directory_lets_encrypt(true)
        .state();

    #[allow(deprecated)]
    let acceptor = state.acceptor();
    let rustls_config = state.default_rustls_config();

    // Drive the ACME state machine (certificate issuance & renewal).
    tokio::spawn(async move {
        while let Some(event) = state.next().await {
            match event {
                Ok(ok) => eprintln!("acme event: {ok:?}"),
                Err(err) => eprintln!("acme error: {err:?}"),
            }
        }
    });

    let listener = TcpListener::bind(addr).await.expect("bind ACME listener");
    println!("Listening on https://{addr} (Let's Encrypt ACME)");

    loop {
        let (tcp, remote_addr) = match listener.accept().await {
            Ok(pair) => pair,
            Err(e) => {
                eprintln!("accept error: {e}");
                continue;
            }
        };
        let acceptor = acceptor.clone();
        let rustls_config = rustls_config.clone();
        let app = app.clone();
        tokio::spawn(async move {
            // Bridge tokio TcpStream to futures-io for rustls-acme.
            let compat_tcp = tcp.compat();
            match acceptor.accept(compat_tcp).await {
                Ok(None) => {} // ACME TLS-ALPN-01 challenge handled
                Ok(Some(start)) => {
                    let Ok(tls) = start.into_stream(rustls_config).await else {
                        return;
                    };
                    // Bridge futures-io TlsStream back to tokio-io for hyper.
                    let io = TokioIo::new(tls.compat());
                    let svc =
                        hyper::service::service_fn(move |mut req: Request<Incoming>| {
                            req.extensions_mut().insert(ConnectInfo(remote_addr));
                            let req = req.map(Body::new);
                            let mut app = app.clone();
                            async move { app.call(req).await }
                        });
                    AutoBuilder::new(TokioExecutor::new())
                        .serve_connection_with_upgrades(io, svc)
                        .await
                        .ok();
                }
                Err(e) => eprintln!("tls accept error: {e}"),
            }
        });
    }
}
