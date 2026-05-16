//! Capability fabric placeholders.
//!
//! The first runtime slice does not implement the full open capability fabric yet.
//! This module exists to keep the boundary visible: plugins, external engines,
//! sidecars, WASM modules, and pi agents should eventually integrate as
//! capability providers instead of calling private runtime internals.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDescriptor {
    pub id: String,
    pub name: String,
    pub version: String,
    pub provider_id: String,
    pub description: String,
    pub streaming: bool,
    pub side_effects: Vec<String>,
    pub permissions_required: Vec<String>,
}
