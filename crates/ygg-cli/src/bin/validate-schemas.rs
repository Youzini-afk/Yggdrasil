use std::fs;
use std::path::Path;

use jsonschema::JSONSchema;
use serde_json::Value;

fn main() -> anyhow::Result<()> {
    let root = Path::new("docs/spec/v1/schemas");
    anyhow::ensure!(
        root.exists(),
        "schema directory missing; run cargo run -p ygg-cli --bin export-schemas"
    );
    let mut files = Vec::new();
    collect_json(root, &mut files)?;
    anyhow::ensure!(!files.is_empty(), "no schema files found");
    for file in &files {
        let text = fs::read_to_string(file)?;
        let schema: Value = serde_json::from_str(&text)?;
        let dialect = schema
            .get("$schema")
            .and_then(Value::as_str)
            .unwrap_or_default();
        anyhow::ensure!(
            dialect == "https://json-schema.org/draft/2020-12/schema",
            "{} is not JSON Schema 2020-12",
            file.display()
        );
        JSONSchema::compile(&schema).map_err(|error| {
            anyhow::anyhow!("{} failed to compile as schema: {error}", file.display())
        })?;
    }

    let method_count = fs::read_dir(root.join("methods"))?
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("json"))
        .count();
    let event_count = fs::read_dir(root.join("events"))?
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("json"))
        .count();
    anyhow::ensure!(
        method_count == ygg_runtime::KernelMethod::all().len(),
        "method schema count {method_count} does not match registry {}",
        ygg_runtime::KernelMethod::all().len()
    );
    anyhow::ensure!(
        event_count >= 30,
        "event schema count {event_count} is unexpectedly low"
    );

    // Additive-only diff hook for CI. Without a previous checkout, we validate the
    // canonical artifacts and leave breaking-change detection to CI that provides
    // BASE_SCHEMA_DIR.
    if let Ok(base) = std::env::var("BASE_SCHEMA_DIR") {
        let base = Path::new(&base);
        if base.exists() {
            let mut old_files = Vec::new();
            collect_json(base, &mut old_files)?;
            for old_file in old_files {
                let rel = old_file.strip_prefix(base)?;
                let new_file = root.join(rel);
                anyhow::ensure!(
                    new_file.exists(),
                    "breaking schema removal: {}",
                    rel.display()
                );
            }
        }
    }
    println!(
        "validated {} schemas (methods: {method_count}, events: {event_count})",
        files.len()
    );
    Ok(())
}

fn collect_json(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> anyhow::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_json(&path, out)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
            out.push(path);
        }
    }
    Ok(())
}
