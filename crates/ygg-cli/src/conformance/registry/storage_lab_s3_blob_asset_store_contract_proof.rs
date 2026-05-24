use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- storage-lab S3 (Blob / Asset Store Contract Proof) ---
        c!(
            "storage_lab.blob_contract_shape",
            ["storage_lab", "storage", "blob"],
            crate::conformance::storage_lab::blob_contract_shape
        ),
        c!(
            "storage_lab.put_blob_preview_content_address_deterministic",
            ["storage_lab", "storage", "blob"],
            crate::conformance::storage_lab::put_blob_preview_content_address_deterministic
        ),
        c!(
            "storage_lab.put_blob_preview_no_storage_no_content_event",
            ["storage_lab", "storage", "blob"],
            crate::conformance::storage_lab::put_blob_preview_no_storage_no_content_event
        ),
        c!(
            "storage_lab.get_blob_metadata_preview_no_content",
            ["storage_lab", "storage", "blob"],
            crate::conformance::storage_lab::get_blob_metadata_preview_no_content
        ),
        c!(
            "storage_lab.export_blob_manifest_refs_only",
            ["storage_lab", "storage", "blob"],
            crate::conformance::storage_lab::export_blob_manifest_refs_only
        ),
        c!(
            "storage_lab.blob_raw_secret_and_unsafe_id_rejected",
            ["storage_lab", "storage", "blob", "secret"],
            crate::conformance::storage_lab::blob_raw_secret_and_unsafe_id_rejected
        ),
    ]
}
