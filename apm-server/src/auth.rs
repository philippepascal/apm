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

    pub fn insert(&self, token: String, username: String) {
        let expires_at = Utc::now() + Duration::days(7);
        let entry = SessionEntry {
            token: token.clone(),
            username,
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
        store.insert("tok1".to_string(), "alice".to_string());
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
        store.insert("tok3".to_string(), "bob".to_string());
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
}
