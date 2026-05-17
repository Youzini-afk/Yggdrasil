use std::fs;

use crate::cli::PackageTemplate;
use crate::commands::{composition, manifest, package};

pub(crate) async fn generated_subprocess_package() -> anyhow::Result<()> {
    let path = std::env::temp_dir().join(format!("ygg-generated-package-{}", std::process::id()));
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }
    package::init_package(
        path.clone(),
        "example/generated-subprocess".to_string(),
        "subprocess".to_string(),
        "python".to_string(),
        None,
    )
    .await?;
    package::package_check(path.join("manifest.yaml")).await?;
    package::package_conformance(path.join("manifest.yaml")).await?;
    fs::remove_dir_all(path)?;
    Ok(())
}

pub(crate) async fn generated_typescript_subprocess_package() -> anyhow::Result<()> {
    let path = std::env::temp_dir().join(format!("ygg-generated-ts-package-{}", std::process::id()));
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }
    package::init_package(
        path.clone(),
        "example/generated-typescript-subprocess".to_string(),
        "subprocess".to_string(),
        "typescript".to_string(),
        None,
    )
    .await?;
    package::package_check(path.join("manifest.yaml")).await?;
    package::package_conformance(path.join("manifest.yaml")).await?;
    fs::remove_dir_all(path)?;
    Ok(())
}

pub(crate) async fn generated_experience_template() -> anyhow::Result<()> {
    let path = std::env::temp_dir().join(format!("ygg-generated-experience-{}", std::process::id()));
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }
    package::init_package(
        path.clone(),
        "example/generated-experience".to_string(),
        "subprocess".to_string(),
        "typescript-experience".to_string(),
        None,
    )
    .await?;
    package::package_check(path.join("manifest.yaml")).await?;
    package::package_conformance(path.join("manifest.yaml")).await?;
    let manifest = manifest::read_manifest(path.join("manifest.yaml")).await?;
    // Legacy experience template (auto-detected from --language typescript-experience)
    // preserves the original 4 surfaces for backward compatibility.
    anyhow::ensure!(manifest.contributes.surfaces.len() >= 4, "legacy experience template should have >= 4 surfaces, got {}", manifest.contributes.surfaces.len());
    fs::remove_dir_all(path)?;
    Ok(())
}

/// Test that the basic template generates no surfaces.
pub(crate) async fn generated_basic_template() -> anyhow::Result<()> {
    let path = std::env::temp_dir().join(format!("ygg-generated-basic-{}", std::process::id()));
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }
    package::init_package(
        path.clone(),
        "example/generated-basic".to_string(),
        "subprocess".to_string(),
        "typescript".to_string(),
        Some(PackageTemplate::Basic),
    )
    .await?;
    package::package_check(path.join("manifest.yaml")).await?;
    package::package_conformance(path.join("manifest.yaml")).await?;
    let manifest = manifest::read_manifest(path.join("manifest.yaml")).await?;
    anyhow::ensure!(manifest.contributes.surfaces.is_empty(), "basic template should have 0 surfaces, got {}", manifest.contributes.surfaces.len());
    fs::remove_dir_all(path)?;
    Ok(())
}

/// Test that the explicit --template experience generates only experience_entry.
pub(crate) async fn generated_explicit_experience_template() -> anyhow::Result<()> {
    let path = std::env::temp_dir().join(format!("ygg-generated-explicit-experience-{}", std::process::id()));
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }
    package::init_package(
        path.clone(),
        "example/generated-explicit-experience".to_string(),
        "subprocess".to_string(),
        "typescript".to_string(),
        Some(PackageTemplate::Experience),
    )
    .await?;
    package::package_check(path.join("manifest.yaml")).await?;
    package::package_conformance(path.join("manifest.yaml")).await?;
    let manifest = manifest::read_manifest(path.join("manifest.yaml")).await?;
    anyhow::ensure!(manifest.contributes.surfaces.len() == 1, "explicit experience template should have 1 surface, got {}", manifest.contributes.surfaces.len());
    anyhow::ensure!(
        matches!(manifest.contributes.surfaces[0].slot, ygg_core::SurfaceSlot::ExperienceEntry),
        "explicit experience template surface slot should be experience_entry"
    );
    fs::remove_dir_all(path)?;
    Ok(())
}

