//! Generate SDKs from docs/spec/v1/schemas/ for TS, Rust, OpenAPI.
//!
//! Outputs:
//!   sdk/typescript/kernel-sdk/src/{types.ts, methods.ts, events.ts, index.ts}
//!   sdk/rust/yg-kernel-sdk/src/{types.rs, methods.rs, events.rs, lib.rs}
//!   sdk/openapi.yaml

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use heck::{ToLowerCamelCase, ToPascalCase, ToSnakeCase};
use schemars::schema::RootSchema;
use serde_json::{json, Map, Value};
use typify::{TypeSpace, TypeSpaceSettings};

const SCHEMA_DIR: &str = "docs/spec/v1/schemas";
const TS_DIR: &str = "sdk/typescript/kernel-sdk/src";
const RUST_DIR: &str = "sdk/rust/yg-kernel-sdk/src";
const OPENAPI: &str = "sdk/openapi.yaml";

#[derive(Clone)]
struct NamedSchema {
    name: String,
    schema: Value,
}

#[derive(Clone)]
struct MethodSpec {
    id: String,
    function_ts: String,
    function_rs: String,
    params_ts: String,
    result_ts: String,
    params_rs: String,
    result_rs: String,
    params_schema: Value,
    result_schema: Value,
}

#[derive(Clone)]
struct EventSpec {
    kind: String,
    payload_alias: String,
    event_name: String,
    payload_ts: String,
    payload_rs: String,
    payload_schema: Value,
}

struct TypeRegistry {
    schemas: BTreeMap<String, Value>,
}

impl TypeRegistry {
    fn new() -> Self {
        Self {
            schemas: BTreeMap::new(),
        }
    }

    fn register(&mut self, preferred: impl AsRef<str>, schema: &Value) -> String {
        let mut name = sanitize_type_name(preferred.as_ref());
        if name.is_empty() {
            name = "GeneratedType".to_string();
        }

        if let Some(existing) = self.schemas.get(&name) {
            if schemas_equivalent(existing, schema) {
                return name;
            }
        } else {
            self.schemas.insert(name.clone(), schema.clone());
            return name;
        }

        let base = name;
        let mut i = 2;
        loop {
            let candidate = format!("{base}{i}");
            match self.schemas.get(&candidate) {
                Some(existing) if schemas_equivalent(existing, schema) => return candidate,
                Some(_) => i += 1,
                None => {
                    self.schemas.insert(candidate.clone(), schema.clone());
                    return candidate;
                }
            }
        }
    }
}

fn main() -> Result<()> {
    fs::create_dir_all(TS_DIR)?;
    fs::create_dir_all(RUST_DIR)?;

    let schemas = read_schema_files(Path::new(SCHEMA_DIR))?;
    let mut registry = TypeRegistry::new();

    let top_level = collect_top_level_types(&schemas, &mut registry);
    let mut methods = collect_methods(&schemas, &mut registry)?;
    let mut events = collect_events(&schemas, &mut registry)?;

    write_typescript(&registry, &methods, &events)?;
    write_rust(&top_level, &mut methods, &mut events)?;
    write_openapi(&methods, &top_level)?;

    Ok(())
}

fn read_schema_files(root: &Path) -> Result<BTreeMap<PathBuf, Value>> {
    let mut out = BTreeMap::new();
    visit_schema_files(root, &mut out)?;
    Ok(out)
}

fn visit_schema_files(dir: &Path, out: &mut BTreeMap<PathBuf, Value>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_schema_files(&path, out)?;
        } else if path
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|name| name.ends_with(".schema.json"))
        {
            let data = fs::read_to_string(&path)?;
            let value: Value = serde_json::from_str(&data)
                .with_context(|| format!("parsing {}", path.display()))?;
            out.insert(path, value);
        }
    }
    Ok(())
}

fn collect_top_level_types(
    schemas: &BTreeMap<PathBuf, Value>,
    registry: &mut TypeRegistry,
) -> Vec<NamedSchema> {
    let mut top_level = Vec::new();
    for (path, schema) in schemas {
        if path.components().any(|c| c.as_os_str() == "methods")
            || path.components().any(|c| c.as_os_str() == "events")
        {
            continue;
        }
        let name = schema_title(schema).unwrap_or_else(|| file_stem_type_name(path));
        let name = registry.register(&name, schema);
        top_level.push(NamedSchema {
            name,
            schema: schema.clone(),
        });
        collect_defs(schema, registry);
    }
    top_level
}

