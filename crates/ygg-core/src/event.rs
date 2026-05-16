use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::ids::{ActorId, EventId, SessionId, StreamId, TurnId};
use crate::{ContextPlan, ModelCall, PromptFrame};

pub type SchemaVersion = u16;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    SessionCreated,
    UserInputReceived,
    TurnStarted,
    ContextPlanCreated,
    PromptFrameCreated,
    ModelCallStarted,
    ModelStreamDelta,
    ModelCallCompleted,
    MessageCommitted,
    TurnCompleted,
    TurnCancelled,
    ErrorOccurred,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventSource {
    Runtime,
    User,
    ModelProvider,
    Cli,
    Service,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: EventId,
    pub stream_id: StreamId,
    pub session_id: SessionId,
    pub turn_id: Option<TurnId>,
    pub actor_id: Option<ActorId>,
    pub kind: EventKind,
    pub schema_version: SchemaVersion,
    pub timestamp: DateTime<Utc>,
    pub causation_id: Option<EventId>,
    pub correlation_id: Option<EventId>,
    pub source: EventSource,
    pub payload: EventPayload,
    #[serde(default)]
    pub metadata: Value,
}

impl EventEnvelope {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: EventId,
        stream_id: StreamId,
        session_id: SessionId,
        turn_id: Option<TurnId>,
        actor_id: Option<ActorId>,
        kind: EventKind,
        source: EventSource,
        payload: EventPayload,
    ) -> Self {
        Self {
            id,
            stream_id,
            session_id,
            turn_id,
            actor_id,
            kind,
            schema_version: 1,
            timestamp: Utc::now(),
            causation_id: None,
            correlation_id: None,
            source,
            payload,
            metadata: json!({}),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum EventPayload {
    SessionCreated { title: String, runtime_profile: String },
    UserInputReceived { content: String },
    TurnStarted { parent_turn_id: Option<TurnId> },
    ContextPlanCreated { context_plan: ContextPlan },
    PromptFrameCreated { prompt_frame: PromptFrame },
    ModelCallStarted { model_call: ModelCall },
    ModelStreamDelta { model_call_id: String, delta: String },
    ModelCallCompleted { model_call: ModelCall },
    MessageCommitted { role: String, content: String },
    TurnCompleted,
    TurnCancelled { reason: Option<String> },
    ErrorOccurred { message: String, recoverable: bool },
    Json(Value),
}
