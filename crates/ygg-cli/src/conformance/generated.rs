use std::fs;

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
    )
    .await?;
    package::package_check(path.join("manifest.yaml")).await?;
    package::package_conformance(path.join("manifest.yaml")).await?;
    let manifest = manifest::read_manifest(path.join("manifest.yaml")).await?;
    anyhow::ensure!(manifest.contributes.surfaces.len() >= 4, "experience template did not generate surface descriptors");
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
    )
    .await?;
    composition::init_composition(composition_path.clone(), "example/composed-experience".to_string()).await?;
    composition::composition_check(composition_path.join("composition.yaml")).await?;
    fs::remove_dir_all(root)?;
    Ok(())
}
