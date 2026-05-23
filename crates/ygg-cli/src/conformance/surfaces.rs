use std::path::PathBuf;

use serde_json::json;
use ygg_runtime::ProtocolContext;

use crate::commands::manifest;

pub(crate) async fn contribution_list() -> anyhow::Result<()> {
    let (_store, runtime) = super::fixtures::runtime();
    runtime
        .load_package(
            manifest::read_manifest(PathBuf::from(
                "examples/packages/echo-rust-inproc/manifest.yaml",
            ))
            .await?,
        )
        .await?;
    runtime
        .load_package(
            manifest::read_manifest(PathBuf::from(
                "examples/packages/thirdparty-surface-fixture/manifest.yaml",
            ))
            .await?,
        )
        .await?;
    let all = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.v1.surface.contribution.list",
            json!({}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(
        all.as_array().map(|items| items.len()).unwrap_or(0) >= 5,
        "surface contributions were not listed"
    );
    let entries = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.v1.surface.contribution.list",
            json!({"slot": "experience_entry"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(
        entries[0]["package_id"] == json!("thirdparty/surface-fixture"),
        "third-party entry surface missing"
    );
    anyhow::ensure!(
        entries[0]["surface"]["activation"]["launch_capability_id"]
            == json!("thirdparty/surface-fixture/start"),
        "entry launch capability missing"
    );
    let described = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.v1.surface.contribution.describe",
            json!({"surface_id": "thirdparty/surface-fixture/assist"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(
        described["surface"]["approval_policy"] == json!("fork_then_approve"),
        "assistant action policy missing"
    );
    Ok(())
}
