mod agentic_forge;
mod audit;
mod core;
mod creator_loop;
mod experience_observability;
mod experience_runtime;
mod fixtures;
mod generated;
mod git_tools;
mod hooks;
mod inference_local;
mod inference_playtest;
mod inproc;
mod install_lab;
mod install_real_smoke;
mod integrity_tools;
mod live_model;
mod memory_lab;
mod network;
mod official_foundation;
mod official_labs;
mod official_play_creation;
mod permissions;
mod playable_creation_board;
mod project_intake_lab;
mod proposals;
mod protocol;
mod replacement;
mod secret_conformance;
mod sharing_lab;
mod storage_backend;
mod storage_lab;
mod streaming;
mod subprocess;
mod substrate;
mod surfaces;
mod tdb_retrieval_lab;
mod tdb_rust_adapter;
mod workspace_lab;

use std::time::Instant;

/// Options parsed from CLI for the conformance command.
pub(crate) struct ConformanceOptions {
    pub(crate) list: bool,
    pub(crate) case: Vec<String>,
    pub(crate) tag: Vec<String>,
    pub(crate) fail_fast: bool,
    pub(crate) slowest: usize,
}

/// A single conformance case with metadata.
struct ConformanceCase {
    id: &'static str,
    tags: &'static [&'static str],
    run: fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send>>,
}

