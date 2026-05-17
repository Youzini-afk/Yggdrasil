use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use ygg_core::{new_id, AssetRecord, PackageId, KERNEL_PACKAGE_ID, EVENT_ASSET_PUT};

use super::{Runtime, StoredAsset};
use crate::{EventStore, redaction};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPutRequest {
    #[serde(default)]
    pub origin_package_id: Option<PackageId>,
    pub mime: String,
    pub content: String,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetGetResponse {
    pub record: AssetRecord,
    pub content: String,
}

impl<S> Runtime<S>
where
    S: EventStore,
{
    pub async fn put_asset(&self, mut request: AssetPutRequest) -> anyhow::Result<AssetRecord> {
        // Scan asset metadata for raw secrets (content is arbitrary user data — excluded)
        let metadata_scan = redaction::scan_value_for_raw_secrets(&request.metadata, "metadata");
        if metadata_scan.has_findings() {
            let findings: Vec<String> = metadata_scan.findings.iter()
                .map(|f| format!("{} ({:?})", f.path, f.detection))
                .collect();
            anyhow::bail!(
                "asset metadata contains raw secret(s) in field(s): {}; use secret_ref references instead",
                findings.join(", ")
            );
        }

        let origin_package_id = request.origin_package_id.take().unwrap_or_else(|| KERNEL_PACKAGE_ID.to_string());
        let mut hasher = DefaultHasher::new();
        request.content.hash(&mut hasher);
        let record = AssetRecord {
            id: new_id("ast"),
            origin_package_id,
            mime: request.mime,
            hash: format!("{:016x}", hasher.finish()),
            size_bytes: request.content.len() as u64,
            created_at: Utc::now(),
            metadata: request.metadata,
        };
        self.assets.write().await.insert(record.id.clone(), StoredAsset { record: record.clone(), content: request.content.clone() });
        self.append_kernel_event_with_metadata(
            &format!("kernel_asset_{}", record.id),
            EVENT_ASSET_PUT,
            serde_json::to_value(&record)?,
            json!({"content": request.content}),
        )
        .await?;
        Ok(record)
    }

    pub async fn get_asset(&self, asset_id: &str) -> anyhow::Result<AssetGetResponse> {
        self.assets
            .read()
            .await
            .get(asset_id)
            .cloned()
            .map(|stored| AssetGetResponse { record: stored.record, content: stored.content })
            .ok_or_else(|| anyhow::anyhow!("asset '{asset_id}' not found"))
    }

    pub async fn list_assets(&self) -> Vec<AssetRecord> {
        let mut assets: Vec<_> = self.assets.read().await.values().map(|stored| stored.record.clone()).collect();
        assets.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        assets
    }
}
