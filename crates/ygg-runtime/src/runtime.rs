use std::sync::Arc;

use chrono::Utc;
use ygg_core::{
    new_id, ContextPlan, ContextSelection, EventEnvelope, EventKind, EventPayload, EventSource,
    ModelCall, ModelCallStatus, ModelMessage, ModelRole, PromptFrame, SamplingParams, Session,
    SessionId, SessionStatus, TurnId,
};

use crate::{EventStore, ModelProvider, ModelStreamEvent};

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub default_runtime_profile: String,
    pub default_title: String,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            default_runtime_profile: "native-rp".to_string(),
            default_title: "Untitled Session".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeOutput {
    pub session_id: SessionId,
    pub turn_id: TurnId,
    pub prompt_frame: PromptFrame,
    pub output: String,
}

#[derive(Clone)]
pub struct Runtime<S, M>
where
    S: EventStore,
    M: ModelProvider,
{
    store: Arc<S>,
    model: Arc<M>,
    config: RuntimeConfig,
}

impl<S, M> Runtime<S, M>
where
    S: EventStore,
    M: ModelProvider,
{
    pub fn new(store: Arc<S>, model: Arc<M>, config: RuntimeConfig) -> Self {
        Self { store, model, config }
    }

    pub fn store(&self) -> Arc<S> {
        self.store.clone()
    }

    pub async fn create_session(&self, title: Option<String>) -> anyhow::Result<Session> {
        let now = Utc::now();
        let session = Session {
            id: new_id("ses"),
            title: title.unwrap_or_else(|| self.config.default_title.clone()),
            runtime_profile: self.config.default_runtime_profile.clone(),
            actor_ids: Vec::new(),
            current_turn_id: None,
            created_at: now,
            updated_at: now,
            status: SessionStatus::Active,
        };

        self.append(
            &session.id,
            None,
            EventKind::SessionCreated,
            EventSource::Runtime,
            EventPayload::SessionCreated {
                title: session.title.clone(),
                runtime_profile: session.runtime_profile.clone(),
            },
        )
        .await?;

        Ok(session)
    }

    pub async fn input(&self, session_id: SessionId, content: String) -> anyhow::Result<RuntimeOutput> {
        self.append(
            &session_id,
            None,
            EventKind::UserInputReceived,
            EventSource::User,
            EventPayload::UserInputReceived { content: content.clone() },
        )
        .await?;

        let turn_id = new_id("turn");
        self.append(
            &session_id,
            Some(turn_id.clone()),
            EventKind::TurnStarted,
            EventSource::Runtime,
            EventPayload::TurnStarted { parent_turn_id: None },
        )
        .await?;

        let context_plan = self.build_context_plan(&session_id, &turn_id, &content);
        self.append(
            &session_id,
            Some(turn_id.clone()),
            EventKind::ContextPlanCreated,
            EventSource::Runtime,
            EventPayload::ContextPlanCreated { context_plan: context_plan.clone() },
        )
        .await?;

        let prompt_frame = self.build_prompt_frame(&session_id, &turn_id, &content, &context_plan);
        self.append(
            &session_id,
            Some(turn_id.clone()),
            EventKind::PromptFrameCreated,
            EventSource::Runtime,
            EventPayload::PromptFrameCreated { prompt_frame: prompt_frame.clone() },
        )
        .await?;

        let model_call = ModelCall {
            id: new_id("mcall"),
            provider: self.model.provider_name().to_string(),
            model: self.model.model_name().to_string(),
            prompt_frame_id: prompt_frame.id.clone(),
            parameters: prompt_frame.sampling.clone(),
            status: ModelCallStatus::Running,
            output: None,
        };

        self.append(
            &session_id,
            Some(turn_id.clone()),
            EventKind::ModelCallStarted,
            EventSource::Runtime,
            EventPayload::ModelCallStarted { model_call: model_call.clone() },
        )
        .await?;

        let mut output = String::new();
        let mut stream = self.model.stream(prompt_frame.clone()).await?;
        while let Some(event) = stream.recv().await {
            match event {
                ModelStreamEvent::Delta(delta) => {
                    output.push_str(&delta);
                    self.append(
                        &session_id,
                        Some(turn_id.clone()),
                        EventKind::ModelStreamDelta,
                        EventSource::ModelProvider,
                        EventPayload::ModelStreamDelta {
                            model_call_id: model_call.id.clone(),
                            delta,
                        },
                    )
                    .await?;
                }
                ModelStreamEvent::Completed(final_output) => {
                    output = final_output;
                }
            }
        }

        let completed_model_call = ModelCall {
            status: ModelCallStatus::Completed,
            output: Some(output.clone()),
            ..model_call
        };

        self.append(
            &session_id,
            Some(turn_id.clone()),
            EventKind::ModelCallCompleted,
            EventSource::ModelProvider,
            EventPayload::ModelCallCompleted { model_call: completed_model_call },
        )
        .await?;

        self.append(
            &session_id,
            Some(turn_id.clone()),
            EventKind::MessageCommitted,
            EventSource::Runtime,
            EventPayload::MessageCommitted { role: "assistant".to_string(), content: output.clone() },
        )
        .await?;

        self.append(
            &session_id,
            Some(turn_id.clone()),
            EventKind::TurnCompleted,
            EventSource::Runtime,
            EventPayload::TurnCompleted,
        )
        .await?;

        Ok(RuntimeOutput { session_id, turn_id, prompt_frame, output })
    }

    fn build_context_plan(&self, session_id: &SessionId, turn_id: &TurnId, content: &str) -> ContextPlan {
        ContextPlan {
            id: new_id("ctx"),
            session_id: session_id.clone(),
            turn_id: turn_id.clone(),
            budget_tokens: None,
            selected: vec![ContextSelection {
                kind: "user_input".to_string(),
                id: None,
                content: content.to_string(),
                rationale: "initial minimal runtime includes the current user input".to_string(),
            }],
            omitted: Vec::new(),
            rationale: "minimal context plan for the first runtime spine".to_string(),
        }
    }

    fn build_prompt_frame(
        &self,
        session_id: &SessionId,
        turn_id: &TurnId,
        content: &str,
        context_plan: &ContextPlan,
    ) -> PromptFrame {
        PromptFrame {
            id: new_id("pf"),
            session_id: session_id.clone(),
            turn_id: turn_id.clone(),
            model_target: format!("{}/{}", self.model.provider_name(), self.model.model_name()),
            messages: vec![
                ModelMessage {
                    role: ModelRole::System,
                    content: "You are running inside the Yggdrasil runtime spine demo.".to_string(),
                },
                ModelMessage { role: ModelRole::User, content: content.to_string() },
            ],
            sampling: SamplingParams::default(),
            token_estimate: None,
            context_plan_id: context_plan.id.clone(),
            render_trace: vec!["system demo block".to_string(), "current user input".to_string()],
        }
    }

    async fn append(
        &self,
        session_id: &SessionId,
        turn_id: Option<TurnId>,
        kind: EventKind,
        source: EventSource,
        payload: EventPayload,
    ) -> anyhow::Result<()> {
        let event = EventEnvelope::new(
            new_id("evt"),
            format!("stream_{session_id}"),
            session_id.clone(),
            turn_id,
            None,
            kind,
            source,
            payload,
        );
        self.store.append(event).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use ygg_core::{EventKind, EventPayload};

    use super::*;
    use crate::{InMemoryEventStore, MockModelProvider};

    #[tokio::test]
    async fn input_records_inspectable_runtime_spine() -> anyhow::Result<()> {
        let store = Arc::new(InMemoryEventStore::default());
        let model = Arc::new(MockModelProvider::default());
        let runtime = Runtime::new(store.clone(), model, RuntimeConfig::default());

        let session = runtime.create_session(Some("test".to_string())).await?;
        let output = runtime.input(session.id.clone(), "hello".to_string()).await?;
        let events = store.list_session(&session.id).await?;

        let kinds: Vec<EventKind> = events.iter().map(|event| event.kind.clone()).collect();
        assert_eq!(
            kinds,
            vec![
                EventKind::SessionCreated,
                EventKind::UserInputReceived,
                EventKind::TurnStarted,
                EventKind::ContextPlanCreated,
                EventKind::PromptFrameCreated,
                EventKind::ModelCallStarted,
                EventKind::ModelStreamDelta,
                EventKind::ModelStreamDelta,
                EventKind::ModelCallCompleted,
                EventKind::MessageCommitted,
                EventKind::TurnCompleted,
            ]
        );

        let prompt_event = events
            .iter()
            .find(|event| event.kind == EventKind::PromptFrameCreated)
            .expect("prompt frame event exists");
        match &prompt_event.payload {
            EventPayload::PromptFrameCreated { prompt_frame } => {
                assert_eq!(prompt_frame.id, output.prompt_frame.id);
                assert_eq!(prompt_frame.turn_id, output.turn_id);
            }
            other => panic!("unexpected prompt payload: {other:?}"),
        }

        let model_completed = events
            .iter()
            .find(|event| event.kind == EventKind::ModelCallCompleted)
            .expect("model completion event exists");
        match &model_completed.payload {
            EventPayload::ModelCallCompleted { model_call } => {
                assert_eq!(model_call.status, ygg_core::ModelCallStatus::Completed);
                assert_eq!(model_call.output.as_deref(), Some(output.output.as_str()));
                assert_eq!(model_call.prompt_frame_id, output.prompt_frame.id);
            }
            other => panic!("unexpected model payload: {other:?}"),
        }

        Ok(())
    }
}
