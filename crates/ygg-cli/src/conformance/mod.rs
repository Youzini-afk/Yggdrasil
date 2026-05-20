mod agentic_forge;
mod core;
mod experience_observability;
mod experience_runtime;
mod fixtures;
mod generated;
mod hooks;
mod inference_local;
mod inference_playtest;
mod inproc;
mod live_model;
mod network;
mod official_foundation;
mod official_labs;
mod official_play_creation;
mod permissions;
mod playable_creation_board;
mod proposals;
mod protocol;
mod replacement;
mod secret_conformance;
mod streaming;
mod subprocess;
mod substrate;
mod surfaces;

fn record_case(
    results: &mut Vec<(&'static str, anyhow::Result<()>)>,
    name: &'static str,
    result: anyhow::Result<()>,
) {
    results.push((name, result));
}

pub(crate) async fn run() -> anyhow::Result<()> {
    let mut results = Vec::new();
    record_case(
        &mut results,
        "session.open_empty",
        core::session_open().await,
    );
    record_case(
        &mut results,
        "event.append_authorized",
        core::event_append_authorized().await,
    );
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
    record_case(
        &mut results,
        "event.range_replay",
        core::event_range_replay().await,
    );
    record_case(
        &mut results,
        "capability.invoke_rust_inproc",
        core::capability_invoke().await,
    );
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
    record_case(
        &mut results,
        "official.no_privilege",
        core::official_no_privilege().await,
    );
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
    record_case(
        &mut results,
        "permission.grant_revoke_audit",
        permissions::permission_grant_revoke_audit().await,
    );
    record_case(
        &mut results,
        "permission.assistant_capability_grant",
        permissions::assistant_capability_grant().await,
    );
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
    record_case(
        &mut results,
        "subprocess.load_ready",
        subprocess::subprocess_load_ready().await,
    );
    record_case(
        &mut results,
        "subprocess.invoke_echo",
        subprocess::subprocess_invoke_echo().await,
    );
    record_case(
        &mut results,
        "package.lifecycle_timeline",
        subprocess::package_lifecycle_timeline().await,
    );
    record_case(
        &mut results,
        "package.logs_capture",
        subprocess::package_logs_capture().await,
    );
    record_case(
        &mut results,
        "package.restart_subprocess",
        subprocess::package_restart_subprocess().await,
    );
    record_case(
        &mut results,
        "host.diagnostics",
        core::host_diagnostics().await,
    );
    record_case(
        &mut results,
        "host.profile_autoload",
        core::host_profile_autoload().await,
    );
    record_case(
        &mut results,
        "surface.contribution_list",
        surfaces::contribution_list().await,
    );
    record_case(
        &mut results,
        "official.foundation_packages",
        official_foundation::foundation_packages().await,
    );
    record_case(
        &mut results,
        "official.assistant_lab_proposal",
        official_labs::assistant_lab_proposal().await,
    );
    record_case(
        &mut results,
        "play_creation.blank_loop",
        official_play_creation::blank_play_creation_loop().await,
    );
    record_case(
        &mut results,
        "proposal.lifecycle_apply",
        proposals::lifecycle_apply().await,
    );
    record_case(
        &mut results,
        "proposal.reject_and_apply_denied",
        proposals::reject_and_apply_denied().await,
    );
    record_case(
        &mut results,
        "asset.put_get_list",
        core::asset_put_get_list().await,
    );
    record_case(
        &mut results,
        "session.fork_branch",
        core::session_fork_branch().await,
    );
    record_case(
        &mut results,
        "projection.rebuild",
        core::projection_rebuild().await,
    );
    record_case(
        &mut results,
        "substrate.sqlite_rehydrate",
        substrate::sqlite_rehydrate().await,
    );
    record_case(
        &mut results,
        "subprocess.bad_handshake",
        subprocess::subprocess_bad_handshake().await,
    );
    record_case(
        &mut results,
        "subprocess.invoke_timeout",
        subprocess::subprocess_timeout().await,
    );
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
    record_case(
        &mut results,
        "protocol.call_host_info",
        protocol::call_host_info().await,
    );
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
    record_case(
        &mut results,
        "hook.ordering_stable",
        hooks::ordering_stable().await,
    );
    record_case(
        &mut results,
        "hook.veto_blocks_event_append",
        hooks::veto_blocks_event_append().await,
    );
    record_case(
        &mut results,
        "hook.metadata_mutation_allowed",
        hooks::metadata_mutation_allowed().await,
    );
    record_case(
        &mut results,
        "hook.package_owned_handler",
        hooks::package_owned_handler().await,
    );
    record_case(
        &mut results,
        "hook.unload_removes_subscription",
        hooks::unload_removes_subscription().await,
    );
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
        "package.generated_experience_runtime_template",
        generated::generated_experience_runtime_template().await,
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
    record_case(
        &mut results,
        "composition.check_descriptor",
        generated::composition_descriptor().await,
    );
    record_case(
        &mut results,
        "composition.check_descriptor_v2",
        generated::composition_descriptor_v2().await,
    );
    record_case(
        &mut results,
        "official.composition_lab",
        official_labs::composition_lab().await,
    );
    record_case(
        &mut results,
        "official.composition_lab_diagnostics",
        official_labs::composition_lab_diagnostics().await,
    );
    record_case(
        &mut results,
        "official.asset_lab",
        official_labs::asset_lab().await,
    );
    record_case(
        &mut results,
        "official.projection_lab",
        official_labs::projection_lab().await,
    );
    record_case(
        &mut results,
        "official.playable_seed",
        official_labs::playable_seed().await,
    );
    record_case(
        &mut results,
        "official.persona_lab",
        official_labs::persona_lab().await,
    );
    record_case(
        &mut results,
        "official.knowledge_lab",
        official_labs::knowledge_lab().await,
    );
    record_case(
        &mut results,
        "official.context_lab",
        official_labs::context_lab().await,
    );
    record_case(
        &mut results,
        "official.text_transform_lab",
        official_labs::text_transform_lab().await,
    );
    record_case(
        &mut results,
        "official.model_connector_lab",
        official_labs::model_connector_lab().await,
    );
    record_case(
        &mut results,
        "official.model_provider_lab",
        official_labs::model_provider_lab().await,
    );
    record_case(
        &mut results,
        "official.model_provider_lab_invoke_core",
        official_labs::model_provider_lab_invoke_core().await,
    );
    record_case(
        &mut results,
        "official.model_provider_lab_normalize_stream",
        official_labs::model_provider_lab_normalize_stream().await,
    );
    record_case(
        &mut results,
        "official.model_routing_lab",
        official_labs::model_routing_lab().await,
    );
    record_case(
        &mut results,
        "official.pi_agent_runtime_lab",
        official_labs::pi_agent_runtime_lab().await,
    );
    record_case(
        &mut results,
        "official.capability_tool_bridge_lab",
        official_labs::capability_tool_bridge_lab().await,
    );
    record_case(
        &mut results,
        "inproc.non_official_preview_rejected",
        inproc::non_official_preview_rejected().await,
    );
    record_case(
        &mut results,
        "inproc.unknown_capability_errors",
        inproc::unknown_inproc_capability_errors().await,
    );
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
    // Phase J6 — third-party agent runtime replacement proof
    record_case(
        &mut results,
        "replacement.thirdparty_agent_runtime_surfaces",
        replacement::thirdparty_agent_runtime_surfaces().await,
    );
    record_case(
        &mut results,
        "replacement.thirdparty_agent_runtime_invocation",
        replacement::thirdparty_agent_runtime_invocation().await,
    );
    record_case(
        &mut results,
        "replacement.composition_agent_runtime_replacement",
        replacement::composition_agent_runtime_replacement().await,
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
        "secret.env_resolver_allowed",
        secret_conformance::env_resolver_allowed().await,
    );
    record_case(
        &mut results,
        "secret.env_resolver_denied",
        secret_conformance::env_resolver_denied().await,
    );
    record_case(
        &mut results,
        "secret.env_resolver_missing_no_leak",
        secret_conformance::env_resolver_missing_no_leak().await,
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
    // Phase M3 — outbound executor boundary
    record_case(
        &mut results,
        "outbound.no_permission_executor_not_called",
        network::outbound_no_permission_executor_not_called().await,
    );
    record_case(
        &mut results,
        "outbound.policy_executor_mismatch_denied",
        network::outbound_policy_executor_mismatch_denied().await,
    );
    record_case(
        &mut results,
        "outbound.allowlisted_fake_executor",
        network::outbound_allowlisted_fake_executor().await,
    );
    record_case(
        &mut results,
        "outbound.raw_body_not_audited",
        network::outbound_raw_body_not_audited().await,
    );
    record_case(
        &mut results,
        "outbound.secret_refs_only",
        network::outbound_secret_refs_only().await,
    );
    record_case(
        &mut results,
        "outbound.host_mismatch_redirect_denied",
        network::outbound_host_mismatch_redirect_denied().await,
    );
    // Phase M4 — model provider invoke adapters + outbound shape conformance
    record_case(
        &mut results,
        "outbound.model_provider_shape_fake_executor",
        network::outbound_model_provider_shape_fake_executor().await,
    );
    // Phase L2 — LiveHttpOutboundExecutor
    record_case(
        &mut results,
        "outbound.live_http_default_disabled",
        network::outbound_live_http_default_disabled().await,
    );
    record_case(
        &mut results,
        "outbound.live_http_rejects_insecure_url",
        network::outbound_live_http_rejects_insecure_url().await,
    );
    record_case(
        &mut results,
        "outbound.live_http_redacted_shape",
        network::outbound_live_http_redacted_shape().await,
    );
    // Phase L3 — kernel.outbound.execute public protocol boundary
    record_case(
        &mut results,
        "outbound.execute_package_allowed",
        network::outbound_execute_package_allowed().await,
    );
    record_case(
        &mut results,
        "outbound.execute_spoofed_package_id_rejected",
        network::outbound_execute_spoofed_package_id_rejected().await,
    );
    record_case(
        &mut results,
        "outbound.execute_no_permission_denied",
        network::outbound_execute_no_permission_denied().await,
    );
    record_case(
        &mut results,
        "outbound.execute_no_raw_secret_in_response",
        network::outbound_execute_no_raw_secret_in_response().await,
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
    // Phase L4 — Live Model Calls: first live provider canary invoke+stream
    record_case(
        &mut results,
        "outbound.secret_headers_parsed",
        live_model::outbound_secret_headers_parsed().await,
    );
    record_case(
        &mut results,
        "outbound.live_loopback_secret_injection",
        live_model::outbound_live_loopback_secret_injection().await,
    );
    record_case(
        &mut results,
        "stream.sse_normalize_deepseek_canary",
        live_model::stream_sse_normalize_deepseek_canary().await,
    );
    record_case(
        &mut results,
        "outbound.live_deepseek_opt_in",
        live_model::outbound_live_deepseek_opt_in().await,
    );
    record_case(
        &mut results,
        "canary.deepseek_profile_shape",
        live_model::canary_deepseek_profile_shape().await,
    );
    // Phase L5 — OpenAI / Anthropic / Gemini live adapter conformance
    record_case(
        &mut results,
        "outbound.openai_chat_loopback",
        live_model::openai_chat_loopback().await,
    );
    record_case(
        &mut results,
        "outbound.openai_responses_loopback",
        live_model::openai_responses_loopback().await,
    );
    record_case(
        &mut results,
        "outbound.anthropic_messages_loopback",
        live_model::anthropic_messages_loopback().await,
    );
    record_case(
        &mut results,
        "outbound.gemini_generate_content_loopback",
        live_model::gemini_generate_content_loopback().await,
    );
    record_case(
        &mut results,
        "outbound.missing_secret_fails_closed",
        live_model::missing_secret_fails_closed().await,
    );
    record_case(
        &mut results,
        "outbound.provider_normalize_request_alignment",
        live_model::provider_normalize_request_alignment().await,
    );
    record_case(
        &mut results,
        "outbound.no_raw_secret_leak_all_providers",
        live_model::no_raw_secret_leak_all_providers().await,
    );
    record_case(
        &mut results,
        "outbound.static_headers_safe_allowlist",
        live_model::static_headers_safe_allowlist().await,
    );
    record_case(
        &mut results,
        "outbound.static_headers_block_secrets",
        live_model::static_headers_block_secrets().await,
    );
    // Phase L6 — OpenRouter / xAI / Fireworks / DeepSeek provider quirks
    record_case(
        &mut results,
        "outbound.openrouter_loopback_headers",
        live_model::openrouter_loopback_headers().await,
    );
    record_case(
        &mut results,
        "outbound.xai_loopback",
        live_model::xai_loopback().await,
    );
    record_case(
        &mut results,
        "outbound.fireworks_loopback",
        live_model::fireworks_loopback().await,
    );
    record_case(
        &mut results,
        "stream.deepseek_reasoning_stream",
        live_model::deepseek_reasoning_stream().await,
    );
    record_case(
        &mut results,
        "stream.openrouter_midstream_error",
        live_model::openrouter_midstream_error().await,
    );
    record_case(
        &mut results,
        "outbound.provider_quirk_fixtures_no_secrets",
        live_model::provider_quirk_fixtures_no_secrets().await,
    );
    record_case(
        &mut results,
        "outbound.static_headers_openrouter_safe",
        live_model::static_headers_openrouter_safe().await,
    );
    // Phase C2 — inference-local-lab deterministic non-HTTP fake local inference provider proof
    record_case(
        &mut results,
        "official.inference_local_lab_describe_capabilities",
        inference_local::inference_local_lab_describe_capabilities().await,
    );
    record_case(
        &mut results,
        "official.inference_local_lab_invoke",
        inference_local::inference_local_lab_invoke().await,
    );
    record_case(
        &mut results,
        "official.inference_local_lab_invoke_rejects_http",
        inference_local::inference_local_lab_invoke_rejects_http().await,
    );
    record_case(
        &mut results,
        "official.inference_local_lab_stream",
        inference_local::inference_local_lab_stream().await,
    );
    record_case(
        &mut results,
        "official.inference_local_lab_explain_error",
        inference_local::inference_local_lab_explain_error().await,
    );
    // Phase C4 — inference-playtest-lab Ygg-native inference proposal vertical slice
    record_case(
        &mut results,
        "official.inference_playtest_lab_draft",
        inference_playtest::inference_playtest_draft().await,
    );
    record_case(
        &mut results,
        "official.inference_playtest_lab_inspect",
        inference_playtest::inference_playtest_inspect().await,
    );
    record_case(
        &mut results,
        "official.inference_playtest_lab_reject_apply_denied",
        inference_playtest::inference_playtest_reject_apply_denied().await,
    );
    record_case(
        &mut results,
        "official.inference_playtest_lab_apply_and_branch",
        inference_playtest::inference_playtest_apply_and_branch().await,
    );
    record_case(
        &mut results,
        "official.inference_playtest_lab_no_chat_kernel_terms",
        inference_playtest::inference_playtest_no_chat_kernel_terms().await,
    );
    // Phase A — Agentic Forge Beta: package-owned run lifecycle / working state / plan graph
    record_case(
        &mut results,
        "agentic_forge.describe_contract",
        agentic_forge::agentic_forge_describe_contract().await,
    );
    record_case(
        &mut results,
        "agentic_forge.start_run_plan_graph_working_state",
        agentic_forge::agentic_forge_start_run().await,
    );
    record_case(
        &mut results,
        "agentic_forge.inspect_cancel_summarize",
        agentic_forge::agentic_forge_inspect_cancel_summarize().await,
    );
    record_case(
        &mut results,
        "agentic_forge.raw_secret_blocked",
        agentic_forge::agentic_forge_raw_secret_blocked().await,
    );
    record_case(
        &mut results,
        "agentic_forge.no_kernel_agent_namespace",
        agentic_forge::agentic_forge_no_kernel_agent_namespace().await,
    );
    // Phase B — Agentic Forge Beta: branch-aware candidate / compare / promote / archive
    record_case(
        &mut results,
        "agentic_forge.create_candidate_branch_aware",
        agentic_forge::agentic_forge_create_candidate().await,
    );
    record_case(
        &mut results,
        "agentic_forge.compare_candidate_stale_detection",
        agentic_forge::agentic_forge_compare_candidate().await,
    );
    record_case(
        &mut results,
        "agentic_forge.draft_promote_proposal_no_mutation",
        agentic_forge::agentic_forge_draft_promote_proposal().await,
    );
    record_case(
        &mut results,
        "agentic_forge.stale_promote_blocked",
        agentic_forge::agentic_forge_stale_promote_blocked().await,
    );
    record_case(
        &mut results,
        "agentic_forge.archive_candidate_target_unchanged",
        agentic_forge::agentic_forge_archive_candidate().await,
    );
    // Phase C — Agentic Forge Beta: inference-backed agent run with deterministic fallback
    record_case(
        &mut results,
        "agentic_forge.inference_node_deterministic_candidate_seed",
        agentic_forge::agentic_forge_inference_node_deterministic().await,
    );
    record_case(
        &mut results,
        "agentic_forge.replay_match_mismatch_flagged",
        agentic_forge::agentic_forge_replay_match_mismatch().await,
    );
    record_case(
        &mut results,
        "agentic_forge.inference_output_privilege_escalation_rejected",
        agentic_forge::agentic_forge_inference_output_validation().await,
    );
    record_case(
        &mut results,
        "agentic_forge.cloud_adapter_needs_host_policy_no_network",
        agentic_forge::agentic_forge_cloud_adapter_no_network().await,
    );
    record_case(
        &mut results,
        "agentic_forge.inference_failure_taxonomy_recovery_hints",
        agentic_forge::agentic_forge_inference_failure_taxonomy().await,
    );
    // Phase D — Agentic Forge Beta: scoped toolchain observation / risk / replay
    record_case(
        &mut results,
        "agentic_forge.explain_tool_call_scoped_no_ambient_authority",
        agentic_forge::agentic_forge_explain_tool_call_scoped().await,
    );
    record_case(
        &mut results,
        "agentic_forge.record_observation_untrusted_large_output_redaction",
        agentic_forge::agentic_forge_record_observation_untrusted().await,
    );
    record_case(
        &mut results,
        "agentic_forge.tool_risk_injection_exfiltration_outbound",
        agentic_forge::agentic_forge_tool_risk_categories().await,
    );
    record_case(
        &mut results,
        "agentic_forge.replay_tool_plan_mismatch_flagged",
        agentic_forge::agentic_forge_replay_tool_mismatch().await,
    );
    record_case(
        &mut results,
        "agentic_forge.plan_toolchain_requires_explicit_provider_nested_delegation_blocked",
        agentic_forge::agentic_forge_plan_toolchain_requires_provider().await,
    );
    // Phase F — Agentic Forge Beta: third-party replacement, hostile, budget/deadline
    record_case(
        &mut results,
        "agentic_forge.thirdparty_replacement_shape_no_official_priority",
        agentic_forge::agentic_forge_thirdparty_replacement_shape().await,
    );
    record_case(
        &mut results,
        "agentic_forge.no_official_priority_ordinary_package",
        agentic_forge::agentic_forge_no_official_priority().await,
    );
    record_case(
        &mut results,
        "agentic_forge.hostile_injection_secret_blocked_cross_package",
        agentic_forge::agentic_forge_hostile_injection_secret_blocked().await,
    );
    record_case(
        &mut results,
        "agentic_forge.budget_deadline_contract_cancellation_consistent",
        agentic_forge::agentic_forge_budget_deadline_contract().await,
    );
    record_case(
        &mut results,
        "agentic_forge.cross_package_replay_mismatch_flagged",
        agentic_forge::agentic_forge_cross_package_replay_consistency().await,
    );
    // Experience Beta 0 — Thin Experience Runtime Contract
    record_case(
        &mut results,
        "experience_runtime.describe_contract_shape",
        experience_runtime::experience_runtime_describe_contract().await,
    );
    record_case(
        &mut results,
        "experience_runtime.checkpoint_recovery_shape",
        experience_runtime::experience_runtime_checkpoint_shape().await,
    );
    record_case(
        &mut results,
        "experience_runtime.recovery_shape",
        experience_runtime::experience_runtime_recovery_shape().await,
    );
    record_case(
        &mut results,
        "experience_runtime.no_kernel_experience_namespace",
        experience_runtime::experience_runtime_no_kernel_namespace().await,
    );
    record_case(
        &mut results,
        "experience_runtime.template_generation",
        experience_runtime::experience_runtime_template_generation().await,
    );
    record_case(
        &mut results,
        "experience_runtime.bind_agent_run_shape",
        experience_runtime::experience_runtime_bind_agent_run().await,
    );
    // Experience Beta 1 — First Real Playable Vertical Slice
    record_case(
        &mut results,
        "playable_board.describe_contract_shape",
        playable_creation_board::playable_board_describe_contract().await,
    );
    record_case(
        &mut results,
        "playable_board.launch_and_player_actions",
        playable_creation_board::playable_board_launch_and_player_actions().await,
    );
    record_case(
        &mut results,
        "playable_board.checkpoint_recovery_shape",
        playable_creation_board::playable_board_checkpoint_recovery().await,
    );
    record_case(
        &mut results,
        "playable_board.request_change_no_chat",
        playable_creation_board::playable_board_request_change_no_chat().await,
    );
    record_case(
        &mut results,
        "playable_board.bind_agent_run_scoped",
        playable_creation_board::playable_board_bind_agent_run_scoped().await,
    );
    record_case(
        &mut results,
        "playable_board.candidate_proposal_no_target_mutation",
        playable_creation_board::playable_board_candidate_proposal_no_target_mutation().await,
    );
    record_case(
        &mut results,
        "playable_board.reject_approve_fork_proof",
        playable_creation_board::playable_board_reject_approve_fork_proof().await,
    );
    record_case(
        &mut results,
        "playable_board.thirdparty_no_official_priority",
        playable_creation_board::playable_board_thirdparty_no_official_priority().await,
    );
    record_case(
        &mut results,
        "playable_board.no_forbidden_namespace",
        playable_creation_board::playable_board_no_forbidden_namespace().await,
    );
    record_case(
        &mut results,
        "playable_board.no_raw_secrets",
        playable_creation_board::playable_board_no_raw_secrets().await,
    );
    // Experience Beta 2 — State + Asset Pipeline Alpha
    record_case(
        &mut results,
        "playable_board.content_address_stable",
        playable_creation_board::playable_board_content_address_stable().await,
    );
    record_case(
        &mut results,
        "playable_board.checkpoint_metadata",
        playable_creation_board::playable_board_checkpoint_metadata().await,
    );
    record_case(
        &mut results,
        "playable_board.provenance_graph",
        playable_creation_board::playable_board_provenance_graph().await,
    );
    record_case(
        &mut results,
        "playable_board.state_diff_preview",
        playable_creation_board::playable_board_state_diff_preview().await,
    );
    record_case(
        &mut results,
        "playable_board.describe_asset_provenance",
        playable_creation_board::playable_board_describe_asset_provenance().await,
    );
    record_case(
        &mut results,
        "playable_board.beta2_no_raw_secrets",
        playable_creation_board::playable_board_beta2_no_raw_secrets().await,
    );
    record_case(
        &mut results,
        "official.asset_lab_content_address",
        official_labs::asset_lab_content_address().await,
    );
    record_case(
        &mut results,
        "official.asset_lab_provenance_graph",
        official_labs::asset_lab_provenance_graph().await,
    );
    record_case(
        &mut results,
        "official.projection_lab_state_snapshot",
        official_labs::projection_lab_state_snapshot().await,
    );
    // Experience Beta 3 — Experience Observability (backend/package part)
    record_case(
        &mut results,
        "experience_observability.contract_shape",
        experience_observability::experience_observability_contract().await,
    );
    record_case(
        &mut results,
        "experience_observability.session_health",
        experience_observability::experience_observability_session_health().await,
    );
    record_case(
        &mut results,
        "experience_observability.package_health",
        experience_observability::experience_observability_package_health().await,
    );
    record_case(
        &mut results,
        "experience_observability.agent_run_health",
        experience_observability::experience_observability_agent_run_health().await,
    );
    record_case(
        &mut results,
        "experience_observability.proposal_causality",
        experience_observability::experience_observability_proposal_causality().await,
    );
    record_case(
        &mut results,
        "experience_observability.cost_latency_summary",
        experience_observability::experience_observability_cost_latency().await,
    );
    record_case(
        &mut results,
        "experience_observability.failure_breadcrumbs",
        experience_observability::experience_observability_failure_breadcrumbs().await,
    );
    record_case(
        &mut results,
        "experience_observability.guardrail_audit_summary",
        experience_observability::experience_observability_guardrail_summary().await,
    );
    record_case(
        &mut results,
        "experience_observability.no_forbidden_namespace",
        experience_observability::experience_observability_no_forbidden_namespace().await,
    );
    record_case(
        &mut results,
        "experience_observability.no_raw_secrets",
        experience_observability::experience_observability_no_raw_secrets().await,
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
