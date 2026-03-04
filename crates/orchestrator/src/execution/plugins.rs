//! Plugin foundation for command orchestration.
//!
//! This module provides a deterministic in-process plugin registry that can
//! hook command execution before and after command dispatch.

use crate::models::ExitStatus;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::panic::{self, AssertUnwindSafe};
use std::sync::{Arc, Mutex, OnceLock};

/// Immutable plugin metadata used for registration and listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginMetadata {
    /// Unique plugin id (stable key).
    pub id: String,
    /// Human readable plugin name.
    pub name: String,
    /// Plugin semantic version string.
    pub version: String,
    /// Short plugin description.
    pub description: String,
}

impl PluginMetadata {
    /// Build plugin metadata from string slices.
    pub fn new(id: &str, name: &str, version: &str, description: &str) -> Self {
        Self {
            id: id.trim().to_string(),
            name: name.trim().to_string(),
            version: version.trim().to_string(),
            description: description.trim().to_string(),
        }
    }
}

/// Read-only plugin listing entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginDescriptor {
    /// Plugin metadata.
    pub metadata: PluginMetadata,
    /// Runtime enabled/disabled state.
    pub enabled: bool,
}

/// Hook context shared with command plugins.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandHookContext {
    /// Correlation id for current command execution.
    pub correlation_id: String,
    /// Raw command string.
    pub command: String,
    /// Normalized metric key for the command.
    pub command_key: String,
    /// Optional final command status (available on after-hook).
    pub status: Option<ExitStatus>,
}

/// Hook invocation stats for metrics and diagnostics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PluginHookStats {
    /// Number of plugins invoked for this hook.
    pub invoked: usize,
    /// Number of hook failures (error or panic) observed.
    pub failures: usize,
}

/// Registration and lifecycle errors in plugin registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginRegistryError {
    /// Plugin id is empty or invalid.
    InvalidId,
    /// Another plugin with same id already exists.
    DuplicateId(String),
    /// Plugin id was not found.
    NotFound(String),
}

impl Display for PluginRegistryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidId => write!(f, "plugin id must not be empty"),
            Self::DuplicateId(id) => write!(f, "plugin '{id}' is already registered"),
            Self::NotFound(id) => write!(f, "plugin '{id}' was not found"),
        }
    }
}

impl std::error::Error for PluginRegistryError {}

/// Plugin contract for command hooks.
pub trait CommandPlugin: Send + Sync {
    /// Static plugin metadata.
    fn metadata(&self) -> PluginMetadata;

    /// Hook executed before command dispatch.
    fn on_before_command(&self, _context: &CommandHookContext) -> Result<(), String> {
        Ok(())
    }

    /// Hook executed after command dispatch.
    fn on_after_command(&self, _context: &CommandHookContext) -> Result<(), String> {
        Ok(())
    }
}

struct RegisteredPlugin {
    metadata: PluginMetadata,
    plugin: Arc<dyn CommandPlugin>,
    enabled: bool,
}

/// In-process plugin registry with deterministic ordering.
#[derive(Default)]
pub struct PluginRegistry {
    plugins: BTreeMap<String, RegisteredPlugin>,
}

impl PluginRegistry {
    /// Register a command plugin. Plugin ids must be unique and non-empty.
    pub fn register(
        &mut self,
        plugin: Arc<dyn CommandPlugin>,
    ) -> Result<PluginMetadata, PluginRegistryError> {
        let metadata = plugin.metadata();
        if metadata.id.trim().is_empty() {
            return Err(PluginRegistryError::InvalidId);
        }
        if self.plugins.contains_key(&metadata.id) {
            return Err(PluginRegistryError::DuplicateId(metadata.id));
        }

        let key = metadata.id.clone();
        let registered = RegisteredPlugin {
            metadata: metadata.clone(),
            plugin,
            enabled: true,
        };
        self.plugins.insert(key, registered);
        Ok(metadata)
    }

    /// Enable/disable a registered plugin by id.
    pub fn set_enabled(
        &mut self,
        plugin_id: &str,
        enabled: bool,
    ) -> Result<(), PluginRegistryError> {
        let Some(plugin) = self.plugins.get_mut(plugin_id) else {
            return Err(PluginRegistryError::NotFound(plugin_id.to_string()));
        };

        plugin.enabled = enabled;
        Ok(())
    }

