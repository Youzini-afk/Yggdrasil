use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use schemars::JsonSchema;

use crate::ids::{AssetId, PackageId};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AssetRecord {
    pub id: AssetId,
    pub origin_package_id: PackageId,
    pub mime: String,
    pub hash: String,
    pub size_bytes: u64,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub metadata: Value,
}
