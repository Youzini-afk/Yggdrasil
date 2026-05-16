use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use ygg_core::{EventEnvelope, SessionId};
use ygg_runtime::{EventStore, InMemoryEventStore, MockModelProvider, Runtime, RuntimeConfig};

pub type AppRuntime = Runtime<InMemoryEventStore, MockModelProvider>;

#[derive(Clone)]
pub struct AppState {
    pub runtime: Arc<AppRuntime>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SessionInputRequest {
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct SessionInputResponse {
    pub session_id: String,
    pub turn_id: String,
    pub output: String,
    pub prompt_frame_id: String,
}

pub fn app() -> Router {
    let store = Arc::new(InMemoryEventStore::default());
    let model = Arc::new(MockModelProvider::default());
    let runtime = Arc::new(Runtime::new(store, model, RuntimeConfig::default()));
    app_with_state(AppState { runtime })
}

pub fn app_with_state(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/sessions", post(create_session))
        .route("/sessions/:session_id/input", post(session_input))
        .route("/sessions/:session_id/events", get(list_events))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

async fn create_session(
    State(state): State<AppState>,
    Json(request): Json<CreateSessionRequest>,
) -> anyhow::Result<Json<ygg_core::Session>, ServiceError> {
    Ok(Json(state.runtime.create_session(request.title).await?))
}

async fn session_input(
    State(state): State<AppState>,
    Path(session_id): Path<SessionId>,
    Json(request): Json<SessionInputRequest>,
) -> anyhow::Result<Json<SessionInputResponse>, ServiceError> {
    let output = state.runtime.input(session_id, request.content).await?;
    Ok(Json(SessionInputResponse {
        session_id: output.session_id,
        turn_id: output.turn_id,
        prompt_frame_id: output.prompt_frame.id,
        output: output.output,
    }))
}

async fn list_events(
    State(state): State<AppState>,
    Path(session_id): Path<SessionId>,
) -> anyhow::Result<Json<Vec<EventEnvelope>>, ServiceError> {
    Ok(Json(state.runtime.store().list_session(&session_id).await?))
}

pub struct ServiceError(anyhow::Error);

impl<E> From<E> for ServiceError
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Self(value.into())
    }
}

impl axum::response::IntoResponse for ServiceError {
    fn into_response(self) -> axum::response::Response {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}
