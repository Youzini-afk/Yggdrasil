use serde::{Deserialize, Serialize};

use crate::ids::{ModelCallId, PromptFrameId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelRole {
    System,
    Developer,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMessage {
    pub role: ModelRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingParams {
    pub temperature: f32,
    pub max_output_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub stop_sequences: Vec<String>,
}

impl Default for SamplingParams {
    fn default() -> Self {
        Self {
            temperature: 0.8,
            max_output_tokens: None,
            top_p: None,
            stop_sequences: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelCallStatus {
    Pending,
    Running,
    Completed,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCall {
    pub id: ModelCallId,
    pub provider: String,
    pub model: String,
    pub prompt_frame_id: PromptFrameId,
    pub parameters: SamplingParams,
    pub status: ModelCallStatus,
    pub output: Option<String>,
}
