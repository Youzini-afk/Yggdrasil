//! Handler for `official/git-tools-lab` capabilities.
//!
//! Pure-Rust git operations used by package installation. All gix operations
//! are blocking and are executed through `tokio::task::spawn_blocking`.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use anyhow::{Context, Result};
use gix::bstr::ByteSlice;
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use super::InprocInvocation;

const PACKAGE_ID: &str = "official/git-tools-lab";

#[derive(Debug, Deserialize)]
struct ResolveRefInput {
    remote_url: String,
    #[serde(rename = "ref")]
    ref_name: String,
}

#[derive(Debug, Deserialize)]
struct FetchRefsInput {
    remote_url: String,
}

#[derive(Debug, Deserialize)]
struct FetchTreeInput {
    remote_url: String,
    commit_sha: String,
    dest_dir: String,
}

#[derive(Debug, Deserialize)]
struct ReadSignedTagInput {
    remote_url: String,
    tag: String,
}

#[derive(Debug, Clone)]
struct RemoteRef {
    name: String,
    sha: String,
    kind: RefKind,
    tag_object: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RefKind {
    Branch,
    Tag,
}

impl RefKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Branch => "branch",
            Self::Tag => "tag",
        }
    }
}

#[derive(Debug, Default)]
struct TreeWriteStats {
    files_written: u64,
    total_bytes: u64,
}

pub fn try_handle(request: &InprocInvocation) -> Option<anyhow::Result<Value>> {
    if request.provider_package_id != PACKAGE_ID {
        return None;
    }
    let id = request.capability_id.as_str();
    if id.ends_with("/resolve_ref") || id == "git.resolve_ref" {
        Some(async_blocking(resolve_ref, request.input.clone()))
    } else if id.ends_with("/fetch_refs") || id == "git.fetch_refs" {
        Some(async_blocking(fetch_refs, request.input.clone()))
    } else if id.ends_with("/fetch_tree") || id == "git.fetch_tree" {
        Some(async_blocking(fetch_tree, request.input.clone()))
    } else if id.ends_with("/read_signed_tag") || id == "git.read_signed_tag" {
        Some(async_blocking(read_signed_tag, request.input.clone()))
    } else {
        None
    }
}

fn async_blocking(f: fn(Value) -> Result<Value>, input: Value) -> Result<Value> {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(tokio::task::spawn_blocking(move || f(input)))
            .context("git task panicked or was cancelled")?
    })
}

fn resolve_ref(input: Value) -> Result<Value> {
    let input: ResolveRefInput = serde_json::from_value(input)?;
    validate_remote_url(&input.remote_url)?;

    if is_full_sha(&input.ref_name) {
        return Ok(serde_json::json!({
            "commit_sha": input.ref_name,
            "ref_kind": "commit",
            "ref_name": input.ref_name,
        }));
    }

    let refs = list_remote_refs_blocking(&input.remote_url)?;
    let wanted = input.ref_name.trim();
    let candidates = [
        wanted.to_string(),
        format!("refs/heads/{wanted}"),
        format!("refs/tags/{wanted}"),
    ];

    let resolved = refs
        .iter()
        .find(|remote_ref| candidates.iter().any(|candidate| candidate == &remote_ref.name))
        .with_context(|| format!("ref '{wanted}' not found on remote"))?;

    Ok(serde_json::json!({
        "commit_sha": resolved.sha,
        "ref_kind": resolved.kind.as_str(),
        "ref_name": resolved.name,
    }))
}

fn fetch_refs(input: Value) -> Result<Value> {
    let input: FetchRefsInput = serde_json::from_value(input)?;
    validate_remote_url(&input.remote_url)?;
    let refs = list_remote_refs_blocking(&input.remote_url)?;
    let refs: Vec<Value> = refs
        .into_iter()
        .map(|remote_ref| {
            serde_json::json!({
                "name": remote_ref.name,
                "sha": remote_ref.sha,
                "kind": remote_ref.kind.as_str(),
            })
        })
        .collect();
    Ok(serde_json::json!({ "refs": refs }))
}

