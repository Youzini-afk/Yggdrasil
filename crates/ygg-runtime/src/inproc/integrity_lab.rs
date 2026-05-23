//! Handler for `official/integrity-lab` capabilities.
//!
//! Provides deterministic SHA-256 hashing plus GPG detached signature
//! verification for package installation.  Sequoia is LGPL-2.0-or-later, which
//! is compatible with AGPL-3.0 deployments; `crypto-rust` keeps the backend
//! pure Rust and avoids system OpenSSL/nettle/botan dependencies.

use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use sequoia_openpgp as openpgp;
use serde_json::Value;
use sha2::{Digest, Sha256};

use openpgp::parse::{stream::*, Parse};
use openpgp::policy::StandardPolicy;
use openpgp::Cert;

use super::InprocInvocation;

const PACKAGE_ID: &str = "official/integrity-lab";

const EXCLUDED_NAMES: &[&str] = &[
    ".git",
    ".gitignore",
    ".DS_Store",
    "node_modules",
    "target",
    "dist",
    "__pycache__",
];

pub fn try_handle(request: &InprocInvocation) -> Option<anyhow::Result<Value>> {
    if request.provider_package_id != PACKAGE_ID {
        return None;
    }

    match request.capability_id.as_str() {
        "integrity.compute_tree_hash" | "official/integrity-lab/compute_tree_hash" => {
            Some(compute_tree_hash(request))
        }
        "integrity.compute_manifest_hash" | "official/integrity-lab/compute_manifest_hash" => {
            Some(compute_manifest_hash(request))
        }
        "integrity.verify_gpg_signature" | "official/integrity-lab/verify_gpg_signature" => {
            Some(verify_gpg_signature(request))
        }
        "integrity.fingerprint_public_key" | "official/integrity-lab/fingerprint_public_key" => {
            Some(fingerprint_public_key(request))
        }
        _ => None,
    }
}

fn input_str<'a>(input: &'a Value, key: &str) -> Result<&'a str> {
    input
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow::anyhow!("missing or invalid string field '{key}'"))
}

fn sha256_prefixed(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("sha256:{}", to_hex(&digest))
}

fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn compute_tree_hash(request: &InprocInvocation) -> Result<Value> {
    let dir = PathBuf::from(input_str(&request.input, "dir")?);
    anyhow::ensure!(dir.is_absolute(), "dir must be an absolute path");
    anyhow::ensure!(dir.is_dir(), "dir must exist and be a directory");

    let mut entries = Vec::new();
    collect_tree_entries(&dir, &dir, &mut entries)?;
    entries.sort_by(|a, b| a.relative.cmp(&b.relative));

    let mut hasher = Sha256::new();
    let mut files_hashed = 0u64;
    let mut total_bytes = 0u64;

    for entry in entries {
        hasher.update(entry.relative.as_bytes());
        hasher.update(b"\0");
        match entry.kind {
            TreeEntryKind::File { path, size } => {
                hasher.update(b"file\0");
                hasher.update(size.to_string().as_bytes());
                hasher.update(b"\0");
                let mut file = fs::File::open(&path)
                    .with_context(|| format!("failed to open file {}", path.display()))?;
                let mut buffer = [0u8; 16 * 1024];
                loop {
                    let read = file.read(&mut buffer)?;
                    if read == 0 {
                        break;
                    }
                    hasher.update(&buffer[..read]);
                }
                hasher.update(b"\0");
                files_hashed += 1;
                total_bytes += size;
            }
            TreeEntryKind::Symlink { target } => {
                hasher.update(b"symlink\0");
                let target = target.to_string_lossy();
                hasher.update(target.as_bytes());
                hasher.update(b"\0");
            }
        }
    }

    let digest = hasher.finalize();
    Ok(serde_json::json!({
        "sha256": format!("sha256:{}", to_hex(&digest)),
        "files_hashed": files_hashed,
        "total_bytes": total_bytes,
    }))
}

struct TreeEntry {
    relative: String,
    kind: TreeEntryKind,
}

enum TreeEntryKind {
    File { path: PathBuf, size: u64 },
    Symlink { target: PathBuf },
}

