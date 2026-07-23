use anyhow::Context;
use reqwest::redirect::Policy;
use reqwest::{Client, Method, StatusCode};
use serde_json::{json, Value};
use std::net::IpAddr;

const VALID_SCOPES: &[&str] = &[
    "observe",
    "project_operate",
    "deploy",
    "develop_propose",
    "develop_approve",
    "develop_execute",
    "access_manage",
];

pub async fn me(endpoint: &str, access_token: &str) -> anyhow::Result<()> {
    print_json(
        request(
            endpoint,
            access_token,
            Method::GET,
            "/host/v1/access/me",
            None,
        )
        .await?,
    )
}

pub async fn list(endpoint: &str, access_token: &str) -> anyhow::Result<()> {
    print_json(request(endpoint, access_token, Method::GET, "/host/v1/access", None).await?)
}

pub async fn pair(
    endpoint: &str,
    access_token: &str,
    device_name: String,
    mut scopes: Vec<String>,
    projects: Vec<String>,
    targets: Vec<String>,
    grant_days: u64,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        !device_name.trim().is_empty(),
        "device_name cannot be empty"
    );
    anyhow::ensure!(
        (1..=365).contains(&grant_days),
        "grant_days must be between 1 and 365"
    );
    scopes.sort();
    scopes.dedup();
    anyhow::ensure!(
        scopes
            .iter()
            .all(|scope| VALID_SCOPES.contains(&scope.as_str())),
        "unknown Host access scope; valid values: {}",
        VALID_SCOPES.join(", ")
    );
    if !scopes.iter().any(|scope| scope == "observe") {
        scopes.push("observe".to_string());
    }
    let mut resources = Vec::new();
    if projects.is_empty() {
        resources.push(json!({"kind": "project"}));
    } else {
        for id in normalized_ids(projects, "project")? {
            resources.push(json!({"kind": "project", "id": id}));
        }
    }
    if targets.is_empty() {
        resources.push(json!({"kind": "target"}));
    } else {
        for id in normalized_ids(targets, "target")? {
            resources.push(json!({"kind": "target", "id": id}));
        }
    }
    let body = json!({
        "device_name": device_name.trim(),
        "scopes": scopes,
        "resources": resources,
        "pairing_ttl_secs": 600,
        "grant_ttl_secs": grant_days.saturating_mul(24 * 60 * 60),
    });
    print_json(
        request(
            endpoint,
            access_token,
            Method::POST,
            "/host/v1/access/pairings",
            Some(body),
        )
        .await?,
    )
}

pub async fn revoke(endpoint: &str, access_token: &str, grant_id: &str) -> anyhow::Result<()> {
    anyhow::ensure!(!grant_id.trim().is_empty(), "grant_id cannot be empty");
    let path = format!(
        "/host/v1/access/grants/{}/revoke",
        url::form_urlencoded::byte_serialize(grant_id.trim().as_bytes()).collect::<String>()
    );
    print_json(request(endpoint, access_token, Method::POST, &path, Some(json!({}))).await?)
}

fn normalized_ids(ids: Vec<String>, kind: &str) -> anyhow::Result<Vec<String>> {
    let mut normalized = ids
        .into_iter()
        .map(|id| id.trim().to_string())
        .collect::<Vec<_>>();
    anyhow::ensure!(
        normalized.iter().all(|id| !id.is_empty()),
        "{kind} ids cannot be empty"
    );
    normalized.sort();
    normalized.dedup();
    Ok(normalized)
}

async fn request(
    endpoint: &str,
    access_token: &str,
    method: Method,
    path: &str,
    body: Option<Value>,
) -> anyhow::Result<Value> {
    let url = host_url(endpoint, path)?;
    // Host control APIs do not use redirects. Refusing them also prevents a
    // bearer credential from ever being replayed to a redirected origin.
    let client = Client::builder().redirect(Policy::none()).build()?;
    let mut request = client
        .request(method, url)
        .bearer_auth(access_token)
        .header("accept", "application/json");
    if let Some(body) = body {
        request = request.json(&body);
    }
    let response = request.send().await.context("Host access request failed")?;
    let status = response.status();
    let bytes = response.bytes().await?;
    if !status.is_success() {
        anyhow::bail!(
            "Host access request returned {}: {}",
            status,
            String::from_utf8_lossy(&bytes)
        );
    }
    if status == StatusCode::NO_CONTENT || bytes.is_empty() {
        return Ok(Value::Null);
    }
    serde_json::from_slice(&bytes).context("Host returned invalid JSON")
}

fn host_url(endpoint: &str, path: &str) -> anyhow::Result<url::Url> {
    let endpoint = endpoint.trim_end_matches('/');
    let url = url::Url::parse(&format!("{endpoint}{path}"))
        .with_context(|| format!("invalid Host endpoint '{endpoint}'"))?;
    anyhow::ensure!(
        matches!(url.scheme(), "http" | "https"),
        "Host endpoint must use http or https"
    );
    if url.scheme() == "http" {
        let host = url.host_str().unwrap_or_default();
        let loopback = host.eq_ignore_ascii_case("localhost")
            || host
                .parse::<IpAddr>()
                .is_ok_and(|address| address.is_loopback());
        anyhow::ensure!(
            loopback,
            "remote Host access requires https; http is allowed only for loopback"
        );
    }
    Ok(url)
}

fn print_json(value: Value) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::host_url;

    #[test]
    fn host_access_http_is_loopback_only() {
        assert!(host_url("http://127.0.0.1:8787", "/host/v1/access/me").is_ok());
        assert!(host_url("http://[::1]:8787", "/host/v1/access/me").is_ok());
        assert!(host_url("http://localhost:8787", "/host/v1/access/me").is_ok());
        assert!(host_url("http://example.test:8787", "/host/v1/access/me").is_err());
        assert!(host_url("https://example.test", "/host/v1/access/me").is_ok());
    }
}
