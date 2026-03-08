//! Command orchestration for NetToolsKit CLI
//!
//! This crate provides the orchestration layer between the CLI interface
//! and command implementations, including:
//! - Command models and menu system
//! - Async command execution with progress tracking
//! - Command processor for dispatch and routing

pub mod execution;
pub mod models;

// Re-export commonly used types
pub use execution::{
    ai::{
        AiChunk, AiMessage, AiProvider, AiProviderError, AiRequest, AiResponse, AiRole, AiUsage,
        MockAiOutcome, MockAiProvider, OpenAiCompatibleProvider, OpenAiCompatibleProviderConfig,
    },
    ai_session::{
        active_ai_session_id, list_local_ai_session_snapshots, load_local_ai_session_from_path,
        prune_local_ai_session_snapshots, resolve_active_ai_session_id, set_active_ai_session_id,
        AiSessionCompressionMode, AiSessionExchange, LocalAiSessionSnapshot, LocalAiSessionState,
        LOCAL_AI_SESSIONS_DIR_NAME, NTK_AI_SESSION_COMPRESSION_MAX_CHARS_ENV,
        NTK_AI_SESSION_COMPRESSION_MODE_ENV, NTK_AI_SESSION_DELTA_MIN_SHARED_PREFIX_CHARS_ENV,
    },
    approval::{
        evaluate_approval, request_approval, ApprovalActionKind, ApprovalDecision, ApprovalRequest,
    },
    chatops::{
        execute_chatops_envelope, parse_chatops_intent, process_chatops_inbox, ChatOpsAdapterError,
        ChatOpsAuditEntry, ChatOpsAuditKind, ChatOpsAuthorizationError, ChatOpsAuthorizationPolicy,
        ChatOpsCommandEnvelope, ChatOpsExecutionError, ChatOpsIngress, ChatOpsIntent,
        ChatOpsLocalAuditStore, ChatOpsNotification, ChatOpsNotificationSeverity, ChatOpsNotifier,
        ChatOpsPlatform, MockChatOpsIngress, RecordingChatOpsNotifier,
    },
    chatops_runtime::{
        build_chatops_runtime, build_chatops_runtime_from_env, ChatOpsRuntime,
        ChatOpsRuntimeConfig, ChatOpsTickSummary, DiscordInteractionIngressOutcome,
    },
    executor::{
        AsyncCommandExecutor, CommandHandle, CommandProgress, CommandResult, ProgressSender,
    },
    plugins::{
        command_plugin_count, list_command_plugins, register_command_plugin,
        set_command_plugin_enabled, CommandHookContext, CommandPlugin, PluginDescriptor,
        PluginMetadata, PluginRegistryError,
    },
    processor::{
        process_command, process_command_with_interrupt, process_control_envelope, process_text,
        TaskSubmissionOutcome,
    },
    repo_workflow::{
        execute_repo_workflow, parse_repo_workflow_payload, validate_repo_workflow_request,
        RepoWorkflowError, RepoWorkflowPlan, RepoWorkflowPolicy, RepoWorkflowRequest,
        RepoWorkflowResult, NTK_REPO_WORKFLOW_ALLOWED_COMMANDS_ENV,
        NTK_REPO_WORKFLOW_ALLOWED_HOSTS_ENV, NTK_REPO_WORKFLOW_ALLOW_PR_ENV,
        NTK_REPO_WORKFLOW_ALLOW_PUSH_ENV, NTK_REPO_WORKFLOW_BASE_DIR_ENV,
        NTK_REPO_WORKFLOW_ENABLED_ENV,
    },
};
pub use models::{get_main_action, ExitStatus, MainAction};
