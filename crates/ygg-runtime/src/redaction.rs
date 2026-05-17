//! Raw-secret redaction and blocking utilities.
//!
//! This module provides conservative scanning of trusted payload paths
//! for leaked raw secrets. It checks known secret field names and
//! value patterns in:
//!
//! - Proposal payloads (operations and expected_effects)
//! - Asset metadata
//! - Audit-like payloads (permission events, kernel events)
//!
//! The scanner is intentionally conservative: it targets obvious API-key
//! field names and high-entropy value patterns. It does NOT scan
//! arbitrary user content fields like `content`, `description`, or `title`
//! to avoid false positives on ordinary text.

use serde_json::Value;
use ygg_core::{is_secret_field_name, looks_like_raw_secret};

/// Result of a raw-secret scan.
#[derive(Debug, Clone)]
pub struct SecretScanResult {
    /// Fields where raw secrets were detected.
    pub findings: Vec<SecretFinding>,
}

/// A single finding of a raw secret in a payload.
#[derive(Debug, Clone)]
pub struct SecretFinding {
    /// The JSON path where the secret was found (dot-separated).
    pub path: String,
    /// The field name that triggered the finding.
    pub field_name: String,
    /// The kind of detection that triggered.
    pub detection: SecretDetection,
}

/// How the secret was detected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretDetection {
    /// The field name matches a known secret field name.
    FieldName,
    /// The value looks like a raw secret (API key pattern).
    ValuePattern,
}

impl SecretScanResult {
    /// Whether any raw secrets were found.
    pub fn has_findings(&self) -> bool {
        !self.findings.is_empty()
    }
}

/// Scan a JSON value for raw secrets.
///
/// This is the entry point for conservative secret scanning.
/// It recursively walks the value, checking field names and
/// string values against known patterns.
///
/// Paths that are excluded from scanning (to avoid false positives):
/// - `content` fields (asset content is arbitrary user data)
/// - `description` fields
/// - `title` fields
/// - `reason` fields
/// - `message` fields
/// - `label` fields
/// - `summary` fields
pub fn scan_value_for_raw_secrets(value: &Value, root_path: &str) -> SecretScanResult {
    let mut result = SecretScanResult { findings: Vec::new() };
    scan_recursive(value, root_path, &mut result, false);
    result
}

/// Fields that are excluded from value-pattern scanning to avoid false positives.
/// Field-name detection still applies, but we don't run looks_like_raw_secret
/// on the values of these fields.
const EXCLUDED_VALUE_SCAN_FIELDS: &[&str] = &[
    "content",
    "description",
    "title",
    "reason",
    "message",
    "label",
    "summary",
    "text",
    "body",
    "note",
    "comment",
    "display_name",
    "author",
];

fn scan_recursive(value: &Value, path: &str, result: &mut SecretScanResult, _in_excluded: bool) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let child_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };
                let is_excluded = EXCLUDED_VALUE_SCAN_FIELDS.iter().any(|f| f == key);

                // Check field name
                if is_secret_field_name(key) {
                    // Only flag if the value is a string (not a SecretRef)
                    if let Value::String(s) = child {
                        if !ygg_core::SecretRef::is_valid_ref(s) {
                            result.findings.push(SecretFinding {
                                path: child_path.clone(),
                                field_name: key.clone(),
                                detection: SecretDetection::FieldName,
                            });
                        }
                    }
                }

                // Check value pattern for non-excluded string fields
                if !is_excluded {
                    if let Value::String(s) = child {
                        if looks_like_raw_secret(s) && !ygg_core::SecretRef::is_valid_ref(s) {
                            result.findings.push(SecretFinding {
                                path: child_path.clone(),
                                field_name: key.clone(),
                                detection: SecretDetection::ValuePattern,
                            });
                        }
                    }
                }

                // Recurse into child
                scan_recursive(child, &child_path, result, is_excluded);
            }
        }
        Value::Array(arr) => {
            for (i, child) in arr.iter().enumerate() {
                let child_path = format!("{}[{}]", path, i);
                scan_recursive(child, &child_path, result, false);
            }
        }
        _ => {}
    }
}

