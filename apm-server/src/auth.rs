use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Duration, Utc};
use rand::distributions::Alphanumeric;
use rand::Rng;

pub fn generate_otp() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(|c| (c as char).to_ascii_uppercase())
        .collect()
}

pub fn generate_token() -> String {
    let bytes: [u8; 32] = rand::thread_rng().gen();
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

struct OtpEntry {
    otp: String,
    created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OtpStore(Arc<Mutex<HashMap<String, OtpEntry>>>);

pub enum OtpError {
    NotFound,
    Expired,
    Invalid,
}

impl OtpStore {
    pub fn new() -> Self {
        OtpStore(Arc::new(Mutex::new(HashMap::new())))
    }

    pub fn insert(&self, username: &str, otp: String) {
        let mut map = self.0.lock().unwrap();
        map.insert(
            username.to_string(),
            OtpEntry {
                otp,
                created_at: Utc::now(),
            },
        );
    }

    pub fn validate(&self, username: &str, otp: &str) -> Result<(), OtpError> {
        let mut map = self.0.lock().unwrap();
        let entry = map.get(username).ok_or(OtpError::NotFound)?;
        if Utc::now() - entry.created_at > Duration::minutes(5) {
            map.remove(username);
            return Err(OtpError::Expired);
        }
        if entry.otp != otp {
            return Err(OtpError::Invalid);
        }
        map.remove(username);
        Ok(())
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct SessionEntry {
    token: String,
    username: String,
    #[serde(default)]
    device_hint: Option<String>,
    #[serde(default = "Utc::now")]
    last_seen: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SessionsFile {
    sessions: Vec<SessionEntry>,
}

#[derive(Clone)]
pub struct SessionStore {
    inner: Arc<Mutex<HashMap<String, SessionEntry>>>,
    path: PathBuf,
}

impl SessionStore {
    pub fn load(path: PathBuf) -> Self {
        let inner = if path.as_os_str().is_empty() || !path.exists() {
            HashMap::new()
        } else {
            match std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str::<SessionsFile>(&s).ok())
            {
                Some(file) => file.sessions.into_iter().map(|e| (e.token.clone(), e)).collect(),
                None => {
                    eprintln!("warn: could not parse {}", path.display());
                    HashMap::new()
                }
            }
        };
        SessionStore {
            inner: Arc::new(Mutex::new(inner)),
            path,
        }
    }

    pub fn insert(&self, token: String, username: String, device_hint: Option<String>) {
        let now = Utc::now();
        let expires_at = now + Duration::days(7);
        let entry = SessionEntry {
            token: token.clone(),
            username,
            device_hint,
            last_seen: now,
            expires_at,
        };
        self.inner.lock().unwrap().insert(token, entry);
        self.save();
    }

    pub fn lookup(&self, token: &str) -> Option<String> {
        let mut map = self.inner.lock().unwrap();
        match map.get(token) {
            None => None,
            Some(entry) => {
                if Utc::now() > entry.expires_at {
                    map.remove(token);
                    None
                } else {
                    Some(entry.username.clone())
                }
            }
        }
    }

    fn save(&self) {
        if self.path.as_os_str().is_empty() {
            return;
        }
        let sessions: Vec<SessionEntry> = self.inner.lock().unwrap().values().cloned().collect();
        let file = SessionsFile { sessions };
        let tmp_path = self.path.with_extension("json.tmp");
        if let Ok(json) = serde_json::to_string_pretty(&file) {
            if std::fs::write(&tmp_path, &json).is_ok() {
                let _ = std::fs::rename(&tmp_path, &self.path);
            }
        }
    }
}

#[derive(serde::Serialize, Clone)]
pub struct SessionInfo {
    pub username: String,
    pub device_hint: Option<String>,
    pub last_seen: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(serde::Deserialize)]
pub struct RevokeRequest {
    pub username: Option<String>,
    pub device: Option<String>,
    #[serde(default)]
    pub all: bool,
}

#[derive(serde::Serialize)]
pub struct RevokeResponse {
    pub revoked: usize,
}

impl SessionStore {
    pub fn list_active(&self) -> Vec<SessionInfo> {
        let map = self.inner.lock().unwrap();
        let now = Utc::now();
        map.values()
            .filter(|e| e.expires_at > now)
            .map(|e| SessionInfo {
                username: e.username.clone(),
                device_hint: e.device_hint.clone(),
                last_seen: e.last_seen,
                expires_at: e.expires_at,
            })
            .collect()
    }

    pub fn revoke(&self, req: &RevokeRequest) -> usize {
        let mut map = self.inner.lock().unwrap();
        let before = map.len();
        if req.all {
            map.clear();
        } else {
            map.retain(|_, e| {
                let username_match = req.username.as_deref().map(|u| e.username != u).unwrap_or(false);
                let device_match = req.device.as_deref().map(|d| e.device_hint.as_deref() != Some(d)).unwrap_or(false);
                if req.device.is_some() {
                    username_match || device_match
                } else {
                    username_match
                }
            });
        }
        let revoked = before - map.len();
        if revoked > 0 {
            drop(map);
            self.save();
        }
        revoked
    }
}

#[cfg(test)]
impl OtpStore {
    pub fn insert_at(&self, username: &str, otp: String, created_at: DateTime<Utc>) {
        self.0
            .lock()
            .unwrap()
            .insert(username.to_string(), OtpEntry { otp, created_at });
    }
}

#[cfg(test)]
impl SessionStore {
    pub fn insert_expiring_at(&self, token: String, username: String, expires_at: DateTime<Utc>) {
        let entry = SessionEntry {
            token: token.clone(),
            username,
            device_hint: None,
            last_seen: Utc::now(),
            expires_at,
        };
        self.inner.lock().unwrap().insert(token, entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn otp_insert_and_validate_happy_path() {
        let store = OtpStore::new();
        store.insert("alice", "ABCD1234".to_string());
        assert!(store.validate("alice", "ABCD1234").is_ok());
    }

    #[test]
    fn otp_validate_expired() {
        let store = OtpStore::new();
        let old = Utc::now() - Duration::minutes(6);
        store.insert_at("alice", "ABCD1234".to_string(), old);
        assert!(matches!(store.validate("alice", "ABCD1234"), Err(OtpError::Expired)));
    }

    #[test]
    fn otp_validate_wrong_value() {
        let store = OtpStore::new();
        store.insert("alice", "ABCD1234".to_string());
        assert!(matches!(store.validate("alice", "WRONGVAL"), Err(OtpError::Invalid)));
    }

    #[test]
    fn otp_validate_twice_second_fails() {
        let store = OtpStore::new();
        store.insert("alice", "ABCD1234".to_string());
        assert!(store.validate("alice", "ABCD1234").is_ok());
        assert!(matches!(store.validate("alice", "ABCD1234"), Err(OtpError::NotFound)));
    }

    #[test]
    fn session_insert_and_lookup() {
        let store = SessionStore::load(PathBuf::new());
        store.insert("tok1".to_string(), "alice".to_string(), None);
        assert_eq!(store.lookup("tok1"), Some("alice".to_string()));
    }

    #[test]
    fn session_lookup_expired() {
        let store = SessionStore::load(PathBuf::new());
        let expired = Utc::now() - Duration::days(8);
        store.insert_expiring_at("tok2".to_string(), "alice".to_string(), expired);
        assert_eq!(store.lookup("tok2"), None);
    }

    #[test]
    fn session_survives_reload() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sessions.json");
        let store = SessionStore::load(path.clone());
        store.insert("tok3".to_string(), "bob".to_string(), None);
        drop(store);
        let store2 = SessionStore::load(path);
        assert_eq!(store2.lookup("tok3"), Some("bob".to_string()));
    }

    #[test]
    fn otp_replace_existing() {
        let store = OtpStore::new();
        store.insert("alice", "FIRST123".to_string());
        store.insert("alice", "SECOND45".to_string());
        assert!(matches!(store.validate("alice", "FIRST123"), Err(OtpError::Invalid)));
        assert!(store.validate("alice", "SECOND45").is_ok());
    }

    #[test]
    fn list_active_excludes_expired() {
        let store = SessionStore::load(PathBuf::new());
        let expired = Utc::now() - Duration::days(8);
        store.insert_expiring_at("tok-exp".to_string(), "eve".to_string(), expired);
        store.insert("tok-ok".to_string(), "alice".to_string(), None);
        let active = store.list_active();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].username, "alice");
    }

    #[test]
    fn revoke_all_clears_store() {
        let store = SessionStore::load(PathBuf::new());
        store.insert("t1".to_string(), "alice".to_string(), None);
        store.insert("t2".to_string(), "bob".to_string(), None);
        let req = RevokeRequest { username: None, device: None, all: true };
        let count = store.revoke(&req);
        assert_eq!(count, 2);
        assert!(store.list_active().is_empty());
    }

    #[test]
    fn revoke_by_username_filters_correctly() {
        let store = SessionStore::load(PathBuf::new());
        store.insert("t1".to_string(), "alice".to_string(), None);
        store.insert("t2".to_string(), "alice".to_string(), None);
        store.insert("t3".to_string(), "bob".to_string(), None);
        let req = RevokeRequest { username: Some("alice".to_string()), device: None, all: false };
        let count = store.revoke(&req);
        assert_eq!(count, 2);
        let remaining = store.list_active();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].username, "bob");
    }

    #[test]
    fn revoke_by_username_and_device() {
        let store = SessionStore::load(PathBuf::new());
        store.insert("t1".to_string(), "alice".to_string(), Some("MacBook".to_string()));
        store.insert("t2".to_string(), "alice".to_string(), Some("iPhone".to_string()));
        let req = RevokeRequest {
            username: Some("alice".to_string()),
            device: Some("MacBook".to_string()),
            all: false,
        };
        let count = store.revoke(&req);
        assert_eq!(count, 1);
        let remaining = store.list_active();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].device_hint.as_deref(), Some("iPhone"));
    }
}
