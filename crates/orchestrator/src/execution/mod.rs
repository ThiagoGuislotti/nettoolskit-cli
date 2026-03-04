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
/// Async command executor with progress tracking.
pub mod executor;
/// Plugin foundation for command pre/post hooks.
pub mod plugins;
/// Command dispatch and text processing routines.
pub mod processor;

// Re-export commonly used types
pub use ai::{
    AiChunk, AiMessage, AiProvider, AiProviderError, AiRequest, AiResponse, AiRole, AiUsage,
    MockAiOutcome, MockAiProvider, OpenAiCompatibleProvider, OpenAiCompatibleProviderConfig,
};
pub use ai_session::{
    active_ai_session_id, list_local_ai_session_snapshots, load_local_ai_session_from_path,
    prune_local_ai_session_snapshots, resolve_active_ai_session_id, set_active_ai_session_id,
    AiSessionExchange, LocalAiSessionSnapshot, LocalAiSessionState, LOCAL_AI_SESSIONS_DIR_NAME,
};
pub use approval::{
    evaluate_approval, request_approval, ApprovalActionKind, ApprovalDecision, ApprovalRequest,
};
pub use executor::{
    AsyncCommandExecutor, CommandHandle, CommandProgress, CommandResult, ProgressSender,
};
pub use plugins::{
    command_plugin_count, list_command_plugins, register_command_plugin,
    set_command_plugin_enabled, CommandHookContext, CommandPlugin, PluginDescriptor,
    PluginMetadata, PluginRegistryError,
};
pub use processor::{process_command, process_command_with_interrupt, process_text};