fn collect_methods(
    schemas: &BTreeMap<PathBuf, Value>,
    registry: &mut TypeRegistry,
) -> Result<Vec<MethodSpec>> {
    let mut methods = Vec::new();
    for (path, schema) in schemas {
        if !path.components().any(|c| c.as_os_str() == "methods") {
            continue;
        }
        let id = schema
            .pointer("/properties/method/const")
            .and_then(Value::as_str)
            .or_else(|| schema.get("title").and_then(Value::as_str))
            .ok_or_else(|| anyhow!("method schema {} has no method const", path.display()))?
            .to_string();
        let defs = schema.get("$defs").or_else(|| schema.get("definitions"));
        let params_schema = defs
            .and_then(|d| d.get("Params"))
            .ok_or_else(|| anyhow!("method schema {id} has no Params"))?
            .clone();
        let result_schema = defs
            .and_then(|d| d.get("Result"))
            .ok_or_else(|| anyhow!("method schema {id} has no Result"))?
            .clone();

        collect_defs(&params_schema, registry);
        collect_defs(&result_schema, registry);

        let base = method_base_name(&id);
        let params_name = schema_title(&params_schema).unwrap_or_else(|| format!("{base}Params"));
        let result_name = schema_title(&result_schema).unwrap_or_else(|| format!("{base}Result"));
        let params_ts = registry.register(params_name, &params_schema);
        let result_ts = registry.register(result_name, &result_schema);
        methods.push(MethodSpec {
            id: id.clone(),
            function_ts: method_function_ts(&id),
            function_rs: id.trim_start_matches("kernel.v1.").to_snake_case(),
            params_rs: params_ts.clone(),
            result_rs: result_ts.clone(),
            params_ts,
            result_ts,
            params_schema,
            result_schema,
        });
    }
    methods.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(methods)
}

fn collect_events(
    schemas: &BTreeMap<PathBuf, Value>,
    registry: &mut TypeRegistry,
) -> Result<Vec<EventSpec>> {
    let mut events = Vec::new();
    for (path, schema) in schemas {
        if !path.components().any(|c| c.as_os_str() == "events") {
            continue;
        }
        let kind = schema
            .pointer("/properties/kind/const")
            .and_then(Value::as_str)
            .or_else(|| schema.get("title").and_then(Value::as_str))
            .ok_or_else(|| anyhow!("event schema {} has no kind const", path.display()))?
            .to_string();
        let payload_schema = schema
            .get("$defs")
            .or_else(|| schema.get("definitions"))
            .and_then(|d| d.get("Payload"))
            .ok_or_else(|| anyhow!("event schema {kind} has no Payload"))?
            .clone();
        collect_defs(&payload_schema, registry);
        let event_name = event_base_name(&kind);
        let payload_name =
            schema_title(&payload_schema).unwrap_or_else(|| format!("{event_name}Payload"));
        let payload_ts = registry.register(payload_name, &payload_schema);
        events.push(EventSpec {
            kind,
            payload_alias: format!("{event_name}Payload"),
            event_name: format!("{event_name}Event"),
            payload_rs: payload_ts.clone(),
            payload_ts,
            payload_schema,
        });
    }
    events.sort_by(|a, b| a.kind.cmp(&b.kind));
    Ok(events)
}

fn collect_defs(schema: &Value, registry: &mut TypeRegistry) {
    if let Some(defs) = schema.get("$defs").or_else(|| schema.get("definitions")) {
        if let Some(map) = defs.as_object() {
            for (key, value) in map {
                let name = schema_title(value).unwrap_or_else(|| key.to_pascal_case());
                registry.register(name, value);
                collect_defs(value, registry);
            }
        }
    }
}