fn collect_tree_entries(root: &Path, dir: &Path, out: &mut Vec<TreeEntry>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        if let Some(name) = name.to_str() {
            if EXCLUDED_NAMES.contains(&name) {
                continue;
            }
        }

        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            out.push(TreeEntry {
                relative: relative_path(root, &path)?,
                kind: TreeEntryKind::Symlink {
                    target: fs::read_link(&path)?,
                },
            });
        } else if metadata.is_dir() {
            collect_tree_entries(root, &path, out)?;
        } else if metadata.is_file() {
            out.push(TreeEntry {
                relative: relative_path(root, &path)?,
                kind: TreeEntryKind::File {
                    path,
                    size: metadata.len(),
                },
            });
        }
    }
    Ok(())
}

fn relative_path(root: &Path, path: &Path) -> Result<String> {
    let relative = path.strip_prefix(root)?;
    Ok(relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/"))
}

fn compute_manifest_hash(request: &InprocInvocation) -> Result<Value> {
    let manifest_path = PathBuf::from(input_str(&request.input, "manifest_path")?);
    let raw = fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read manifest {}", manifest_path.display()))?;
    let value: Value = match manifest_path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => serde_json::from_str(&raw).or_else(|_| serde_yaml::from_str(&raw))?,
        _ => serde_yaml::from_str(&raw)?,
    };
    let canonical = canonicalize_json(value);
    let bytes = serde_json::to_vec(&canonical)?;
    Ok(serde_json::json!({ "sha256": sha256_prefixed(&bytes) }))
}

fn canonicalize_json(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let sorted: BTreeMap<String, Value> = map
                .into_iter()
                .map(|(key, value)| (key, canonicalize_json(value)))
                .collect();
            Value::Object(sorted.into_iter().collect())
        }
        Value::Array(values) => Value::Array(values.into_iter().map(canonicalize_json).collect()),
        other => other,
    }
}

fn verify_gpg_signature(request: &InprocInvocation) -> Result<Value> {
    let public_keys = request
        .input
        .get("public_keys")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if public_keys.is_empty() {
        return Ok(verify_error("no public keys provided"));
    }

    let data = match input_str(&request.input, "data").and_then(|data| {
        BASE64
            .decode(data)
            .map_err(|error| anyhow::anyhow!("invalid base64 data: {error}"))
    }) {
        Ok(data) => data,
        Err(error) => return Ok(verify_error(error.to_string())),
    };
    let signature = match input_str(&request.input, "signature") {
        Ok(signature) => signature,
        Err(error) => return Ok(verify_error(format!("invalid signature format: {error}"))),
    };

    let mut certs = Vec::new();
    for (idx, key) in public_keys.iter().enumerate() {
        let Some(key) = key.as_str() else {
            return Ok(verify_error(format!("invalid public key at index {idx}: expected armored string")));
        };
        match Cert::from_bytes(key.as_bytes()) {
            Ok(cert) => certs.push(cert),
            Err(error) => {
                return Ok(verify_error(format!(
                    "invalid public key at index {idx}: {error}"
                )));
            }
        }
    }

    let helper = IntegrityVerificationHelper::new(certs);
    let policy = StandardPolicy::new();
    let mut verifier = match DetachedVerifierBuilder::from_bytes(signature.as_bytes())
        .and_then(|builder| builder.with_policy(&policy, None, helper))
    {
        Ok(verifier) => verifier,
        Err(error) => {
            return Ok(verify_error(format!("invalid signature format: {error}")));
        }
    };

    match verifier.verify_bytes(&data) {
        Ok(()) => {
            let helper = verifier.into_helper();
            Ok(serde_json::json!({
                "verified": true,
                "key_fingerprint": helper.key_fingerprint,
                "signing_time": helper.signing_time,
                "error": Value::Null,
            }))
        }
        Err(error) => Ok(verify_error(error.to_string())),
    }
}

fn verify_error(message: impl Into<String>) -> Value {
    serde_json::json!({
        "verified": false,
        "key_fingerprint": Value::Null,
        "signing_time": Value::Null,
        "error": message.into(),
    })
}

struct IntegrityVerificationHelper {
    certs: Vec<Cert>,
    key_fingerprint: Option<String>,
    signing_time: Option<String>,
}

impl IntegrityVerificationHelper {
    fn new(certs: Vec<Cert>) -> Self {
        Self {
            certs,
            key_fingerprint: None,
            signing_time: None,
        }
    }
}

