use super::{case, ConformanceCase};

pub(super) fn cases() -> Vec<ConformanceCase> {
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- inference local ---
        c!(
            "official.inference_local_lab_describe_capabilities",
            ["official", "slow"],
            crate::conformance::inference_local::inference_local_lab_describe_capabilities
        ),
        c!(
            "official.inference_local_lab_invoke",
            ["official", "slow"],
            crate::conformance::inference_local::inference_local_lab_invoke
        ),
        c!(
            "official.inference_local_lab_invoke_rejects_http",
            ["official", "slow"],
            crate::conformance::inference_local::inference_local_lab_invoke_rejects_http
        ),
        c!(
            "official.inference_local_lab_stream",
            ["official", "slow"],
            crate::conformance::inference_local::inference_local_lab_stream
        ),
        c!(
            "official.inference_local_lab_explain_error",
            ["official", "slow"],
            crate::conformance::inference_local::inference_local_lab_explain_error
        ),
    ]
}
