//! Handler for `official/install-lab` capabilities.
//!
//! Orchestrates package installation by composing git-tools-lab and
//! integrity-lab through normal capability dispatch.

use anyhow::Result;
use serde_json::Value;
use std::path::Path;

use super::InprocInvocation;

mod executor;
mod fs_copy;
mod gc;
mod layout;
mod planner;
mod project_kind;
mod source;
mod types;
mod update_check;

const PACKAGE_ID: &str = "official/install-lab";

pub use layout::StoreSchemaMigration;

pub fn ensure_store_schema(data_dir: &Path) -> Result<Option<StoreSchemaMigration>> {
    layout::ensure_store_schema(data_dir)
}

pub async fn try_handle(request: &InprocInvocation) -> Option<Result<Value>> {
    if request.provider_package_id != PACKAGE_ID {
        return None;
    }
    match request.capability_id.as_str() {
        "install.resolve_plan" | "official/install-lab/resolve_plan" => {
            Some(planner::resolve_plan(request.input.clone()).await)
        }
        "install.execute_plan" | "official/install-lab/execute_plan" => {
            Some(executor::execute_plan(request.input.clone(), request.session_id.as_deref()).await)
        }
        "install.detect_kind" | "official/install-lab/detect_kind" => {
            Some(project_kind::detect_kind(request.input.clone()).await)
        }
        "install.register_project" | "official/install-lab/register_project" => {
            Some(executor::register_project_capability(request.input.clone()).await)
        }
        "install.uninstall" | "official/install-lab/uninstall" => {
            Some(executor::uninstall(request.input.clone(), request.session_id.as_deref()).await)
        }
        "install.list_installed" | "official/install-lab/list_installed" => {
            Some(executor::list_installed(request.input.clone()).await)
        }
        "install.check_lockfile" | "official/install-lab/check_lockfile" => {
            Some(executor::check_lockfile(request.input.clone()).await)
        }
        "install.check_for_updates" | "official/install-lab/check_for_updates" => {
            Some(update_check::check_for_updates(request.input.clone()).await)
        }
        _ => None,
    }
}