/// Test that the assistant-action template generates one surface with fork_then_approve.
pub(crate) async fn generated_assistant_action_template() -> anyhow::Result<()> {
    let path = std::env::temp_dir().join(format!("ygg-generated-assistant-action-{}", std::process::id()));
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }
    package::init_package(
        path.clone(),
        "example/generated-assistant-action".to_string(),
        "subprocess".to_string(),
        "typescript".to_string(),
        Some(PackageTemplate::AssistantAction),
    )
    .await?;
    package::package_check(path.join("manifest.yaml")).await?;
    package::package_conformance(path.join("manifest.yaml")).await?;
    let manifest = manifest::read_manifest(path.join("manifest.yaml")).await?;
    anyhow::ensure!(manifest.contributes.surfaces.len() == 1, "assistant-action template should have 1 surface, got {}", manifest.contributes.surfaces.len());
    anyhow::ensure!(
        matches!(manifest.contributes.surfaces[0].slot, ygg_core::SurfaceSlot::AssistantAction),
        "assistant-action template surface slot should be assistant_action"
    );
    anyhow::ensure!(
        manifest.contributes.surfaces[0].approval_policy == Some(ygg_core::SurfaceApprovalPolicy::ForkThenApprove),
        "assistant-action template should have fork_then_approve policy"
    );
    fs::remove_dir_all(path)?;
    Ok(())
}

/// Test that the asset-editor template generates one surface with the asset_editor slot.
pub(crate) async fn generated_asset_editor_template() -> anyhow::Result<()> {
    let path = std::env::temp_dir().join(format!("ygg-generated-asset-editor-{}", std::process::id()));
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }
    package::init_package(
        path.clone(),
        "example/generated-asset-editor".to_string(),
        "subprocess".to_string(),
        "typescript".to_string(),
        Some(PackageTemplate::AssetEditor),
    )
    .await?;
    package::package_check(path.join("manifest.yaml")).await?;
    package::package_conformance(path.join("manifest.yaml")).await?;
    let manifest = manifest::read_manifest(path.join("manifest.yaml")).await?;
    anyhow::ensure!(manifest.contributes.surfaces.len() == 1, "asset-editor template should have 1 surface, got {}", manifest.contributes.surfaces.len());
    anyhow::ensure!(
        matches!(manifest.contributes.surfaces[0].slot, ygg_core::SurfaceSlot::AssetEditor),
        "asset-editor template surface slot should be asset_editor"
    );
    fs::remove_dir_all(path)?;
    Ok(())
}

/// Test that the full-surface template generates all 5 surfaces.
pub(crate) async fn generated_full_surface_template() -> anyhow::Result<()> {
    let path = std::env::temp_dir().join(format!("ygg-generated-full-surface-{}", std::process::id()));
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }
    package::init_package(
        path.clone(),
        "example/generated-full-surface".to_string(),
        "subprocess".to_string(),
        "typescript".to_string(),
        Some(PackageTemplate::FullSurface),
    )
    .await?;
    package::package_check(path.join("manifest.yaml")).await?;
    package::package_conformance(path.join("manifest.yaml")).await?;
    let manifest = manifest::read_manifest(path.join("manifest.yaml")).await?;
    anyhow::ensure!(manifest.contributes.surfaces.len() == 5, "full-surface template should have 5 surfaces, got {}", manifest.contributes.surfaces.len());
    let slots: Vec<&str> = manifest.contributes.surfaces.iter().map(|s| match s.slot {
        ygg_core::SurfaceSlot::ExperienceEntry => "experience_entry",
        ygg_core::SurfaceSlot::PlayRenderer => "play_renderer",
        ygg_core::SurfaceSlot::ForgePanel => "forge_panel",
        ygg_core::SurfaceSlot::AssistantAction => "assistant_action",
        ygg_core::SurfaceSlot::AssetEditor => "asset_editor",
        ygg_core::SurfaceSlot::HomeCard => "home_card",
    }).collect();
    anyhow::ensure!(slots.contains(&"experience_entry"), "full-surface should include experience_entry");
    anyhow::ensure!(slots.contains(&"play_renderer"), "full-surface should include play_renderer");
    anyhow::ensure!(slots.contains(&"forge_panel"), "full-surface should include forge_panel");
    anyhow::ensure!(slots.contains(&"assistant_action"), "full-surface should include assistant_action");
    anyhow::ensure!(slots.contains(&"asset_editor"), "full-surface should include asset_editor");
    // Verify assistant_action has fork_then_approve
    let assist = manifest.contributes.surfaces.iter().find(|s| matches!(s.slot, ygg_core::SurfaceSlot::AssistantAction)).unwrap();
    anyhow::ensure!(assist.approval_policy == Some(ygg_core::SurfaceApprovalPolicy::ForkThenApprove), "full-surface assistant_action should have fork_then_approve");
    fs::remove_dir_all(path)?;
    Ok(())
}

pub(crate) async fn composition_descriptor() -> anyhow::Result<()> {
    let root = std::env::temp_dir().join(format!("ygg-composition-{}", std::process::id()));
    let package_path = root.join("package");
    let composition_path = root.join("composition");
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    fs::create_dir_all(&root)?;
    package::init_package(
        package_path,
        "example/composed-experience".to_string(),
        "subprocess".to_string(),
        "typescript-experience".to_string(),
        None,
    )
    .await?;
    composition::init_composition(composition_path.clone(), "example/composed-experience".to_string()).await?;
    composition::composition_check(composition_path.join("composition.yaml")).await?;
    fs::remove_dir_all(root)?;
    Ok(())
}
