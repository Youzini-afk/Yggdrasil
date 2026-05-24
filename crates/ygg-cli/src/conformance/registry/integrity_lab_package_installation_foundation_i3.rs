use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- integrity-lab (Package Installation Foundation I3) ---
        c!(
            "integrity.tree_hash_deterministic",
            ["official", "integrity", "package_install"],
            crate::conformance::integrity_tools::tree_hash_deterministic
        ),
        c!(
            "integrity.tree_hash_excludes_metadata",
            ["official", "integrity", "package_install"],
            crate::conformance::integrity_tools::tree_hash_excludes_metadata
        ),
        c!(
            "integrity.manifest_hash_yaml_json_equivalent",
            ["official", "integrity", "package_install"],
            crate::conformance::integrity_tools::manifest_hash_yaml_json_equivalent
        ),
        c!(
            "integrity.gpg_verify_valid_signature",
            ["official", "integrity", "package_install"],
            crate::conformance::integrity_tools::gpg_verify_valid_signature
        ),
        c!(
            "integrity.gpg_verify_wrong_key_fails",
            ["official", "integrity", "package_install"],
            crate::conformance::integrity_tools::gpg_verify_wrong_key_fails
        ),
        c!(
            "integrity.gpg_verify_invalid_signature_no_panic",
            ["official", "integrity", "package_install"],
            crate::conformance::integrity_tools::gpg_verify_invalid_signature_no_panic
        ),
        c!(
            "integrity.fingerprint_extraction_consistent",
            ["official", "integrity", "package_install"],
            crate::conformance::integrity_tools::fingerprint_extraction_consistent
        ),
    ]
}
