use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;

#[async_trait]
pub trait KernelTransport: Send + Sync {
    async fn invoke(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value>;

    fn invoke_stream(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Box<dyn Stream<Item = Result<serde_json::Value>> + Unpin + Send>;
}

pub struct KernelClient {
    pub transport: Box<dyn KernelTransport>,
}

impl KernelClient {
    pub fn new(transport: Box<dyn KernelTransport>) -> Self {
        Self { transport }
    }
}
