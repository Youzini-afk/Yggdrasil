//! Streaming invocation registry and lifecycle (Phase S3).
//!
//! This module provides the in-memory registry for ongoing streaming
//! capability invocations and the runtime methods for start/append/
//! end/cancel/timeout lifecycle. It is content-free — no model, prompt,
//! agent, or message semantics.

use std::collections::HashMap;

use chrono::Utc;
use serde_json::{json, Value};
use tokio::sync::RwLock;
use ygg_core::{
    new_id, CapabilityId, InvocationId, PackageId, RedactionState, SessionId, StreamFrameEnvelope,
    StreamFrameType, StreamInvocationRecord, StreamInvocationState, EVENT_STREAM_CANCELLED,
    EVENT_STREAM_CHUNK, EVENT_STREAM_ENDED, EVENT_STREAM_ERROR, EVENT_STREAM_PROGRESS,
    EVENT_STREAM_STARTED, EVENT_STREAM_TIMEOUT,
};

use super::Runtime;
use crate::EventStore;

/// The in-memory streaming invocation registry.
///
/// Tracks ongoing and completed streaming invocations. The registry
/// enforces lifecycle rules:
/// - Only `Active` invocations can receive chunk/progress frames.
/// - Cancel marks invocation `Cancelled` and blocks further frames.
/// - Timeout marks invocation `Timeout` and blocks further frames.
/// - Error and End are terminal states.
#[derive(Debug, Default)]
pub struct StreamRegistry {
    invocations: RwLock<HashMap<InvocationId, StreamInvocationRecord>>,
}

impl StreamRegistry {
    /// Start a new streaming invocation.
    ///
    /// Returns the invocation record with state `Active`.
    /// Emits a `kernel/v1/stream.started` event.
    pub async fn start_invocation(
        &self,
        capability_id: CapabilityId,
        provider_package_id: PackageId,
        session_id: SessionId,
        metadata: Value,
    ) -> StreamInvocationRecord {
        let invocation_id = new_id("inv");
        let stream_id = new_id("str");
        let now = Utc::now();
        let record = StreamInvocationRecord {
            invocation_id: invocation_id.clone(),
            stream_id: stream_id.clone(),
            capability_id,
            provider_package_id,
            session_id,
            state: StreamInvocationState::Active,
            frame_count: 0,
            started_at: now,
            ended_at: None,
            metadata,
        };
        self.invocations
            .write()
            .await
            .insert(invocation_id, record.clone());
        record
    }

    /// Get a streaming invocation record by id.
    pub async fn get_invocation(
        &self,
        invocation_id: &InvocationId,
    ) -> Option<StreamInvocationRecord> {
        self.invocations.read().await.get(invocation_id).cloned()
    }

    /// Get a streaming invocation record by stream id.
    pub async fn get_invocation_by_stream_id(
        &self,
        stream_id: &str,
    ) -> Option<StreamInvocationRecord> {
        self.invocations
            .read()
            .await
            .values()
            .find(|record| record.stream_id == stream_id)
            .cloned()
    }

    /// Append a chunk frame to an active invocation.
    ///
    /// Returns an error if the invocation is terminal or not found.
    /// Increments `frame_count` and returns the frame envelope.
    pub async fn append_chunk(
        &self,
        invocation_id: &InvocationId,
        payload: Value,
        redaction_state: RedactionState,
    ) -> anyhow::Result<StreamFrameEnvelope> {
        let mut invocations = self.invocations.write().await;
        let record = invocations
            .get_mut(invocation_id)
            .ok_or_else(|| anyhow::anyhow!("invocation '{}' not found", invocation_id))?;
        if record.is_terminal() {
            anyhow::bail!(
                "invocation '{}' is in terminal state {:?}; cannot append chunk",
                invocation_id,
                record.state
            );
        }
        record.frame_count += 1;
        let frame = StreamFrameEnvelope {
            invocation_id: invocation_id.clone(),
            stream_id: record.stream_id.clone(),
            frame_type: StreamFrameType::Chunk,
            sequence: record.frame_count,
            redaction_state,
            timestamp: Utc::now(),
            payload,
            metadata: json!({}),
        };
        Ok(frame)
    }