/// Build the full ordered list of conformance cases.
fn build_cases() -> Vec<ConformanceCase> {
    // Helper to erase the async function type.
    fn case(
        id: &'static str,
        tags: &'static [&'static str],
        run: fn()
            -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send>>,
    ) -> ConformanceCase {
        ConformanceCase { id, tags, run }
    }
    // Macro to reduce boilerplate for the common pattern.
    macro_rules! c {
        ($id:expr, [$($tag:expr),*], $func:path) => {
            case($id, &[$($tag),*], || Box::pin($func()))
        };
    }

    vec![
        // --- core ---
        c!(
            "session.open_empty",
            ["runtime", "session"],
            core::session_open
        ),
        c!(
            "event.append_authorized",
            ["runtime", "event"],
            core::event_append_authorized
        ),
        c!(
            "event.append_without_permission_denied",
            ["runtime", "event"],
            core::event_append_without_permission_denied
        ),
        c!(
            "event.kernel_namespace_denied",
            ["runtime", "event"],
            core::kernel_namespace_denied
        ),
        c!(
            "event.read_without_permission_denied",
            ["runtime", "event"],
            core::event_read_without_permission_denied
        ),
        c!(
            "event.closed_session_rejects_append",
            ["runtime", "event"],
            core::closed_session_rejects_append
        ),
        c!(
            "event.range_replay",
            ["runtime", "event"],
            core::event_range_replay
        ),
        c!(
            "capability.invoke_rust_inproc",
            ["runtime", "capability"],
            core::capability_invoke
        ),
        c!(
            "capability.handle_invoke",
            ["runtime", "capability", "handle"],
            core::capability_handle_invoke
        ),
        c!(
            "capability.handle_attenuate_invoke",
            ["runtime", "capability", "handle"],
            core::capability_handle_attenuate_invoke
        ),
        c!(
            "capability.handle_revoke_blocks_invoke",
            ["runtime", "capability", "handle"],
            core::capability_handle_revoke_blocks_invoke
        ),
        c!(
            "capability.auto_mint_legacy_invoke",
            ["runtime", "capability", "handle"],
            core::capability_auto_mint_legacy_invoke
        ),
        c!(
            "capability.invoke_events_completed",
            ["runtime", "capability", "audit"],
            core::capability_invoke_events_completed
        ),
        c!(
            "capability.invoke_events_failed",
            ["runtime", "capability", "audit"],
            core::capability_invoke_events_failed
        ),
        c!(
            "package.audit_report",
            ["runtime", "audit"],
            audit::package_audit_report
        ),
        c!(
            "capability.ambiguous_provider_denied",
            ["runtime", "capability"],
            core::ambiguous_provider_denied
        ),
        c!(
            "capability.explicit_provider_selected",
            ["runtime", "capability"],
            core::explicit_provider_selected
        ),
        c!(
            "package.unload_removes_capability",
            ["runtime", "package"],
            core::unload_removes_capability
        ),
        c!(
            "official.no_privilege",
            ["official"],
            core::official_no_privilege
        ),
        c!(
            "schema.capability_input_rejects_invalid",
            ["runtime", "schema"],
            core::capability_schema_rejects_invalid
        ),
        c!(
            "schema.event_payload_rejects_invalid",
            ["runtime", "schema"],
            core::event_schema_rejects_invalid
        ),
        // --- permissions ---
        c!(
            "protocol.structured_permission_error",
            ["protocol", "permission"],
            permissions::structured_permission_error
        ),
        c!(
            "permission.grant_revoke_audit",
            ["runtime", "permission"],
            permissions::permission_grant_revoke_audit
        ),
        c!(
            "permission.assistant_capability_grant",
            ["runtime", "permission"],
            permissions::assistant_capability_grant
        ),
        c!(
            "principal.package_cannot_self_assert_writer",
            ["runtime", "permission"],
            permissions::principal_cannot_self_assert_writer
        ),
        c!(
            "principal.package_cannot_self_assert_capability_caller",
            ["runtime", "permission"],
            permissions::principal_cannot_self_assert_capability_caller
        ),
        // --- subprocess ---
        c!(
            "subprocess.load_ready",
            ["subprocess", "slow"],
            subprocess::subprocess_load_ready
        ),
        c!(
            "subprocess.invoke_echo",
            ["subprocess", "slow"],
            subprocess::subprocess_invoke_echo
        ),
        c!(
            "subprocess.bindings_injected_invokable",
            ["subprocess", "binding", "slow"],
            subprocess::subprocess_bindings_injected_and_invokable
        ),
        c!(
            "package.lifecycle_timeline",
            ["subprocess", "package", "slow"],
            subprocess::package_lifecycle_timeline
        ),
        c!(
            "package.path_b_self_contained",
            ["subprocess", "package", "path_b", "slow"],
            subprocess::path_b_self_contained_contract_none
        ),
        c!(
            "package.logs_capture",
            ["subprocess", "package", "slow"],
            subprocess::package_logs_capture
        ),
        c!(
            "package.restart_subprocess",
            ["subprocess", "package", "slow"],
            subprocess::package_restart_subprocess
        ),
        // --- host ---
        c!(
            "host.diagnostics",
            ["runtime", "host"],
            core::host_diagnostics
        ),
        c!(
            "host.profile_autoload",
            ["runtime", "host", "slow"],
            core::host_profile_autoload
        ),
        // --- surfaces ---
        c!(
            "surface.contribution_list",
            ["surface"],
            surfaces::contribution_list
        ),
        // --- official foundation / labs ---
        c!(
            "official.foundation_packages",
            ["official", "slow"],
            official_foundation::foundation_packages
        ),
        c!(
            "official.assistant_lab_proposal",
            ["official", "slow"],
            official_labs::assistant_lab_proposal
        ),
        c!(
            "play_creation.blank_loop",
            ["official", "slow"],
            official_play_creation::blank_play_creation_loop
        ),
        // --- proposals ---
        c!(
            "proposal.lifecycle_apply",
            ["runtime", "proposal"],
            proposals::lifecycle_apply
        ),
        c!(
            "proposal.reject_and_apply_denied",
            ["runtime", "proposal"],
            proposals::reject_and_apply_denied
        ),
        // --- asset ---
        c!(
            "asset.put_get_list",
            ["runtime", "asset"],
            core::asset_put_get_list
        ),
        // --- session fork ---
        c!(
            "session.fork_branch",
            ["runtime", "session"],
            core::session_fork_branch
        ),
        // --- projection ---
        c!(
            "projection.rebuild",
            ["runtime", "projection"],
            core::projection_rebuild
        ),
        // --- substrate ---
        c!(
            "substrate.sqlite_rehydrate",
            ["substrate", "slow"],
            substrate::sqlite_rehydrate
        ),
        // --- subprocess error cases ---
        c!(
            "subprocess.bad_handshake",
            ["subprocess", "slow"],
            subprocess::subprocess_bad_handshake
        ),
        c!(
            "subprocess.invoke_timeout",
            ["subprocess", "slow"],
            subprocess::subprocess_timeout
        ),
        c!(
            "subprocess.invalid_output_schema",
            ["subprocess", "slow"],
            subprocess::subprocess_invalid_output_schema
        ),
        c!(
            "subprocess.unload_removes_capability",
            ["subprocess", "slow"],
            subprocess::subprocess_unload_removes_capability
        ),
        // --- protocol ---
        c!(
            "protocol.call_host_info",
            ["protocol"],
            protocol::call_host_info
        ),
        c!(
            "protocol.call_capability_in_process",
            ["protocol"],
            protocol::call_capability_in_process
        ),
        // --- package check / reload ---
        c!(
            "package.check_diagnostics",
            ["subprocess", "package", "slow"],
            subprocess::package_check_diagnostics
        ),
        c!(
            "package.reload_smoke",
            ["subprocess", "package", "slow"],
            subprocess::package_reload_smoke
        ),
        // --- hooks ---
        c!(
            "hook.ordering_stable",
            ["runtime", "hook"],
            hooks::ordering_stable
        ),
        c!(
            "hook.veto_blocks_event_append",
            ["runtime", "hook"],
            hooks::veto_blocks_event_append
        ),
        c!(
            "hook.metadata_mutation_allowed",
            ["runtime", "hook"],
            hooks::metadata_mutation_allowed
        ),
        c!(
            "hook.package_owned_handler",
            ["runtime", "hook"],
            hooks::package_owned_handler
        ),
        c!(
            "hook.unload_removes_subscription",
            ["runtime", "hook"],
            hooks::unload_removes_subscription
        ),
        // --- generated packages ---
        c!(
            "package.generated_subprocess_conformance",
            ["generated", "slow"],
            generated::generated_subprocess_package
        ),
        c!(
            "package.generated_typescript_subprocess_conformance",
            ["generated", "slow"],
            generated::generated_typescript_subprocess_package
        ),
        c!(
            "package.generated_experience_template",
            ["generated", "slow"],
            generated::generated_experience_template
        ),
        c!(
            "package.generated_basic_template",
            ["generated", "slow"],
            generated::generated_basic_template
        ),
        c!(
            "package.generated_explicit_experience_template",
            ["generated", "slow"],
            generated::generated_explicit_experience_template
        ),
        c!(
            "package.generated_assistant_action_template",
            ["generated", "slow"],
            generated::generated_assistant_action_template
        ),
        c!(
            "package.generated_asset_editor_template",
            ["generated", "slow"],
            generated::generated_asset_editor_template
        ),
        c!(
            "package.generated_full_surface_template",
            ["generated", "slow"],
            generated::generated_full_surface_template
        ),
        c!(
            "package.generated_networked_template",
            ["generated", "network", "slow"],
            generated::generated_networked_template
        ),
        c!(
            "package.generated_streaming_template",
            ["generated", "stream", "slow"],
            generated::generated_streaming_template
        ),
        c!(
            "package.generated_agent_runtime_template",
            ["generated", "agentic", "slow"],
            generated::generated_agent_runtime_template
        ),
        c!(
            "package.generated_experience_runtime_template",
            ["generated", "experience", "slow"],
            generated::generated_experience_runtime_template
        ),
        c!(
            "package.faux_model_readiness",
            ["generated", "slow"],
            generated::faux_model_readiness_package
        ),
        c!(
            "package.faux_agent_readiness",
            ["generated", "agentic", "slow"],
            generated::faux_agent_readiness_package
        ),
        // --- composition ---
        c!(
            "composition.check_descriptor",
            ["composition"],
            generated::composition_descriptor
        ),
        c!(
            "composition.check_descriptor_v2",
            ["composition"],
            generated::composition_descriptor_v2
        ),
        c!(
            "official.composition_lab",
            ["official", "composition", "slow"],
            official_labs::composition_lab
        ),
        c!(
            "official.composition_lab_diagnostics",
            ["official", "composition", "slow"],
            official_labs::composition_lab_diagnostics
        ),
        c!(
            "official.asset_lab",
            ["official", "slow"],
            official_labs::asset_lab
        ),
        c!(
            "official.projection_lab",
            ["official", "slow"],
            official_labs::projection_lab
        ),
        c!(
            "official.playable_seed",
            ["official", "slow"],
            official_labs::playable_seed
        ),
        c!(
            "official.persona_lab",
            ["official", "slow"],
            official_labs::persona_lab
        ),
        c!(
            "official.knowledge_lab",
            ["official", "slow"],
            official_labs::knowledge_lab
        ),
        c!(
            "official.context_lab",
            ["official", "slow"],
            official_labs::context_lab
        ),
        c!(
            "official.text_transform_lab",
            ["official", "slow"],
            official_labs::text_transform_lab
        ),
        c!(
            "official.model_connector_lab",
            ["official", "slow"],
            official_labs::model_connector_lab
        ),
        c!(
            "official.model_provider_lab",
            ["official", "slow"],
            official_labs::model_provider_lab
        ),
        c!(
            "official.model_provider_lab_invoke_core",
            ["official", "slow"],
            official_labs::model_provider_lab_invoke_core
        ),
        c!(
            "official.model_provider_lab_normalize_stream",
            ["official", "slow"],
            official_labs::model_provider_lab_normalize_stream
        ),
        c!(
            "official.model_routing_lab",
            ["official", "slow"],
            official_labs::model_routing_lab
        ),
        c!(
            "official.pi_agent_runtime_lab",
            ["official", "agentic", "slow"],
            official_labs::pi_agent_runtime_lab
        ),
        c!(
            "official.capability_tool_bridge_lab",
            ["official", "agentic", "slow"],
            official_labs::capability_tool_bridge_lab
        ),
        // --- inproc ---
        c!(
            "inproc.non_official_preview_rejected",
            ["runtime"],
            inproc::non_official_preview_rejected
        ),
        c!(
            "inproc.unknown_capability_errors",
            ["runtime"],
            inproc::unknown_inproc_capability_errors
        ),
        c!(
            "inproc.bindings_init",
            ["runtime", "binding"],
            inproc::inproc_bindings_init_receives_manifest_bindings
        ),
        // --- replacement ---
        c!(
            "replacement.thirdparty_seed_surfaces",
            ["replacement"],
            replacement::thirdparty_seed_surfaces
        ),
        c!(
            "replacement.thirdparty_seed_invocation",
            ["replacement"],
            replacement::thirdparty_seed_invocation
        ),
        c!(
            "replacement.ambiguous_no_official_priority",
            ["replacement"],
            replacement::ambiguous_no_official_priority
        ),
        c!(
            "replacement.composition_thirdparty",
            ["replacement", "composition"],
            replacement::composition_thirdparty
        ),
        c!(
            "replacement.thirdparty_agent_runtime_surfaces",
            ["replacement", "agentic"],
            replacement::thirdparty_agent_runtime_surfaces
        ),
        c!(
            "replacement.thirdparty_agent_runtime_invocation",
            ["replacement", "agentic"],
            replacement::thirdparty_agent_runtime_invocation
        ),
        c!(
            "replacement.composition_agent_runtime_replacement",
            ["replacement", "agentic", "composition"],
            replacement::composition_agent_runtime_replacement
        ),
        // --- secret conformance ---
        c!(
            "substrate.permission_grant_rehydrate",
            ["substrate", "secret"],
            secret_conformance::permission_grant_rehydrate
        ),
        c!(
            "secret.ref_validation",
            ["secret"],
            secret_conformance::secret_ref_validation
        ),
        c!(
            "secret.raw_blocked_in_proposal",
            ["secret"],
            secret_conformance::raw_secret_blocked_in_proposal
        ),
        c!(
            "secret.raw_blocked_in_asset_metadata",
            ["secret"],
            secret_conformance::raw_secret_blocked_in_asset_metadata
        ),
        c!(
            "official.no_secret_bypass",
            ["official", "secret"],
            secret_conformance::no_secret_bypass
        ),
        c!(
            "secret.env_resolver_allowed",
            ["secret"],
            secret_conformance::env_resolver_allowed
        ),
        c!(
            "secret.env_resolver_denied",
            ["secret"],
            secret_conformance::env_resolver_denied
        ),
        c!(
            "secret.env_resolver_missing_no_leak",
            ["secret"],
            secret_conformance::env_resolver_missing_no_leak
        ),
        // --- network ---
        c!(
            "network.no_permission_denied",
            ["network", "outbound"],
            network::no_network_permission_denied
        ),
        c!(
            "network.allowlisted_host_method_allowed",
            ["network", "outbound"],
            network::allowlisted_host_method_allowed
        ),
        c!(
            "network.host_method_mismatch_denied",
            ["network", "outbound"],
            network::host_method_mismatch_denied
        ),
        c!(
            "network.official_no_network_bypass",
            ["network", "outbound"],
            network::official_no_network_bypass
        ),
        c!(
            "network.audit_no_raw_secrets",
            ["network", "outbound"],
            network::audit_no_raw_secrets
        ),
        c!(
            "network.policy_pure_function",
            ["network", "outbound"],
            network::network_policy_pure_function
        ),
        // --- outbound ---
        c!(
            "outbound.no_permission_executor_not_called",
            ["outbound", "network"],
            network::outbound_no_permission_executor_not_called
        ),
        c!(
            "outbound.policy_executor_mismatch_denied",
            ["outbound", "network"],
            network::outbound_policy_executor_mismatch_denied
        ),
        c!(
            "outbound.allowlisted_fake_executor",
            ["outbound", "network"],
            network::outbound_allowlisted_fake_executor
        ),
        c!(
            "outbound.raw_body_not_audited",
            ["outbound", "network"],
            network::outbound_raw_body_not_audited
        ),
        c!(
            "outbound.secret_refs_only",
            ["outbound", "network", "secret"],
            network::outbound_secret_refs_only
        ),
        c!(
            "outbound.host_mismatch_redirect_denied",
            ["outbound", "network"],
            network::outbound_host_mismatch_redirect_denied
        ),
        c!(
            "outbound.model_provider_shape_fake_executor",
            ["outbound", "network"],
            network::outbound_model_provider_shape_fake_executor
        ),
        // --- live http outbound ---
        c!(
            "outbound.live_http_default_disabled",
            ["outbound", "network", "live"],
            network::outbound_live_http_default_disabled
        ),
        c!(
            "outbound.live_http_rejects_insecure_url",
            ["outbound", "network", "live"],
            network::outbound_live_http_rejects_insecure_url
        ),
        c!(
            "outbound.live_http_redacted_shape",
            ["outbound", "network", "live"],
            network::outbound_live_http_redacted_shape
        ),
        // --- kernel.v1.outbound.execute ---
        c!(
            "outbound.execute_package_allowed",
            ["outbound", "network"],
            network::outbound_execute_package_allowed
        ),
        c!(
            "outbound.execute_spoofed_package_id_rejected",
            ["outbound", "network"],
            network::outbound_execute_spoofed_package_id_rejected
        ),
        c!(
            "outbound.execute_no_permission_denied",
            ["outbound", "network"],
            network::outbound_execute_no_permission_denied
        ),
        c!(
            "outbound.execute_no_raw_secret_in_response",
            ["outbound", "network", "secret"],
            network::outbound_execute_no_raw_secret_in_response
        ),
        // --- Y1: outbound execute profile conformance ---
        c!(
            "outbound_execute.profile_default_deny_all",
            ["outbound", "network", "profile"],
            network::outbound_execute_profile_default_deny_all
        ),
        c!(
            "outbound_execute.profile_fake_executor_works",
            ["outbound", "network", "profile"],
            network::outbound_execute_profile_fake_executor_works
        ),
        c!(
            "outbound_execute.profile_live_disabled_returns_deny",
            ["outbound", "network", "profile"],
            network::outbound_execute_profile_live_disabled_returns_deny
        ),
        c!(
            "outbound_websocket.profile_default_deny_all",
            ["outbound", "network", "websocket"],
            network::outbound_websocket_profile_default_deny_all
        ),
        c!(
            "outbound_websocket.profile_fake_executor_works",
            ["outbound", "network", "websocket"],
            network::outbound_websocket_profile_fake_executor_works
        ),
        c!(
            "outbound_websocket.profile_live_disabled_returns_deny",
            ["outbound", "network", "websocket"],
            network::outbound_websocket_profile_live_disabled_returns_deny
        ),
        c!(
            "outbound_websocket.secret_ref_undeclared_fails",
            ["outbound", "network", "websocket", "secret", "manifest"],
            network::outbound_websocket_secret_ref_undeclared_fails
        ),
        c!(
            "outbound_websocket.capability_namespace_enforced",
            ["outbound", "network", "websocket", "permission"],
            network::outbound_websocket_capability_namespace_enforced
        ),
        c!(
            "outbound_websocket.wss_only_default",
            ["outbound", "network", "websocket"],
            network::outbound_websocket_wss_only_default
        ),
        c!(
            "outbound_websocket.idle_timeout_emits_error_and_completed",
            ["outbound", "network", "websocket", "audit"],
            network::outbound_websocket_idle_timeout_emits_error_and_completed
        ),
        c!(
            "outbound_websocket.max_total_bytes_inbound_terminates",
            ["outbound", "network", "websocket", "audit"],
            network::outbound_websocket_max_total_bytes_inbound_terminates
        ),
        c!(
            "outbound_websocket.max_concurrent_connections_enforced",
            ["outbound", "network", "websocket"],
            network::outbound_websocket_max_concurrent_connections_enforced
        ),
        c!(
            "outbound_websocket.cancel_via_capability_cancel",
            ["outbound", "network", "websocket"],
            network::outbound_websocket_cancel_via_capability_cancel
        ),
        c!(
            "outbound.execute_completed_audit_emitted",
            ["outbound", "network", "audit"],
            network::outbound_execute_completed_audit_emitted
        ),
        c!(
            "outbound.execute_correlation_id_propagates",
            ["outbound", "network", "audit"],
            network::outbound_execute_correlation_id_propagates
        ),
        c!(
            "outbound.stream_completed_audit_emitted",
            ["outbound", "network", "stream", "audit"],
            network::outbound_stream_completed_audit_emitted
        ),
        c!(
            "outbound.websocket_completed_audit_emitted",
            ["outbound", "network", "websocket", "audit"],
            network::outbound_websocket_completed_audit_emitted
        ),
        // --- Y2: manifest permissions.secret_refs conformance ---
        c!(
            "outbound_execute.secret_ref_undeclared_fails",
            ["outbound", "network", "secret", "manifest"],
            network::outbound_execute_secret_ref_undeclared_fails
        ),
        c!(
            "outbound_execute.secret_ref_declared_resolves",
            ["outbound", "network", "secret", "manifest"],
            network::outbound_execute_secret_ref_declared_resolves
        ),
        // --- Y3: kernel.v1.outbound.stream conformance ---
        c!(
            "outbound_stream.profile_default_deny_all",
            ["outbound", "network", "stream", "profile"],
            network::outbound_stream_profile_default_deny_all
        ),
        c!(
            "outbound_stream.fake_executor_emits_canned_frames",
            ["outbound", "network", "stream"],
            network::outbound_stream_fake_executor_emits_canned_frames
        ),
        c!(
            "outbound_stream.secret_ref_undeclared_fails",
            ["outbound", "network", "stream", "secret", "manifest"],
            network::outbound_stream_secret_ref_undeclared_fails
        ),
        c!(
            "outbound_stream.secret_ref_declared_resolves",
            ["outbound", "network", "stream", "secret", "manifest"],
            network::outbound_stream_secret_ref_declared_resolves
        ),
        c!(
            "outbound_stream.capability_namespace_enforced",
            ["outbound", "network", "stream", "permission"],
            network::outbound_stream_capability_namespace_enforced
        ),
        c!(
            "outbound_stream.https_only",
            ["outbound", "network", "stream", "profile"],
            network::outbound_stream_https_only
        ),
        c!(
            "subprocess.reverse_kernel_call_dispatched",
            ["subprocess", "outbound", "network"],
            network::subprocess_reverse_kernel_call_dispatched
        ),
        c!(
            "subprocess.reverse_kernel_call_principal_locked",
            ["subprocess", "outbound", "network"],
            network::subprocess_reverse_kernel_call_principal_locked
        ),
        c!(
            "subprocess.reverse_stream_chunks_piped",
            ["subprocess", "outbound", "network", "stream"],
            network::subprocess_reverse_stream_chunks_piped
        ),
        c!(
            "sse_parser.basic_smoke",
            ["outbound", "network", "stream", "sse"],
            network::sse_parser_basic_smoke
        ),
        c!(
            "sse_parser.partial_chunks",
            ["outbound", "network", "stream", "sse"],
            network::sse_parser_partial_chunks
        ),
        // --- streaming ---
        c!(
            "stream.normal_lifecycle",
            ["stream"],
            streaming::stream_normal_lifecycle
        ),
        c!(
            "stream.cancel_blocks_chunks",
            ["stream"],
            streaming::stream_cancel_blocks_chunks
        ),
        c!(
            "stream.timeout_blocks_chunks",
            ["stream"],
            streaming::stream_timeout_blocks_chunks
        ),
        c!(
            "stream.error_terminal",
            ["stream"],
            streaming::stream_error_terminal
        ),
        c!(
            "stream.non_streaming_rejected",
            ["stream"],
            streaming::stream_non_streaming_rejected
        ),
        c!(
            "stream.no_model_agent_methods",
            ["stream"],
            streaming::stream_no_model_agent_methods
        ),
        c!(
            "stream.protocol_dispatch",
            ["stream", "protocol"],
            streaming::stream_protocol_dispatch
        ),
        // --- live model ---
        c!(
            "outbound.secret_headers_parsed",
            ["outbound", "network", "live", "secret"],
            live_model::outbound_secret_headers_parsed
        ),
        c!(
            "outbound.live_loopback_secret_injection",
            ["outbound", "network", "live", "secret"],
            live_model::outbound_live_loopback_secret_injection
        ),
        c!(
            "stream.sse_normalize_deepseek_canary",
            ["stream", "live"],
            live_model::stream_sse_normalize_deepseek_canary
        ),
        c!(
            "live_model.default_disabled_when_env_unset",
            ["live", "outbound", "network"],
            live_model::live_model_default_disabled_when_env_unset
        ),
        c!(
            "live_model.smoke_skipped_in_default_run",
            ["live", "outbound", "network"],
            live_model::live_model_smoke_skipped_in_default_run
        ),
        c!(
            "outbound.live_deepseek_opt_in",
            ["outbound", "network", "live"],
            live_model::outbound_live_deepseek_opt_in
        ),
        c!(
            "canary.deepseek_profile_shape",
            ["live", "outbound"],
            live_model::canary_deepseek_profile_shape
        ),
        // --- live model providers ---
        c!(
            "outbound.openai_chat_loopback",
            ["outbound", "network", "live"],
            live_model::openai_chat_loopback
        ),
        c!(
            "outbound.openai_responses_loopback",
            ["outbound", "network", "live"],
            live_model::openai_responses_loopback
        ),
        c!(
            "outbound.anthropic_messages_loopback",
            ["outbound", "network", "live"],
            live_model::anthropic_messages_loopback
        ),
        c!(
            "outbound.gemini_generate_content_loopback",
            ["outbound", "network", "live"],
            live_model::gemini_generate_content_loopback
        ),
        c!(
            "outbound.missing_secret_fails_closed",
            ["outbound", "network", "live", "secret"],
            live_model::missing_secret_fails_closed
        ),
        c!(
            "outbound.provider_normalize_request_alignment",
            ["outbound", "network", "live"],
            live_model::provider_normalize_request_alignment
        ),
        c!(
            "outbound.no_raw_secret_leak_all_providers",
            ["outbound", "network", "live", "secret"],
            live_model::no_raw_secret_leak_all_providers
        ),
        c!(
            "outbound.static_headers_safe_allowlist",
            ["outbound", "network", "live"],
            live_model::static_headers_safe_allowlist
        ),
        c!(
            "outbound.static_headers_block_secrets",
            ["outbound", "network", "live", "secret"],
            live_model::static_headers_block_secrets
        ),
        // --- live model quirks ---
        c!(
            "outbound.openrouter_loopback_headers",
            ["outbound", "network", "live"],
            live_model::openrouter_loopback_headers
        ),
        c!(
            "outbound.xai_loopback",
            ["outbound", "network", "live"],
            live_model::xai_loopback
        ),
        c!(
            "outbound.fireworks_loopback",
            ["outbound", "network", "live"],
            live_model::fireworks_loopback
        ),
        c!(
            "stream.deepseek_reasoning_stream",
            ["stream", "live"],
            live_model::deepseek_reasoning_stream
        ),
        c!(
            "stream.openrouter_midstream_error",
            ["stream", "live"],
            live_model::openrouter_midstream_error
        ),
        c!(
            "outbound.provider_quirk_fixtures_no_secrets",
            ["outbound", "network", "live", "secret"],
            live_model::provider_quirk_fixtures_no_secrets
        ),
        c!(
            "outbound.static_headers_openrouter_safe",
            ["outbound", "network", "live"],
            live_model::static_headers_openrouter_safe
        ),
        // --- inference local ---
        c!(
            "official.inference_local_lab_describe_capabilities",
            ["official", "slow"],
            inference_local::inference_local_lab_describe_capabilities
        ),
        c!(
            "official.inference_local_lab_invoke",
            ["official", "slow"],
            inference_local::inference_local_lab_invoke
        ),
        c!(
            "official.inference_local_lab_invoke_rejects_http",
            ["official", "slow"],
            inference_local::inference_local_lab_invoke_rejects_http
        ),
        c!(
            "official.inference_local_lab_stream",
            ["official", "slow"],
            inference_local::inference_local_lab_stream
        ),
        c!(
            "official.inference_local_lab_explain_error",
            ["official", "slow"],
            inference_local::inference_local_lab_explain_error
        ),
        // --- inference playtest ---
        c!(
            "official.inference_playtest_lab_draft",
            ["official", "slow"],
            inference_playtest::inference_playtest_draft
        ),
        c!(
            "official.inference_playtest_lab_inspect",
            ["official", "slow"],
            inference_playtest::inference_playtest_inspect
        ),
        c!(
            "official.inference_playtest_lab_reject_apply_denied",
            ["official", "slow"],
            inference_playtest::inference_playtest_reject_apply_denied
        ),
        c!(
            "official.inference_playtest_lab_apply_and_branch",
            ["official", "slow"],
            inference_playtest::inference_playtest_apply_and_branch
        ),
        c!(
            "official.inference_playtest_lab_no_chat_kernel_terms",
            ["official", "slow"],
            inference_playtest::inference_playtest_no_chat_kernel_terms
        ),
        // --- agentic forge Phase A ---
        c!(
            "agentic_forge.describe_contract",
            ["agentic"],
            agentic_forge::agentic_forge_describe_contract
        ),
        c!(
            "agentic_forge.start_run_plan_graph_working_state",
            ["agentic"],
            agentic_forge::agentic_forge_start_run
        ),
        c!(
            "agentic_forge.inspect_cancel_summarize",
            ["agentic"],
            agentic_forge::agentic_forge_inspect_cancel_summarize
        ),
        c!(
            "agentic_forge.raw_secret_blocked",
            ["agentic", "secret"],
            agentic_forge::agentic_forge_raw_secret_blocked
        ),
        c!(
            "agentic_forge.no_kernel_agent_namespace",
            ["agentic"],
            agentic_forge::agentic_forge_no_kernel_agent_namespace
        ),
        // --- agentic forge Phase B ---
        c!(
            "agentic_forge.create_candidate_branch_aware",
            ["agentic"],
            agentic_forge::agentic_forge_create_candidate
        ),
        c!(
            "agentic_forge.compare_candidate_stale_detection",
            ["agentic"],
            agentic_forge::agentic_forge_compare_candidate
        ),
        c!(
            "agentic_forge.draft_promote_proposal_no_mutation",
            ["agentic"],
            agentic_forge::agentic_forge_draft_promote_proposal
        ),
        c!(
            "agentic_forge.stale_promote_blocked",
            ["agentic"],
            agentic_forge::agentic_forge_stale_promote_blocked
        ),
        c!(
            "agentic_forge.archive_candidate_target_unchanged",
            ["agentic"],
            agentic_forge::agentic_forge_archive_candidate
        ),
        // --- agentic forge Phase C ---
        c!(
            "agentic_forge.inference_node_deterministic_candidate_seed",
            ["agentic"],
            agentic_forge::agentic_forge_inference_node_deterministic
        ),
        c!(
            "agentic_forge.replay_match_mismatch_flagged",
            ["agentic"],
            agentic_forge::agentic_forge_replay_match_mismatch
        ),
        c!(
            "agentic_forge.inference_output_privilege_escalation_rejected",
            ["agentic"],
            agentic_forge::agentic_forge_inference_output_validation
        ),
        c!(
            "agentic_forge.cloud_adapter_needs_host_policy_no_network",
            ["agentic", "network"],
            agentic_forge::agentic_forge_cloud_adapter_no_network
        ),
        c!(
            "agentic_forge.inference_failure_taxonomy_recovery_hints",
            ["agentic"],
            agentic_forge::agentic_forge_inference_failure_taxonomy
        ),
        // --- agentic forge Phase D ---
        c!(
            "agentic_forge.explain_tool_call_scoped_no_ambient_authority",
            ["agentic"],
            agentic_forge::agentic_forge_explain_tool_call_scoped
        ),
        c!(
            "agentic_forge.record_observation_untrusted_large_output_redaction",
            ["agentic"],
            agentic_forge::agentic_forge_record_observation_untrusted
        ),
        c!(
            "agentic_forge.tool_risk_injection_exfiltration_outbound",
            ["agentic", "network"],
            agentic_forge::agentic_forge_tool_risk_categories
        ),
        c!(
            "agentic_forge.replay_tool_plan_mismatch_flagged",
            ["agentic"],
            agentic_forge::agentic_forge_replay_tool_mismatch
        ),
        c!(
            "agentic_forge.plan_toolchain_requires_explicit_provider_nested_delegation_blocked",
            ["agentic"],
            agentic_forge::agentic_forge_plan_toolchain_requires_provider
        ),
        // --- agentic forge Phase F ---
        c!(
            "agentic_forge.thirdparty_replacement_shape_no_official_priority",
            ["agentic", "replacement"],
            agentic_forge::agentic_forge_thirdparty_replacement_shape
        ),
        c!(
            "agentic_forge.no_official_priority_ordinary_package",
            ["agentic", "official"],
            agentic_forge::agentic_forge_no_official_priority
        ),
        c!(
            "agentic_forge.hostile_injection_secret_blocked_cross_package",
            ["agentic", "secret"],
            agentic_forge::agentic_forge_hostile_injection_secret_blocked
        ),
        c!(
            "agentic_forge.budget_deadline_contract_cancellation_consistent",
            ["agentic"],
            agentic_forge::agentic_forge_budget_deadline_contract
        ),
        c!(
            "agentic_forge.cross_package_replay_mismatch_flagged",
            ["agentic"],
            agentic_forge::agentic_forge_cross_package_replay_consistency
        ),
        // --- experience runtime ---
        c!(
            "experience_runtime.describe_contract_shape",
            ["experience"],
            experience_runtime::experience_runtime_describe_contract
        ),
        c!(
            "experience_runtime.checkpoint_recovery_shape",
            ["experience"],
            experience_runtime::experience_runtime_checkpoint_shape
        ),
        c!(
            "experience_runtime.recovery_shape",
            ["experience"],
            experience_runtime::experience_runtime_recovery_shape
        ),
        c!(
            "experience_runtime.no_kernel_experience_namespace",
            ["experience"],
            experience_runtime::experience_runtime_no_kernel_namespace
        ),
        c!(
            "experience_runtime.template_generation",
            ["experience", "generated"],
            experience_runtime::experience_runtime_template_generation
        ),
        c!(
            "experience_runtime.bind_agent_run_shape",
            ["experience", "agentic"],
            experience_runtime::experience_runtime_bind_agent_run
        ),
        // --- playable board Beta 1 ---
        c!(
            "playable_board.describe_contract_shape",
            ["experience"],
            playable_creation_board::playable_board_describe_contract
        ),
        c!(
            "playable_board.launch_and_player_actions",
            ["experience"],
            playable_creation_board::playable_board_launch_and_player_actions
        ),
        c!(
            "playable_board.checkpoint_recovery_shape",
            ["experience"],
            playable_creation_board::playable_board_checkpoint_recovery
        ),
        c!(
            "playable_board.request_change_no_chat",
            ["experience"],
            playable_creation_board::playable_board_request_change_no_chat
        ),
        c!(
            "playable_board.bind_agent_run_scoped",
            ["experience", "agentic"],
            playable_creation_board::playable_board_bind_agent_run_scoped
        ),
        c!(
            "playable_board.candidate_proposal_no_target_mutation",
            ["experience"],
            playable_creation_board::playable_board_candidate_proposal_no_target_mutation
        ),
        c!(
            "playable_board.reject_approve_fork_proof",
            ["experience"],
            playable_creation_board::playable_board_reject_approve_fork_proof
        ),
        c!(
            "playable_board.thirdparty_no_official_priority",
            ["experience", "replacement"],
            playable_creation_board::playable_board_thirdparty_no_official_priority
        ),
        c!(
            "playable_board.no_forbidden_namespace",
            ["experience"],
            playable_creation_board::playable_board_no_forbidden_namespace
        ),
        c!(
            "playable_board.no_raw_secrets",
            ["experience", "secret"],
            playable_creation_board::playable_board_no_raw_secrets
        ),
        // --- playable board Beta 2 ---
        c!(
            "playable_board.content_address_stable",
            ["experience"],
            playable_creation_board::playable_board_content_address_stable
        ),
        c!(
            "playable_board.checkpoint_metadata",
            ["experience"],
            playable_creation_board::playable_board_checkpoint_metadata
        ),
        c!(
            "playable_board.provenance_graph",
            ["experience"],
            playable_creation_board::playable_board_provenance_graph
        ),
        c!(
            "playable_board.state_diff_preview",
            ["experience"],
            playable_creation_board::playable_board_state_diff_preview
        ),
        c!(
            "playable_board.describe_asset_provenance",
            ["experience"],
            playable_creation_board::playable_board_describe_asset_provenance
        ),
        c!(
            "playable_board.beta2_no_raw_secrets",
            ["experience", "secret"],
            playable_creation_board::playable_board_beta2_no_raw_secrets
        ),
        c!(
            "official.asset_lab_content_address",
            ["official", "slow"],
            official_labs::asset_lab_content_address
        ),
        c!(
            "official.asset_lab_provenance_graph",
            ["official", "slow"],
            official_labs::asset_lab_provenance_graph
        ),
        c!(
            "official.projection_lab_state_snapshot",
            ["official", "slow"],
            official_labs::projection_lab_state_snapshot
        ),
        // --- experience observability Beta 3 ---
        c!(
            "experience_observability.contract_shape",
            ["experience"],
            experience_observability::experience_observability_contract
        ),
        c!(
            "experience_observability.session_health",
            ["experience"],
            experience_observability::experience_observability_session_health
        ),
        c!(
            "experience_observability.package_health",
            ["experience"],
            experience_observability::experience_observability_package_health
        ),
        c!(
            "experience_observability.agent_run_health",
            ["experience", "agentic"],
            experience_observability::experience_observability_agent_run_health
        ),
        c!(
            "experience_observability.proposal_causality",
            ["experience"],
            experience_observability::experience_observability_proposal_causality
        ),
        c!(
            "experience_observability.cost_latency_summary",
            ["experience"],
            experience_observability::experience_observability_cost_latency
        ),
        c!(
            "experience_observability.failure_breadcrumbs",
            ["experience"],
            experience_observability::experience_observability_failure_breadcrumbs
        ),
        c!(
            "experience_observability.guardrail_audit_summary",
            ["experience"],
            experience_observability::experience_observability_guardrail_summary
        ),
        c!(
            "experience_observability.no_forbidden_namespace",
            ["experience"],
            experience_observability::experience_observability_no_forbidden_namespace
        ),
        c!(
            "experience_observability.no_raw_secrets",
            ["experience", "secret"],
            experience_observability::experience_observability_no_raw_secrets
        ),
        // --- memory lab Beta 4 ---
        c!(
            "memory_lab.contract_shape",
            ["memory"],
            memory_lab::memory_lab_contract
        ),
        c!(
            "memory_lab.record_memory",
            ["memory"],
            memory_lab::memory_lab_record_memory
        ),
        c!(
            "memory_lab.retrieve_memory",
            ["memory"],
            memory_lab::memory_lab_retrieve_memory
        ),
        c!(
            "memory_lab.trace_retrieval",
            ["memory"],
            memory_lab::memory_lab_trace_retrieval
        ),
        c!(
            "memory_lab.draft_update_proposal_only",
            ["memory"],
            memory_lab::memory_lab_draft_update
        ),
        c!(
            "memory_lab.correction_proposal_gated",
            ["memory"],
            memory_lab::memory_lab_correction
        ),
        c!(
            "memory_lab.forget_redaction_plan",
            ["memory"],
            memory_lab::memory_lab_forget_redaction
        ),
        c!(
            "memory_lab.branch_view",
            ["memory"],
            memory_lab::memory_lab_branch_view
        ),
        c!(
            "memory_lab.no_forbidden_namespace",
            ["memory"],
            memory_lab::memory_lab_no_forbidden_namespace
        ),
        c!(
            "memory_lab.no_raw_secrets",
            ["memory", "secret"],
            memory_lab::memory_lab_no_raw_secrets
        ),
        // --- creator loop Beta 5 ---
        c!(
            "creator_loop.playable_board_template",
            ["experience", "generated"],
            creator_loop::creator_loop_playable_board_template
        ),
        c!(
            "creator_loop.playable_experience_template",
            ["experience", "generated"],
            creator_loop::creator_loop_playable_experience_template
        ),
        c!(
            "creator_loop.experience_surface_warnings",
            ["experience", "generated"],
            creator_loop::creator_loop_experience_surface_warnings
        ),
        c!(
            "creator_loop.missing_checkpoint_warning",
            ["experience", "generated"],
            creator_loop::creator_loop_missing_checkpoint_warning
        ),
        c!(
            "creator_loop.dangerous_permissions_warning",
            ["experience", "generated"],
            creator_loop::creator_loop_dangerous_permissions_warning
        ),
        c!(
            "creator_loop.network_nondeterministic_hint",
            ["experience", "generated", "network"],
            creator_loop::creator_loop_network_nondeterministic_hint
        ),
        c!(
            "creator_loop.composition_experience_diagnostics",
            ["experience", "composition", "generated"],
            creator_loop::creator_loop_composition_experience_diagnostics
        ),
        c!(
            "creator_loop.walkthrough_reference",
            ["experience", "generated"],
            creator_loop::creator_loop_walkthrough_reference
        ),
        c!(
            "creator_loop.thirdparty_no_privilege",
            ["experience", "replacement", "generated"],
            creator_loop::creator_loop_thirdparty_no_privilege
        ),
        // --- project-intake-lab (External Project Operating Plane Alpha E1) ---
        c!(
            "project_intake.contract_shape",
            [
                "official",
                "external_project",
                "project_intake",
                "no_execution"
            ],
            project_intake_lab::project_intake_contract
        ),
        c!(
            "project_intake.source_classification",
            [
                "official",
                "external_project",
                "project_intake",
                "no_execution"
            ],
            project_intake_lab::project_intake_source_classification
        ),
        c!(
            "project_intake.stack_detection_npm_lifecycle",
            [
                "official",
                "external_project",
                "project_intake",
                "no_execution"
            ],
            project_intake_lab::project_intake_stack_detection
        ),
        c!(
            "project_intake.workspace_plan_no_execution",
            [
                "official",
                "external_project",
                "project_intake",
                "no_execution"
            ],
            project_intake_lab::project_intake_workspace_plan
        ),
        c!(
            "project_intake.local_path_rejection",
            [
                "official",
                "external_project",
                "project_intake",
                "no_execution",
                "secret"
            ],
            project_intake_lab::project_intake_local_path_rejection
        ),
        c!(
            "project_intake.adapter_plan_no_execution",
            [
                "official",
                "external_project",
                "project_intake",
                "no_execution"
            ],
            project_intake_lab::project_intake_adapter_plan
        ),
        c!(
            "project_intake.no_forbidden_namespace",
            [
                "official",
                "external_project",
                "project_intake",
                "no_execution",
                "protocol"
            ],
            project_intake_lab::project_intake_no_forbidden_namespace
        ),
        c!(
            "project_intake.no_raw_secrets",
            [
                "official",
                "external_project",
                "project_intake",
                "no_execution",
                "secret"
            ],
            project_intake_lab::project_intake_no_raw_secrets
        ),
        // --- project-intake-lab E5 (External Project Operating Plane Alpha E5: Adapter/Wrapper Generation Proof) ---
        c!(
            "project_intake.adapter_manifest_preview_no_write",
            [
                "official",
                "external_project",
                "project_intake",
                "no_execution"
            ],
            project_intake_lab::project_intake_adapter_manifest_preview_no_write
        ),
        c!(
            "project_intake.rejects_official_adapter_id",
            [
                "official",
                "external_project",
                "project_intake",
                "no_execution",
                "secret"
            ],
            project_intake_lab::project_intake_rejects_official_adapter_id
        ),
        c!(
            "project_intake.rejects_path_traversal_adapter_id",
            [
                "official",
                "external_project",
                "project_intake",
                "no_execution"
            ],
            project_intake_lab::project_intake_rejects_path_traversal_adapter_id
        ),
        c!(
            "project_intake.capability_namespace_mismatch_rejected",
            [
                "official",
                "external_project",
                "project_intake",
                "no_execution",
                "protocol"
            ],
            project_intake_lab::project_intake_capability_namespace_mismatch_rejected
        ),
        c!(
            "project_intake.wrapper_preview_no_execution",
            [
                "official",
                "external_project",
                "project_intake",
                "no_execution"
            ],
            project_intake_lab::project_intake_wrapper_preview_no_execution
        ),
        c!(
            "project_intake.fixture_preview_redacted",
            [
                "official",
                "external_project",
                "project_intake",
                "no_execution",
                "secret"
            ],
            project_intake_lab::project_intake_fixture_preview_redacted
        ),
        c!(
            "project_intake.readiness_checklist_ok",
            [
                "official",
                "external_project",
                "project_intake",
                "no_execution"
            ],
            project_intake_lab::project_intake_readiness_checklist_ok
        ),
        c!(
            "project_intake.e5_no_forbidden_namespace_no_raw_secret",
            [
                "official",
                "external_project",
                "project_intake",
                "no_execution",
                "secret",
                "protocol"
            ],
            project_intake_lab::project_intake_e5_no_forbidden_namespace_no_raw_secret
        ),
        // --- workspace-lab (External Project Operating Plane Alpha E2) ---
        c!(
            "workspace_lab.contract_shape",
            [
                "official",
                "external_project",
                "workspace_lab",
                "policy",
                "no_execution"
            ],
            workspace_lab::workspace_lab_contract
        ),
        c!(
            "workspace_lab.action_taxonomy_deny_default",
            [
                "official",
                "external_project",
                "workspace_lab",
                "policy",
                "no_execution"
            ],
            workspace_lab::workspace_lab_action_deny_default
        ),
        c!(
            "workspace_lab.policy_mismatch_fail_closed",
            [
                "official",
                "external_project",
                "workspace_lab",
                "policy",
                "no_execution"
            ],
            workspace_lab::workspace_lab_policy_mismatch
        ),
        c!(
            "workspace_lab.raw_secret_blocked",
            [
                "official",
                "external_project",
                "workspace_lab",
                "policy",
                "no_execution",
                "secret"
            ],
            workspace_lab::workspace_lab_raw_secret_blocked
        ),
        c!(
            "workspace_lab.audit_redacted",
            [
                "official",
                "external_project",
                "workspace_lab",
                "policy",
                "no_execution"
            ],
            workspace_lab::workspace_lab_audit_redacted
        ),
        c!(
            "workspace_lab.no_forbidden_namespace",
            [
                "official",
                "external_project",
                "workspace_lab",
                "policy",
                "no_execution",
                "protocol"
            ],
            workspace_lab::workspace_lab_no_forbidden_namespace
        ),
        c!(
            "workspace_lab.no_execution",
            [
                "official",
                "external_project",
                "workspace_lab",
                "policy",
                "no_execution"
            ],
            workspace_lab::workspace_lab_no_execution
        ),
        // --- workspace-lab E3 (External Project Operating Plane Alpha E3: Managed Workspace Deterministic Proof) ---
        c!(
            "workspace_lab.fixture_workspace_creation",
            [
                "official",
                "external_project",
                "workspace_lab",
                "managed_workspace",
                "no_execution",
                "fixture"
            ],
            workspace_lab::workspace_lab_fixture_workspace_creation
        ),
        c!(
            "workspace_lab.inspect_read_no_filesystem",
            [
                "official",
                "external_project",
                "workspace_lab",
                "managed_workspace",
                "no_execution",
                "fixture"
            ],
            workspace_lab::workspace_lab_inspect_read_no_filesystem
        ),
        c!(
            "workspace_lab.run_plan_requires_approval",
            [
                "official",
                "external_project",
                "workspace_lab",
                "managed_workspace",
                "no_execution",
                "fixture"
            ],
            workspace_lab::workspace_lab_run_plan_requires_approval
        ),
        c!(
            "workspace_lab.fixture_process_result_redacted",
            [
                "official",
                "external_project",
                "workspace_lab",
                "managed_workspace",
                "no_execution",
                "fixture"
            ],
            workspace_lab::workspace_lab_fixture_process_result_redacted
        ),
        c!(
            "workspace_lab.entrypoint_discovery",
            [
                "official",
                "external_project",
                "workspace_lab",
                "managed_workspace",
                "no_execution",
                "fixture"
            ],
            workspace_lab::workspace_lab_entrypoint_discovery
        ),
        c!(
            "workspace_lab.patch_draft_proposal",
            [
                "official",
                "external_project",
                "workspace_lab",
                "managed_workspace",
                "no_execution",
                "fixture"
            ],
            workspace_lab::workspace_lab_patch_draft_proposal
        ),
        c!(
            "workspace_lab.e3_raw_secret_no_forbidden_namespace",
            [
                "official",
                "external_project",
                "workspace_lab",
                "managed_workspace",
                "no_execution",
                "fixture",
                "secret",
                "protocol"
            ],
            workspace_lab::workspace_lab_e3_raw_secret_no_forbidden_namespace
        ),
        // --- integrity-lab (Package Installation Foundation I3) ---
        c!(
            "integrity.tree_hash_deterministic",
            ["official", "integrity", "package_install"],
            integrity_tools::tree_hash_deterministic
        ),
        c!(
            "integrity.tree_hash_excludes_metadata",
            ["official", "integrity", "package_install"],
            integrity_tools::tree_hash_excludes_metadata
        ),
        c!(
            "integrity.manifest_hash_yaml_json_equivalent",
            ["official", "integrity", "package_install"],
            integrity_tools::manifest_hash_yaml_json_equivalent
        ),
        c!(
            "integrity.gpg_verify_valid_signature",
            ["official", "integrity", "package_install"],
            integrity_tools::gpg_verify_valid_signature
        ),
        c!(
            "integrity.gpg_verify_wrong_key_fails",
            ["official", "integrity", "package_install"],
            integrity_tools::gpg_verify_wrong_key_fails
        ),
        c!(
            "integrity.gpg_verify_invalid_signature_no_panic",
            ["official", "integrity", "package_install"],
            integrity_tools::gpg_verify_invalid_signature_no_panic
        ),
        c!(
            "integrity.fingerprint_extraction_consistent",
            ["official", "integrity", "package_install"],
            integrity_tools::fingerprint_extraction_consistent
        ),
        // --- sharing lab Beta 6 ---
        c!(
            "sharing_lab.contract_shape",
            ["sharing"],
            sharing_lab::sharing_contract
        ),
        c!(
            "sharing_lab.export_composition_bundle",
            ["sharing", "composition"],
            sharing_lab::sharing_export_bundle
        ),
        c!(
            "sharing_lab.import_composition_bundle",
            ["sharing", "composition"],
            sharing_lab::sharing_import_bundle
        ),
        c!(
            "sharing_lab.branch_session_bundle",
            ["sharing"],
            sharing_lab::sharing_branch_session_bundle
        ),
        c!(
            "sharing_lab.package_set_lockfile",
            ["sharing"],
            sharing_lab::sharing_package_set_lockfile
        ),
        c!(
            "sharing_lab.compatibility_report",
            ["sharing"],
            sharing_lab::sharing_compatibility_report
        ),
        c!(
            "sharing_lab.ai_disclosure_bundle",
            ["sharing"],
            sharing_lab::sharing_ai_disclosure_bundle
        ),
        c!(
            "sharing_lab.read_only_share_manifest",
            ["sharing"],
            sharing_lab::sharing_read_only_manifest
        ),
        c!(
            "sharing_lab.async_fork_share_plan",
            ["sharing"],
            sharing_lab::sharing_async_fork_plan
        ),
        c!(
            "sharing_lab.no_marketplace_no_raw_secrets",
            ["sharing", "secret"],
            sharing_lab::sharing_no_marketplace_no_raw_secrets
        ),
        // --- storage backend neutrality S1 ---
        c!(
            "storage_backend.in_memory_event_store_contract_append_range",
            ["storage", "substrate"],
            storage_backend::in_memory_event_store_contract_append_range
        ),
        c!(
            "storage_backend.sqlite_event_store_contract_append_range",
            ["storage", "substrate"],
            storage_backend::sqlite_event_store_contract_append_range
        ),
        c!(
            "storage_backend.backend_parity_kind_prefix",
            ["storage", "substrate"],
            storage_backend::backend_parity_kind_prefix
        ),
        c!(
            "storage_backend.backend_parity_concurrent_append",
            ["storage", "substrate"],
            storage_backend::backend_parity_concurrent_append
        ),
        c!(
            "storage_backend.backend_parity_subscription",
            ["storage", "substrate"],
            storage_backend::backend_parity_subscription
        ),
        c!(
            "storage_backend.rehydrate_parity",
            ["storage", "substrate"],
            storage_backend::storage_backend_rehydrate_parity
        ),
        // --- storage backend neutrality S1 — PostgreSQL (opt-in) ---
        #[cfg(feature = "postgres")]
        c!(
            "storage_backend.postgres_event_store_contract_append_range",
            ["storage", "substrate", "postgres"],
            storage_backend::postgres_event_store_contract_append_range
        ),
        #[cfg(feature = "postgres")]
        c!(
            "storage_backend.postgres_backend_parity_kind_prefix",
            ["storage", "substrate", "postgres"],
            storage_backend::postgres_backend_parity_kind_prefix
        ),
        #[cfg(feature = "postgres")]
        c!(
            "storage_backend.postgres_backend_parity_concurrent_append",
            ["storage", "substrate", "postgres"],
            storage_backend::postgres_backend_parity_concurrent_append
        ),
        #[cfg(feature = "postgres")]
        c!(
            "storage_backend.postgres_backend_parity_subscription",
            ["storage", "substrate", "postgres"],
            storage_backend::postgres_backend_parity_subscription
        ),
        #[cfg(feature = "postgres")]
        c!(
            "storage_backend.postgres_rehydrate_parity",
            ["storage", "substrate", "postgres"],
            storage_backend::postgres_rehydrate_parity
        ),
        // --- storage-lab (Storage Backend Neutrality Alpha S2) ---
        c!(
            "storage_lab.contract_shape_no_kernel_database_terms",
            ["storage_lab", "storage", "official"],
            storage_lab::contract_shape_no_kernel_database_terms
        ),
        c!(
            "storage_lab.backend_classes_no_credentials",
            ["storage_lab", "storage"],
            storage_lab::backend_classes_no_secret_config
        ),
        c!(
            "storage_lab.package_state_plan_scoped",
            ["storage_lab", "storage"],
            storage_lab::package_state_plan_scoped
        ),
        c!(
            "storage_lab.put_document_preview_no_write",
            ["storage_lab", "storage"],
            storage_lab::put_document_preview_no_write
        ),
        c!(
            "storage_lab.get_document_preview_no_read",
            ["storage_lab", "storage"],
            storage_lab::get_document_preview_no_read
        ),
        c!(
            "storage_lab.query_prefix_preview_no_query_execution",
            ["storage_lab", "storage"],
            storage_lab::query_prefix_preview_no_query_execution
        ),
        c!(
            "storage_lab.delete_tombstone_preview_no_delete",
            ["storage_lab", "storage"],
            storage_lab::delete_tombstone_preview_no_delete
        ),
        c!(
            "storage_lab.export_snapshot_preview_redacted",
            ["storage_lab", "storage"],
            storage_lab::export_snapshot_preview_redacted
        ),
        c!(
            "storage_lab.raw_secret_rejected",
            ["storage_lab", "storage", "secret"],
            storage_lab::raw_secret_rejected
        ),
        c!(
            "storage_lab.unsafe_id_rejected",
            ["storage_lab", "storage"],
            storage_lab::unsafe_id_rejected
        ),
        // --- storage-lab S3 (Blob / Asset Store Contract Proof) ---
        c!(
            "storage_lab.blob_contract_shape",
            ["storage_lab", "storage", "blob"],
            storage_lab::blob_contract_shape
        ),
        c!(
            "storage_lab.put_blob_preview_content_address_deterministic",
            ["storage_lab", "storage", "blob"],
            storage_lab::put_blob_preview_content_address_deterministic
        ),
        c!(
            "storage_lab.put_blob_preview_no_storage_no_content_event",
            ["storage_lab", "storage", "blob"],
            storage_lab::put_blob_preview_no_storage_no_content_event
        ),
        c!(
            "storage_lab.get_blob_metadata_preview_no_content",
            ["storage_lab", "storage", "blob"],
            storage_lab::get_blob_metadata_preview_no_content
        ),
        c!(
            "storage_lab.export_blob_manifest_refs_only",
            ["storage_lab", "storage", "blob"],
            storage_lab::export_blob_manifest_refs_only
        ),
        c!(
            "storage_lab.blob_raw_secret_and_unsafe_id_rejected",
            ["storage_lab", "storage", "blob", "secret"],
            storage_lab::blob_raw_secret_and_unsafe_id_rejected
        ),
        // --- storage-lab S4 (Projection / Index Materialization Contract Proof) ---
        c!(
            "storage_lab.projection_contract_shape",
            ["storage_lab", "storage", "projection"],
            storage_lab::projection_contract_shape
        ),
        c!(
            "storage_lab.projection_materialization_plan_only",
            ["storage_lab", "storage", "projection"],
            storage_lab::projection_materialization_plan_only
        ),
        c!(
            "storage_lab.projection_query_preview_no_execution",
            ["storage_lab", "storage", "projection"],
            storage_lab::projection_query_preview_no_execution
        ),
        c!(
            "storage_lab.projection_migration_plan_no_rewrite",
            ["storage_lab", "storage", "projection"],
            storage_lab::projection_migration_plan_no_rewrite
        ),
        c!(
            "storage_lab.projection_rejects_raw_secret",
            ["storage_lab", "storage", "projection", "secret"],
            storage_lab::projection_rejects_raw_secret
        ),
        c!(
            "storage_lab.projection_no_db_table_leakage",
            ["storage_lab", "storage", "projection"],
            storage_lab::projection_no_db_table_leakage
        ),
        // --- storage-lab S5 (Retrieval / Vector / Multimodal Provider Contract) ---
        c!(
            "storage_lab.retrieval_contract_shape",
            ["storage_lab", "storage", "retrieval"],
            storage_lab::retrieval_contract_shape
        ),
        c!(
            "storage_lab.multimodal_index_plan_no_embedding_no_storage",
            ["storage_lab", "storage", "retrieval"],
            storage_lab::multimodal_index_plan_no_embedding_no_storage
        ),
        c!(
            "storage_lab.multimodal_index_rejects_invalid_modality_or_too_many_refs",
            ["storage_lab", "storage", "retrieval"],
            storage_lab::multimodal_index_rejects_invalid_modality_or_too_many_refs
        ),
        c!(
            "storage_lab.vector_search_plan_no_execution",
            ["storage_lab", "storage", "retrieval"],
            storage_lab::vector_search_plan_no_execution
        ),
        c!(
            "storage_lab.backend_fit_mentions_tdb_future_only",
            ["storage_lab", "storage", "retrieval", "tdb"],
            storage_lab::backend_fit_mentions_tdb_future_only
        ),
        c!(
            "storage_lab.retrieval_rejects_raw_secret",
            ["storage_lab", "storage", "retrieval", "secret"],
            storage_lab::retrieval_rejects_raw_secret
        ),
        c!(
            "storage_lab.retrieval_no_kernel_vector_namespace_or_credentials",
            ["storage_lab", "storage", "retrieval"],
            storage_lab::retrieval_no_kernel_vector_namespace_or_secret_config
        ),
        c!(
            "tdb_retrieval_lab.contract_shape",
            ["tdb", "retrieval", "storage", "official"],
            tdb_retrieval_lab::contract_shape
        ),
        c!(
            "tdb_retrieval_lab.index_plan_no_execution",
            ["tdb", "retrieval", "storage"],
            tdb_retrieval_lab::index_plan_no_execution
        ),
        c!(
            "tdb_retrieval_lab.query_plan_no_execution",
            ["tdb", "retrieval", "storage"],
            tdb_retrieval_lab::query_plan_no_execution
        ),
        c!(
            "tdb_retrieval_lab.backend_fit_boundary",
            ["tdb", "retrieval", "storage"],
            tdb_retrieval_lab::backend_fit_boundary
        ),
        c!(
            "tdb_retrieval_lab.invalid_input_rejected",
            ["tdb", "retrieval", "storage"],
            tdb_retrieval_lab::invalid_input_rejected
        ),
        c!(
            "tdb_retrieval_lab.raw_secret_and_unsafe_id_rejected",
            ["tdb", "retrieval", "storage", "secret"],
            tdb_retrieval_lab::raw_secret_and_unsafe_id_rejected
        ),
        c!(
            "tdb_retrieval_lab.real_tdb_opt_in_seam_crate_adapter_available",
            ["tdb", "retrieval", "storage"],
            tdb_retrieval_lab::real_tdb_opt_in_seam_crate_adapter_available
        ),
        // --- git-tools-lab (Package Installation Foundation I2) ---
        c!(
            "git_tools.url_validation_https_only",
            ["official", "git_tools", "install"],
            git_tools::url_validation_https_only
        ),
        c!(
            "git_tools.url_validation_no_userinfo",
            ["official", "git_tools", "install", "secret"],
            git_tools::url_validation_no_userinfo
        ),
        c!(
            "git_tools.path_validation_absolute",
            ["official", "git_tools", "install"],
            git_tools::path_validation_absolute
        ),
        c!(
            "git_tools.path_validation_no_traversal",
            ["official", "git_tools", "install"],
            git_tools::path_validation_no_traversal
        ),
        c!(
            "git_tools.read_signed_tag_unsigned",
            ["official", "git_tools", "install", "fixture"],
            git_tools::read_signed_tag_unsigned
        ),
        // --- install-lab (Package Installation Foundation I4) ---
        c!(
            "install_lab.resolve_plan_local_source",
            ["official", "install", "package_install", "fixture"],
            install_lab::resolve_plan_local_source
        ),
        c!(
            "install_lab.resolve_plan_runs_conformance",
            ["official", "install", "package_install", "fixture"],
            install_lab::resolve_plan_runs_conformance
        ),
        c!(
            "install_lab.resolve_plan_blocks_on_conformance_failure",
            ["official", "install", "package_install", "fixture"],
            install_lab::resolve_plan_blocks_on_conformance_failure
        ),
        c!(
            "install_lab.resolve_plan_ignore_conformance_overrides",
            ["official", "install", "package_install", "fixture"],
            install_lab::resolve_plan_ignore_conformance_overrides
        ),
        c!(
            "install_lab.transitive_conformance_propagates",
            ["official", "install", "package_install", "fixture"],
            install_lab::transitive_conformance_propagates
        ),
        c!(
            "install_lab.resolve_plan_with_transitive",
            ["official", "install", "package_install", "fixture"],
            install_lab::resolve_plan_with_transitive
        ),
        c!(
            "install_lab.resolve_plan_cycle_detection",
            ["official", "install", "package_install", "fixture"],
            install_lab::resolve_plan_cycle_detection
        ),
        c!(
            "install_lab.execute_plan_local",
            ["official", "install", "package_install", "fixture"],
            install_lab::execute_plan_local
        ),
        c!(
            "install_lab.execute_plan_consent_mismatch",
            ["official", "install", "package_install", "fixture"],
            install_lab::execute_plan_consent_mismatch
        ),
        c!(
            "install_lab.uninstall_removes_from_profile",
            ["official", "install", "package_install", "fixture"],
            install_lab::uninstall_removes_from_profile
        ),
        c!(
            "install_lab.list_installed_reflects_lockfile",
            ["official", "install", "package_install", "fixture"],
            install_lab::list_installed_reflects_lockfile
        ),
        c!(
            "install_lab.check_lockfile_drift_detection",
            ["official", "install", "package_install", "fixture"],
            install_lab::check_lockfile_drift_detection
        ),
        c!(
            "install.real_github_smoke",
            ["install", "real-network", "opt-in"],
            install_real_smoke::real_github_smoke
        ),
        // --- real TDB Rust adapter subprocess proof ---
        c!(
            "tdb_rust_adapter.subprocess_adapter_shell_invokes_disabled_smoke",
            ["tdb", "retrieval", "subprocess", "slow"],
            tdb_rust_adapter::subprocess_adapter_shell_invokes_disabled_smoke
        ),
        c!(
            "tdb_rust_adapter.subprocess_adapter_rejects_secret_and_raw_path",
            ["tdb", "retrieval", "subprocess", "secret", "slow"],
            tdb_rust_adapter::subprocess_adapter_rejects_secret_and_raw_path
        ),
        c!(
            "tdb_rust_adapter.real_crate_smoke_opt_in",
            ["tdb", "tdb_real", "retrieval", "slow"],
            tdb_rust_adapter::real_crate_smoke_opt_in
        ),
    ]
}

