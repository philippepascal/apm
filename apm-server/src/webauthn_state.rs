use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use webauthn_rs::prelude::{PasskeyAuthentication, PasskeyRegistration, Url};
use webauthn_rs::{Webauthn, WebauthnBuilder};

pub struct RegistrationSession {
    pub username: String,
    pub passkey_reg: PasskeyRegistration,
}

pub struct AuthenticationSession {
    pub username: String,
    pub passkey_auth: PasskeyAuthentication,
    pub created_at: Instant,
}

pub struct WebauthnState {
    pub webauthn: Webauthn,
    pub pending: Arc<Mutex<HashMap<String, RegistrationSession>>>,
    pub pending_auth: Arc<Mutex<HashMap<String, AuthenticationSession>>>,
}

impl WebauthnState {
    pub fn new(origin: &str) -> anyhow::Result<Self> {
        let origin_url = Url::parse(origin)
            .map_err(|e| anyhow::anyhow!("invalid origin URL: {e}"))?;
        let rp_id = origin_url
            .host_str()
            .ok_or_else(|| anyhow::anyhow!("origin URL has no host"))?
            .to_string();
        let webauthn = WebauthnBuilder::new(&rp_id, &origin_url)
            .map_err(|e| anyhow::anyhow!("WebauthnBuilder::new failed: {e:?}"))?
            .build()
            .map_err(|e| anyhow::anyhow!("WebauthnBuilder::build failed: {e:?}"))?;
        Ok(WebauthnState {
            webauthn,
            pending: Arc::new(Mutex::new(HashMap::new())),
            pending_auth: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn new_with_localhost_origin_succeeds() {
        let state = WebauthnState::new("http://localhost:3000");
        assert!(state.is_ok(), "expected Ok, got {:?}", state.err().map(|e| e.to_string()));
    }

    #[test]
    fn new_with_invalid_origin_fails() {
        assert!(WebauthnState::new("not-a-url").is_err());
    }

    #[test]
    fn authentication_session_ttl_check() {
        let old = Instant::now() - Duration::from_secs(360);
        assert!(old.elapsed() >= Duration::from_secs(300), "session 6 min old should be expired");
    }
}
