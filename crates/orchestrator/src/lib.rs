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
        AiSessionExchange, LocalAiSessionSnapshot, LocalAiSessionState, LOCAL_AI_SESSIONS_DIR_NAME,
    },
    approval::{
        evaluate_approval, request_approval, ApprovalActionKind, ApprovalDecision, ApprovalRequest,
    },
    executor::{
        AsyncCommandExecutor, CommandHandle, CommandProgress, CommandResult, ProgressSender,
    },
    plugins::{
        command_plugin_count, list_command_plugins, register_command_plugin,
        set_command_plugin_enabled, CommandHookContext, CommandPlugin, PluginDescriptor,
        PluginMetadata, PluginRegistryError,
    },
    processor::{process_command, process_command_with_interrupt, process_text},
};
pub use models::{get_main_action, ExitStatus, MainAction};
