//! pi integration placeholders.
//!
//! pi should read events and produce proposals. Yggdrasil validates and commits
//! proposals through events.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTaskStub {
    pub id: String,
    pub kind: String,
    pub session_id: String,
    pub status: String,
}
