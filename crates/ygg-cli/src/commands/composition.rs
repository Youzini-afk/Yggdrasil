use std::fs;
use std::path::PathBuf;

use anyhow::Result;

use super::manifest::read_manifest;
use super::package::init_package;
use crate::cli::CompositionDescriptor;

pub(crate) async fn init_composition(path: PathBuf, id: String) -> Result<()> {
    fs::create_dir_all(&path)?;
    let package_path = path.join("package");
    init_package(
        package_path.clone(),
        id.clone(),
        "subprocess".to_string(),
        "python-experience".to_string(),
        None,
    )
    .await?;
    fs::write(
        path.join("composition.yaml"),
        format!(
            r#"id: {id}
version: 0.1.0
entry_surface_id: {id}/entry
# title: "{id} composition"
# description: "A composition descriptor for {id}"
packages:
  - package/manifest.yaml
required_surfaces:
  - experience_entry
  - play_renderer
  - forge_panel
# optional_packages:
#   - ../extras/optional-package/manifest.yaml
# required_capabilities:
#   - official/composition-lab/launch_plan
# default_activation:
#   auto_launch: true
# permission_expectations:
#   - capabilities.invoke
# replacement_candidates:
#   - official/experience-alt
# compatibility_notes:
#   - "Requires kernel v0.1.0 or later"
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

    // Collect surface and capability info from required packages
    let mut surface_ids = Vec::new();
    let mut slots = Vec::new();
    let mut capability_ids = Vec::new();
    let mut warnings = Vec::new();
    let mut loaded_packages = Vec::new();

    for package_path in &composition.packages {
        let resolved = if package_path.is_absolute() { package_path.clone() } else { base.join(package_path) };
        match read_manifest(resolved).await {
            Ok(manifest) => {
                manifest.validate_basic()?;
                for surface in manifest.contributes.surfaces {
                    let slot = serde_json::to_value(&surface.slot)?.as_str().unwrap_or_default().to_string();
                    surface_ids.push(surface.id);
                    slots.push(slot);
                }
                for cap in &manifest.provides {
                    capability_ids.push(cap.id.clone());
                }
                loaded_packages.push(package_path.display().to_string());
            }
            Err(e) => {
                anyhow::bail!("required package '{}' failed to load: {e}", package_path.display());
            }
        }
    }

    // Collect info from optional packages (warn only on missing)
    let mut optional_loaded = Vec::new();
    for package_path in &composition.optional_packages {
        let resolved = if package_path.is_absolute() { package_path.clone() } else { base.join(package_path) };
        match read_manifest(resolved).await {
            Ok(manifest) => {
                manifest.validate_basic()?;
                for surface in manifest.contributes.surfaces {
                    let slot = serde_json::to_value(&surface.slot)?.as_str().unwrap_or_default().to_string();
                    surface_ids.push(surface.id);
                    slots.push(slot);
                }
                for cap in &manifest.provides {
                    capability_ids.push(cap.id.clone());
                }
                optional_loaded.push(package_path.display().to_string());
            }
            Err(e) => {
                warnings.push(format!("optional package '{}' not loaded: {e}", package_path.display()));
            }
        }
    }

    // Check entry surface
    let entry_present = surface_ids.iter().any(|id| id == &composition.entry_surface_id);
    if !entry_present {
        anyhow::bail!("entry surface '{}' not provided by composition packages", composition.entry_surface_id);
    }

    // Check required surfaces
    let mut missing_surfaces = Vec::new();
    for required in &composition.required_surfaces {
        if !slots.iter().any(|slot| slot == required) {
            missing_surfaces.push(required.clone());
        }
    }
    if !missing_surfaces.is_empty() {
        anyhow::bail!("required surface slots missing: {}", missing_surfaces.join(", "));
    }

    // Check required capabilities
    let mut missing_capabilities = Vec::new();
    for required_cap in &composition.required_capabilities {
        if !capability_ids.iter().any(|cap| cap == required_cap) {
            missing_capabilities.push(required_cap.clone());
        }
    }
    if !missing_capabilities.is_empty() {
        anyhow::bail!("required capabilities missing: {}", missing_capabilities.join(", "));
    }

    // Print structured diagnostics
    println!("composition: {}@{}", composition.id, composition.version);
    if let Some(ref title) = composition.title {
        println!("  title: {title}");
    }
    if let Some(ref desc) = composition.description {
        println!("  description: {desc}");
    }
    println!("  entry_surface_id: {}", composition.entry_surface_id);

    println!("  required packages ({}):", loaded_packages.len());
    for pkg in &loaded_packages {
        println!("    - {pkg}");
    }

    println!("  optional packages ({} loaded, {} total):", optional_loaded.len(), composition.optional_packages.len());
    for pkg in &optional_loaded {
        println!("    - {pkg}");
    }

    println!("  surfaces by slot ({}):", slots.len());
    let mut slot_counts: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for slot in &slots {
        *slot_counts.entry(slot.as_str()).or_insert(0) += 1;
    }
    for (slot, count) in &slot_counts {
        println!("    {slot}: {count}");
    }

    println!("  capabilities ({}):", capability_ids.len());
    for cap in &capability_ids {
        println!("    - {cap}");
    }

    if !composition.required_capabilities.is_empty() {
        println!("  required capabilities: {} (all present)", composition.required_capabilities.len());
    }

    println!("  entry activation: {}", if composition.default_activation.is_some() { "present" } else { "missing" });

    if !composition.permission_expectations.is_empty() {
        println!("  permission expectations:");
        for perm in &composition.permission_expectations {
            println!("    - {perm}");
        }
    }

    if !composition.replacement_candidates.is_empty() {
        println!("  replacement candidates:");
        for cand in &composition.replacement_candidates {
            println!("    - {cand}");
        }
    }

    if !composition.compatibility_notes.is_empty() {
        println!("  compatibility notes:");
        for note in &composition.compatibility_notes {
            println!("    - {note}");
        }
    }

    for warning in &warnings {
        println!("  WARNING: {warning}");
    }

    println!("composition check: {}@{} ok", composition.id, composition.version);
    Ok(())
}