    /// Number of currently enabled plugins.
    pub fn enabled_count(&self) -> usize {
        self.plugins
            .values()
            .filter(|plugin| plugin.enabled)
            .count()
    }

    /// List registered plugins with current state.
    pub fn list(&self) -> Vec<PluginDescriptor> {
        self.plugins
            .values()
            .map(|plugin| PluginDescriptor {
                metadata: plugin.metadata.clone(),
                enabled: plugin.enabled,
            })
            .collect()
    }

    /// Execute before-command hooks for all enabled plugins.
    pub fn run_before_hooks(&self, context: &CommandHookContext) -> PluginHookStats {
        self.run_hooks("before", context, |plugin, hook_context| {
            plugin.on_before_command(hook_context)
        })
    }

    /// Execute after-command hooks for all enabled plugins.
    pub fn run_after_hooks(&self, context: &CommandHookContext) -> PluginHookStats {
        self.run_hooks("after", context, |plugin, hook_context| {
            plugin.on_after_command(hook_context)
        })
    }

    fn run_hooks(
        &self,
        phase: &str,
        context: &CommandHookContext,
        invoke: impl Fn(&dyn CommandPlugin, &CommandHookContext) -> Result<(), String>,
    ) -> PluginHookStats {
        let mut stats = PluginHookStats::default();

        for registered in self.plugins.values().filter(|plugin| plugin.enabled) {
            stats.invoked += 1;
            let plugin_id = registered.metadata.id.as_str();
            let hook_result = panic::catch_unwind(AssertUnwindSafe(|| {
                invoke(registered.plugin.as_ref(), context)
            }));

            match hook_result {
                Ok(Ok(())) => {}
                Ok(Err(err)) => {
                    stats.failures += 1;
                    tracing::warn!(
                        plugin_id = plugin_id,
                        phase = phase,
                        command = context.command,
                        "Plugin hook error: {err}"
                    );
                }
                Err(_) => {
                    stats.failures += 1;
                    tracing::warn!(
                        plugin_id = plugin_id,
                        phase = phase,
                        command = context.command,
                        "Plugin hook panicked"
                    );
                }
            }
        }

        stats
    }
}

static PLUGIN_REGISTRY: OnceLock<Mutex<PluginRegistry>> = OnceLock::new();

fn with_plugin_registry<T>(f: impl FnOnce(&mut PluginRegistry) -> T) -> T {
    let registry = PLUGIN_REGISTRY.get_or_init(|| Mutex::new(PluginRegistry::default()));
    let mut guard = registry
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    f(&mut guard)
}

/// Register a plugin in the process-wide command registry.
pub fn register_command_plugin(
    plugin: Arc<dyn CommandPlugin>,
) -> Result<PluginMetadata, PluginRegistryError> {
    with_plugin_registry(|registry| registry.register(plugin))
}

/// Toggle a plugin on or off in the process-wide command registry.
pub fn set_command_plugin_enabled(
    plugin_id: &str,
    enabled: bool,
) -> Result<(), PluginRegistryError> {
    with_plugin_registry(|registry| registry.set_enabled(plugin_id, enabled))
}

/// List process-wide registered plugins.
pub fn list_command_plugins() -> Vec<PluginDescriptor> {
    with_plugin_registry(|registry| registry.list())
}

/// Count enabled plugins in process-wide registry.
pub fn command_plugin_count() -> usize {
    with_plugin_registry(|registry| registry.enabled_count())
}

/// Execute before-command hooks in process-wide registry.
pub fn run_before_command_plugins(context: &CommandHookContext) -> PluginHookStats {
    with_plugin_registry(|registry| registry.run_before_hooks(context))
}

