use async_trait::async_trait;
use tokio::sync::mpsc;
use ygg_core::PromptFrame;

#[derive(Debug, Clone)]
pub enum ModelStreamEvent {
    Delta(String),
    Completed(String),
}

#[async_trait]
pub trait ModelProvider: Send + Sync + 'static {
    fn provider_name(&self) -> &str;
    fn model_name(&self) -> &str;
    async fn stream(&self, prompt: PromptFrame) -> anyhow::Result<mpsc::Receiver<ModelStreamEvent>>;
}

#[derive(Debug, Clone)]
pub struct MockModelProvider {
    pub provider: String,
    pub model: String,
}

impl Default for MockModelProvider {
    fn default() -> Self {
        Self { provider: "mock".to_string(), model: "mock-roleplay-model".to_string() }
    }
}

#[async_trait]
impl ModelProvider for MockModelProvider {
    fn provider_name(&self) -> &str {
        &self.provider
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    async fn stream(&self, prompt: PromptFrame) -> anyhow::Result<mpsc::Receiver<ModelStreamEvent>> {
        let (tx, rx) = mpsc::channel(16);
        tokio::spawn(async move {
            let last_user = prompt
                .messages
                .iter()
                .rev()
                .find(|message| matches!(message.role, ygg_core::ModelRole::User))
                .map(|message| message.content.clone())
                .unwrap_or_else(|| "...".to_string());
            let output = format!("[mock assistant] I received: {last_user}");
            let midpoint = output.len() / 2;
            let (first, second) = output.split_at(midpoint);
            let _ = tx.send(ModelStreamEvent::Delta(first.to_string())).await;
            let _ = tx.send(ModelStreamEvent::Delta(second.to_string())).await;
            let _ = tx.send(ModelStreamEvent::Completed(output)).await;
        });
        Ok(rx)
    }
}
