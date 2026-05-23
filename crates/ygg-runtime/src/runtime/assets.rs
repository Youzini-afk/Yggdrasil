use chrono::Utc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use ygg_core::{new_id, AssetRecord, PackageId, EVENT_ASSET_PUT, KERNEL_PACKAGE_ID};

use super::{Runtime, StoredAsset};
use crate::{redaction, EventStore};

// ---------------------------------------------------------------------------
// Stable content-address helper (FNV-1a 64-bit, deterministic across runs)
// ---------------------------------------------------------------------------
//
// DefaultHasher is explicitly NOT used because its output is not guaranteed
// stable across Rust versions or platforms. FNV-1a is a simple, well-known,
// deterministic hash suitable for content-addressed asset metadata.
// Prefix "fnv1a64:" makes the scheme explicit and distinguishes it from
// future stronger schemes (e.g. sha256:) if needed.

const FNV1A_64_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV1A_64_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Compute a deterministic FNV-1a 64-bit hash of the input bytes.
fn fnv1a_64(data: &[u8]) -> u64 {
    let mut hash = FNV1A_64_OFFSET_BASIS;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV1A_64_PRIME);
    }
    hash
}

/// Compute a stable content address for arbitrary string content.
/// Returns `"fnv1a64:<lowercase-hex>"`.
pub fn content_address(content: &str) -> String {
    let hash = fnv1a_64(content.as_bytes());
    format!("fnv1a64:{:016x}", hash)
}

/// Build standard Beta 2 metadata convention fields for an asset record.
///
/// Callers should merge this into their `AssetPutRequest.metadata`.
/// No raw secrets are included; secret fields use `secret_ref:` references.
pub fn standard_asset_metadata(origin_package_id: &str, disclosure: &str) -> Value {
    json!({
        "content_address_scheme": "fnv1a64",
        "provenance": {
            "origin_package_id": origin_package_id,
        },
        "disclosure": disclosure,
        "source_refs": [],
        "derived_refs": [],
        "large_output_policy": "inline",
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AssetPutRequest {
    #[serde(default)]
    pub origin_package_id: Option<PackageId>,
    pub mime: String,
    pub content: String,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
            let findings: Vec<String> = metadata_scan
                .findings
                .iter()
                .map(|f| format!("{} ({:?})", f.path, f.detection))
                .collect();
            anyhow::bail!(
                "asset metadata contains raw secret(s) in field(s): {}; use secret_ref references instead",
                findings.join(", ")
            );
        }

        let origin_package_id = request
            .origin_package_id
            .take()
            .unwrap_or_else(|| KERNEL_PACKAGE_ID.to_string());
        let ca = content_address(&request.content);
        let record = AssetRecord {
            id: new_id("ast"),
            origin_package_id,
            mime: request.mime,
            hash: ca.clone(),
            size_bytes: request.content.len() as u64,
            created_at: Utc::now(),
            metadata: request.metadata,
        };
        self.assets.write().await.insert(
            record.id.clone(),
            StoredAsset {
                record: record.clone(),
                content: request.content.clone(),
            },
        );
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
            .map(|stored| AssetGetResponse {
                record: stored.record,
                content: stored.content,
            })
            .ok_or_else(|| anyhow::anyhow!("asset '{asset_id}' not found"))
    }

    pub async fn list_assets(&self) -> Vec<AssetRecord> {
        let mut assets: Vec<_> = self
            .assets
            .read()
            .await
            .values()
            .map(|stored| stored.record.clone())
            .collect();
        assets.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        assets
    }
}