fn fetch_tree(input: Value) -> Result<Value> {
    let input: FetchTreeInput = serde_json::from_value(input)?;
    validate_remote_url(&input.remote_url)?;
    let dest = PathBuf::from(&input.dest_dir);
    validate_dest_dir(&dest)?;
    if dest.exists() {
        anyhow::bail!("dest_dir already exists: {}", dest.display());
    }
    if !is_full_sha(&input.commit_sha) {
        anyhow::bail!("commit_sha must be a 40-character hex SHA");
    }

    let parent = dest
        .parent()
        .with_context(|| format!("dest_dir has no parent: {}", dest.display()))?;
    let file_name = dest
        .file_name()
        .and_then(|name| name.to_str())
        .with_context(|| format!("dest_dir must end in a valid UTF-8 directory name: {}", dest.display()))?;
    fs::create_dir_all(parent)?;
    let tmp = parent.join(format!("{file_name}.tmp.{}", Uuid::new_v4()));

    let outcome = (|| -> Result<Value> {
        let repo_tmp = parent.join(format!("{file_name}.repo.tmp.{}", Uuid::new_v4()));
        let repo = clone_shallow(&input.remote_url, &input.commit_sha, &repo_tmp)?;
        let commit_id = gix::ObjectId::from_hex(input.commit_sha.as_bytes())?;
        let commit = repo.find_object(commit_id)?.peel_to_commit()?;
        let tree = commit.tree()?;
        let tree_hash = tree.id.to_string();

        fs::create_dir(&tmp)?;
        let stats = write_tree_recursive(&tree, &tmp)
            .with_context(|| format!("failed to write tree to {}", tmp.display()))?;
        fs::rename(&tmp, &dest).with_context(|| {
            format!(
                "failed to atomically rename {} to {}",
                tmp.display(),
                dest.display()
            )
        })?;
        fs::remove_dir_all(&repo_tmp).ok();
        Ok(serde_json::json!({
            "files_written": stats.files_written,
            "total_bytes": stats.total_bytes,
            "tree_hash": tree_hash,
        }))
    })();

    if outcome.is_err() {
        fs::remove_dir_all(&tmp).ok();
    }
    outcome
}

fn read_signed_tag(input: Value) -> Result<Value> {
    let input: ReadSignedTagInput = serde_json::from_value(input)?;
    validate_remote_url(&input.remote_url)?;
    let refs = list_remote_refs_blocking(&input.remote_url)?;
    let wanted = input.tag.trim();
    let candidates = [wanted.to_string(), format!("refs/tags/{wanted}")];
    let tag_ref = refs
        .iter()
        .find(|remote_ref| {
            remote_ref.kind == RefKind::Tag
                && candidates.iter().any(|candidate| candidate == &remote_ref.name)
        })
        .with_context(|| format!("tag '{wanted}' not found on remote"))?;

    let object_to_fetch = tag_ref.tag_object.as_deref().unwrap_or(&tag_ref.sha);
    let tmp = std::env::temp_dir().join(format!("ygg-git-tag-{}", Uuid::new_v4()));
    let repo = clone_shallow(&input.remote_url, object_to_fetch, &tmp)?;
    let output = if let Some(tag_object) = &tag_ref.tag_object {
        let id = gix::ObjectId::from_hex(tag_object.as_bytes())?;
        let tag = repo.find_object(id)?.try_into_tag()?;
        let decoded = tag.decode()?;
        let signed_data = signed_data_before_pgp(&tag.data);
        serde_json::json!({
            "tag_object": tag_object,
            "pgp_signature": decoded.pgp_signature.map(bstr_to_string),
            "signed_data": bytes_to_string(signed_data),
            "tagger": decoded.tagger()?.map(signature_to_json),
            "message": bstr_to_string(decoded.message),
        })
    } else {
        let id = gix::ObjectId::from_hex(tag_ref.sha.as_bytes())?;
        let commit = repo.find_object(id)?.peel_to_commit()?;
        let decoded = commit.decode()?;
        serde_json::json!({
            "tag_object": Value::Null,
            "pgp_signature": Value::Null,
            "signed_data": bytes_to_string(&commit.data),
            "tagger": signature_to_json(decoded.committer()?),
            "message": bstr_to_string(decoded.message),
        })
    };
    fs::remove_dir_all(&tmp).ok();
    Ok(output)
}

