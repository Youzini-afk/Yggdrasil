use super::*;

#[derive(Debug, Serialize)]
struct ResolvedSurfaceBundle {
    surface_id: String,
    bundle_url: String,
    export_name: String,
    stylesheets: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    wrapper_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    project_id: Option<String>,
    source: SurfaceBundleSource,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum SurfaceBundleSource {
    InstalledProject,
    DevPath,
}

fn surface_prefix_matches(surface_id: &str, prefix: &str) -> bool {
    surface_id == prefix
        || surface_id
            .strip_prefix(prefix)
            .is_some_and(|rest| rest.starts_with('/'))
}

fn default_surface_export_name(surface_id: &str) -> String {
    if surface_id.starts_with("ydltavern/") {
        match surface_id {
            "ydltavern/play" | "ydltavern/surface" => "mountTavernPlaySurface".to_string(),
            "ydltavern/settings" => "mountTavernSettingsSurface".to_string(),
            "ydltavern/extensions" => "mountTavernExtensionsSurface".to_string(),
            "ydltavern/character" => "mountTavernCharactersSurface".to_string(),
            "ydltavern/world-info" => "mountTavernWorldInfoSurface".to_string(),
            "ydltavern/persona" => "mountTavernPersonaSurface".to_string(),
            "ydltavern/ai-response-config" => "mountTavernAIResponseConfigSurface".to_string(),
            "ydltavern/user-settings" => "mountTavernUserSettingsSurface".to_string(),
            "ydltavern/backgrounds" => "mountTavernBackgroundsSurface".to_string(),
            _ => "mountTavernPlaySurface".to_string(),
        }
    } else {
        "mountSurface".to_string()
    }
}

fn default_surface_stylesheets(prefix: &str) -> Vec<String> {
    if prefix == "ydltavern" {
        vec![
            "/surface-bundles/ydltavern/styles/surface.css".to_string(),
            "/surface-bundles/ydltavern/styles/mobile.css".to_string(),
        ]
    } else {
        Vec::new()
    }
}

impl<S> Runtime<S>
where
    S: EventStore,
{
    // --- Surface ---

    pub(crate) async fn dispatch_surface_resolve_bundle(
        &self,
        context: &ProtocolContext,
        params: &Value,
    ) -> anyhow::Result<Value> {
        if !matches!(
            context.principal,
            ProtocolPrincipal::HostAdmin | ProtocolPrincipal::HostDev
        ) {
            anyhow::bail!(
                "kernel.v1.surface.resolve_bundle permission denied: requires host admin/dev principal"
            );
        }

        let surface_id = params
            .get("surface_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("surface_id required"))?;

        for entry in self.config.project_registry.list() {
            if let Some(bundle) = self.try_resolve_via_project(&entry, surface_id)? {
                return Ok(serde_json::to_value(bundle)?);
            }
        }

        if let Some(bundle) = self.try_resolve_via_dev_path(surface_id)? {
            return Ok(serde_json::to_value(bundle)?);
        }

        anyhow::bail!("surface_not_found: {surface_id}")
    }

    fn try_resolve_via_project(
        &self,
        entry: &crate::ProjectEntry,
        surface_id: &str,
    ) -> anyhow::Result<Option<ResolvedSurfaceBundle>> {
        if entry.descriptor.project.entry_surface_id.as_deref() != Some(surface_id) {
            return Ok(None);
        }

        let project_id = entry.descriptor.project.id.as_str();
        Ok(Some(ResolvedSurfaceBundle {
            surface_id: surface_id.to_string(),
            bundle_url: format!("/surface-bundles/projects/{project_id}/bundle.mjs"),
            export_name: default_surface_export_name(surface_id),
            stylesheets: Vec::new(),
            wrapper_class: None,
            project_id: Some(project_id.to_string()),
            source: SurfaceBundleSource::InstalledProject,
        }))
    }

    fn try_resolve_via_dev_path(
        &self,
        surface_id: &str,
    ) -> anyhow::Result<Option<ResolvedSurfaceBundle>> {
        let Some((prefix, _path)) = self
            .config
            .surface_dev_paths
            .iter()
            .filter(|(prefix, _)| surface_prefix_matches(surface_id, prefix))
            .max_by_key(|(prefix, _)| prefix.len())
        else {
            return Ok(None);
        };

        Ok(Some(ResolvedSurfaceBundle {
            surface_id: surface_id.to_string(),
            bundle_url: format!("/surface-bundles/{prefix}/bundle.mjs"),
            export_name: default_surface_export_name(surface_id),
            stylesheets: default_surface_stylesheets(prefix),
            wrapper_class: Some(format!("{}-surface", prefix.replace(['/', '_'], "-"))),
            project_id: None,
            source: SurfaceBundleSource::DevPath,
        }))
    }

    pub(crate) async fn dispatch_surface_list(&self, params: &Value) -> anyhow::Result<Value> {
        let slot = params
            .get("slot")
            .and_then(Value::as_str)
            .map(str::to_string);
        Ok(self.list_surface_contributions(slot).await)
    }

    pub(crate) async fn dispatch_surface_describe(&self, params: &Value) -> anyhow::Result<Value> {
        let surface_id = params
            .get("surface_id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                anyhow::anyhow!("kernel.v1.surface.contribution.describe requires surface_id")
            })?;
        self.describe_surface_contribution(surface_id).await
    }
}
