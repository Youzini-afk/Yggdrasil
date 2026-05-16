use serde::{Deserialize, Serialize};

use crate::ids::{PromptFrameId, SessionId, TurnId};
use crate::model::{ModelMessage, SamplingParams};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSelection {
    pub kind: String,
    pub id: Option<String>,
    pub content: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPlan {
    pub id: String,
    pub session_id: SessionId,
    pub turn_id: TurnId,
    pub budget_tokens: Option<u32>,
    pub selected: Vec<ContextSelection>,
    pub omitted: Vec<ContextSelection>,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptFrame {
    pub id: PromptFrameId,
    pub session_id: SessionId,
    pub turn_id: TurnId,
    pub model_target: String,
    pub messages: Vec<ModelMessage>,
    pub sampling: SamplingParams,
    pub token_estimate: Option<u32>,
    pub context_plan_id: String,
    pub render_trace: Vec<String>,
}