fn list_remote_refs_blocking(remote_url: &str) -> Result<Vec<RemoteRef>> {
    let tmp = std::env::temp_dir().join(format!("ygg-git-refs-{}", Uuid::new_v4()));
    let result = (|| -> Result<Vec<RemoteRef>> {
        let repo = gix::init_bare(&tmp)?;
        let remote = repo.remote_at(remote_url)?.with_refspecs(
            ["+refs/heads/*:refs/remotes/origin/*", "+refs/tags/*:refs/tags/*"],
            gix::remote::Direction::Fetch,
        )?;
        let connection = remote.connect(gix::remote::Direction::Fetch)?;
        let (ref_map, _) = connection.ref_map(gix::progress::Discard, Default::default())?;
        let refs = ref_map
            .remote_refs
            .iter()
            .filter_map(remote_ref_from_gix)
            .collect();
        Ok(refs)
    })();
    fs::remove_dir_all(&tmp).ok();
    result
}

fn remote_ref_from_gix(remote_ref: &gix::protocol::handshake::Ref) -> Option<RemoteRef> {
    match remote_ref {
        gix::protocol::handshake::Ref::Peeled {
            full_ref_name,
            tag,
            object,
        } => classify_remote_ref(full_ref_name.as_bstr(), object.to_string(), Some(tag.to_string())),
        gix::protocol::handshake::Ref::Direct {
            full_ref_name,
            object,
        } => classify_remote_ref(full_ref_name.as_bstr(), object.to_string(), None),
        gix::protocol::handshake::Ref::Symbolic {
            full_ref_name,
            tag,
            object,
            ..
        } => classify_remote_ref(
            full_ref_name.as_bstr(),
            object.to_string(),
            tag.as_ref().map(ToString::to_string),
        ),
        gix::protocol::handshake::Ref::Unborn { .. } => None,
    }
}

fn classify_remote_ref(
    full_ref_name: &gix::bstr::BStr,
    sha: String,
    tag_object: Option<String>,
) -> Option<RemoteRef> {
    let name = bstr_to_string(full_ref_name);
    if name.starts_with("refs/heads/") {
        Some(RemoteRef {
            name,
            sha,
            kind: RefKind::Branch,
            tag_object: None,
        })
    } else if name.starts_with("refs/tags/") {
        Some(RemoteRef {
            name,
            sha,
            kind: RefKind::Tag,
            tag_object,
        })
    } else {
        None
    }
}

fn clone_shallow(remote_url: &str, ref_name: &str, path: &Path) -> Result<gix::Repository> {
    let mut prep = gix::prepare_clone(remote_url, path)?
        .with_ref_name(Some(ref_name))?
        .with_shallow(gix::remote::fetch::Shallow::DepthAtRemote(
            std::num::NonZeroU32::new(1).expect("non-zero"),
        ))
        .configure_remote(|remote| {
            remote
                .with_refspecs(
                    ["+refs/heads/*:refs/remotes/origin/*", "+refs/tags/*:refs/tags/*"],
                    gix::remote::Direction::Fetch,
                )
                .map_err(|err| Box::new(err) as Box<dyn std::error::Error + Send + Sync>)
        });
    let interrupt = AtomicBool::new(false);
    let (repo, _) = prep.fetch_only(gix::progress::Discard, &interrupt)?;
    Ok(repo)
}