/// Execute after-command hooks in process-wide registry.
pub fn run_after_command_plugins(context: &CommandHookContext) -> PluginHookStats {
    with_plugin_registry(|registry| registry.run_after_hooks(context))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountingPlugin {
        metadata: PluginMetadata,
        before: Arc<AtomicUsize>,
        after: Arc<AtomicUsize>,
        fail_before: bool,
        fail_after: bool,
    }

    impl CountingPlugin {
        fn new(
            id: &str,
            before: Arc<AtomicUsize>,
            after: Arc<AtomicUsize>,
            fail_before: bool,
            fail_after: bool,
        ) -> Self {
            Self {
                metadata: PluginMetadata::new(id, "Counter", "1.0.0", "test plugin"),
                before,
                after,
                fail_before,
                fail_after,
            }
        }
    }

    impl CommandPlugin for CountingPlugin {
        fn metadata(&self) -> PluginMetadata {
            self.metadata.clone()
        }

        fn on_before_command(&self, _context: &CommandHookContext) -> Result<(), String> {
            self.before.fetch_add(1, Ordering::SeqCst);
            if self.fail_before {
                return Err("before hook failed".to_string());
            }
            Ok(())
        }

        fn on_after_command(&self, _context: &CommandHookContext) -> Result<(), String> {
            self.after.fetch_add(1, Ordering::SeqCst);
            if self.fail_after {
                return Err("after hook failed".to_string());
            }
            Ok(())
        }
    }

    fn sample_context() -> CommandHookContext {
        CommandHookContext {
            correlation_id: "cmd-1".to_string(),
            command: "/help".to_string(),
            command_key: "help".to_string(),
            status: Some(ExitStatus::Success),
        }
    }

    #[test]
    fn registry_registers_and_lists_plugins() {
        let before = Arc::new(AtomicUsize::new(0));
        let after = Arc::new(AtomicUsize::new(0));
        let plugin = Arc::new(CountingPlugin::new(
            "com.example.counter",
            before,
            after,
            false,
            false,
        ));

        let mut registry = PluginRegistry::default();
        let metadata = registry.register(plugin).expect("registration must pass");
        assert_eq!(metadata.id, "com.example.counter");

        let listed = registry.list();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].metadata.id, "com.example.counter");
        assert!(listed[0].enabled);
    }

    #[test]
    fn registry_rejects_duplicate_plugin_ids() {
        let before = Arc::new(AtomicUsize::new(0));
        let after = Arc::new(AtomicUsize::new(0));

        let plugin_a = Arc::new(CountingPlugin::new(
            "com.example.dup",
            Arc::clone(&before),
            Arc::clone(&after),
            false,
            false,
        ));
        let plugin_b = Arc::new(CountingPlugin::new(
            "com.example.dup",
            before,
            after,
            false,
            false,
        ));

        let mut registry = PluginRegistry::default();
        registry.register(plugin_a).expect("first registration");
        let err = registry
            .register(plugin_b)
            .expect_err("duplicate id must fail");
        assert_eq!(
            err,
            PluginRegistryError::DuplicateId("com.example.dup".to_string())
        );
    }

    #[test]
    fn disabled_plugins_are_skipped_in_hook_execution() {
        let before = Arc::new(AtomicUsize::new(0));
        let after = Arc::new(AtomicUsize::new(0));
        let plugin = Arc::new(CountingPlugin::new(
            "com.example.toggle",
            Arc::clone(&before),
            Arc::clone(&after),
            false,
            false,
        ));

        let mut registry = PluginRegistry::default();
        registry.register(plugin).expect("registration must pass");
        registry
            .set_enabled("com.example.toggle", false)
            .expect("toggle must pass");

        let before_stats = registry.run_before_hooks(&sample_context());
        let after_stats = registry.run_after_hooks(&sample_context());

        assert_eq!(before_stats.invoked, 0);
        assert_eq!(after_stats.invoked, 0);
        assert_eq!(before.load(Ordering::SeqCst), 0);
        assert_eq!(after.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn hook_failures_are_counted_and_non_blocking() {
        let before = Arc::new(AtomicUsize::new(0));
        let after = Arc::new(AtomicUsize::new(0));
        let plugin = Arc::new(CountingPlugin::new(
            "com.example.failing",
            Arc::clone(&before),
            Arc::clone(&after),
            true,
            true,
        ));

        let mut registry = PluginRegistry::default();
        registry.register(plugin).expect("registration must pass");

        let before_stats = registry.run_before_hooks(&sample_context());
        let after_stats = registry.run_after_hooks(&sample_context());

        assert_eq!(before_stats.invoked, 1);
        assert_eq!(before_stats.failures, 1);
        assert_eq!(after_stats.invoked, 1);
        assert_eq!(after_stats.failures, 1);
        assert_eq!(before.load(Ordering::SeqCst), 1);
        assert_eq!(after.load(Ordering::SeqCst), 1);
    }
}
