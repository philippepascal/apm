// Ticket DTOs
#[derive(serde::Serialize)]
pub struct TransitionOption {
    pub to: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

#[derive(serde::Serialize)]
pub struct TicketResponse {
    #[serde(flatten)]
    pub frontmatter: apm_core::ticket::Frontmatter,
    pub body: String,
    pub has_open_questions: bool,
    pub has_pending_amendments: bool,
    pub blocking_deps: Vec<BlockingDep>,
    pub owner: Option<String>,
}

#[derive(serde::Serialize)]
pub struct TicketsEnvelope {
    pub tickets: Vec<TicketResponse>,
    pub supervisor_states: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct BlockingDep {
    pub id: String,
    pub state: String,
}

#[derive(serde::Serialize)]
pub struct TicketDetailResponse {
    #[serde(flatten)]
    pub frontmatter: apm_core::ticket::Frontmatter,
    pub body: String,
    pub raw: String,
    pub valid_transitions: Vec<TransitionOption>,
    pub blocking_deps: Vec<BlockingDep>,
    pub owner: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct TransitionRequest {
    pub to: String,
}

#[derive(serde::Deserialize)]
pub struct BatchTransitionRequest {
    pub ids: Vec<String>,
    pub to: String,
}

#[derive(serde::Deserialize)]
pub struct BatchPriorityRequest {
    pub ids: Vec<String>,
    pub priority: u8,
}

#[derive(serde::Serialize)]
pub struct BatchFailure {
    pub id: String,
    pub error: String,
}

#[derive(serde::Serialize)]
pub struct BatchResult {
    pub succeeded: Vec<String>,
    pub failed: Vec<BatchFailure>,
}

#[derive(serde::Deserialize)]
pub struct PutBodyRequest {
    pub content: String,
}

#[derive(serde::Deserialize)]
pub struct PatchTicketRequest {
    pub effort: Option<u8>,
    pub risk: Option<u8>,
    pub priority: Option<u8>,
    pub owner: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct CreateTicketRequest {
    pub title: Option<String>,
    pub sections: Option<std::collections::HashMap<String, String>>,
    pub epic: Option<String>,
    pub depends_on: Option<Vec<String>>,
}

// Epic DTOs
#[derive(serde::Serialize)]
pub struct EpicSummary {
    pub id: String,
    pub title: String,
    pub branch: String,
    pub state: String,
    pub ticket_counts: std::collections::HashMap<String, usize>,
}

#[derive(serde::Serialize)]
pub struct EpicDetailResponse {
    #[serde(flatten)]
    pub summary: EpicSummary,
    pub tickets: Vec<TicketResponse>,
}

#[derive(serde::Deserialize)]
pub struct CreateEpicRequest {
    pub title: Option<String>,
}

// Misc handler DTOs
#[derive(serde::Deserialize, Default)]
pub struct CleanRequest {
    pub dry_run:    Option<bool>,
    pub force:      Option<bool>,
    pub branches:   Option<bool>,
    pub remote:     Option<bool>,
    pub older_than: Option<String>,
    pub untracked:  Option<bool>,
    pub epics:      Option<bool>,
}

#[derive(serde::Deserialize, Default)]
pub struct ListTicketsQuery {
    pub include_closed: Option<bool>,
    pub author: Option<String>,
    pub owner: Option<String>,
}

// Auth/WebAuthn DTOs
#[derive(serde::Deserialize)]
pub struct RegisterChallengeRequest {
    pub username: String,
    pub otp: String,
}

#[derive(serde::Serialize)]
pub struct RegisterChallengeResponse {
    pub reg_id: String,
    #[serde(rename = "publicKey")]
    pub public_key: serde_json::Value,
}

#[derive(serde::Deserialize)]
pub struct RegisterCompleteRequest {
    pub reg_id: String,
    pub response: webauthn_rs::prelude::RegisterPublicKeyCredential,
}

#[derive(serde::Deserialize)]
pub struct LoginChallengeRequest {
    pub username: String,
}

#[derive(serde::Serialize)]
pub struct LoginChallengeResponse {
    pub login_id: String,
    #[serde(rename = "publicKey")]
    pub public_key: serde_json::Value,
}

#[derive(serde::Deserialize)]
pub struct LoginCompleteRequest {
    pub login_id: String,
    pub response: webauthn_rs::prelude::PublicKeyCredential,
}
