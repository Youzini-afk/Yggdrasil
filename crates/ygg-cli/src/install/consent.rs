use anyhow::Result;
use serde_json::{json, Value};

pub fn approve_all(plan: &Value) -> Value {
    let summary = &plan["permissions_summary"];
    json!({
        "approved_capabilities": summary["new_capabilities"].clone(),
        "approved_network_hosts": summary["new_network_hosts"].clone(),
        "approved_secret_refs": summary["new_secret_refs"].clone(),
    })
}

pub fn prompt_for_consent(_plan: &Value) -> Result<Value> {
    anyhow::bail!("Use --yes to install (consent prompt coming in I6)")
}
