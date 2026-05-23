use std::collections::BTreeMap;

use serde_json::json;
use ygg_core::project::{ExternalProjectData, ProjectDescriptor, ProjectId, ProjectInner, ProjectType, SecretPolicy};
use ygg_runtime::{EventStore, ProtocolContext};

use super::fixtures::*;

fn descriptor(id: &str) -> ProjectDescriptor {
    ProjectDescriptor {
        schema_version: 1,
        project: ProjectInner {
            id: ProjectId::new(id).expect("valid id"),
            title: id.to_string(),
            description: "protocol project test".to_string(),
            project_type: ProjectType::ExternalWorkspace,
            icon: None,
            entry_surface_id: Some("official/workspace-lab/workspace_view".to_string()),
            packages: vec![],
            optional_packages: vec![],
            required_surfaces: vec![],
            required_capabilities: vec![],
            secret_policy: SecretPolicy::default(),
            external: Some(ExternalProjectData {
                source: "/tmp/protocol-project".to_string(),
                source_ref: None,
                adapter_manifest: None,
                workspace_root: Some("/tmp/protocol-project".to_string()),
            }),
            metadata: BTreeMap::new(),
        },
    }
}

pub(crate) async fn project_list_returns_registered_projects() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.config().project_registry.register(descriptor("proto-list-one__abc12345"))?;
    runtime.config().project_registry.register(descriptor("proto-list-two__def67890"))?;
    let value = runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.v1.project.list", json!({}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let projects = value["projects"].as_array().ok_or_else(|| anyhow::anyhow!("projects missing"))?;
    anyhow::ensure!(projects.iter().any(|p| p["id"] == json!("proto-list-one__abc12345")));
    anyhow::ensure!(projects.iter().any(|p| p["id"] == json!("proto-list-two__def67890")));
    Ok(())
}

pub(crate) async fn project_get_returns_full_descriptor() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.config().project_registry.register(descriptor("proto-get__abc12345"))?;
    let value = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.v1.project.get",
            json!({"project_id":"proto-get__abc12345"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(value["project"]["id"] == json!("proto-get__abc12345"));
    anyhow::ensure!(value["state"] == json!("installed"));
    anyhow::ensure!(value["paths"].is_object());
    Ok(())
}

pub(crate) async fn project_start_transitions_state() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    runtime.config().project_registry.register(descriptor("proto-start__abc12345"))?;
    let value = runtime
        .call_protocol(
            &ProtocolContext::host_dev("conformance"),
            "kernel.v1.project.start",
            json!({"project_id":"proto-start__abc12345"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    anyhow::ensure!(value["previous_state"] == json!("installed"));
    anyhow::ensure!(value["new_state"] == json!("running"));
    let list = runtime
        .call_protocol(&ProtocolContext::host_dev("conformance"), "kernel.v1.project.list", json!({}))
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let project = list["projects"].as_array().unwrap().iter().find(|p| p["id"] == json!("proto-start__abc12345")).unwrap();
    anyhow::ensure!(project["state"] == json!("running"));
    Ok(())
}

pub(crate) async fn project_methods_require_admin_principal() -> anyhow::Result<()> {
    let (_store, runtime) = runtime();
    let err = runtime
        .call_protocol(
            &ProtocolContext::package("example/not-admin", "conformance"),
            "kernel.v1.project.list",
            json!({}),
        )
        .await
        .expect_err("package principal should be denied");
    anyhow::ensure!(err.code == "kernel/v1/error/permission_denied");
    Ok(())
}

pub(crate) async fn project_lifecycle_event_emitted_on_start() -> anyhow::Result<()> {
    let (store, runtime) = runtime();
    runtime.config().project_registry.register(descriptor("proto-event__abc12345"))?;
    let mut context = ProtocolContext::host_dev("conformance");
    context.session_id = Some("project-events".to_string());
    runtime
        .call_protocol(
            &context,
            "kernel.v1.project.start",
            json!({"project_id":"proto-event__abc12345"}),
        )
        .await
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let events = store.list_session(&"project-events".to_string()).await?;
    anyhow::ensure!(events.iter().any(|event| event.kind == ygg_core::PROJECT_STARTED && event.payload["project_id"] == json!("proto-event__abc12345")));
    Ok(())
}
