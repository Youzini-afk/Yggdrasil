use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ygg_core::{CapabilityId, PackageId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InprocInvocation {
    pub capability_id: CapabilityId,
    pub provider_package_id: PackageId,
    #[serde(default)]
    pub input: Value,
}

#[async_trait]
pub trait InprocPackage: Send + Sync {
    async fn invoke(&self, request: InprocInvocation) -> anyhow::Result<Value>;
}

#[derive(Clone, Default)]
pub struct InprocPackageCatalog {
    entries: Arc<HashMap<String, Arc<dyn InprocPackage>>>,
}

impl InprocPackageCatalog {
    pub fn with_default_examples() -> Self {
        let mut entries: HashMap<String, Arc<dyn InprocPackage>> = HashMap::new();
        entries.insert(entry_key("example-echo-rust-inproc", "register"), Arc::new(EchoInprocPackage));
        Self { entries: Arc::new(entries) }
    }

    pub fn lookup(&self, crate_ref: &str, symbol: &str) -> Option<Arc<dyn InprocPackage>> {
        self.entries.get(&entry_key(crate_ref, symbol)).cloned()
    }
}

fn entry_key(crate_ref: &str, symbol: &str) -> String {
    format!("{crate_ref}::{symbol}")
}

struct EchoInprocPackage;

#[async_trait]
impl InprocPackage for EchoInprocPackage {
    async fn invoke(&self, request: InprocInvocation) -> anyhow::Result<Value> {
        Ok(request.input)
    }
}