fn write_typescript(
    registry: &TypeRegistry,
    methods: &[MethodSpec],
    events: &[EventSpec],
) -> Result<()> {
    fs::write(Path::new(TS_DIR).join("types.ts"), emit_ts_types(registry))?;
    fs::write(
        Path::new(TS_DIR).join("methods.ts"),
        emit_ts_methods(methods),
    )?;
    fs::write(Path::new(TS_DIR).join("events.ts"), emit_ts_events(events))?;
    fs::write(
        Path::new(TS_DIR).join("index.ts"),
        "export * from \"./client\";\nexport * from \"./types\";\nexport * from \"./events\";\nexport * from \"./methods\";\n",
    )?;
    Ok(())
}

fn emit_ts_types(registry: &TypeRegistry) -> String {
    let mut out = generated_header("TypeScript types generated from docs/spec/v1/schemas/.");
    let ctx = TsContext { registry };
    for (name, schema) in &registry.schemas {
        out.push_str("\n");
        if let Some(description) = schema.get("description").and_then(Value::as_str) {
            out.push_str(&doc_comment(description));
        }
        if schema
            .get("properties")
            .and_then(Value::as_object)
            .is_some()
            && schema.get("oneOf").is_none()
            && schema.get("anyOf").is_none()
            && schema.get("allOf").is_none()
        {
            out.push_str(&format!("export interface {name} "));
            out.push_str(&ctx.object_body(schema, 0));
            out.push_str("\n");
        } else {
            out.push_str(&format!(
                "export type {name} = {};\n",
                ctx.type_expr(schema)
            ));
        }
    }
    out
}

fn emit_ts_methods(methods: &[MethodSpec]) -> String {
    let mut out =
        generated_header("TypeScript client methods generated from docs/spec/v1/schemas/methods/.");
    out.push_str("import type { KernelClient } from \"./client\";\n");
    out.push_str("import type { ");
    let imports = methods
        .iter()
        .flat_map(|m| [m.params_ts.as_str(), m.result_ts.as_str()])
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(", ");
    out.push_str(&imports);
    out.push_str(" } from \"./types\";\n\n");

    out.push_str("export interface KernelMethods {\n");
    for method in methods {
        out.push_str(&format!(
            "  {}(params: {}): Promise<{}>;\n",
            method.function_ts, method.params_ts, method.result_ts
        ));
    }
    out.push_str("}\n\n");

    out.push_str(
        "declare module \"./client\" {\n  interface KernelClient extends KernelMethods {}\n}\n\n",
    );

    for method in methods {
        out.push_str(&format!(
            "export async function {}(\n  this: KernelClient,\n  params: {},\n): Promise<{}> {{\n  return this.transport.invoke(\"{}\", params) as Promise<{}>;\n}}\n\n",
            method.function_ts, method.params_ts, method.result_ts, method.id, method.result_ts
        ));
    }

    out.push_str(
        "export function attach<T extends KernelClient>(client: T): T & KernelMethods {\n",
    );
    for method in methods {
        out.push_str(&format!(
            "  (client as T & KernelMethods).{} = {}.bind(client);\n",
            method.function_ts, method.function_ts
        ));
    }
    out.push_str("  return client as T & KernelMethods;\n}\n");
    out
}

fn emit_ts_events(events: &[EventSpec]) -> String {
    let mut out =
        generated_header("TypeScript event payloads generated from docs/spec/v1/schemas/events/.");
    out.push_str("import type { ");
    let imports = events
        .iter()
        .map(|e| e.payload_ts.as_str())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(", ");
    out.push_str(&imports);
    out.push_str(" } from \"./types\";\n\n");

    for event in events {
        out.push_str(&format!(
            "export interface {} {{\n  kind: {};\n  payload: {};\n}}\n\n",
            event.event_name,
            json_string(&event.kind),
            event.payload_ts
        ));
    }
    out.push_str("export interface KernelEventPayloadMap {\n");
    for event in events {
        out.push_str(&format!(
            "  {}: {};\n",
            json_string(&event.kind),
            event.payload_ts
        ));
    }
    out.push_str("}\n\nexport type KernelEvent =\n");
    for event in events {
        out.push_str(&format!("  | {}\n", event.event_name));
    }
    out.push_str(";\n");
    out
}

struct TsContext<'a> {
    registry: &'a TypeRegistry,
}

