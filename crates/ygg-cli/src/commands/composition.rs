use std::fs;
use std::path::PathBuf;

use anyhow::Result;

use super::manifest::read_manifest;
use crate::cli::CompositionDescriptor;

pub(crate) async fn init_composition(path: PathBuf, id: String) -> Result<()> {
    fs::create_dir_all(&path)?;
    fs::write(
        path.join("composition.yaml"),
        format!(
            r#"id: {id}
version: 0.1.0
entry_surface_id: {id}/entry
packages:
  - ../package/manifest.yaml
required_surfaces:
  - experience_entry
  - play_renderer
  - forge_panel
"#
        ),
    )?;
    println!("initialized composition descriptor at {}", path.join("composition.yaml").display());
    Ok(())
}

pub(crate) async fn composition_check(path: PathBuf) -> Result<()> {
    let raw = fs::read_to_string(&path)?;
    let composition: CompositionDescriptor = match path.extension().and_then(|ext| ext.to_str()) {
        Some("yaml") | Some("yml") => serde_yaml::from_str(&raw)?,
        _ => serde_json::from_str(&raw)?,
    };
    anyhow::ensure!(!composition.id.trim().is_empty(), "composition id is required");
    anyhow::ensure!(!composition.version.trim().is_empty(), "composition version is required");
    anyhow::ensure!(!composition.entry_surface_id.trim().is_empty(), "composition entry_surface_id is required");
    let base = path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let mut surface_ids = Vec::new();
    let mut slots = Vec::new();
    for package_path in &composition.packages {
        let resolved = if package_path.is_absolute() { package_path.clone() } else { base.join(package_path) };
        let manifest = read_manifest(resolved).await?;
        manifest.validate_basic()?;
        for surface in manifest.contributes.surfaces {
            let slot = serde_json::to_value(&surface.slot)?.as_str().unwrap_or_default().to_string();
            surface_ids.push(surface.id);
            slots.push(slot);
        }
    }
    anyhow::ensure!(surface_ids.iter().any(|id| id == &composition.entry_surface_id), "entry surface not provided by composition packages");
    for required in &composition.required_surfaces {
        anyhow::ensure!(slots.iter().any(|slot| slot == required), "required surface slot '{required}' missing");
    }
    println!("composition check: {}@{} ok", composition.id, composition.version);
    Ok(())
}