    /// Append a progress frame to an active invocation.
    ///
    /// Returns an error if the invocation is terminal or not found.
    /// Increments `frame_count` and returns the frame envelope.
    /// Progress frames carry no data payload.
    pub async fn append_progress(
        &self,
        invocation_id: &InvocationId,
        metadata: Value,
    ) -> anyhow::Result<StreamFrameEnvelope> {
        let mut invocations = self.invocations.write().await;
        let record = invocations
            .get_mut(invocation_id)
            .ok_or_else(|| anyhow::anyhow!("invocation '{}' not found", invocation_id))?;
        if record.is_terminal() {
            anyhow::bail!(
                "invocation '{}' is in terminal state {:?}; cannot append progress",
                invocation_id,
                record.state
            );
        }
        record.frame_count += 1;
        let frame = StreamFrameEnvelope {
            invocation_id: invocation_id.clone(),
            stream_id: record.stream_id.clone(),
            frame_type: StreamFrameType::Progress,
            sequence: record.frame_count,
            redaction_state: RedactionState::NotCaptured,
            timestamp: Utc::now(),
            payload: Value::Null,
            metadata,
        };
        Ok(frame)
    }

    /// End an active invocation normally.
    ///
    /// Sets state to `Ended` and returns the terminal frame envelope.
    pub async fn end_invocation(
        &self,
        invocation_id: &InvocationId,
    ) -> anyhow::Result<StreamFrameEnvelope> {
        let mut invocations = self.invocations.write().await;
        let record = invocations
            .get_mut(invocation_id)
            .ok_or_else(|| anyhow::anyhow!("invocation '{}' not found", invocation_id))?;
        if record.is_terminal() {
            anyhow::bail!(
                "invocation '{}' is already in terminal state {:?}",
                invocation_id,
                record.state
            );
        }
        record.frame_count += 1;
        record.state = StreamInvocationState::Ended;
        record.ended_at = Some(Utc::now());
        let frame = StreamFrameEnvelope {
            invocation_id: invocation_id.clone(),
            stream_id: record.stream_id.clone(),
            frame_type: StreamFrameType::End,
            sequence: record.frame_count,
            redaction_state: RedactionState::NotCaptured,
            timestamp: Utc::now(),
            payload: Value::Null,
            metadata: json!({}),
        };
        Ok(frame)
    }

    /// Mark an active invocation as errored.
    ///
    /// Sets state to `Error` and returns the terminal frame envelope.
    pub async fn error_invocation(
        &self,
        invocation_id: &InvocationId,
        error_message: &str,
    ) -> anyhow::Result<StreamFrameEnvelope> {
        let mut invocations = self.invocations.write().await;
        let record = invocations
            .get_mut(invocation_id)
            .ok_or_else(|| anyhow::anyhow!("invocation '{}' not found", invocation_id))?;
        if record.is_terminal() {
            anyhow::bail!(
                "invocation '{}' is already in terminal state {:?}",
                invocation_id,
                record.state
            );
        }
        record.frame_count += 1;
        record.state = StreamInvocationState::Error;
        record.ended_at = Some(Utc::now());
        let frame = StreamFrameEnvelope {
            invocation_id: invocation_id.clone(),
            stream_id: record.stream_id.clone(),
            frame_type: StreamFrameType::Error,
            sequence: record.frame_count,
            redaction_state: RedactionState::NotCaptured,
            timestamp: Utc::now(),
            payload: json!({"error": error_message}),
            metadata: json!({}),
        };
        Ok(frame)
    }

    /// Cancel an active invocation.
    ///
    /// Sets state to `Cancelled` and blocks further frames.
    /// Returns the terminal frame envelope.
    pub async fn cancel_invocation(
        &self,
        invocation_id: &InvocationId,
    ) -> anyhow::Result<StreamFrameEnvelope> {
        let mut invocations = self.invocations.write().await;
        let record = invocations
            .get_mut(invocation_id)
            .ok_or_else(|| anyhow::anyhow!("invocation '{}' not found", invocation_id))?;
        if record.is_terminal() {
            anyhow::bail!(
                "invocation '{}' is already in terminal state {:?}",
                invocation_id,
                record.state
            );
        }
        record.frame_count += 1;
        record.state = StreamInvocationState::Cancelled;
        record.ended_at = Some(Utc::now());
        let frame = StreamFrameEnvelope {
            invocation_id: invocation_id.clone(),
            stream_id: record.stream_id.clone(),
            frame_type: StreamFrameType::Cancelled,
            sequence: record.frame_count,
            redaction_state: RedactionState::NotCaptured,
            timestamp: Utc::now(),
            payload: Value::Null,
            metadata: json!({}),
        };
        Ok(frame)
    }