impl TsContext<'_> {
    fn type_expr(&self, schema: &Value) -> String {
        match schema {
            Value::Bool(true) => "unknown".to_string(),
            Value::Bool(false) => "never".to_string(),
            Value::Object(map) => self.type_expr_object(map),
            _ => "unknown".to_string(),
        }
    }

    fn type_expr_object(&self, map: &Map<String, Value>) -> String {
        if let Some(reference) = map.get("$ref").and_then(Value::as_str) {
            return self.ref_name(reference);
        }
        if let Some(value) = map.get("const") {
            return literal_ts(value);
        }
        if let Some(values) = map.get("enum").and_then(Value::as_array) {
            return values
                .iter()
                .map(literal_ts)
                .collect::<Vec<_>>()
                .join(" | ");
        }
        if let Some(values) = map.get("oneOf").and_then(Value::as_array) {
            return union_ts(values.iter().map(|v| self.type_expr(v)).collect());
        }
        if let Some(values) = map.get("anyOf").and_then(Value::as_array) {
            return union_ts(values.iter().map(|v| self.type_expr(v)).collect());
        }
        if let Some(values) = map.get("allOf").and_then(Value::as_array) {
            return values
                .iter()
                .map(|v| self.type_expr(v))
                .collect::<Vec<_>>()
                .join(" & ");
        }
        if let Some(type_value) = map.get("type") {
            if let Some(types) = type_value.as_array() {
                return union_ts(
                    types
                        .iter()
                        .map(|t| self.type_for_json_type(t, map))
                        .collect(),
                );
            }
            return self.type_for_json_type(type_value, map);
        }
        if map.contains_key("properties") {
            return self.object_body(&Value::Object(map.clone()), 0);
        }
        "unknown".to_string()
    }

    fn type_for_json_type(&self, type_value: &Value, map: &Map<String, Value>) -> String {
        match type_value.as_str() {
            Some("string") => "string".to_string(),
            Some("integer" | "number") => "number".to_string(),
            Some("boolean") => "boolean".to_string(),
            Some("null") => "null".to_string(),
            Some("array") => {
                let item = map
                    .get("items")
                    .map(|v| self.type_expr(v))
                    .unwrap_or_else(|| "unknown".to_string());
                format!("Array<{item}>")
            }
            Some("object") => self.object_body(&Value::Object(map.clone()), 0),
            _ => "unknown".to_string(),
        }
    }

    fn object_body(&self, schema: &Value, indent: usize) -> String {
        let Some(map) = schema.as_object() else {
            return "Record<string, unknown>".to_string();
        };
        let properties = map.get("properties").and_then(Value::as_object);
        let additional = map.get("additionalProperties");
        if properties.is_none() {
            return match additional {
                Some(Value::Bool(false)) => "Record<string, never>".to_string(),
                Some(Value::Object(_)) | Some(Value::Bool(true)) => format!(
                    "Record<string, {}>",
                    additional
                        .map(|v| self.type_expr(v))
                        .unwrap_or_else(|| "unknown".to_string())
                ),
                _ => "Record<string, unknown>".to_string(),
            };
        }

        let required = map
            .get("required")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        let mut out = String::from("{\n");
        let prop_indent = " ".repeat(indent + 2);
        for (name, prop_schema) in properties.unwrap() {
            if let Some(description) = prop_schema.get("description").and_then(Value::as_str) {
                for line in doc_comment(description).lines() {
                    out.push_str(&prop_indent);
                    out.push_str(line);
                    out.push('\n');
                }
            }
            let optional = if required.contains(name.as_str()) {
                ""
            } else {
                "?"
            };
            out.push_str(&format!(
                "{}{}{}: {};\n",
                prop_indent,
                json_string(name),
                optional,
                self.type_expr(prop_schema)
            ));
        }
        if let Some(Value::Object(_)) = additional {
            out.push_str(&format!(
                "{}[key: string]: {};\n",
                prop_indent,
                self.type_expr(additional.unwrap())
            ));
        }
        out.push_str(&" ".repeat(indent));
        out.push('}');
        out
    }

    fn ref_name(&self, reference: &str) -> String {
        let name = reference
            .strip_prefix("#/$defs/")
            .or_else(|| reference.strip_prefix("#/definitions/"))
            .or_else(|| reference.strip_prefix("#/components/schemas/"))
            .unwrap_or(reference)
            .split('/')
            .last()
            .unwrap_or(reference)
            .to_pascal_case();
        if self.registry.schemas.contains_key(&name) {
            name
        } else {
            sanitize_type_name(&name)
        }
    }
}

