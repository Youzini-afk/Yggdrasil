use serde_json::{json, Value};

use super::safety::contains_raw_secret;
use super::InprocInvocation;

pub fn try_handle(request: &InprocInvocation) -> Option<anyhow::Result<Value>> {
    if request.provider_package_id != "official/package-installer-lab" {
        return None;
    }
    let local = request
        .capability_id
        .strip_prefix("official/package-installer-lab/")?;
    Some(match local {
        "describe_install_contract" => describe_install_contract(),
        "plan_install" => plan_install(&request.input),
        "apply_install" => apply_install(&request.input),
        "list_installed" => list_installed(&request.input),
        "uninstall" => uninstall(&request.input),
        "update" => update(&request.input),
        "inspect_lockfile" => inspect_lockfile(&request.input),
        _ => return None,
    })
}

fn describe_install_contract() -> anyhow::Result<Value> {
    Ok(json!({
        "package_id": "official/package-installer-lab",
        "kind": "git_package_installer_contract",
        "stage": "profile_lockfile_loop",
        "transport": {
            "method": "kernel.outbound.git_fetch",
            "first_round": "public_https_git_only",
            "private_repos": "deferred",
            "ssh": "deferred"
        },
        "capabilities": [
            "describe_install_contract",
            "plan_install",
            "apply_install",
            "list_installed",
            "uninstall",
            "update",
            "inspect_lockfile"
        ],
        "red_lines": [
            "no kernel.git namespace",
            "no kernel.package.install namespace",
            "no post-install scripts",
            "approval required before apply",
            "profile-scoped lockfile",
            "apply requires approved proposal with pinned commit/content hash"
        ],
        "lockfile": {
            "scope": "profile",
            "shape": "<profile-name>.lock.yaml",
            "pinned_fields": ["remote_url", "ref", "commit_sha", "content_hash", "manifest_path"]
        }
    }))
}

fn plan_install(input: &Value) -> anyhow::Result<Value> {
    if contains_raw_secret(input) {
        anyhow::bail!("plan_install rejects raw secrets; use secret_ref fields when private repositories are supported")
    }
    let remote_url = input
        .get("remote_url")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("plan_install requires remote_url"))?;
    let reference = input.get("ref").and_then(Value::as_str).unwrap_or("main");
    let manifest_path = input
        .get("manifest_path")
        .and_then(Value::as_str)
        .unwrap_or("manifest.yaml");
    let preferred_package_id = input
        .get("preferred_package_id")
        .and_then(Value::as_str)
        .unwrap_or("unresolved/from-git");

    validate_public_https_git_url(remote_url)?;
    validate_safe_ref(reference)?;
    validate_manifest_path(manifest_path)?;

    Ok(json!({
        "kind": "package_install_proposal_draft",
        "status": "requires_git_fetch_before_apply",
        "requires_user_approval": true,
        "source_ref": "official/package-installer-lab/plan_install",
        "remote_url": remote_url,
        "ref": reference,
        "manifest_path": manifest_path,
        "preferred_package_id": preferred_package_id,
        "operations": [{
            "kind": "package.install_from_git",
            "remote_url": remote_url,
            "ref": reference,
            "manifest_path": manifest_path,
            "preferred_package_id": preferred_package_id,
            "requires_git_fetch": true,
            "requires_user_approval": true
        }],
        "expected_effects": [
            "resolve ref through kernel.outbound.git_fetch",
            "pin commit_sha and content_hash before apply",
            "validate package manifest",
            "write profile-scoped lockfile entry",
            "load package through the ordinary kernel.package.load path"
        ],
        "requested_host_policy": {
            "outbound_git_enabled": true,
            "https_only": true,
            "allowed_host_required": host_from_url(remote_url)?
        },
        "pinned": {
            "commit_sha": null,
            "content_hash": null,
            "manifest_content_hash": null,
            "why_null": "plan_install is proposal-only in this stage; apply resolves and pins before mutation"
        }
    }))
}

fn apply_install(input: &Value) -> anyhow::Result<Value> {
    reject_raw_secret(input)?;
    let approved = input.get("approved").and_then(Value::as_bool).unwrap_or(false);
    if !approved {
        anyhow::bail!("apply_install requires an approved proposal")
    }
    let remote_url = string_field(input, "remote_url")?;
    let package_id = string_field(input, "package_id")?;
    let reference = input.get("ref").and_then(Value::as_str).unwrap_or("main");
    let commit_sha = string_field(input, "commit_sha")?;
    let content_hash = string_field(input, "content_hash")?;
    let manifest_path = input.get("manifest_path").and_then(Value::as_str).unwrap_or("manifest.yaml");
    validate_public_https_git_url(remote_url)?;
    validate_package_id(package_id)?;
    validate_safe_ref(reference)?;
    validate_commit_sha(commit_sha)?;
    validate_content_hash(content_hash)?;
    validate_manifest_path(manifest_path)?;

    Ok(json!({
        "kind": "package_install_apply_plan",
        "status": "ready_for_host_lockfile_write",
        "requires_user_approval": false,
        "lockfile_entry": {
            "package_id": package_id,
            "remote_url": remote_url,
            "ref": reference,
            "commit_sha": commit_sha,
            "content_hash": content_hash,
            "manifest_path": manifest_path,
            "install_root_subdir": install_root_subdir(package_id, commit_sha)
        },
        "host_actions": [
            "write profile-scoped lockfile entry",
            "copy fetched tree into install_root_subdir",
            "load package through kernel.package.load"
        ]
    }))
}