fn write_tree_recursive(tree: &gix::Tree<'_>, dest: &Path) -> Result<TreeWriteStats> {
    let mut stats = TreeWriteStats::default();
    for entry in tree.iter() {
        let entry = entry?;
        let name = bstr_to_string(entry.filename());
        if name == ".git" || name.contains('/') || name.contains('\\') || name == ".." {
            anyhow::bail!("unsafe tree entry name: {name}");
        }
        let out = dest.join(&name);
        if entry.mode().is_tree() {
            fs::create_dir(&out)?;
            let child = entry.object()?.try_into_tree()?;
            let child_stats = write_tree_recursive(&child, &out)?;
            stats.files_written += child_stats.files_written;
            stats.total_bytes += child_stats.total_bytes;
        } else if entry.mode().is_blob_or_symlink() {
            let blob = entry.object()?.try_into_blob()?;
            if entry.mode().is_link() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::symlink;
                    let target = bytes_to_string(&blob.data);
                    symlink(target, &out)?;
                }
                #[cfg(not(unix))]
                {
                    let mut file = fs::File::create(&out)?;
                    file.write_all(&blob.data)?;
                }
            } else {
                let mut file = fs::File::create(&out)?;
                file.write_all(&blob.data)?;
            }
            stats.files_written += 1;
            stats.total_bytes += blob.data.len() as u64;
        }
    }
    Ok(stats)
}

fn validate_remote_url(url: &str) -> Result<()> {
    let parsed = url::Url::parse(url)?;
    if parsed.scheme() != "https" {
        anyhow::bail!("only HTTPS URLs supported, got: {}", parsed.scheme());
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        anyhow::bail!("URL must not contain userinfo");
    }
    if parsed.host_str().is_none() {
        anyhow::bail!("URL must have a host");
    }
    Ok(())
}

fn validate_dest_dir(dest: &Path) -> Result<()> {
    if !dest.is_absolute() {
        anyhow::bail!("dest_dir must be absolute, got: {}", dest.display());
    }
    for component in dest.components() {
        if matches!(component, std::path::Component::ParentDir) {
            anyhow::bail!("dest_dir must not contain ..");
        }
    }
    Ok(())
}

fn is_full_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn signed_data_before_pgp(data: &[u8]) -> &[u8] {
    const MARKER: &[u8] = b"-----BEGIN PGP SIGNATURE-----";
    match data.windows(MARKER.len()).position(|window| window == MARKER) {
        Some(0) => &data[..0],
        Some(pos) if data.get(pos.wrapping_sub(1)) == Some(&b'\n') => &data[..pos - 1],
        Some(pos) => &data[..pos],
        None => data,
    }
}

fn signature_to_json(signature: gix::actor::SignatureRef<'_>) -> Value {
    serde_json::json!({
        "name": bstr_to_string(signature.name),
        "email": bstr_to_string(signature.email),
        "date": signature.time,
    })
}

fn bstr_to_string(value: &gix::bstr::BStr) -> String {
    String::from_utf8_lossy(value.as_ref()).into_owned()
}

fn bytes_to_string(value: &[u8]) -> String {
    String::from_utf8_lossy(value).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_https_remote_urls() {
        for url in [
            "ssh://github.com/example/repo.git",
            "git://github.com/example/repo.git",
            "file:///tmp/repo.git",
        ] {
            assert!(validate_remote_url(url).is_err(), "accepted {url}");
        }
    }

    #[test]
    fn rejects_remote_url_userinfo() {
        assert!(validate_remote_url("https://user:pass@example.com/repo.git").is_err());
    }

    #[test]
    fn rejects_relative_dest_dir() {
        assert!(validate_dest_dir(Path::new("relative/path")).is_err());
    }

    #[test]
    fn rejects_parent_components() {
        assert!(validate_dest_dir(Path::new("/tmp/../repo")).is_err());
    }
}