/// Redact a JSON value by replacing detected raw secret values
/// with `<secret:redacted>`.
///
/// Returns the redacted value and the scan result.
pub fn redact_secrets_in_value(value: &Value) -> (Value, SecretScanResult) {
    let scan = scan_value_for_raw_secrets(value, "");
    if scan.findings.is_empty() {
        return (value.clone(), scan);
    }
    let mut redacted = value.clone();
    for finding in &scan.findings {
        apply_redaction(&mut redacted, &finding.path);
    }
    (redacted, scan)
}

fn apply_redaction(value: &mut Value, path: &str) {
    let parts: Vec<&str> = path.split('.').collect();
    redact_path_recursive(value, &parts);
}

fn redact_path_recursive(value: &mut Value, parts: &[&str]) {
    if parts.is_empty() {
        return;
    }

    // Handle array indexing like "path[0]"
    let (current_key, remaining) = parts[0].split_once('[').unwrap_or((parts[0], ""));
    let array_index = if remaining.ends_with(']') && !remaining.is_empty() {
        remaining.trim_end_matches(']').parse::<usize>().ok()
    } else {
        None
    };

    if parts.len() == 1 {
        // Terminal: redact the value
        if let Value::Object(map) = value {
            if let Some(v) = map.get_mut(current_key) {
                *v = Value::String("<secret:redacted>".to_string());
            }
        } else if let Value::Array(arr) = value {
            if let Some(idx) = array_index {
                if idx < arr.len() {
                    arr[idx] = Value::String("<secret:redacted>".to_string());
                }
            }
        }
        return;
    }

    // Recurse
    let rest = &parts[1..];
    if let Value::Object(map) = value {
        if let Some(child) = map.get_mut(current_key) {
            redact_path_recursive(child, rest);
        }
    } else if let Value::Array(arr) = value {
        if let Some(idx) = array_index {
            if idx < arr.len() {
                redact_path_recursive(&mut arr[idx], rest);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use super::*;

    #[test]
    fn detects_api_key_field_name() {
        let value = json!({"api_key": "somevalue"});
        let result = scan_value_for_raw_secrets(&value, "");
        assert!(result.has_findings());
        assert_eq!(result.findings[0].detection, SecretDetection::FieldName);
    }

    #[test]
    fn allows_secret_ref_in_secret_field() {
        let value = json!({"secret": "secret_ref:env:MY_KEY"});
        let result = scan_value_for_raw_secrets(&value, "");
        assert!(!result.has_findings());
    }

    #[test]
    fn detects_raw_secret_value_pattern() {
        let value = json!({"config": {"bearer": "Bearer sk-abc123def456ghi789jkl012mno345pqr"}});
        let result = scan_value_for_raw_secrets(&value, "");
        assert!(result.has_findings());
    }

    #[test]
    fn excludes_content_field_from_value_scan() {
        // Content fields hold arbitrary user data — don't scan for value patterns
        let value = json!({"content": "sk-abc123def456ghi789jkl012mno345pqr678stu901vwx"});
        let result = scan_value_for_raw_secrets(&value, "");
        // Field-name detection wouldn't fire either since "content" isn't a secret field name
        assert!(!result.has_findings());
    }

    #[test]
    fn detects_nested_secret() {
        let value = json!({
            "operations": [
                {"op": "asset.put", "payload": {"api_key": "sk-real-key-abc123def456ghi789"}}
            ]
        });
        let result = scan_value_for_raw_secrets(&value, "");
        assert!(result.has_findings());
    }

    #[test]
    fn redaction_replaces_value() {
        let value = json!({"api_key": "sk-abc123", "title": "My Package"});
        let (redacted, scan) = redact_secrets_in_value(&value);
        assert!(scan.has_findings());
        assert_eq!(redacted["api_key"], "<secret:redacted>");
        assert_eq!(redacted["title"], "My Package");
    }

    #[test]
    fn no_false_positive_on_normal_text() {
        let value = json!({
            "title": "My Package",
            "description": "A useful package",
            "version": "0.1.0",
            "reason": "needed for work",
            "expected_effects": {"summary": "writes an asset"}
        });
        let result = scan_value_for_raw_secrets(&value, "");
        assert!(!result.has_findings());
    }
}