impl VerificationHelper for IntegrityVerificationHelper {
    fn get_certs(&mut self, ids: &[openpgp::KeyHandle]) -> openpgp::Result<Vec<Cert>> {
        Ok(self
            .certs
            .iter()
            .filter(|cert| {
                ids.is_empty()
                    || cert
                        .keys()
                        .any(|key| ids.iter().any(|id| key.key().key_handle().aliases(id)))
            })
            .cloned()
            .collect())
    }

    fn check(&mut self, structure: MessageStructure) -> openpgp::Result<()> {
        for layer in structure.into_iter() {
            if let MessageLayer::SignatureGroup { results } = layer {
                for result in results {
                    if let Ok(good) = result {
                        self.key_fingerprint = Some(good.ka.key().fingerprint().to_string());
                        self.signing_time = good
                            .sig
                            .signature_creation_time()
                            .map(system_time_to_rfc3339);
                        return Ok(());
                    }
                }
            }
        }
        Err(anyhow::anyhow!("no valid signature"))
    }
}

fn system_time_to_rfc3339(time: SystemTime) -> String {
    let datetime: chrono::DateTime<chrono::Utc> = time
        .duration_since(UNIX_EPOCH)
        .map(|duration| chrono::DateTime::<chrono::Utc>::from(UNIX_EPOCH + duration))
        .unwrap_or_else(|_| chrono::DateTime::<chrono::Utc>::from(UNIX_EPOCH));
    datetime.to_rfc3339()
}

fn fingerprint_public_key(request: &InprocInvocation) -> Result<Value> {
    let public_key = input_str(&request.input, "public_key")?;
    let cert = Cert::from_bytes(public_key.as_bytes())?;
    let user_ids: Vec<String> = cert
        .userids()
        .map(|userid| String::from_utf8_lossy(userid.userid().value()).to_string())
        .collect();
    Ok(serde_json::json!({
        "fingerprint": cert.fingerprint().to_string(),
        "user_ids": user_ids,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    use openpgp::armor;
    use openpgp::cert::prelude::*;
    use openpgp::serialize::stream::{Armorer, Message, Signer};
    use openpgp::serialize::SerializeInto;

    fn request(capability_id: &str, input: Value) -> InprocInvocation {
        InprocInvocation {
            capability_id: capability_id.to_string(),
            provider_package_id: PACKAGE_ID.to_string(),
            input,
        }
    }

    #[test]
    fn manifest_hash_yaml_json_equivalent_unit() -> Result<()> {
        let yaml: Value = serde_yaml::from_str("b: 2\na: 1\n")?;
        let json: Value = serde_json::from_str(r#"{"a":1,"b":2}"#)?;
        assert_eq!(canonicalize_json(yaml), canonicalize_json(json));
        Ok(())
    }

    #[test]
    fn gpg_signature_roundtrip_unit() -> Result<()> {
        let data = b"integrity-lab runtime unit data";
        let (cert, _) = CertBuilder::general_purpose(None, Some("Runtime Test <runtime@example.test>"))
            .generate()?;
        let policy = StandardPolicy::new();
        let signing_keypair = cert
            .keys()
            .secret()
            .with_policy(&policy, None)
            .supported()
            .alive()
            .revoked(false)
            .for_signing()
            .next()
            .context("missing signing key")?
            .key()
            .clone()
            .into_keypair()?;
        let signing_fingerprint = signing_keypair.public().fingerprint().to_string();
        let mut signature = Vec::new();
        {
            let message = Message::new(&mut signature);
            let message = Armorer::new(message).kind(armor::Kind::Signature).build()?;
            let mut signer = Signer::new(message, signing_keypair).detached().build()?;
            signer.write_all(data)?;
            signer.finalize()?;
        }

        let public_key = String::from_utf8(cert.armored().to_vec()?)?;
        let output = verify_gpg_signature(&request(
            "integrity.verify_gpg_signature",
            serde_json::json!({
                "data": BASE64.encode(data),
                "signature": String::from_utf8(signature)?,
                "public_keys": [public_key],
            }),
        ))?;
        assert_eq!(output["verified"], true);
        assert_eq!(output["key_fingerprint"], signing_fingerprint);
        Ok(())
    }
}
