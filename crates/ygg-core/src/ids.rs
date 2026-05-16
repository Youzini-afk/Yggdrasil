use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type EventId = String;
pub type StreamId = String;
pub type SessionId = String;
pub type TurnId = String;
pub type ActorId = String;
pub type PromptFrameId = String;
pub type ModelCallId = String;

pub fn new_id(prefix: &str) -> String {
    format!("{prefix}_{}", Uuid::new_v4().simple())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdRef {
    pub id: String,
}
