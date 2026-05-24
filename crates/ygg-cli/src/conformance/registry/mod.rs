#![allow(dead_code)]

use crate::conformance::runner::{CaseFn, ConformanceCase};

mod agentic_forge_phase_a;
mod agentic_forge_phase_b;
mod agentic_forge_phase_c;
mod agentic_forge_phase_d;
mod agentic_forge_phase_f;
mod asset;
mod composition;
mod core;
mod creator_loop_beta_5;
mod experience_observability_beta_3;
mod experience_runtime;
mod generated_packages;
mod git_tools_lab_package_installation_foundation_i2;
mod hooks;
mod host;
mod inference_local;
mod inference_playtest;
mod inproc;
mod install_lab_package_installation_foundation_i4;
mod integrity_lab_package_installation_foundation_i3;
mod kernel_v1_outbound_execute;
mod live_http_outbound;
mod live_model;
mod live_model_providers;
mod live_model_quirks;
mod memory_lab_beta_4;
mod network;
mod official_foundation_labs;
mod outbound;
mod package_check_reload;
mod permissions;
mod playable_board_beta_1;
mod playable_board_beta_2;
mod project_intake_lab_e5_external_project_operating_plane_alpha_e5_adapte;
mod project_intake_lab_external_project_operating_plane_alpha_e1;
mod project_lifecycle_round_10a_2_wave_3;
mod project_scoped_secrets_round_10a_2_wave_2c;
mod projection;
mod proposals;
mod protocol;
mod real_tdb_rust_adapter_subprocess_proof;
mod replacement;
mod secret_conformance;
mod secret_store_lab_round_10a_1_phase_b;
mod session_fork;
mod sharing_lab_beta_6;
mod storage_backend_neutrality_s1;
mod storage_backend_neutrality_s1_postgresql_opt_in;
mod storage_lab_s3_blob_asset_store_contract_proof;
mod storage_lab_s4_projection_index_materialization_contract_proof;
mod storage_lab_s5_retrieval_vector_multimodal_provider_contract;
mod storage_lab_storage_backend_neutrality_alpha_s2;
mod streaming;
mod subprocess;
mod subprocess_error_cases;
mod substrate;
mod surfaces;
mod workspace_lab_e3_external_project_operating_plane_alpha_e3_managed_wor;
mod workspace_lab_external_project_operating_plane_alpha_e2;
mod y1_outbound_execute_profile_conformance;
mod y2_manifest_permissions_secret_refs_conformance;
mod y3_kernel_v1_outbound_stream_conformance;

pub(super) fn case(
    id: &'static str,
    tags: &'static [&'static str],
    run: CaseFn,
) -> ConformanceCase {
    ConformanceCase { id, tags, run }
}

/// Build the full ordered list of conformance cases.
pub(super) fn build_cases() -> Vec<ConformanceCase> {
    let mut cases = Vec::new();
    cases.extend(core::cases());
    cases.extend(permissions::cases());
    cases.extend(subprocess::cases());
    cases.extend(host::cases());
    cases.extend(surfaces::cases());
    cases.extend(official_foundation_labs::cases());
    cases.extend(proposals::cases());
    cases.extend(asset::cases());
    cases.extend(session_fork::cases());
    cases.extend(projection::cases());
    cases.extend(substrate::cases());
    cases.extend(subprocess_error_cases::cases());
    cases.extend(protocol::cases());
    cases.extend(package_check_reload::cases());
    cases.extend(hooks::cases());
    cases.extend(generated_packages::cases());
    cases.extend(composition::cases());
    cases.extend(inproc::cases());
    cases.extend(replacement::cases());
    cases.extend(secret_conformance::cases());
    cases.extend(network::cases());
    cases.extend(outbound::cases());
    cases.extend(live_http_outbound::cases());
    cases.extend(kernel_v1_outbound_execute::cases());
    cases.extend(y1_outbound_execute_profile_conformance::cases());
    cases.extend(y2_manifest_permissions_secret_refs_conformance::cases());
    cases.extend(y3_kernel_v1_outbound_stream_conformance::cases());
    cases.extend(streaming::cases());
    cases.extend(live_model::cases());
    cases.extend(live_model_providers::cases());
    cases.extend(live_model_quirks::cases());
    cases.extend(inference_local::cases());
    cases.extend(inference_playtest::cases());
    cases.extend(agentic_forge_phase_a::cases());
    cases.extend(agentic_forge_phase_b::cases());
    cases.extend(agentic_forge_phase_c::cases());
    cases.extend(agentic_forge_phase_d::cases());
    cases.extend(agentic_forge_phase_f::cases());
    cases.extend(experience_runtime::cases());
    cases.extend(playable_board_beta_1::cases());
    cases.extend(playable_board_beta_2::cases());
    cases.extend(experience_observability_beta_3::cases());
    cases.extend(memory_lab_beta_4::cases());
    cases.extend(creator_loop_beta_5::cases());
    cases.extend(project_intake_lab_external_project_operating_plane_alpha_e1::cases());
    cases.extend(project_intake_lab_e5_external_project_operating_plane_alpha_e5_adapte::cases());
    cases.extend(workspace_lab_external_project_operating_plane_alpha_e2::cases());
    cases.extend(workspace_lab_e3_external_project_operating_plane_alpha_e3_managed_wor::cases());
    cases.extend(integrity_lab_package_installation_foundation_i3::cases());
    cases.extend(secret_store_lab_round_10a_1_phase_b::cases());
    cases.extend(project_scoped_secrets_round_10a_2_wave_2c::cases());
    cases.extend(project_lifecycle_round_10a_2_wave_3::cases());
    cases.extend(sharing_lab_beta_6::cases());
    cases.extend(storage_backend_neutrality_s1::cases());
    cases.extend(storage_backend_neutrality_s1_postgresql_opt_in::cases());
    cases.extend(storage_lab_storage_backend_neutrality_alpha_s2::cases());
    cases.extend(storage_lab_s3_blob_asset_store_contract_proof::cases());
    cases.extend(storage_lab_s4_projection_index_materialization_contract_proof::cases());
    cases.extend(storage_lab_s5_retrieval_vector_multimodal_provider_contract::cases());
    cases.extend(git_tools_lab_package_installation_foundation_i2::cases());
    cases.extend(install_lab_package_installation_foundation_i4::cases());
    cases.extend(real_tdb_rust_adapter_subprocess_proof::cases());
    cases
}