    /// Mark an active invocation as timed out.
    ///
    /// Sets state to `Timeout` and blocks further frames.
    /// Returns the terminal frame envelope.
    pub async fn timeout_invocation(
        &self,
        invocation_id: &InvocationId,
    ) -> anyhow::Result<StreamFrameEnvelope> {
        let mut invocations = self.invocations.write().await;
        let record = invocations
            .get_mut(invocation_id)
            .ok_or_else(|| anyhow::anyhow!("invocation '{}' not found", invocation_id))?;
        if record.is_terminal() {
            anyhow::bail!(
                "invocation '{}' is already in terminal state {:?}",
                invocation_id,
                record.state
            );
        }
        record.frame_count += 1;
        record.state = StreamInvocationState::Timeout;
        record.ended_at = Some(Utc::now());
        let frame = StreamFrameEnvelope {
            invocation_id: invocation_id.clone(),
            stream_id: record.stream_id.clone(),
            frame_type: StreamFrameType::Timeout,
            sequence: record.frame_count,
            redaction_state: RedactionState::NotCaptured,
            timestamp: Utc::now(),
            payload: Value::Null,
            metadata: json!({}),
        };
        Ok(frame)
    }

    /// List all invocation records.
    pub async fn list_invocations(&self) -> Vec<StreamInvocationRecord> {
        self.invocations.read().await.values().cloned().collect()
    }
}

// ---------------------------------------------------------------------------
// Runtime methods for streaming lifecycle
// ---------------------------------------------------------------------------

