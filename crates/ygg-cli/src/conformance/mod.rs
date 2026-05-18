mod core;
mod fixtures;
mod generated;
mod hooks;
mod inproc;
mod network;
mod official_foundation;
mod official_labs;
mod official_play_creation;
mod permissions;
mod proposals;
mod protocol;
mod replacement;
mod secret_conformance;
mod streaming;
mod subprocess;
mod substrate;
mod surfaces;

fn record_case(results: &mut Vec<(&'static str, anyhow::Result<()>)>, name: &'static str, result: anyhow::Result<()>) {
    results.push((name, result));
}

pub(crate) async fn run() -> anyhow::Result<()> {
    let mut results = Vec::new();
    record_case(&mut results, "session.open_empty", core::session_open().await);
    record_case(&mut results, "event.append_authorized", core::event_append_authorized().await);
    record_case(
        &mut results,
        "event.append_without_permission_denied",
        core::event_append_without_permission_denied().await,
    );
    record_case(
        &mut results,
        "event.kernel_namespace_denied",
        core::kernel_namespace_denied().await,
    );
    record_case(
        &mut results,
        "event.read_without_permission_denied",
        core::event_read_without_permission_denied().await,
    );
    record_case(
        &mut results,
        "event.closed_session_rejects_append",
        core::closed_session_rejects_append().await,
    );
    record_case(&mut results, "event.range_replay", core::event_range_replay().await);
    record_case(&mut results, "capability.invoke_rust_inproc", core::capability_invoke().await);
    record_case(
        &mut results,
        "capability.ambiguous_provider_denied",
        core::ambiguous_provider_denied().await,
    );
    record_case(
        &mut results,
        "capability.explicit_provider_selected",
        core::explicit_provider_selected().await,
    );
    record_case(
        &mut results,
        "package.unload_removes_capability",
        core::unload_removes_capability().await,
    );
    record_case(&mut results, "official.no_privilege", core::official_no_privilege().await);
    record_case(
        &mut results,
        "schema.capability_input_rejects_invalid",
        core::capability_schema_rejects_invalid().await,
    );
    record_case(
        &mut results,
        "schema.event_payload_rejects_invalid",
        core::event_schema_rejects_invalid().await,
    );
    record_case(
        &mut results,
        "protocol.structured_permission_error",
        permissions::structured_permission_error().await,
    );
    record_case(&mut results, "permission.grant_revoke_audit", permissions::permission_grant_revoke_audit().await);
    record_case(&mut results, "permission.assistant_capability_grant", permissions::assistant_capability_grant().await);
    record_case(
        &mut results,
        "principal.package_cannot_self_assert_writer",
        permissions::principal_cannot_self_assert_writer().await,
    );
    record_case(
        &mut results,
        "principal.package_cannot_self_assert_capability_caller",
        permissions::principal_cannot_self_assert_capability_caller().await,
    );
    record_case(&mut results, "subprocess.load_ready", subprocess::subprocess_load_ready().await);
    record_case(&mut results, "subprocess.invoke_echo", subprocess::subprocess_invoke_echo().await);
    record_case(&mut results, "package.lifecycle_timeline", subprocess::package_lifecycle_timeline().await);
    record_case(&mut results, "package.logs_capture", subprocess::package_logs_capture().await);
    record_case(&mut results, "package.restart_subprocess", subprocess::package_restart_subprocess().await);
    record_case(&mut results, "host.diagnostics", core::host_diagnostics().await);
    record_case(&mut results, "host.profile_autoload", core::host_profile_autoload().await);
    record_case(&mut results, "surface.contribution_list", surfaces::contribution_list().await);
    record_case(&mut results, "official.foundation_packages", official_foundation::foundation_packages().await);
    record_case(&mut results, "official.assistant_lab_proposal", official_labs::assistant_lab_proposal().await);
    record_case(&mut results, "play_creation.blank_loop", official_play_creation::blank_play_creation_loop().await);
    record_case(&mut results, "proposal.lifecycle_apply", proposals::lifecycle_apply().await);
    record_case(&mut results, "proposal.reject_and_apply_denied", proposals::reject_and_apply_denied().await);
    record_case(&mut results, "asset.put_get_list", core::asset_put_get_list().await);
    record_case(&mut results, "session.fork_branch", core::session_fork_branch().await);
    record_case(&mut results, "projection.rebuild", core::projection_rebuild().await);
    record_case(&mut results, "substrate.sqlite_rehydrate", substrate::sqlite_rehydrate().await);
    record_case(&mut results, "subprocess.bad_handshake", subprocess::subprocess_bad_handshake().await);
    record_case(&mut results, "subprocess.invoke_timeout", subprocess::subprocess_timeout().await);
    record_case(
        &mut results,
        "subprocess.invalid_output_schema",
        subprocess::subprocess_invalid_output_schema().await,
    );
    record_case(
        &mut results,
        "subprocess.unload_removes_capability",
        subprocess::subprocess_unload_removes_capability().await,
    );
    record_case(&mut results, "protocol.call_host_info", protocol::call_host_info().await);
    record_case(
        &mut results,
        "protocol.call_capability_in_process",
        protocol::call_capability_in_process().await,
    );
    record_case(
        &mut results,
        "package.check_diagnostics",
        subprocess::package_check_diagnostics().await,
    );
    record_case(
        &mut results,
        "package.reload_smoke",
        subprocess::package_reload_smoke().await,
    );
    record_case(&mut results, "hook.ordering_stable", hooks::ordering_stable().await);
    record_case(&mut results, "hook.veto_blocks_event_append", hooks::veto_blocks_event_append().await);
    record_case(&mut results, "hook.metadata_mutation_allowed", hooks::metadata_mutation_allowed().await);
    record_case(&mut results, "hook.package_owned_handler", hooks::package_owned_handler().await);
    record_case(&mut results, "hook.unload_removes_subscription", hooks::unload_removes_subscription().await);
    record_case(
        &mut results,
        "package.generated_subprocess_conformance",
        generated::generated_subprocess_package().await,
    );
    record_case(
        &mut results,
        "package.generated_typescript_subprocess_conformance",
        generated::generated_typescript_subprocess_package().await,
    );
    record_case(
        &mut results,
        "package.generated_experience_template",
        generated::generated_experience_template().await,
    );
    record_case(
        &mut results,
        "package.generated_basic_template",
        generated::generated_basic_template().await,
    );
    record_case(
        &mut results,
        "package.generated_explicit_experience_template",
        generated::generated_explicit_experience_template().await,
    );
    record_case(
        &mut results,
        "package.generated_assistant_action_template",
        generated::generated_assistant_action_template().await,
    );
    record_case(
        &mut results,
        "package.generated_asset_editor_template",
        generated::generated_asset_editor_template().await,
    );
    record_case(
        &mut results,
        "package.generated_full_surface_template",
        generated::generated_full_surface_template().await,
    );
    record_case(
        &mut results,
        "package.generated_networked_template",
        generated::generated_networked_template().await,
    );
    record_case(
        &mut results,
        "package.generated_streaming_template",
        generated::generated_streaming_template().await,
    );
    record_case(
        &mut results,
        "package.generated_agent_runtime_template",
        generated::generated_agent_runtime_template().await,
    );
    record_case(
        &mut results,
        "package.faux_model_readiness",
        generated::faux_model_readiness_package().await,
    );
    record_case(
        &mut results,
        "package.faux_agent_readiness",
        generated::faux_agent_readiness_package().await,
    );
    record_case(&mut results, "composition.check_descriptor", generated::composition_descriptor().await);
    record_case(&mut results, "composition.check_descriptor_v2", generated::composition_descriptor_v2().await);
    record_case(&mut results, "official.composition_lab", official_labs::composition_lab().await);
    record_case(&mut results, "official.composition_lab_diagnostics", official_labs::composition_lab_diagnostics().await);
    record_case(&mut results, "official.asset_lab", official_labs::asset_lab().await);
    record_case(&mut results, "official.projection_lab", official_labs::projection_lab().await);
    record_case(&mut results, "official.playable_seed", official_labs::playable_seed().await);
    record_case(&mut results, "official.persona_lab", official_labs::persona_lab().await);
    record_case(&mut results, "official.knowledge_lab", official_labs::knowledge_lab().await);
    record_case(&mut results, "official.context_lab", official_labs::context_lab().await);
    record_case(&mut results, "official.text_transform_lab", official_labs::text_transform_lab().await);
    record_case(&mut results, "official.model_connector_lab", official_labs::model_connector_lab().await);
    record_case(&mut results, "official.model_routing_lab", official_labs::model_routing_lab().await);
    record_case(&mut results, "inproc.non_official_preview_rejected", inproc::non_official_preview_rejected().await);
    record_case(&mut results, "inproc.unknown_capability_errors", inproc::unknown_inproc_capability_errors().await);
    record_case(
        &mut results,
        "replacement.thirdparty_seed_surfaces",
        replacement::thirdparty_seed_surfaces().await,
    );
    record_case(
        &mut results,
        "replacement.thirdparty_seed_invocation",
        replacement::thirdparty_seed_invocation().await,
    );
    record_case(
        &mut results,
        "replacement.ambiguous_no_official_priority",
        replacement::ambiguous_no_official_priority().await,
    );
    record_case(
        &mut results,
        "replacement.composition_thirdparty",
        replacement::composition_thirdparty().await,
    );
    record_case(
        &mut results,
        "substrate.permission_grant_rehydrate",
        secret_conformance::permission_grant_rehydrate().await,
    );
    record_case(
        &mut results,
        "secret.ref_validation",
        secret_conformance::secret_ref_validation().await,
    );
    record_case(
        &mut results,
        "secret.raw_blocked_in_proposal",
        secret_conformance::raw_secret_blocked_in_proposal().await,
    );
    record_case(
        &mut results,
        "secret.raw_blocked_in_asset_metadata",
        secret_conformance::raw_secret_blocked_in_asset_metadata().await,
    );
    record_case(
        &mut results,
        "official.no_secret_bypass",
        secret_conformance::no_secret_bypass().await,
    );
    record_case(
        &mut results,
        "network.no_permission_denied",
        network::no_network_permission_denied().await,
    );
    record_case(
        &mut results,
        "network.allowlisted_host_method_allowed",
        network::allowlisted_host_method_allowed().await,
    );
    record_case(
        &mut results,
        "network.host_method_mismatch_denied",
        network::host_method_mismatch_denied().await,
    );
    record_case(
        &mut results,
        "network.official_no_network_bypass",
        network::official_no_network_bypass().await,
    );
    record_case(
        &mut results,
        "network.audit_no_raw_secrets",
        network::audit_no_raw_secrets().await,
    );
    record_case(
        &mut results,
        "network.policy_pure_function",
        network::network_policy_pure_function().await,
    );
    // Phase S3 — streaming and cancellation lifecycle
    record_case(
        &mut results,
        "stream.normal_lifecycle",
        streaming::stream_normal_lifecycle().await,
    );
    record_case(
        &mut results,
        "stream.cancel_blocks_chunks",
        streaming::stream_cancel_blocks_chunks().await,
    );
    record_case(
        &mut results,
        "stream.timeout_blocks_chunks",
        streaming::stream_timeout_blocks_chunks().await,
    );
    record_case(
        &mut results,
        "stream.error_terminal",
        streaming::stream_error_terminal().await,
    );
    record_case(
        &mut results,
        "stream.non_streaming_rejected",
        streaming::stream_non_streaming_rejected().await,
    );
    record_case(
        &mut results,
        "stream.no_model_agent_methods",
        streaming::stream_no_model_agent_methods().await,
    );
    record_case(
        &mut results,
        "stream.protocol_dispatch",
        streaming::stream_protocol_dispatch().await,
    );

    let mut failed = false;
    for (name, result) in &results {
        match result {
            Ok(()) => println!("{name:<45} PASS"),
            Err(err) => {
                failed = true;
                println!("{name:<45} FAIL {err}");
            }
        }
    }
    if failed {
        anyhow::bail!("conformance failed");
    }
    println!("conformance: ok ({} cases)", results.len());
    Ok(())
}
