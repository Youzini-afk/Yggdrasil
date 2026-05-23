use anyhow::{Context, Result};
use console::style;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::io::IsTerminal;
use ygg_core::lockfile::Lockfile;

pub fn approve_all(plan: &Value) -> Value {
    let summary = plan
        .pointer("/permissions_summary")
        .cloned()
        .unwrap_or(json!({}));
    let new_caps = summary
        .pointer("/new_capabilities")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let new_hosts = summary
        .pointer("/new_network_hosts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let new_secrets = summary
        .pointer("/new_secret_refs")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    json!({
        "approved_capabilities": new_caps,
        "approved_network_hosts": new_hosts,
        "approved_secret_refs": new_secrets,
    })
}

pub fn prompt_for_consent(plan: &Value, existing_lockfile: Option<&str>) -> Result<Value> {
    let requested = aggregate_plan_permissions(plan);
    let existing = match existing_lockfile {
        Some(lockfile_toml) => read_existing_grants(lockfile_toml)?,
        None => AggregatedPermissions::default(),
    };
    let diff = diff_permissions(&requested, &existing);

    if !std::io::stdin().is_terminal() {
        anyhow::bail!("no TTY; use --yes for non-interactive consent");
    }

    if !diff.has_new_permissions() {
        return Ok(consent_from_permissions(&requested));
    }

    if render_prompt(&diff, plan)? {
        Ok(consent_from_permissions(&requested))
    } else {
        anyhow::bail!("install declined by user")
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct AggregatedPermissions {
    capabilities: BTreeSet<String>,
    network_hosts: BTreeSet<String>,
    secret_refs: BTreeSet<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct PermissionDiff {
    new_capabilities: BTreeSet<String>,
    new_network_hosts: BTreeSet<String>,
    new_secret_refs: BTreeSet<String>,
    already_granted: AggregatedPermissions,
}

impl PermissionDiff {
    fn has_new_permissions(&self) -> bool {
        !self.new_capabilities.is_empty()
            || !self.new_network_hosts.is_empty()
            || !self.new_secret_refs.is_empty()
    }
}

fn aggregate_plan_permissions(plan: &Value) -> AggregatedPermissions {
    let mut permissions = AggregatedPermissions::default();
    if let Some(packages) = plan.pointer("/packages").and_then(Value::as_array) {
        for pkg in packages {
            collect_strings(
                pkg.pointer("/permissions/capabilities_invoke"),
                &mut permissions.capabilities,
            );
            collect_strings(
                pkg.pointer("/permissions/network_hosts"),
                &mut permissions.network_hosts,
            );
            collect_strings(
                pkg.pointer("/permissions/secret_refs"),
                &mut permissions.secret_refs,
            );
        }
    }
    permissions
}

fn collect_strings(value: Option<&Value>, output: &mut BTreeSet<String>) {
    if let Some(values) = value.and_then(Value::as_array) {
        output.extend(values.iter().filter_map(Value::as_str).map(str::to_string));
    }
}

fn read_existing_grants(lockfile_toml: &str) -> Result<AggregatedPermissions> {
    let lockfile: Lockfile =
        toml::from_str(lockfile_toml).context("failed to parse lockfile TOML")?;
    lockfile.validate()?;

    let mut permissions = AggregatedPermissions::default();
    for entry in lockfile.package {
        permissions.capabilities.extend(entry.granted_capabilities);
        permissions.network_hosts.extend(entry.granted_network);
        permissions.secret_refs.extend(entry.granted_secrets);
    }
    Ok(permissions)
}

fn diff_permissions(
    plan: &AggregatedPermissions,
    existing: &AggregatedPermissions,
) -> PermissionDiff {
    PermissionDiff {
        new_capabilities: plan
            .capabilities
            .difference(&existing.capabilities)
            .cloned()
            .collect(),
        new_network_hosts: plan
            .network_hosts
            .difference(&existing.network_hosts)
            .cloned()
            .collect(),
        new_secret_refs: plan
            .secret_refs
            .difference(&existing.secret_refs)
            .cloned()
            .collect(),
        already_granted: AggregatedPermissions {
            capabilities: plan
                .capabilities
                .intersection(&existing.capabilities)
                .cloned()
                .collect(),
            network_hosts: plan
                .network_hosts
                .intersection(&existing.network_hosts)
                .cloned()
                .collect(),
            secret_refs: plan
                .secret_refs
                .intersection(&existing.secret_refs)
                .cloned()
                .collect(),
        },
    }
}

fn consent_from_permissions(permissions: &AggregatedPermissions) -> Value {
    json!({
        "approved_capabilities": permissions.capabilities.iter().cloned().collect::<Vec<_>>(),
        "approved_network_hosts": permissions.network_hosts.iter().cloned().collect::<Vec<_>>(),
        "approved_secret_refs": permissions.secret_refs.iter().cloned().collect::<Vec<_>>(),
    })
}

fn render_prompt(diff: &PermissionDiff, plan: &Value) -> Result<bool> {
    println!();
    println!("{}", style("Install plan:").bold().underlined());

    if let Some(packages) = plan.pointer("/packages").and_then(Value::as_array) {
        for pkg in packages {
            let id = pkg.pointer("/id").and_then(Value::as_str).unwrap_or("?");
            let version = pkg
                .pointer("/version")
                .and_then(Value::as_str)
                .unwrap_or("");
            let signed = pkg
                .pointer("/signed")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let signed_marker = if signed {
                style("✓ signed").green()
            } else {
                style("⚠ unsigned").yellow()
            };
            println!("  {} @ {}  {}", style(id).cyan(), version, signed_marker);
        }
    }

    if !diff.new_capabilities.is_empty() {
        println!();
        println!(
            "{}",
            style("New capability invocations requested:")
                .bold()
                .yellow()
        );
        for cap in &diff.new_capabilities {
            println!("  - {}", style(cap).red());
        }
    }
    if !diff.new_network_hosts.is_empty() {
        println!();
        println!("{}", style("New network hosts requested:").bold().yellow());
        for host in &diff.new_network_hosts {
            println!("  - {}", style(host).red());
        }
    }
    if !diff.new_secret_refs.is_empty() {
        println!();
        println!(
            "{}",
            style("New secret references requested:").bold().yellow()
        );
        for secret in &diff.new_secret_refs {
            println!("  - {}", style(secret).red());
        }
    }
    if !diff.already_granted.capabilities.is_empty()
        || !diff.already_granted.network_hosts.is_empty()
        || !diff.already_granted.secret_refs.is_empty()
    {
        println!();
        println!("{}", style("Already granted (reused):").dim());
        for cap in &diff.already_granted.capabilities {
            println!("  - {}", style(format!("{cap} (capability)")).dim());
        }
        for host in &diff.already_granted.network_hosts {
            println!("  - {}", style(format!("{host} (network host)")).dim());
        }
        for secret in &diff.already_granted.secret_refs {
            println!("  - {}", style(format!("{secret} (secret ref)")).dim());
        }
    }

    println!();
    let confirmed = dialoguer::Confirm::new()
        .with_prompt("Proceed with install?")
        .default(false)
        .interact()?;

    Ok(confirmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approve_all_extracts_all_new_permissions() {
        let plan = json!({
            "permissions_summary": {
                "new_capabilities": ["model.live_call", "vector.search"],
                "new_network_hosts": ["api.openai.com"],
                "new_secret_refs": ["OPENAI_API_KEY"]
            }
        });

        let consent = approve_all(&plan);

        assert_eq!(
            consent["approved_capabilities"],
            json!(["model.live_call", "vector.search"])
        );
        assert_eq!(consent["approved_network_hosts"], json!(["api.openai.com"]));
        assert_eq!(consent["approved_secret_refs"], json!(["OPENAI_API_KEY"]));
    }

    #[test]
    fn diff_detects_new_capabilities() {
        let plan = AggregatedPermissions {
            capabilities: ["a", "b", "c"].into_iter().map(String::from).collect(),
            network_hosts: BTreeSet::default(),
            secret_refs: BTreeSet::default(),
        };
        let existing = AggregatedPermissions {
            capabilities: ["a", "b"].into_iter().map(String::from).collect(),
            network_hosts: BTreeSet::default(),
            secret_refs: BTreeSet::default(),
        };

        let diff = diff_permissions(&plan, &existing);

        assert_eq!(
            diff.new_capabilities,
            ["c"].into_iter().map(String::from).collect()
        );
        assert_eq!(
            diff.already_granted.capabilities,
            ["a", "b"].into_iter().map(String::from).collect()
        );
    }

    #[test]
    fn aggregate_plan_permissions_collects_all_package_permissions() {
        let plan = json!({
            "packages": [
                {
                    "permissions": {
                        "capabilities_invoke": ["model.live_call"],
                        "network_hosts": ["api.openai.com"],
                        "secret_refs": ["secret_ref:env:OPENAI_API_KEY"]
                    }
                },
                {
                    "permissions": {
                        "capabilities_invoke": ["vector.search", "model.live_call"],
                        "network_hosts": ["example.com"],
                        "secret_refs": []
                    }
                }
            ]
        });

        let permissions = aggregate_plan_permissions(&plan);

        assert_eq!(
            permissions.capabilities,
            ["model.live_call", "vector.search"]
                .into_iter()
                .map(String::from)
                .collect()
        );
        assert_eq!(
            permissions.network_hosts,
            ["api.openai.com", "example.com"]
                .into_iter()
                .map(String::from)
                .collect()
        );
        assert_eq!(
            permissions.secret_refs,
            ["secret_ref:env:OPENAI_API_KEY"]
                .into_iter()
                .map(String::from)
                .collect()
        );
    }

    #[test]
    fn consent_from_permissions_approves_requested_permissions() {
        let plan = json!({
            "packages": [{
                "permissions": {
                    "capabilities_invoke": ["model.live_call"],
                    "network_hosts": ["api.openai.com"],
                    "secret_refs": ["secret_ref:env:OPENAI_API_KEY"]
                }
            }]
        });
        let permissions = aggregate_plan_permissions(&plan);

        let consent = consent_from_permissions(&permissions);

        assert_eq!(consent["approved_capabilities"], json!(["model.live_call"]));
        assert_eq!(consent["approved_network_hosts"], json!(["api.openai.com"]));
        assert_eq!(
            consent["approved_secret_refs"],
            json!(["secret_ref:env:OPENAI_API_KEY"])
        );
    }
}