fn list_installed(input: &Value) -> anyhow::Result<Value> {
    reject_raw_secret(input)?;
    let packages = input
        .get("packages")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    Ok(json!({
        "kind": "installed_package_list",
        "package_count": packages.len(),
        "packages": packages
    }))
}

fn uninstall(input: &Value) -> anyhow::Result<Value> {
    reject_raw_secret(input)?;
    let package_id = string_field(input, "package_id")?;
    validate_package_id(package_id)?;
    Ok(json!({
        "kind": "package_uninstall_plan",
        "package_id": package_id,
        "requires_user_approval": true,
        "host_actions": ["kernel.package.unload", "remove lockfile entry", "remove install_root_subdir"]
    }))
}

fn update(input: &Value) -> anyhow::Result<Value> {
    reject_raw_secret(input)?;
    let package_id = string_field(input, "package_id")?;
    validate_package_id(package_id)?;
    let target_ref = input.get("target_ref").and_then(Value::as_str).unwrap_or("main");
    validate_safe_ref(target_ref)?;
    Ok(json!({
        "kind": "package_update_proposal_draft",
        "package_id": package_id,
        "target_ref": target_ref,
        "requires_user_approval": true,
        "operations": [{"kind": "package.update_from_git", "package_id": package_id, "target_ref": target_ref}],
        "expected_effects": ["resolve target ref", "compare old/new pinned commit", "write updated profile lockfile entry"]
    }))
}

fn inspect_lockfile(input: &Value) -> anyhow::Result<Value> {
    reject_raw_secret(input)?;
    let packages = input
        .get("packages")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    Ok(json!({
        "kind": "profile_lockfile_inspection",
        "format_version": input.get("format_version").cloned().unwrap_or(json!(1)),
        "package_count": packages.len(),
        "valid": true,
        "packages": packages
    }))
}

fn validate_public_https_git_url(remote_url: &str) -> anyhow::Result<()> {
    let url = reqwest::Url::parse(remote_url)
        .map_err(|_| anyhow::anyhow!("remote_url must be a valid HTTPS git URL"))?;
    if url.scheme() != "https" {
        anyhow::bail!("remote_url must use HTTPS")
    }
    if url.host_str().is_none() {
        anyhow::bail!("remote_url must include a host")
    }
    if !url.username().is_empty() || url.password().is_some() || url.query().is_some() {
        anyhow::bail!("remote_url must not include credentials or query strings")
    }
    Ok(())
}

fn reject_raw_secret(input: &Value) -> anyhow::Result<()> {
    if contains_raw_secret(input) {
        anyhow::bail!("package-installer-lab rejects raw secrets")
    }
    Ok(())
}

fn string_field<'a>(input: &'a Value, key: &str) -> anyhow::Result<&'a str> {
    input
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("{key} is required"))
}

fn validate_package_id(package_id: &str) -> anyhow::Result<()> {
    if package_id.trim().is_empty() || package_id.contains("..") || package_id.starts_with('/') {
        anyhow::bail!("package_id is not safe")
    }
    Ok(())
}

fn validate_commit_sha(commit_sha: &str) -> anyhow::Result<()> {
    if commit_sha.len() != 40 || !commit_sha.chars().all(|ch| ch.is_ascii_hexdigit()) {
        anyhow::bail!("commit_sha must be a 40-character SHA-1")
    }
    Ok(())
}

fn validate_content_hash(content_hash: &str) -> anyhow::Result<()> {
    if !content_hash.starts_with("sha256:") && !content_hash.starts_with("fnv1a64:") {
        anyhow::bail!("content_hash must start with sha256: or fnv1a64:")
    }
    Ok(())
}

fn install_root_subdir(package_id: &str, commit_sha: &str) -> String {
    format!("{}-{}", package_id.replace('/', "-"), &commit_sha[..12])
}

fn host_from_url(remote_url: &str) -> anyhow::Result<String> {
    Ok(reqwest::Url::parse(remote_url)?
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("remote_url must include a host"))?
        .to_string())
}

fn validate_safe_ref(reference: &str) -> anyhow::Result<()> {
    if reference.trim().is_empty()
        || reference.contains("..")
        || reference.starts_with('/')
        || reference.ends_with('/')
        || reference.contains(' ')
        || reference.contains('\\')
    {
        anyhow::bail!("ref is not safe")
    }
    Ok(())
}

fn validate_manifest_path(path: &str) -> anyhow::Result<()> {
    if path.trim().is_empty()
        || path.starts_with('/')
        || path.contains("..")
        || path.contains('\\')
        || !path.ends_with(".yaml") && !path.ends_with(".yml")
    {
        anyhow::bail!("manifest_path is not safe")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_install_returns_proposal_draft() {
        let value = plan_install(&json!({
            "remote_url": "https://github.com/example/pkg",
            "ref": "main",
            "preferred_package_id": "thirdparty/pkg"
        }))
        .unwrap();
        assert_eq!(value["kind"], "package_install_proposal_draft");
        assert_eq!(value["requires_user_approval"], true);
        assert_eq!(value["pinned"]["commit_sha"], Value::Null);
    }

    #[test]
    fn plan_install_rejects_raw_secret_query() {
        let result = plan_install(&json!({
            "remote_url": "https://github.com/example/pkg?token=raw-secret-placeholder"
        }));
        assert!(result.is_err());
    }

    #[test]
    fn plan_install_rejects_unsafe_manifest_path() {
        let result = plan_install(&json!({
            "remote_url": "https://github.com/example/pkg",
            "manifest_path": "../manifest.yaml"
        }));
        assert!(result.is_err());
    }
}