fn write_rust(
    top_level: &[NamedSchema],
    methods: &mut [MethodSpec],
    events: &mut [EventSpec],
) -> Result<()> {
    let mut settings = TypeSpaceSettings::default();
    settings
        .with_derive("PartialEq".to_string())
        .with_attr("#[allow(clippy::large_enum_variant)]".to_string());
    let mut type_space = TypeSpace::new(&settings);

    for item in top_level {
        add_rust_type(&mut type_space, &item.schema, &item.name)?;
    }
    for method in methods.iter_mut() {
        method.params_rs =
            add_rust_type(&mut type_space, &method.params_schema, &method.params_rs)?;
        method.result_rs =
            add_rust_type(&mut type_space, &method.result_schema, &method.result_rs)?;
    }
    for event in events.iter_mut() {
        event.payload_rs =
            add_rust_type(&mut type_space, &event.payload_schema, &event.payload_rs)?;
    }

    let mut types = generated_rust_header("Rust types generated from docs/spec/v1/schemas/.");
    types.push_str("#![allow(clippy::large_enum_variant)]\n#![allow(clippy::derive_partial_eq_without_eq)]\n#![allow(clippy::module_name_repetitions)]\n\n");
    types.push_str(&format_rust_tokens(type_space.to_stream().to_string())?);
    fs::write(Path::new(RUST_DIR).join("types.rs"), types)?;
    fs::write(
        Path::new(RUST_DIR).join("methods.rs"),
        emit_rust_methods(methods)?,
    )?;
    fs::write(
        Path::new(RUST_DIR).join("events.rs"),
        emit_rust_events(events)?,
    )?;
    fs::write(
        Path::new(RUST_DIR).join("lib.rs"),
        "pub mod client;\npub mod events;\npub mod methods;\npub mod types;\n\npub use client::{KernelClient, KernelTransport};\npub use events::*;\npub use methods::*;\npub use types::*;\n",
    )?;
    Ok(())
}

fn add_rust_type(type_space: &mut TypeSpace, schema: &Value, name_hint: &str) -> Result<String> {
    let root = root_schema_for_typify(schema, name_hint)?;
    let type_id = type_space
        .add_root_schema(root)
        .map_err(|e| anyhow!("typify failed for {name_hint}: {e}"))?
        .ok_or_else(|| anyhow!("typify did not create root type for {name_hint}"))?;
    Ok(type_space
        .get_type(&type_id)
        .map_err(|e| anyhow!("typify lookup failed for {name_hint}: {e}"))?
        .ident()
        .to_string())
}

fn root_schema_for_typify(schema: &Value, name_hint: &str) -> Result<RootSchema> {
    let mut value = schema.clone();
    if let Some(map) = value.as_object_mut() {
        map.entry("title".to_string())
            .or_insert_with(|| Value::String(sanitize_type_name(name_hint)));
    }
    flatten_definitions(&mut value);
    normalize_for_typify(&mut value);
    serde_json::from_value(value).with_context(|| format!("building RootSchema for {name_hint}"))
}

fn flatten_definitions(value: &mut Value) {
    let mut defs = Map::new();
    collect_definition_entries(value, &mut defs);
    if !defs.is_empty() {
        let map = value.as_object_mut().expect("schema root is object");
        let entry = map
            .entry("$defs".to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        if let Some(existing) = entry.as_object_mut() {
            for (key, value) in defs {
                existing.entry(key).or_insert(value);
            }
        }
    }
}

fn collect_definition_entries(value: &Value, defs: &mut Map<String, Value>) {
    match value {
        Value::Object(map) => {
            for key in ["$defs", "definitions"] {
                if let Some(Value::Object(local_defs)) = map.get(key) {
                    for (name, schema) in local_defs {
                        defs.entry(name.clone()).or_insert_with(|| schema.clone());
                        collect_definition_entries(schema, defs);
                    }
                }
            }
            for (key, child) in map {
                if key != "$defs" && key != "definitions" {
                    collect_definition_entries(child, defs);
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                collect_definition_entries(value, defs);
            }
        }
        _ => {}
    }
}

fn normalize_for_typify(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if let Some(defs) = map.remove("$defs") {
                map.insert("definitions".to_string(), defs);
            }
            if let Some(Value::String(reference)) = map.get_mut("$ref") {
                if let Some(rest) = reference.strip_prefix("#/$defs/") {
                    *reference = format!("#/definitions/{rest}");
                }
            }
            for value in map.values_mut() {
                normalize_for_typify(value);
            }
        }
        Value::Array(values) => {
            for value in values {
                normalize_for_typify(value);
            }
        }
        _ => {}
    }
}