/// Format a duration for display.
fn fmt_duration(d: std::time::Duration) -> String {
    if d < std::time::Duration::from_millis(1) {
        format!("{:.0}µs", d.as_micros())
    } else if d < std::time::Duration::from_secs(1) {
        format!("{:.1}ms", d.as_secs_f64() * 1000.0)
    } else {
        format!("{:.2}s", d.as_secs_f64())
    }
}

pub(crate) async fn run(opts: ConformanceOptions) -> anyhow::Result<()> {
    let all_cases = build_cases();

    // --- list mode: print ids + tags and exit ---
    if opts.list {
        for case in &all_cases {
            let tags = case.tags.join(", ");
            println!("{:<55} [{}]", case.id, tags);
        }
        println!("total: {} cases", all_cases.len());
        return Ok(());
    }

    // --- filter cases ---
    let selected: Vec<&ConformanceCase> = all_cases
        .iter()
        .filter(|case| {
            // --case <substring>: case id must contain at least one matching substring
            let case_match =
                opts.case.is_empty() || opts.case.iter().any(|p| case.id.contains(p.as_str()));
            // --tag <tag>: case must have at least one matching tag
            let tag_match =
                opts.tag.is_empty() || opts.tag.iter().any(|t| case.tags.contains(&t.as_str()));
            case_match && tag_match
        })
        .collect();

    if selected.is_empty() {
        eprintln!("no conformance cases matched the given filters");
        anyhow::bail!("no cases selected");
    }

    // --- execute cases ---
    let mut results: Vec<(&ConformanceCase, anyhow::Result<()>, std::time::Duration)> = Vec::new();
    let mut failed = false;

    for case in &selected {
        let start = Instant::now();
        let result = (case.run)().await;
        let elapsed = start.elapsed();
        let ok = result.is_ok();
        if !ok {
            failed = true;
        }
        match &result {
            Ok(()) => println!("{:<55} PASS  {}", case.id, fmt_duration(elapsed)),
            Err(err) => println!("{:<55} FAIL  {}  {}", case.id, fmt_duration(elapsed), err),
        }
        results.push((case, result, elapsed));
        if !ok && opts.fail_fast {
            break;
        }
    }

    // --- slowest report ---
    if opts.slowest > 0 && results.len() > 1 {
        let mut timed: Vec<(&ConformanceCase, std::time::Duration)> =
            results.iter().map(|(case, _, dur)| (*case, *dur)).collect();
        timed.sort_by(|a, b| b.1.cmp(&a.1));
        let n = opts.slowest.min(timed.len());
        println!();
        println!("slowest {} cases:", n);
        for (case, elapsed) in timed.iter().take(n) {
            let was_ok = results.iter().any(|(c, r, _)| c.id == case.id && r.is_ok());
            let status = if was_ok { "PASS" } else { "FAIL" };
            println!("  {:<55} {}  {}", case.id, status, fmt_duration(*elapsed));
        }
    }

    // --- summary ---
    let pass_count = results.iter().filter(|r| r.1.is_ok()).count();
    let fail_count = results.len() - pass_count;
    if failed {
        println!();
        anyhow::bail!(
            "conformance: {}/{} passed, {} failed",
            pass_count,
            results.len(),
            fail_count
        );
    }
    println!(
        "\nconformance: ok ({} cases, {})",
        results.len(),
        fmt_duration(results.iter().map(|r| r.2).sum::<std::time::Duration>())
    );
    Ok(())
}
