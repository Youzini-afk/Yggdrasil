//! Tavern runtime placeholders.
//!
//! Tavern compatibility is a runtime profile, not the kernel. The first thin
//! slice should import Character Card V2 into a native asset/actor projection
//! while preserving the original payload.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TavernImportPlan {
    pub preserve_original_payload: bool,
    pub create_native_projection: bool,
    pub supported_resource: String,
}

impl TavernImportPlan {
    pub fn character_card_v2() -> Self {
        Self {
            preserve_original_payload: true,
            create_native_projection: true,
            supported_resource: "sillytavern.character_card_v2".to_string(),
        }
    }
}