fn emit_rust_methods(methods: &[MethodSpec]) -> Result<String> {
    let header =
        generated_rust_header("Rust client methods generated from docs/spec/v1/schemas/methods/.");
    let mut out = String::new();
    out.push_str(
        "use anyhow::Result;\n\nuse crate::client::KernelClient;\nuse crate::types::*;\n\n",
    );
    out.push_str("impl KernelClient {\n");
    for method in methods {
        out.push_str(&format!(
            "    pub async fn {}(&self, params: {}) -> Result<{}> {{\n        let raw = self.transport.invoke(\"{}\", serde_json::to_value(params)?).await?;\n        Ok(serde_json::from_value(raw)?)\n    }}\n\n",
            method.function_rs, method.params_rs, method.result_rs, method.id
        ));
    }
    out.push_str("}\n");
    Ok(format!("{}{}", header, format_rust(out)?))
}

fn emit_rust_events(events: &[EventSpec]) -> Result<String> {
    let header = generated_rust_header(
        "Rust event payload aliases generated from docs/spec/v1/schemas/events/.",
    );
    let mut out = String::new();
    out.push_str("use crate::types::*;\n\n");
    for event in events {
        out.push_str(&format!(
            "pub const {}: &str = \"{}\";\n",
            event.kind.to_shouty_snake_case(),
            event.kind
        ));
        if event.payload_alias != event.payload_rs {
            out.push_str(&format!(
                "pub type {} = {};\n",
                event.payload_alias, event.payload_rs
            ));
        }
        out.push('\n');
    }
    Ok(format!("{}{}", header, format_rust(out)?))
}

fn write_openapi(methods: &[MethodSpec], top_level: &[NamedSchema]) -> Result<()> {
    let mut paths = Map::new();
    for method in methods {
        let operation_id = method.function_ts.clone();
        let mut request_schema = json_rpc_request_schema(&method.id, &method.params_schema);
        let mut response_schema = json_rpc_response_schema(&method.result_schema);
        normalize_openapi_refs(&mut request_schema);
        normalize_openapi_refs(&mut response_schema);
        paths.insert(
            format!("/rpc/{}", method.id),
            json!({
                "post": {
                    "operationId": operation_id,
                    "summary": format!("Invoke {}", method.id),
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": request_schema
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "JSON-RPC response envelope",
                            "content": {
                                "application/json": {
                                    "schema": response_schema
                                }
                            }
                        }
                    }
                }
            }),
        );
    }

    let mut components = Map::new();
    for schema in top_level {
        let mut component = schema.schema.clone();
        normalize_openapi_refs(&mut component);
        components.insert(schema.name.clone(), component);
    }

    let openapi = json!({
        "openapi": "3.1.0",
        "info": {
            "title": "Yggdrasil Kernel RPC API",
            "version": "1.0.0",
            "description": "Generated from docs/spec/v1/schemas/. JSON-RPC methods are exposed as typed /rpc/{method} operations for code generators."
        },
        "paths": paths,
        "components": { "schemas": components }
    });
    fs::write(OPENAPI, serde_yaml::to_string(&openapi)?)?;
    Ok(())
}

fn normalize_openapi_refs(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if let Some(defs) = map.remove("definitions") {
                map.entry("$defs".to_string()).or_insert(defs);
            }
            if let Some(Value::String(reference)) = map.get_mut("$ref") {
                if let Some(rest) = reference.strip_prefix("#/definitions/") {
                    *reference = format!("#/$defs/{rest}");
                }
            }
            for value in map.values_mut() {
                normalize_openapi_refs(value);
            }
        }
        Value::Array(values) => {
            for value in values {
                normalize_openapi_refs(value);
            }
        }
        _ => {}
    }
}