impl<S> Runtime<S>
where
    S: EventStore,
{
    /// Access the streaming invocation registry.
    pub fn stream_registry(&self) -> &StreamRegistry {
        &self.streams
    }

    /// Start a streaming capability invocation.
    ///
    /// Validates that the capability has `streaming: true` in its descriptor,
    /// creates a registry record, emits `kernel/v1/stream.started`, and
    /// returns the start frame plus the invocation record.
    pub async fn stream_capability_start(
        &self,
        session_id: &SessionId,
        capability_id: &CapabilityId,
        provider_package_id: Option<&str>,
        version: Option<&str>,
        metadata: Value,
    ) -> anyhow::Result<(StreamFrameEnvelope, StreamInvocationRecord)> {
        let provider = self
            .capabilities
            .resolve(
                capability_id,
                provider_package_id.map(|s| s.to_string()).as_ref(),
                version,
            )
            .await?;

        if !provider.descriptor.streaming {
            anyhow::bail!(
                "capability '{}' is not a streaming capability (descriptor streaming=false)",
                capability_id
            );
        }

        let record = self
            .streams
            .start_invocation(
                capability_id.clone(),
                provider.provider_package_id.clone(),
                session_id.clone(),
                metadata,
            )
            .await;

        // Emit kernel/v1/stream.started event
        let event_payload = json!({
            "invocation_id": record.invocation_id,
            "stream_id": record.stream_id,
            "capability_id": capability_id,
            "provider_package_id": provider.provider_package_id,
            "session_id": session_id,
        });
        self.append_kernel_event(session_id, EVENT_STREAM_STARTED, event_payload)
            .await?;

        // Build the start frame
        let frame = StreamFrameEnvelope {
            invocation_id: record.invocation_id.clone(),
            stream_id: record.stream_id.clone(),
            frame_type: StreamFrameType::Start,
            sequence: 0,
            redaction_state: RedactionState::NotCaptured,
            timestamp: Utc::now(),
            payload: json!({}),
            metadata: json!({}),
        };

        Ok((frame, record))
    }

    /// Append a chunk frame to an active streaming invocation.
    ///
    /// Emits `kernel/v1/stream.chunk` and returns the frame envelope.
    pub async fn stream_capability_chunk(
        &self,
        session_id: &SessionId,
        invocation_id: &InvocationId,
        payload: Value,
        redaction_state: RedactionState,
    ) -> anyhow::Result<StreamFrameEnvelope> {
        let frame = self
            .streams
            .append_chunk(invocation_id, payload.clone(), redaction_state)
            .await?;

        let event_payload = json!({
            "invocation_id": invocation_id,
            "stream_id": frame.stream_id,
            "sequence": frame.sequence,
            "redaction_state": serde_json::to_value(redaction_state)?,
            "data": payload,
        });
        self.append_kernel_event(session_id, EVENT_STREAM_CHUNK, event_payload)
            .await?;

        Ok(frame)
    }

    /// Append a progress frame to an active streaming invocation.
    ///
    /// Emits `kernel/v1/stream.progress` and returns the frame envelope.
    pub async fn stream_capability_progress(
        &self,
        session_id: &SessionId,
        invocation_id: &InvocationId,
        metadata: Value,
    ) -> anyhow::Result<StreamFrameEnvelope> {
        let frame = self
            .streams
            .append_progress(invocation_id, metadata.clone())
            .await?;

        let event_payload = json!({
            "invocation_id": invocation_id,
            "stream_id": frame.stream_id,
            "sequence": frame.sequence,
        });
        self.append_kernel_event(session_id, EVENT_STREAM_PROGRESS, event_payload)
            .await?;

        Ok(frame)
    }

    /// End a streaming invocation normally.
    ///
    /// Emits `kernel/v1/stream.ended` and returns the terminal frame.
    pub async fn stream_capability_end(
        &self,
        session_id: &SessionId,
        invocation_id: &InvocationId,
    ) -> anyhow::Result<StreamFrameEnvelope> {
        let frame = self.streams.end_invocation(invocation_id).await?;

        let event_payload = json!({
            "invocation_id": invocation_id,
            "stream_id": frame.stream_id,
            "sequence": frame.sequence,
            "frame_count": frame.sequence,
        });
        self.append_kernel_event(session_id, EVENT_STREAM_ENDED, event_payload)
            .await?;

        Ok(frame)
    }

    /// Error-terminate a streaming invocation.
    ///
    /// Emits `kernel/v1/stream.error` and returns the terminal frame.
    pub async fn stream_capability_error(
        &self,
        session_id: &SessionId,
        invocation_id: &InvocationId,
        error_message: &str,
    ) -> anyhow::Result<StreamFrameEnvelope> {
        let frame = self
            .streams
            .error_invocation(invocation_id, error_message)
            .await?;

        let event_payload = json!({
            "invocation_id": invocation_id,
            "stream_id": frame.stream_id,
            "sequence": frame.sequence,
            "error": error_message,
        });
        self.append_kernel_event(session_id, EVENT_STREAM_ERROR, event_payload)
            .await?;

        Ok(frame)
    }

    /// Cancel a streaming invocation.
    ///
    /// Emits `kernel/v1/stream.cancelled` and returns the terminal frame.
    pub async fn stream_capability_cancel(
        &self,
        session_id: &SessionId,
        invocation_id: &InvocationId,
    ) -> anyhow::Result<StreamFrameEnvelope> {
        let frame = self.streams.cancel_invocation(invocation_id).await?;

        let event_payload = json!({
            "invocation_id": invocation_id,
            "stream_id": frame.stream_id,
            "sequence": frame.sequence,
        });
        self.append_kernel_event(session_id, EVENT_STREAM_CANCELLED, event_payload)
            .await?;

        Ok(frame)
    }

    /// Timeout a streaming invocation.
    ///
    /// Emits `kernel/v1/stream.timeout` and returns the terminal frame.
    pub async fn stream_capability_timeout(
        &self,
        session_id: &SessionId,
        invocation_id: &InvocationId,
    ) -> anyhow::Result<StreamFrameEnvelope> {
        let frame = self.streams.timeout_invocation(invocation_id).await?;

        let event_payload = json!({
            "invocation_id": invocation_id,
            "stream_id": frame.stream_id,
            "sequence": frame.sequence,
        });
        self.append_kernel_event(session_id, EVENT_STREAM_TIMEOUT, event_payload)
            .await?;

        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_registry() -> StreamRegistry {
        StreamRegistry::default()
    }

    #[tokio::test]
    async fn start_invocation_creates_active_record() {
        let registry = test_registry();
        let record = registry
            .start_invocation(
                "example/cap".to_string(),
                "example/pkg".to_string(),
                "session_1".to_string(),
                json!({}),
            )
            .await;
        assert_eq!(record.state, StreamInvocationState::Active);
        assert_eq!(record.frame_count, 0);
        assert!(record.invocation_id.starts_with("inv_"));
        assert!(record.stream_id.starts_with("str_"));
    }

    #[tokio::test]
    async fn normal_lifecycle_emits_ordered_frames() {
        let registry = test_registry();
        let record = registry
            .start_invocation(
                "example/cap".to_string(),
                "example/pkg".to_string(),
                "session_1".to_string(),
                json!({}),
            )
            .await;

        let chunk1 = registry
            .append_chunk(
                &record.invocation_id,
                json!({"n": 1}),
                RedactionState::NotCaptured,
            )
            .await
            .unwrap();
        assert_eq!(chunk1.frame_type, StreamFrameType::Chunk);
        assert_eq!(chunk1.sequence, 1);

        let chunk2 = registry
            .append_chunk(
                &record.invocation_id,
                json!({"n": 2}),
                RedactionState::NotCaptured,
            )
            .await
            .unwrap();
        assert_eq!(chunk2.sequence, 2);

        let end = registry
            .end_invocation(&record.invocation_id)
            .await
            .unwrap();
        assert_eq!(end.frame_type, StreamFrameType::End);
        assert_eq!(end.sequence, 3);

        // After end, no more chunks
        let result = registry
            .append_chunk(
                &record.invocation_id,
                json!({}),
                RedactionState::NotCaptured,
            )
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn cancel_blocks_further_chunks() {
        let registry = test_registry();
        let record = registry
            .start_invocation(
                "example/cap".to_string(),
                "example/pkg".to_string(),
                "session_1".to_string(),
                json!({}),
            )
            .await;

        let cancel = registry
            .cancel_invocation(&record.invocation_id)
            .await
            .unwrap();
        assert_eq!(cancel.frame_type, StreamFrameType::Cancelled);

        let result = registry
            .append_chunk(
                &record.invocation_id,
                json!({}),
                RedactionState::NotCaptured,
            )
            .await;
        assert!(result.is_err());

        let updated = registry
            .get_invocation(&record.invocation_id)
            .await
            .unwrap();
        assert_eq!(updated.state, StreamInvocationState::Cancelled);
    }

    #[tokio::test]
    async fn timeout_blocks_further_chunks() {
        let registry = test_registry();
        let record = registry
            .start_invocation(
                "example/cap".to_string(),
                "example/pkg".to_string(),
                "session_1".to_string(),
                json!({}),
            )
            .await;

        let timeout = registry
            .timeout_invocation(&record.invocation_id)
            .await
            .unwrap();
        assert_eq!(timeout.frame_type, StreamFrameType::Timeout);

        let result = registry
            .append_chunk(
                &record.invocation_id,
                json!({}),
                RedactionState::NotCaptured,
            )
            .await;
        assert!(result.is_err());

        let updated = registry
            .get_invocation(&record.invocation_id)
            .await
            .unwrap();
        assert_eq!(updated.state, StreamInvocationState::Timeout);
    }

    #[tokio::test]
    async fn error_terminal_frame_works() {
        let registry = test_registry();
        let record = registry
            .start_invocation(
                "example/cap".to_string(),
                "example/pkg".to_string(),
                "session_1".to_string(),
                json!({}),
            )
            .await;

        let error = registry
            .error_invocation(&record.invocation_id, "something broke")
            .await
            .unwrap();
        assert_eq!(error.frame_type, StreamFrameType::Error);
        assert_eq!(error.payload["error"], "something broke");

        let result = registry
            .append_chunk(
                &record.invocation_id,
                json!({}),
                RedactionState::NotCaptured,
            )
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn double_end_is_rejected() {
        let registry = test_registry();
        let record = registry
            .start_invocation(
                "example/cap".to_string(),
                "example/pkg".to_string(),
                "session_1".to_string(),
                json!({}),
            )
            .await;

        let _ = registry
            .end_invocation(&record.invocation_id)
            .await
            .unwrap();
        let result = registry.end_invocation(&record.invocation_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn progress_frame_has_no_payload() {
        let registry = test_registry();
        let record = registry
            .start_invocation(
                "example/cap".to_string(),
                "example/pkg".to_string(),
                "session_1".to_string(),
                json!({}),
            )
            .await;

        let progress = registry
            .append_progress(&record.invocation_id, json!({"percent": 50}))
            .await
            .unwrap();
        assert_eq!(progress.frame_type, StreamFrameType::Progress);
        assert!(progress.payload.is_null());
        assert_eq!(progress.metadata["percent"], 50);
    }

    #[tokio::test]
    async fn unknown_invocation_returns_not_found() {
        let registry = test_registry();
        let nonexistent = "nonexistent".to_string();
        let result = registry
            .append_chunk(&nonexistent, json!({}), RedactionState::NotCaptured)
            .await;
        assert!(result.is_err());
    }
}
