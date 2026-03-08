//! Execution Module
//!
//! Command execution and processing logic.

/// AI provider abstraction and deterministic mock provider.
pub mod ai;
/// Local AI session persistence and resume primitives.
pub mod ai_session;
/// Approval gateway for AI side-effect operations.
pub mod approval;
/// Internal command cache primitives used by processor runtime.
mod cache;
/// ChatOps contracts and execution pipeline for Telegram/Discord adapters.
pub mod chatops;
/// ChatOps runtime adapters and service-loop orchestration.
pub mod chatops_runtime;
/// Async command executor with progress tracking.
pub mod executor;
/// Plugin foundation for command pre/post hooks.
pub mod plugins;
/// Command dispatch and text processing routines.
pub mod processor;
/// Repository workflow automation with explicit policy gates.
pub mod repo_workflow;

// Re-export commonly used types
pub use ai::{
    AiChunk, AiMessage, AiProvider, AiProviderError, AiRequest, AiResponse, AiRole, AiUsage,
    MockAiOutcome, MockAiProvider, OpenAiCompatibleProvider, OpenAiCompatibleProviderConfig,
};
pub use ai_session::{
    active_ai_session_id, list_local_ai_session_snapshots, load_local_ai_session_from_path,
    prune_local_ai_session_snapshots, resolve_active_ai_session_id, set_active_ai_session_id,
    AiSessionCompressionMode, AiSessionExchange, LocalAiSessionSnapshot, LocalAiSessionState,
    LOCAL_AI_SESSIONS_DIR_NAME, NTK_AI_SESSION_COMPRESSION_MAX_CHARS_ENV,
    NTK_AI_SESSION_COMPRESSION_MODE_ENV, NTK_AI_SESSION_DELTA_MIN_SHARED_PREFIX_CHARS_ENV,
};
pub use approval::{
    evaluate_approval, request_approval, ApprovalActionKind, ApprovalDecision, ApprovalRequest,
};
pub use chatops::{
    execute_chatops_envelope, parse_chatops_intent, process_chatops_inbox, ChatOpsAdapterError,
    ChatOpsAuditEntry, ChatOpsAuditKind, ChatOpsAuthorizationError, ChatOpsAuthorizationPolicy,
    ChatOpsCommandEnvelope, ChatOpsExecutionError, ChatOpsIngress, ChatOpsIntent,
    ChatOpsLocalAuditStore, ChatOpsNotification, ChatOpsNotificationSeverity, ChatOpsNotifier,
    ChatOpsPlatform, MockChatOpsIngress, RecordingChatOpsNotifier,
};
pub use chatops_runtime::{
    build_chatops_runtime, build_chatops_runtime_from_env, ChatOpsRuntime, ChatOpsRuntimeConfig,
    ChatOpsTickSummary, DiscordInteractionIngressOutcome,
};
pub use executor::{
    AsyncCommandExecutor, CommandHandle, CommandProgress, CommandResult, ProgressSender,
};
pub use plugins::{
    command_plugin_count, list_command_plugins, register_command_plugin,
    set_command_plugin_enabled, CommandHookContext, CommandPlugin, PluginDescriptor,
    PluginMetadata, PluginRegistryError,
};
pub use processor::{
    process_command, process_command_with_interrupt, process_control_envelope, process_text,
    TaskSubmissionOutcome,
};
pub use repo_workflow::{
    execute_repo_workflow, parse_repo_workflow_payload, validate_repo_workflow_request,
    RepoWorkflowError, RepoWorkflowPlan, RepoWorkflowPolicy, RepoWorkflowRequest,
    RepoWorkflowResult, NTK_REPO_WORKFLOW_ALLOWED_COMMANDS_ENV,
    NTK_REPO_WORKFLOW_ALLOWED_HOSTS_ENV, NTK_REPO_WORKFLOW_ALLOW_PR_ENV,
    NTK_REPO_WORKFLOW_ALLOW_PUSH_ENV, NTK_REPO_WORKFLOW_BASE_DIR_ENV,
    NTK_REPO_WORKFLOW_ENABLED_ENV,
};