fn json_rpc_request_schema(method: &str, params: &Value) -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["jsonrpc", "id", "method", "params"],
        "properties": {
            "jsonrpc": { "const": "2.0" },
            "id": { "oneOf": [{"type":"string"}, {"type":"integer"}, {"type":"null"}] },
            "method": { "const": method },
            "params": params
        }
    })
}

fn json_rpc_response_schema(result: &Value) -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["jsonrpc", "id"],
        "properties": {
            "jsonrpc": { "const": "2.0" },
            "id": { "oneOf": [{"type":"string"}, {"type":"integer"}, {"type":"null"}] },
            "result": result,
            "error": {
                "type": "object",
                "required": ["code", "message"],
                "properties": {
                    "code": { "type": "integer" },
                    "message": { "type": "string" },
                    "data": true
                }
            }
        }
    })
}

fn schema_title(schema: &Value) -> Option<String> {
    schema
        .get("title")
        .and_then(Value::as_str)
        .map(sanitize_type_name)
}

fn file_stem_type_name(path: &Path) -> String {
    path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("GeneratedType")
        .trim_end_matches(".schema.json")
        .to_pascal_case()
}

fn sanitize_type_name(name: &str) -> String {
    let pascal = name
        .replace("kernel.v1.", "")
        .replace("kernel/v1/", "")
        .replace(['.', '/', '-'], "_")
        .to_pascal_case();
    if pascal.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        format!("T{pascal}")
    } else {
        pascal
    }
}

fn method_base_name(id: &str) -> String {
    sanitize_type_name(id.trim_start_matches("kernel.v1."))
}

fn event_base_name(kind: &str) -> String {
    sanitize_type_name(kind.trim_start_matches("kernel/v1/"))
}

fn method_function_ts(id: &str) -> String {
    id.trim_start_matches("kernel.v1.")
        .replace(['.', '-', '/'], "_")
        .to_lower_camel_case()
}

fn schemas_equivalent(a: &Value, b: &Value) -> bool {
    let mut a = a.clone();
    let mut b = b.clone();
    strip_metadata(&mut a);
    strip_metadata(&mut b);
    a == b
}

fn strip_metadata(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.remove("title");
            map.remove("description");
            map.remove("$schema");
            map.remove("$id");
            for value in map.values_mut() {
                strip_metadata(value);
            }
        }
        Value::Array(values) => {
            for value in values {
                strip_metadata(value);
            }
        }
        _ => {}
    }
}

fn union_ts(types: Vec<String>) -> String {
    let unique = types.into_iter().collect::<BTreeSet<_>>();
    if unique.is_empty() {
        "unknown".to_string()
    } else {
        unique.into_iter().collect::<Vec<_>>().join(" | ")
    }
}

fn literal_ts(value: &Value) -> String {
    match value {
        Value::String(s) => json_string(s),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        _ => "unknown".to_string(),
    }
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value).expect("string serializes")
}

fn doc_comment(text: &str) -> String {
    let mut out = String::new();
    out.push_str("/**\n");
    for line in text.lines() {
        out.push_str(" * ");
        out.push_str(line.trim());
        out.push('\n');
    }
    out.push_str(" */\n");
    out
}

fn generated_header(message: &str) -> String {
    format!("// @generated by cargo run -p ygg-cli --bin generate-sdks\n// {message}\n")
}

fn generated_rust_header(message: &str) -> String {
    format!("// @generated by cargo run -p ygg-cli --bin generate-sdks\n// {message}\n")
}

fn format_rust_tokens(tokens: String) -> Result<String> {
    format_rust(tokens)
}

fn format_rust(source: String) -> Result<String> {
    let file = syn::parse_file(&source).with_context(|| {
        let preview: String = source.chars().take(4000).collect();
        format!("generated Rust did not parse. Preview:\n{preview}")
    })?;
    Ok(prettyplease::unparse(&file))
}

trait ShoutySnake {
    fn to_shouty_snake_case(&self) -> String;
}

impl ShoutySnake for str {
    fn to_shouty_snake_case(&self) -> String {
        self.replace(['/', '.', '-'], "_")
            .to_snake_case()
            .to_uppercase()
    }
}
