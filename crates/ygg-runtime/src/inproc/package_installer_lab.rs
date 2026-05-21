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
        "apply_install" => not_implemented_yet("apply_install"),
        "list_installed" => not_implemented_yet("list_installed"),
        "uninstall" => not_implemented_yet("uninstall"),
        "update" => not_implemented_yet("update"),
        "inspect_lockfile" => not_implemented_yet("inspect_lockfile"),
        _ => return None,
    })
}

fn describe_install_contract() -> anyhow::Result<Value> {
    Ok(json!({
        "package_id": "official/package-installer-lab",
        "kind": "git_package_installer_contract",
        "stage": "plan_only",
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
            "profile-scoped lockfile"
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

fn not_implemented_yet(capability: &str) -> anyhow::Result<Value> {
    Ok(json!({
        "kind": "package_installer_pending_capability",
        "capability": capability,
        "status": "reserved_for_apply_stage",
        "requires_user_approval": true
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
