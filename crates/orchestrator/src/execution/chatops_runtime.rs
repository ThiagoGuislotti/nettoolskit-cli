//! ChatOps runtime adapters and service-loop orchestration.
//!
//! This module wires platform adapters (Telegram/Discord) into the existing
//! ChatOps command pipeline while preserving local-first persistence and
//! explicit authorization policy controls.

use super::chatops::{
    execute_chatops_envelope, ChatOpsAdapterError, ChatOpsAuditEntry, ChatOpsAuditKind,
    ChatOpsAuthorizationPolicy, ChatOpsCommandEnvelope, ChatOpsLocalAuditStore,
    ChatOpsNotification, ChatOpsNotificationSeverity, ChatOpsNotifier, ChatOpsPlatform,
};
use crate::models::ExitStatus;
use nettoolskit_core::IngressTransport;
use serde::Deserialize;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const DEFAULT_CHATOPS_POLL_INTERVAL_MS: u64 = 3_000;
const DEFAULT_CHATOPS_MAX_BATCH_SIZE: usize = 16;
const DEFAULT_CHATOPS_RATE_LIMIT_WINDOW_SECONDS: u64 = 60;
const DEFAULT_TELEGRAM_API_BASE: &str = "https://api.telegram.org";
const DEFAULT_DISCORD_API_BASE: &str = "https://discord.com/api/v10";

/// Environment variable to enable ChatOps runtime.
pub const NTK_CHATOPS_ENABLED_ENV: &str = "NTK_CHATOPS_ENABLED";
/// Environment variable for allowlisted user ids.
pub const NTK_CHATOPS_ALLOWED_USERS_ENV: &str = "NTK_CHATOPS_ALLOWED_USERS";
/// Environment variable for allowlisted channel ids.
pub const NTK_CHATOPS_ALLOWED_CHANNELS_ENV: &str = "NTK_CHATOPS_ALLOWED_CHANNELS";
/// Environment variable for allowlisted command scopes.
pub const NTK_CHATOPS_ALLOWED_COMMANDS_ENV: &str = "NTK_CHATOPS_ALLOWED_COMMANDS";
/// Environment variable for polling interval (milliseconds).
pub const NTK_CHATOPS_POLL_INTERVAL_MS_ENV: &str = "NTK_CHATOPS_POLL_INTERVAL_MS";
/// Environment variable for max envelopes processed per poll.
pub const NTK_CHATOPS_MAX_BATCH_ENV: &str = "NTK_CHATOPS_MAX_BATCH";
/// Environment variable for per-user rate-limit budget.
pub const NTK_CHATOPS_RATE_LIMIT_PER_USER_ENV: &str = "NTK_CHATOPS_RATE_LIMIT_PER_USER";
/// Environment variable for per-channel rate-limit budget.
pub const NTK_CHATOPS_RATE_LIMIT_PER_CHANNEL_ENV: &str = "NTK_CHATOPS_RATE_LIMIT_PER_CHANNEL";
/// Environment variable for rate-limit window (seconds).
pub const NTK_CHATOPS_RATE_LIMIT_WINDOW_SECONDS_ENV: &str = "NTK_CHATOPS_RATE_LIMIT_WINDOW_SECONDS";
/// Environment variable for rate-limit strategy (`fixed_window` or `token_bucket`).
pub const NTK_CHATOPS_RATE_LIMIT_STRATEGY_ENV: &str = "NTK_CHATOPS_RATE_LIMIT_STRATEGY";
/// Environment variable for per-user token-bucket burst capacity.
pub const NTK_CHATOPS_RATE_LIMIT_BURST_PER_USER_ENV: &str = "NTK_CHATOPS_RATE_LIMIT_BURST_PER_USER";
/// Environment variable for per-channel token-bucket burst capacity.
pub const NTK_CHATOPS_RATE_LIMIT_BURST_PER_CHANNEL_ENV: &str =
    "NTK_CHATOPS_RATE_LIMIT_BURST_PER_CHANNEL";
/// Environment variable for ingress-driven rate-limit auto-tuning profile.
pub const NTK_CHATOPS_RATE_LIMIT_AUTOTUNE_PROFILE_ENV: &str =
    "NTK_CHATOPS_RATE_LIMIT_AUTOTUNE_PROFILE";
/// Environment variable for custom audit file path.
pub const NTK_CHATOPS_AUDIT_PATH_ENV: &str = "NTK_CHATOPS_AUDIT_PATH";
/// Environment variable for Telegram bot token.
pub const NTK_CHATOPS_TELEGRAM_TOKEN_ENV: &str = "NTK_CHATOPS_TELEGRAM_TOKEN";
/// Environment variable for Telegram API base URL override.
pub const NTK_CHATOPS_TELEGRAM_API_BASE_ENV: &str = "NTK_CHATOPS_TELEGRAM_API_BASE";
/// Environment variable to switch Telegram ingress to webhook queue mode.
pub const NTK_CHATOPS_TELEGRAM_WEBHOOK_ENABLED_ENV: &str = "NTK_CHATOPS_TELEGRAM_WEBHOOK_ENABLED";
/// Environment variable for Discord bot token.
pub const NTK_CHATOPS_DISCORD_TOKEN_ENV: &str = "NTK_CHATOPS_DISCORD_TOKEN";
/// Environment variable for Discord API base URL override.
pub const NTK_CHATOPS_DISCORD_API_BASE_ENV: &str = "NTK_CHATOPS_DISCORD_API_BASE";
/// Environment variable for Discord channel ids (comma/semicolon-separated).
pub const NTK_CHATOPS_DISCORD_CHANNELS_ENV: &str = "NTK_CHATOPS_DISCORD_CHANNELS";
/// Environment variable to switch Discord ingress to interaction webhook queue mode.
pub const NTK_CHATOPS_DISCORD_INTERACTIONS_ENABLED_ENV: &str =
    "NTK_CHATOPS_DISCORD_INTERACTIONS_ENABLED";

/// Rate-limit strategy for ChatOps ingress throttling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatOpsRateLimitStrategy {
    /// Fixed-window counters (legacy/default behavior).
    FixedWindow,
    /// Token-bucket with configurable burst budget.
    TokenBucket,
}

/// Auto-tuning profile for switching rate-limit strategy based on observed ingress traffic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatOpsRateLimitAutoTuneProfile {
    /// Keep static strategy selection.
    Disabled,
    /// Requires sustained traffic change before switching strategy.
    Conservative,
    /// Balanced strategy switching thresholds.
    Balanced,
    /// React faster to traffic changes.
    Aggressive,
}

impl ChatOpsRateLimitAutoTuneProfile {
    const fn is_enabled(self) -> bool {
        !matches!(self, Self::Disabled)
    }

    const fn required_high_windows(self) -> usize {
        match self {
            Self::Disabled => usize::MAX,
            Self::Conservative => 3,
            Self::Balanced => 2,
            Self::Aggressive => 1,
        }
    }

    const fn required_low_windows(self) -> usize {
        match self {
            Self::Disabled => usize::MAX,
            Self::Conservative => 4,
            Self::Balanced => 3,
            Self::Aggressive => 2,
        }
    }

    fn thresholds(self, baseline_budget: usize) -> (usize, usize) {
        let baseline = baseline_budget.max(4);
        match self {
            Self::Disabled => (usize::MAX, 0),
            Self::Conservative => (baseline.saturating_mul(3), baseline),
            Self::Balanced => (baseline.saturating_mul(2), (baseline / 2).max(2)),
            Self::Aggressive => (baseline, (baseline / 3).max(1)),
        }
    }
}

/// Runtime configuration resolved from environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatOpsRuntimeConfig {
    /// Enables ChatOps background polling loop.
    pub enabled: bool,
    /// Poll interval used by service loop.
    pub poll_interval: Duration,
    /// Max envelopes consumed per ingress per tick.
    pub max_batch_size: usize,
    /// Allowlisted remote user ids.
    pub allowed_user_ids: Vec<String>,
    /// Allowlisted remote channel ids.
    pub allowed_channel_ids: Vec<String>,
    /// Optional command scope allowlist (`submit`, `submit:ai-plan`, `list`, `*`).
    pub allowed_command_scopes: Vec<String>,
    /// Optional per-user rate limit budget.
    pub rate_limit_per_user: Option<usize>,
    /// Optional per-channel rate limit budget.
    pub rate_limit_per_channel: Option<usize>,
    /// Rate-limit strategy.
    pub rate_limit_strategy: ChatOpsRateLimitStrategy,
    /// Optional per-user burst budget for token-bucket strategy.
    pub rate_limit_burst_per_user: Option<usize>,
    /// Optional per-channel burst budget for token-bucket strategy.
    pub rate_limit_burst_per_channel: Option<usize>,
    /// Optional ingress-driven auto-tuning profile for strategy switching.
    pub rate_limit_auto_tune_profile: ChatOpsRateLimitAutoTuneProfile,
    /// Rate-limit window duration.
    pub rate_limit_window: Duration,
    /// Telegram bot token.
    pub telegram_bot_token: Option<String>,
    /// Telegram API base URL override.
    pub telegram_api_base: String,
    /// Enables Telegram webhook ingress queue mode (instead of polling).
    pub telegram_webhook_enabled: bool,
    /// Discord bot token.
    pub discord_bot_token: Option<String>,
    /// Discord API base URL override.
    pub discord_api_base: String,
    /// Enables Discord interaction ingress queue mode (instead of channel polling).
    pub discord_interactions_enabled: bool,
    /// Discord channel ids to poll.
    pub discord_channel_ids: Vec<String>,
    /// Optional custom audit store path.
    pub audit_path: Option<std::path::PathBuf>,
}

impl Default for ChatOpsRuntimeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            poll_interval: Duration::from_millis(DEFAULT_CHATOPS_POLL_INTERVAL_MS),
            max_batch_size: DEFAULT_CHATOPS_MAX_BATCH_SIZE,
            allowed_user_ids: Vec::new(),
            allowed_channel_ids: Vec::new(),
            allowed_command_scopes: Vec::new(),
            rate_limit_per_user: None,
            rate_limit_per_channel: None,
            rate_limit_strategy: ChatOpsRateLimitStrategy::FixedWindow,
            rate_limit_burst_per_user: None,
            rate_limit_burst_per_channel: None,
            rate_limit_auto_tune_profile: ChatOpsRateLimitAutoTuneProfile::Disabled,
            rate_limit_window: Duration::from_secs(DEFAULT_CHATOPS_RATE_LIMIT_WINDOW_SECONDS),
            telegram_bot_token: None,
            telegram_api_base: DEFAULT_TELEGRAM_API_BASE.to_string(),
            telegram_webhook_enabled: false,
            discord_bot_token: None,
            discord_api_base: DEFAULT_DISCORD_API_BASE.to_string(),
            discord_interactions_enabled: false,
            discord_channel_ids: Vec::new(),
            audit_path: None,
        }
    }
}

impl ChatOpsRuntimeConfig {
    /// Resolve runtime configuration from environment variables.
    #[must_use]
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(value) = std::env::var(NTK_CHATOPS_ENABLED_ENV) {
            if let Some(parsed) = parse_bool(&value) {
                config.enabled = parsed;
            }
        }
        if let Ok(value) = std::env::var(NTK_CHATOPS_POLL_INTERVAL_MS_ENV) {
            if let Ok(parsed) = value.trim().parse::<u64>() {
                config.poll_interval = Duration::from_millis(parsed.max(100));
            }
        }
        if let Ok(value) = std::env::var(NTK_CHATOPS_MAX_BATCH_ENV) {
            if let Ok(parsed) = value.trim().parse::<usize>() {
                config.max_batch_size = parsed.max(1);
            }
        }
        if let Ok(value) = std::env::var(NTK_CHATOPS_ALLOWED_USERS_ENV) {
            config.allowed_user_ids = parse_list(&value);
        }
        if let Ok(value) = std::env::var(NTK_CHATOPS_ALLOWED_CHANNELS_ENV) {
            config.allowed_channel_ids = parse_list(&value);
        }
        if let Ok(value) = std::env::var(NTK_CHATOPS_ALLOWED_COMMANDS_ENV) {
            config.allowed_command_scopes = parse_list(&value)
                .into_iter()
                .map(|scope| scope.to_ascii_lowercase())
                .collect();
        }
        if let Ok(value) = std::env::var(NTK_CHATOPS_RATE_LIMIT_PER_USER_ENV) {
            if let Ok(parsed) = value.trim().parse::<usize>() {
                config.rate_limit_per_user = Some(parsed.max(1));
            }
        }
        if let Ok(value) = std::env::var(NTK_CHATOPS_RATE_LIMIT_PER_CHANNEL_ENV) {
            if let Ok(parsed) = value.trim().parse::<usize>() {
                config.rate_limit_per_channel = Some(parsed.max(1));
            }
        }
        if let Ok(value) = std::env::var(NTK_CHATOPS_RATE_LIMIT_STRATEGY_ENV) {
            if let Some(parsed) = parse_rate_limit_strategy(&value) {
                config.rate_limit_strategy = parsed;
            }
        }
        if let Ok(value) = std::env::var(NTK_CHATOPS_RATE_LIMIT_BURST_PER_USER_ENV) {
            if let Ok(parsed) = value.trim().parse::<usize>() {
                config.rate_limit_burst_per_user = Some(parsed.max(1));
            }
        }
        if let Ok(value) = std::env::var(NTK_CHATOPS_RATE_LIMIT_BURST_PER_CHANNEL_ENV) {
            if let Ok(parsed) = value.trim().parse::<usize>() {
                config.rate_limit_burst_per_channel = Some(parsed.max(1));
            }
        }
        if let Ok(value) = std::env::var(NTK_CHATOPS_RATE_LIMIT_AUTOTUNE_PROFILE_ENV) {
            if let Some(parsed) = parse_rate_limit_auto_tune_profile(&value) {
                config.rate_limit_auto_tune_profile = parsed;
            }
        }
        if let Ok(value) = std::env::var(NTK_CHATOPS_RATE_LIMIT_WINDOW_SECONDS_ENV) {
            if let Ok(parsed) = value.trim().parse::<u64>() {
                config.rate_limit_window = Duration::from_secs(parsed.max(1));
            }
        }
        if let Ok(value) = std::env::var(NTK_CHATOPS_TELEGRAM_TOKEN_ENV) {
            let token = value.trim();
            if !token.is_empty() {
                config.telegram_bot_token = Some(token.to_string());
            }
        }
        if let Ok(value) = std::env::var(NTK_CHATOPS_TELEGRAM_API_BASE_ENV) {
            let base = value.trim();
            if !base.is_empty() {
                config.telegram_api_base = base.to_string();
            }
        }
        if let Ok(value) = std::env::var(NTK_CHATOPS_TELEGRAM_WEBHOOK_ENABLED_ENV) {
            if let Some(parsed) = parse_bool(&value) {
                config.telegram_webhook_enabled = parsed;
            }
        }
        if let Ok(value) = std::env::var(NTK_CHATOPS_DISCORD_TOKEN_ENV) {
            let token = value.trim();
            if !token.is_empty() {
                config.discord_bot_token = Some(token.to_string());
            }
        }
        if let Ok(value) = std::env::var(NTK_CHATOPS_DISCORD_API_BASE_ENV) {
            let base = value.trim();
            if !base.is_empty() {
                config.discord_api_base = base.to_string();
            }
        }
        if let Ok(value) = std::env::var(NTK_CHATOPS_DISCORD_INTERACTIONS_ENABLED_ENV) {
            if let Some(parsed) = parse_bool(&value) {
                config.discord_interactions_enabled = parsed;
            }
        }
        if let Ok(value) = std::env::var(NTK_CHATOPS_DISCORD_CHANNELS_ENV) {
            config.discord_channel_ids = parse_list(&value);
        }
        if let Ok(value) = std::env::var(NTK_CHATOPS_AUDIT_PATH_ENV) {
            let path = value.trim();
            if !path.is_empty() {
                config.audit_path = Some(std::path::PathBuf::from(path));
            }
        }

        config
    }
}

/// Tick summary returned by ChatOps runtime loop.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ChatOpsTickSummary {
    /// Number of envelopes fetched from ingress adapters.
    pub envelopes_received: usize,
    /// Number of command executions that returned success.
    pub executed_success: usize,
    /// Number of command executions that returned error/interrupted or failed before execution.
    pub executed_failed: usize,
    /// Number of envelopes rejected due to rate limits.
    pub rate_limited: usize,
    /// Number of ingress polling failures.
    pub ingress_errors: usize,
    /// Number of outbound notification dispatch failures.
    pub notification_errors: usize,
}

/// Result of a Discord interaction ingress payload handling attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiscordInteractionIngressOutcome {
    /// True when payload was a Discord ping interaction (`type=1`).
    pub ping: bool,
    /// Number of command envelopes queued from this payload.
    pub queued: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ChatOpsRateLimitPolicy {
    strategy: ChatOpsRateLimitStrategy,
    per_user_limit: Option<usize>,
    per_channel_limit: Option<usize>,
    burst_per_user: Option<usize>,
    burst_per_channel: Option<usize>,
    auto_tune_profile: ChatOpsRateLimitAutoTuneProfile,
    window: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ChatOpsRateCounter {
    window_started_unix_ms: u64,
    used: usize,
}

#[derive(Debug, Clone, Copy)]
struct ChatOpsTokenBucket {
    capacity: f64,
    refill_per_ms: f64,
    available_tokens: f64,
    last_refill_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChatOpsRateLimitViolation {
    scope: &'static str,
    identifier: String,
    limit: usize,
    retry_after: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ChatOpsRateLimitStrategySwitch {
    from: ChatOpsRateLimitStrategy,
    to: ChatOpsRateLimitStrategy,
    observed_requests: usize,
}

#[derive(Debug, Clone, Copy)]
struct ChatOpsRateAutoTuneState {
    profile: ChatOpsRateLimitAutoTuneProfile,
    window_started_unix_ms: u64,
    observed_requests: usize,
    consecutive_high_windows: usize,
    consecutive_low_windows: usize,
}

impl ChatOpsRateAutoTuneState {
    fn new(policy: &ChatOpsRateLimitPolicy) -> Option<Self> {
        if !policy.auto_tune_profile.is_enabled() {
            return None;
        }
        if policy.per_user_limit.is_none() && policy.per_channel_limit.is_none() {
            return None;
        }
        Some(Self {
            profile: policy.auto_tune_profile,
            window_started_unix_ms: 0,
            observed_requests: 0,
            consecutive_high_windows: 0,
            consecutive_low_windows: 0,
        })
    }

    fn observe(
        &mut self,
        now_unix_ms: u64,
        policy: &ChatOpsRateLimitPolicy,
        effective_strategy: &mut ChatOpsRateLimitStrategy,
    ) -> Option<ChatOpsRateLimitStrategySwitch> {
        let mut strategy_switch = None;
        if self.window_started_unix_ms == 0 {
            self.window_started_unix_ms = now_unix_ms;
        }

        let window_unix_ms = u64::try_from(policy.window.as_millis())
            .unwrap_or(u64::MAX)
            .max(1);
        if now_unix_ms.saturating_sub(self.window_started_unix_ms) >= window_unix_ms {
            strategy_switch =
                self.evaluate_window(policy, *effective_strategy)
                    .map(|next_strategy| ChatOpsRateLimitStrategySwitch {
                        from: *effective_strategy,
                        to: next_strategy,
                        observed_requests: self.observed_requests,
                    });
            if let Some(next_strategy) = strategy_switch.map(|item| item.to) {
                *effective_strategy = next_strategy;
            }
            self.window_started_unix_ms = now_unix_ms;
            self.observed_requests = 0;
        }

        self.observed_requests = self.observed_requests.saturating_add(1);
        strategy_switch
    }

    fn evaluate_window(
        &mut self,
        policy: &ChatOpsRateLimitPolicy,
        current_strategy: ChatOpsRateLimitStrategy,
    ) -> Option<ChatOpsRateLimitStrategy> {
        let baseline_budget = policy
            .per_user_limit
            .unwrap_or_default()
            .max(policy.per_channel_limit.unwrap_or_default())
            .max(4);
        let (high_threshold, low_threshold) = self.profile.thresholds(baseline_budget);
        let observed = self.observed_requests;

        if observed >= high_threshold {
            self.consecutive_high_windows = self.consecutive_high_windows.saturating_add(1);
            self.consecutive_low_windows = 0;
        } else if observed <= low_threshold {
            self.consecutive_low_windows = self.consecutive_low_windows.saturating_add(1);
            self.consecutive_high_windows = 0;
        } else {
            self.consecutive_high_windows = 0;
            self.consecutive_low_windows = 0;
        }

        if matches!(current_strategy, ChatOpsRateLimitStrategy::FixedWindow)
            && self.consecutive_high_windows >= self.profile.required_high_windows()
        {
            self.consecutive_high_windows = 0;
            self.consecutive_low_windows = 0;
            return Some(ChatOpsRateLimitStrategy::TokenBucket);
        }
        if matches!(current_strategy, ChatOpsRateLimitStrategy::TokenBucket)
            && self.consecutive_low_windows >= self.profile.required_low_windows()
        {
            self.consecutive_high_windows = 0;
            self.consecutive_low_windows = 0;
            return Some(ChatOpsRateLimitStrategy::FixedWindow);
        }

        None
    }
}

#[derive(Debug)]
struct ChatOpsRateLimiter {
    policy: ChatOpsRateLimitPolicy,
    effective_strategy: ChatOpsRateLimitStrategy,
    auto_tune: Option<ChatOpsRateAutoTuneState>,
    last_strategy_switch: Option<ChatOpsRateLimitStrategySwitch>,
    user_counters: HashMap<String, ChatOpsRateCounter>,
    channel_counters: HashMap<String, ChatOpsRateCounter>,
    user_buckets: HashMap<String, ChatOpsTokenBucket>,
    channel_buckets: HashMap<String, ChatOpsTokenBucket>,
}

impl ChatOpsRateLimiter {
    fn new(policy: ChatOpsRateLimitPolicy) -> Self {
        let effective_strategy = policy.strategy;
        let auto_tune = ChatOpsRateAutoTuneState::new(&policy);
        Self {
            policy,
            effective_strategy,
            auto_tune,
            last_strategy_switch: None,
            user_counters: HashMap::new(),
            channel_counters: HashMap::new(),
            user_buckets: HashMap::new(),
            channel_buckets: HashMap::new(),
        }
    }

    fn check_and_record(
        &mut self,
        envelope: &ChatOpsCommandEnvelope,
    ) -> Result<(), ChatOpsRateLimitViolation> {
        let now_unix_ms = if envelope.received_at_unix_ms == 0 {
            current_unix_timestamp_ms()
        } else {
            envelope.received_at_unix_ms
        };
        if let Some(auto_tune) = self.auto_tune.as_mut() {
            self.last_strategy_switch =
                auto_tune.observe(now_unix_ms, &self.policy, &mut self.effective_strategy);
        }
        match self.effective_strategy {
            ChatOpsRateLimitStrategy::FixedWindow => {
                if let Some(limit) = self.policy.per_user_limit {
                    check_counter_budget(
                        &mut self.user_counters,
                        envelope.user_id.trim(),
                        limit,
                        now_unix_ms,
                        self.policy.window,
                        "user",
                    )?;
                }
                if let Some(limit) = self.policy.per_channel_limit {
                    check_counter_budget(
                        &mut self.channel_counters,
                        envelope.channel_id.trim(),
                        limit,
                        now_unix_ms,
                        self.policy.window,
                        "channel",
                    )?;
                }
            }
            ChatOpsRateLimitStrategy::TokenBucket => {
                if let Some(limit) = self.policy.per_user_limit {
                    check_token_bucket_budget(
                        &mut self.user_buckets,
                        envelope.user_id.trim(),
                        limit,
                        self.policy.burst_per_user,
                        now_unix_ms,
                        self.policy.window,
                        "user",
                    )?;
                }
                if let Some(limit) = self.policy.per_channel_limit {
                    check_token_bucket_budget(
                        &mut self.channel_buckets,
                        envelope.channel_id.trim(),
                        limit,
                        self.policy.burst_per_channel,
                        now_unix_ms,
                        self.policy.window,
                        "channel",
                    )?;
                }
            }
        }
        Ok(())
    }

    fn take_strategy_switch(&mut self) -> Option<ChatOpsRateLimitStrategySwitch> {
        self.last_strategy_switch.take()
    }
}

/// Runtime orchestrator for asynchronous ChatOps adapters.
pub struct ChatOpsRuntime {
    poll_interval: Duration,
    max_batch_size: usize,
    policy: ChatOpsAuthorizationPolicy,
    audit_store: Option<ChatOpsLocalAuditStore>,
    rate_limiter: Option<Mutex<ChatOpsRateLimiter>>,
    telegram_webhook_ingress: Option<Arc<TelegramWebhookIngressAdapter>>,
    discord_interaction_ingress: Option<Arc<DiscordInteractionIngressAdapter>>,
    ingresses: Vec<Arc<dyn AsyncChatOpsIngress>>,
    notifiers: Vec<(ChatOpsPlatform, Arc<dyn AsyncChatOpsNotifier>)>,
}

impl ChatOpsRuntime {
    /// Poll interval configured for this runtime.
    #[must_use]
    pub const fn poll_interval(&self) -> Duration {
        self.poll_interval
    }

    /// Enabled adapter platforms.
    #[must_use]
    pub fn enabled_platforms(&self) -> Vec<ChatOpsPlatform> {
        self.ingresses
            .iter()
            .map(|adapter| adapter.platform())
            .collect()
    }

    /// Returns true when Telegram webhook queue mode is enabled.
    #[must_use]
    pub fn is_telegram_webhook_enabled(&self) -> bool {
        self.telegram_webhook_ingress.is_some()
    }

    /// Returns true when Discord interaction queue mode is enabled.
    #[must_use]
    pub fn is_discord_interactions_enabled(&self) -> bool {
        self.discord_interaction_ingress.is_some()
    }

    /// Enqueue raw Telegram webhook payload for processing in the next runtime tick.
    ///
    /// # Errors
    ///
    /// Returns error when webhook mode is disabled or payload is invalid.
    pub fn enqueue_telegram_webhook_payload(
        &self,
        payload: &str,
    ) -> Result<usize, ChatOpsAdapterError> {
        let Some(ingress) = &self.telegram_webhook_ingress else {
            return Err(ChatOpsAdapterError::new(
                "Telegram webhook mode is disabled in ChatOps runtime",
            ));
        };
        ingress.enqueue_payload(payload)
    }

    /// Enqueue raw Discord interaction payload for processing in the next runtime tick.
    ///
    /// # Errors
    ///
    /// Returns error when interaction mode is disabled or payload is invalid.
    pub fn enqueue_discord_interaction_payload(
        &self,
        payload: &str,
    ) -> Result<DiscordInteractionIngressOutcome, ChatOpsAdapterError> {
        let Some(ingress) = &self.discord_interaction_ingress else {
            return Err(ChatOpsAdapterError::new(
                "Discord interaction mode is disabled in ChatOps runtime",
            ));
        };
        ingress.enqueue_payload(payload)
    }

    /// Execute one polling iteration across all configured adapters.
    pub async fn tick(&self) -> ChatOpsTickSummary {
        let mut summary = ChatOpsTickSummary::default();

        for ingress in &self.ingresses {
            let envelopes = match ingress.pull_pending(self.max_batch_size).await {
                Ok(envelopes) => envelopes,
                Err(error) => {
                    summary.ingress_errors += 1;
                    tracing::warn!(
                        platform = %ingress.platform(),
                        error = %error,
                        "chatops ingress polling failed"
                    );
                    continue;
                }
            };

            summary.envelopes_received += envelopes.len();
            for envelope in envelopes {
                if let Some(rate_limiter) = &self.rate_limiter {
                    let (rate_limit_result, strategy_switch) = {
                        let mut guard = rate_limiter
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        let result = guard.check_and_record(&envelope);
                        let strategy_switch = guard.take_strategy_switch();
                        (result, strategy_switch)
                    };
                    if let Some(strategy_switch) = strategy_switch {
                        let note = format!(
                            "Rate-limit strategy auto-tuned from {:?} to {:?} (observed {} requests in window).",
                            strategy_switch.from,
                            strategy_switch.to,
                            strategy_switch.observed_requests
                        );
                        append_runtime_audit(
                            self.audit_store.as_ref(),
                            &envelope,
                            ChatOpsAuditKind::NotificationSent,
                            &note,
                        );
                        tracing::info!(
                            from = ?strategy_switch.from,
                            to = ?strategy_switch.to,
                            observed_requests = strategy_switch.observed_requests,
                            "chatops rate-limit strategy auto-tuned"
                        );
                    }

                    if let Err(violation) = rate_limit_result {
                        summary.executed_failed += 1;
                        summary.rate_limited += 1;
                        let rejection_message = format!(
                            "Rate limit exceeded for {} `{}` (limit: {} requests per {}s, retry in {}s).",
                            violation.scope,
                            violation.identifier,
                            violation.limit,
                            self.policy_rate_window_seconds(),
                            violation.retry_after.as_secs().max(1)
                        );
                        append_rate_limit_audit(
                            self.audit_store.as_ref(),
                            &envelope,
                            &rejection_message,
                        );

                        let notification = ChatOpsNotification {
                            platform: envelope.platform,
                            channel_id: envelope.channel_id.clone(),
                            message_text: rejection_message,
                            severity: ChatOpsNotificationSeverity::Warning,
                        };
                        if let Some(notifier) = self.notifier_for(envelope.platform) {
                            if let Err(error) = notifier.send(&notification).await {
                                summary.notification_errors += 1;
                                tracing::warn!(
                                    platform = %envelope.platform,
                                    error = %error,
                                    "chatops rate-limit notification dispatch failed"
                                );
                            } else {
                                append_notification_audit(
                                    self.audit_store.as_ref(),
                                    &envelope,
                                    "rate-limit notification sent",
                                );
                            }
                        } else {
                            summary.notification_errors += 1;
                            tracing::warn!(
                                platform = %envelope.platform,
                                "chatops notifier is not configured for platform"
                            );
                        }
                        continue;
                    }
                }

                let queued_notifier = QueuedNotificationNotifier::default();
                let execution = execute_chatops_envelope(
                    &envelope,
                    &self.policy,
                    &queued_notifier,
                    self.audit_store.as_ref(),
                )
                .await;
                match execution {
                    Ok(ExitStatus::Success) => summary.executed_success += 1,
                    Ok(ExitStatus::Error | ExitStatus::Interrupted) | Err(_) => {
                        summary.executed_failed += 1
                    }
                }

                let pending_notifications = queued_notifier.drain();
                if pending_notifications.is_empty() {
                    continue;
                }

                if let Some(notifier) = self.notifier_for(envelope.platform) {
                    for notification in pending_notifications {
                        if let Err(error) = notifier.send(&notification).await {
                            summary.notification_errors += 1;
                            tracing::warn!(
                                platform = %envelope.platform,
                                error = %error,
                                "chatops notification dispatch failed"
                            );
                        }
                    }
                } else {
                    summary.notification_errors += pending_notifications.len();
                    tracing::warn!(
                        platform = %envelope.platform,
                        "chatops notifier is not configured for platform"
                    );
                }
            }
        }

        summary
    }

    fn notifier_for(&self, platform: ChatOpsPlatform) -> Option<&Arc<dyn AsyncChatOpsNotifier>> {
        self.notifiers
            .iter()
            .find(|(adapter_platform, _)| *adapter_platform == platform)
            .map(|(_, notifier)| notifier)
    }

    fn policy_rate_window_seconds(&self) -> u64 {
        self.rate_limiter
            .as_ref()
            .and_then(|limiter| {
                limiter
                    .lock()
                    .ok()
                    .map(|guard| guard.policy.window.as_secs())
            })
            .unwrap_or(DEFAULT_CHATOPS_RATE_LIMIT_WINDOW_SECONDS)
    }
}

/// Build runtime from environment-derived configuration.
///
/// # Errors
///
/// Returns error when enabled runtime has invalid/missing adapter configuration.
pub fn build_chatops_runtime_from_env() -> Result<Option<ChatOpsRuntime>, ChatOpsAdapterError> {
    build_chatops_runtime(ChatOpsRuntimeConfig::from_env())
}

/// Build runtime from explicit configuration.
///
/// # Errors
///
/// Returns error when enabled runtime has invalid/missing adapter configuration.
pub fn build_chatops_runtime(
    config: ChatOpsRuntimeConfig,
) -> Result<Option<ChatOpsRuntime>, ChatOpsAdapterError> {
    if !config.enabled {
        return Ok(None);
    }
    if config.telegram_webhook_enabled && config.telegram_bot_token.is_none() {
        return Err(ChatOpsAdapterError::new(
            "Telegram webhook mode requires NTK_CHATOPS_TELEGRAM_TOKEN",
        ));
    }

    let policy = ChatOpsAuthorizationPolicy::new_with_scopes(
        config.allowed_user_ids.clone(),
        config.allowed_channel_ids.clone(),
        config.allowed_command_scopes.clone(),
    );
    let rate_limit_policy = ChatOpsRateLimitPolicy {
        strategy: config.rate_limit_strategy,
        per_user_limit: config.rate_limit_per_user,
        per_channel_limit: config.rate_limit_per_channel,
        burst_per_user: config.rate_limit_burst_per_user,
        burst_per_channel: config.rate_limit_burst_per_channel,
        auto_tune_profile: config.rate_limit_auto_tune_profile,
        window: config.rate_limit_window,
    };
    let rate_limiter = if rate_limit_policy.per_user_limit.is_some()
        || rate_limit_policy.per_channel_limit.is_some()
    {
        Some(Mutex::new(ChatOpsRateLimiter::new(rate_limit_policy)))
    } else {
        None
    };
    let audit_store = config
        .audit_path
        .as_ref()
        .map(ChatOpsLocalAuditStore::from_path)
        .or_else(ChatOpsLocalAuditStore::from_default_data_dir);

    let mut ingresses: Vec<Arc<dyn AsyncChatOpsIngress>> = Vec::new();
    let mut notifiers: Vec<(ChatOpsPlatform, Arc<dyn AsyncChatOpsNotifier>)> = Vec::new();
    let mut telegram_webhook_ingress: Option<Arc<TelegramWebhookIngressAdapter>> = None;
    let mut discord_interaction_ingress: Option<Arc<DiscordInteractionIngressAdapter>> = None;

    if let Some(token) = config.telegram_bot_token.clone() {
        let telegram_adapter = Arc::new(TelegramChatOpsAdapter::new(
            token,
            config.telegram_api_base.clone(),
        )?);
        notifiers.push((ChatOpsPlatform::Telegram, telegram_adapter.clone()));

        if config.telegram_webhook_enabled {
            let webhook_ingress = Arc::new(TelegramWebhookIngressAdapter::default());
            ingresses.push(webhook_ingress.clone());
            telegram_webhook_ingress = Some(webhook_ingress);
        } else {
            ingresses.push(telegram_adapter);
        }
    }

    if let Some(token) = config.discord_bot_token.clone() {
        let notifier = Arc::new(DiscordChatOpsNotifierAdapter::new(
            token.clone(),
            config.discord_api_base.clone(),
        )?);
        notifiers.push((ChatOpsPlatform::Discord, notifier));

        if config.discord_interactions_enabled {
            let interaction_ingress = Arc::new(DiscordInteractionIngressAdapter::default());
            ingresses.push(interaction_ingress.clone());
            discord_interaction_ingress = Some(interaction_ingress);
        } else {
            if config.discord_channel_ids.is_empty() {
                return Err(ChatOpsAdapterError::new(
                    "Discord polling adapter requires NTK_CHATOPS_DISCORD_CHANNELS when interaction mode is disabled",
                ));
            }
            let adapter = Arc::new(DiscordChatOpsAdapter::new(
                token,
                config.discord_api_base.clone(),
                config.discord_channel_ids.clone(),
            )?);
            ingresses.push(adapter);
        }
    }

    if ingresses.is_empty() {
        return Err(ChatOpsAdapterError::new(
            "ChatOps enabled but no Telegram/Discord token is configured",
        ));
    }

    Ok(Some(ChatOpsRuntime {
        poll_interval: config.poll_interval,
        max_batch_size: config.max_batch_size,
        policy,
        audit_store,
        rate_limiter,
        telegram_webhook_ingress,
        discord_interaction_ingress,
        ingresses,
        notifiers,
    }))
}

type ChatOpsRuntimeFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

trait AsyncChatOpsIngress: Send + Sync {
    fn platform(&self) -> ChatOpsPlatform;
    fn pull_pending(
        &self,
        max_items: usize,
    ) -> ChatOpsRuntimeFuture<'_, Result<Vec<ChatOpsCommandEnvelope>, ChatOpsAdapterError>>;
}

trait AsyncChatOpsNotifier: Send + Sync {
    fn send(
        &self,
        notification: &ChatOpsNotification,
    ) -> ChatOpsRuntimeFuture<'_, Result<(), ChatOpsAdapterError>>;
}

#[derive(Default)]
struct QueuedNotificationNotifier {
    notifications: Mutex<Vec<ChatOpsNotification>>,
}

impl QueuedNotificationNotifier {
    fn drain(&self) -> Vec<ChatOpsNotification> {
        let mut guard = self
            .notifications
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::mem::take(&mut *guard)
    }
}

impl ChatOpsNotifier for QueuedNotificationNotifier {
    fn send(&self, notification: &ChatOpsNotification) -> Result<(), ChatOpsAdapterError> {
        self.notifications
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(notification.clone());
        Ok(())
    }
}

#[derive(Default)]
struct TelegramWebhookIngressAdapter {
    queue: Mutex<VecDeque<ChatOpsCommandEnvelope>>,
}

impl TelegramWebhookIngressAdapter {
    fn enqueue_payload(&self, payload: &str) -> Result<usize, ChatOpsAdapterError> {
        let parsed = parse_telegram_webhook_payload(payload)?;
        if parsed.is_empty() {
            return Ok(0);
        }
        let parsed_len = parsed.len();
        let mut queue = self
            .queue
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        queue.extend(parsed);
        Ok(parsed_len)
    }
}

impl AsyncChatOpsIngress for TelegramWebhookIngressAdapter {
    fn platform(&self) -> ChatOpsPlatform {
        ChatOpsPlatform::Telegram
    }

    fn pull_pending(
        &self,
        max_items: usize,
    ) -> ChatOpsRuntimeFuture<'_, Result<Vec<ChatOpsCommandEnvelope>, ChatOpsAdapterError>> {
        Box::pin(async move {
            if max_items == 0 {
                return Ok(Vec::new());
            }
            let mut queue = self
                .queue
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let mut items = Vec::new();
            for _ in 0..max_items {
                if let Some(next) = queue.pop_front() {
                    items.push(next);
                } else {
                    break;
                }
            }
            Ok(items)
        })
    }
}

#[derive(Default)]
struct DiscordInteractionIngressAdapter {
    queue: Mutex<VecDeque<ChatOpsCommandEnvelope>>,
}

impl DiscordInteractionIngressAdapter {
    fn enqueue_payload(
        &self,
        payload: &str,
    ) -> Result<DiscordInteractionIngressOutcome, ChatOpsAdapterError> {
        match parse_discord_interaction_payload(payload)? {
            DiscordInteractionIngressPayload::Ping => Ok(DiscordInteractionIngressOutcome {
                ping: true,
                queued: 0,
            }),
            DiscordInteractionIngressPayload::Command(envelope) => {
                let mut queue = self
                    .queue
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                queue.push_back(envelope);
                Ok(DiscordInteractionIngressOutcome {
                    ping: false,
                    queued: 1,
                })
            }
        }
    }
}

impl AsyncChatOpsIngress for DiscordInteractionIngressAdapter {
    fn platform(&self) -> ChatOpsPlatform {
        ChatOpsPlatform::Discord
    }

    fn pull_pending(
        &self,
        max_items: usize,
    ) -> ChatOpsRuntimeFuture<'_, Result<Vec<ChatOpsCommandEnvelope>, ChatOpsAdapterError>> {
        Box::pin(async move {
            if max_items == 0 {
                return Ok(Vec::new());
            }
            let mut queue = self
                .queue
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let mut items = Vec::new();
            for _ in 0..max_items {
                if let Some(next) = queue.pop_front() {
                    items.push(next);
                } else {
                    break;
                }
            }
            Ok(items)
        })
    }
}

struct TelegramChatOpsAdapter {
    client: reqwest::Client,
    bot_token: String,
    api_base: String,
    next_update_id: Mutex<i64>,
}

impl TelegramChatOpsAdapter {
    fn new(bot_token: String, api_base: String) -> Result<Self, ChatOpsAdapterError> {
        if bot_token.trim().is_empty() {
            return Err(ChatOpsAdapterError::new("Telegram token must not be empty"));
        }
        if api_base.trim().is_empty() {
            return Err(ChatOpsAdapterError::new(
                "Telegram API base URL must not be empty",
            ));
        }
        Ok(Self {
            client: reqwest::Client::new(),
            bot_token,
            api_base: api_base.trim_end_matches('/').to_string(),
            next_update_id: Mutex::new(0),
        })
    }

    fn endpoint(&self, method: &str) -> String {
        format!("{}/bot{}/{}", self.api_base, self.bot_token, method)
    }
}

impl AsyncChatOpsIngress for TelegramChatOpsAdapter {
    fn platform(&self) -> ChatOpsPlatform {
        ChatOpsPlatform::Telegram
    }

    fn pull_pending(
        &self,
        max_items: usize,
    ) -> ChatOpsRuntimeFuture<'_, Result<Vec<ChatOpsCommandEnvelope>, ChatOpsAdapterError>> {
        Box::pin(async move {
            if max_items == 0 {
                return Ok(Vec::new());
            }

            let offset = *self
                .next_update_id
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let response = self
                .client
                .get(self.endpoint("getUpdates"))
                .query(&[
                    ("offset", offset.to_string()),
                    ("limit", max_items.to_string()),
                    ("timeout", "0".to_string()),
                ])
                .send()
                .await
                .map_err(|error| ChatOpsAdapterError::new(error.to_string()))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(ChatOpsAdapterError::new(format!(
                    "Telegram getUpdates failed ({status}): {body}"
                )));
            }

            let payload: TelegramUpdatesResponse = response
                .json()
                .await
                .map_err(|error| ChatOpsAdapterError::new(error.to_string()))?;
            if !payload.ok {
                return Err(ChatOpsAdapterError::new(
                    payload
                        .description
                        .unwrap_or_else(|| "Telegram API returned ok=false".to_string()),
                ));
            }

            let mut envelopes = Vec::new();
            let mut latest_update_id = offset;
            for update in payload.result {
                if update.update_id > latest_update_id {
                    latest_update_id = update.update_id;
                }
                if let Some(envelope) = update.into_chatops_envelope() {
                    envelopes.push(envelope);
                }
            }

            if latest_update_id >= offset {
                let mut guard = self
                    .next_update_id
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                *guard = latest_update_id.saturating_add(1);
            }

            Ok(envelopes)
        })
    }
}

impl AsyncChatOpsNotifier for TelegramChatOpsAdapter {
    fn send(
        &self,
        notification: &ChatOpsNotification,
    ) -> ChatOpsRuntimeFuture<'_, Result<(), ChatOpsAdapterError>> {
        let channel_id = notification.channel_id.clone();
        let text = notification.message_text.clone();

        Box::pin(async move {
            let response = self
                .client
                .post(self.endpoint("sendMessage"))
                .json(&serde_json::json!({
                    "chat_id": channel_id,
                    "text": text,
                }))
                .send()
                .await
                .map_err(|error| ChatOpsAdapterError::new(error.to_string()))?;

            if response.status().is_success() {
                Ok(())
            } else {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                Err(ChatOpsAdapterError::new(format!(
                    "Telegram sendMessage failed ({status}): {body}"
                )))
            }
        })
    }
}

struct DiscordChatOpsNotifierAdapter {
    client: reqwest::Client,
    bot_token: String,
    api_base: String,
}

impl DiscordChatOpsNotifierAdapter {
    fn new(bot_token: String, api_base: String) -> Result<Self, ChatOpsAdapterError> {
        if bot_token.trim().is_empty() {
            return Err(ChatOpsAdapterError::new("Discord token must not be empty"));
        }
        if api_base.trim().is_empty() {
            return Err(ChatOpsAdapterError::new(
                "Discord API base URL must not be empty",
            ));
        }

        Ok(Self {
            client: reqwest::Client::new(),
            bot_token,
            api_base: api_base.trim_end_matches('/').to_string(),
        })
    }

    fn messages_endpoint(&self, channel_id: &str) -> String {
        format!("{}/channels/{}/messages", self.api_base, channel_id)
    }
}

impl AsyncChatOpsNotifier for DiscordChatOpsNotifierAdapter {
    fn send(
        &self,
        notification: &ChatOpsNotification,
    ) -> ChatOpsRuntimeFuture<'_, Result<(), ChatOpsAdapterError>> {
        let channel_id = notification.channel_id.clone();
        let text = notification.message_text.clone();
        let endpoint = self.messages_endpoint(&channel_id);
        let auth_header = format!("Bot {}", self.bot_token);

        Box::pin(async move {
            let response = self
                .client
                .post(endpoint)
                .header(reqwest::header::AUTHORIZATION, auth_header.as_str())
                .header(reqwest::header::USER_AGENT, "nettoolskit-chatops/1.0")
                .json(&serde_json::json!({
                    "content": text,
                }))
                .send()
                .await
                .map_err(|error| ChatOpsAdapterError::new(error.to_string()))?;

            if response.status().is_success() {
                Ok(())
            } else {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                Err(ChatOpsAdapterError::new(format!(
                    "Discord message send failed for channel {channel_id} ({status}): {body}"
                )))
            }
        })
    }
}

struct DiscordChatOpsAdapter {
    client: reqwest::Client,
    bot_token: String,
    api_base: String,
    channel_ids: Vec<String>,
    last_seen_by_channel: Mutex<HashMap<String, u128>>,
}

impl DiscordChatOpsAdapter {
    fn new(
        bot_token: String,
        api_base: String,
        channel_ids: Vec<String>,
    ) -> Result<Self, ChatOpsAdapterError> {
        if bot_token.trim().is_empty() {
            return Err(ChatOpsAdapterError::new("Discord token must not be empty"));
        }
        if api_base.trim().is_empty() {
            return Err(ChatOpsAdapterError::new(
                "Discord API base URL must not be empty",
            ));
        }
        if channel_ids.is_empty() {
            return Err(ChatOpsAdapterError::new(
                "Discord channel list must not be empty",
            ));
        }

        Ok(Self {
            client: reqwest::Client::new(),
            bot_token,
            api_base: api_base.trim_end_matches('/').to_string(),
            channel_ids,
            last_seen_by_channel: Mutex::new(HashMap::new()),
        })
    }

    fn messages_endpoint(&self, channel_id: &str) -> String {
        format!("{}/channels/{}/messages", self.api_base, channel_id)
    }

    fn parse_snowflake(value: &str) -> Option<u128> {
        value.trim().parse::<u128>().ok()
    }
}

impl AsyncChatOpsIngress for DiscordChatOpsAdapter {
    fn platform(&self) -> ChatOpsPlatform {
        ChatOpsPlatform::Discord
    }

    fn pull_pending(
        &self,
        max_items: usize,
    ) -> ChatOpsRuntimeFuture<'_, Result<Vec<ChatOpsCommandEnvelope>, ChatOpsAdapterError>> {
        Box::pin(async move {
            if max_items == 0 {
                return Ok(Vec::new());
            }

            let mut envelopes = Vec::new();
            let per_channel_limit = max_items.min(50).to_string();
            let auth_header = format!("Bot {}", self.bot_token);

            for channel_id in &self.channel_ids {
                let response = self
                    .client
                    .get(self.messages_endpoint(channel_id))
                    .query(&[("limit", per_channel_limit.clone())])
                    .header(reqwest::header::AUTHORIZATION, auth_header.as_str())
                    .header(reqwest::header::USER_AGENT, "nettoolskit-chatops/1.0")
                    .send()
                    .await
                    .map_err(|error| ChatOpsAdapterError::new(error.to_string()))?;

                if !response.status().is_success() {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    return Err(ChatOpsAdapterError::new(format!(
                        "Discord message poll failed for channel {channel_id} ({status}): {body}"
                    )));
                }

                let mut messages: Vec<DiscordMessage> = response
                    .json()
                    .await
                    .map_err(|error| ChatOpsAdapterError::new(error.to_string()))?;

                messages.sort_by(|left, right| left.id.cmp(&right.id));

                let last_seen = self
                    .last_seen_by_channel
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .get(channel_id)
                    .copied()
                    .unwrap_or_default();
                let mut new_last_seen = last_seen;

                for message in messages {
                    let message_id = Self::parse_snowflake(&message.id).unwrap_or_default();
                    if message_id <= last_seen {
                        continue;
                    }
                    if message.author.bot.unwrap_or(false) {
                        continue;
                    }
                    if message.content.trim().is_empty() {
                        continue;
                    }

                    envelopes.push(ChatOpsCommandEnvelope::new(
                        ChatOpsPlatform::Discord,
                        message.channel_id.unwrap_or_else(|| channel_id.clone()),
                        message.author.id,
                        message.content,
                        current_unix_timestamp_ms(),
                    ));
                    new_last_seen = new_last_seen.max(message_id);
                }

                let mut guard = self
                    .last_seen_by_channel
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                guard.insert(channel_id.clone(), new_last_seen);
            }

            if envelopes.len() > max_items {
                envelopes.truncate(max_items);
            }

            Ok(envelopes)
        })
    }
}

#[derive(Debug, Deserialize)]
struct TelegramUpdatesResponse {
    ok: bool,
    #[serde(default)]
    result: Vec<TelegramUpdate>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TelegramUpdate {
    update_id: i64,
    message: Option<TelegramMessage>,
}

impl TelegramUpdate {
    fn into_chatops_envelope(self) -> Option<ChatOpsCommandEnvelope> {
        let message = self.message?;
        let text = message.text?;
        if text.trim().is_empty() {
            return None;
        }

        let received_at_unix_ms = u64::try_from(message.date.max(0))
            .unwrap_or_default()
            .saturating_mul(1_000)
            .max(current_unix_timestamp_ms());
        Some(
            ChatOpsCommandEnvelope::new(
                ChatOpsPlatform::Telegram,
                message.chat.id.to_string(),
                message
                    .from
                    .map(|author| author.id.to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                text,
                received_at_unix_ms,
            )
            .with_transport(IngressTransport::TelegramPolling),
        )
    }
}

#[derive(Debug, Deserialize)]
struct TelegramMessage {
    date: i64,
    text: Option<String>,
    chat: TelegramChat,
    from: Option<TelegramUser>,
}

#[derive(Debug, Deserialize)]
struct TelegramChat {
    id: i64,
}

#[derive(Debug, Deserialize)]
struct TelegramUser {
    id: i64,
}

#[derive(Debug, Deserialize)]
struct DiscordMessage {
    id: String,
    #[serde(default)]
    channel_id: Option<String>,
    content: String,
    author: DiscordAuthor,
}

#[derive(Debug, Deserialize)]
struct DiscordAuthor {
    id: String,
    #[serde(default)]
    bot: Option<bool>,
}

enum DiscordInteractionIngressPayload {
    Ping,
    Command(ChatOpsCommandEnvelope),
}

#[derive(Debug, Deserialize)]
struct DiscordInteractionPayload {
    #[serde(rename = "type")]
    interaction_type: u8,
    #[serde(default)]
    channel_id: Option<String>,
    #[serde(default)]
    member: Option<DiscordInteractionMember>,
    #[serde(default)]
    user: Option<DiscordInteractionUser>,
    #[serde(default)]
    data: Option<DiscordInteractionData>,
}

impl DiscordInteractionPayload {
    fn resolve_user_id(&self) -> Option<String> {
        self.member
            .as_ref()
            .and_then(|member| member.user.as_ref())
            .map(|user| user.id.clone())
            .or_else(|| self.user.as_ref().map(|user| user.id.clone()))
    }
}

#[derive(Debug, Deserialize)]
struct DiscordInteractionMember {
    #[serde(default)]
    user: Option<DiscordInteractionUser>,
}

#[derive(Debug, Deserialize)]
struct DiscordInteractionUser {
    id: String,
}

#[derive(Debug, Deserialize)]
struct DiscordInteractionData {
    name: String,
    #[serde(default)]
    options: Vec<DiscordInteractionOption>,
}

impl DiscordInteractionData {
    fn into_message_text(self) -> Option<String> {
        let mut tokens = Vec::new();
        let command_name = self.name.trim();
        if !command_name.is_empty() {
            tokens.push(command_name.to_string());
        }
        for option in self.options {
            collect_discord_interaction_option_tokens(option, &mut tokens);
        }
        if tokens.is_empty() {
            None
        } else {
            Some(tokens.join(" "))
        }
    }
}

#[derive(Debug, Deserialize)]
struct DiscordInteractionOption {
    #[serde(rename = "type")]
    option_type: u8,
    name: String,
    #[serde(default)]
    value: Option<serde_json::Value>,
    #[serde(default)]
    options: Vec<DiscordInteractionOption>,
}

fn collect_discord_interaction_option_tokens(
    option: DiscordInteractionOption,
    tokens: &mut Vec<String>,
) {
    let option_name = option.name.trim().to_string();
    if option_name.is_empty() {
        return;
    }

    if option.option_type == 1 || option.option_type == 2 {
        tokens.push(option_name);
        for nested in option.options {
            collect_discord_interaction_option_tokens(nested, tokens);
        }
        return;
    }

    if let Some(value) = option.value {
        if let Some(token) = discord_option_value_to_token(value) {
            tokens.push(token);
            return;
        }
    }

    if option.options.is_empty() {
        tokens.push(option_name);
    } else {
        for nested in option.options {
            collect_discord_interaction_option_tokens(nested, tokens);
        }
    }
}

fn discord_option_value_to_token(value: serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(value) => Some(value),
        serde_json::Value::Number(value) => Some(value.to_string()),
        serde_json::Value::Bool(value) => Some(value.to_string()),
        serde_json::Value::Null => None,
        other => Some(other.to_string()),
    }
}

fn parse_telegram_webhook_payload(
    payload: &str,
) -> Result<Vec<ChatOpsCommandEnvelope>, ChatOpsAdapterError> {
    if payload.trim().is_empty() {
        return Err(ChatOpsAdapterError::new(
            "Telegram webhook payload must not be empty",
        ));
    }

    let update: TelegramUpdate = serde_json::from_str(payload).map_err(|error| {
        ChatOpsAdapterError::new(format!("invalid Telegram webhook payload: {error}"))
    })?;
    Ok(update
        .into_chatops_envelope()
        .map(|envelope| vec![envelope])
        .unwrap_or_default())
}

fn parse_discord_interaction_payload(
    payload: &str,
) -> Result<DiscordInteractionIngressPayload, ChatOpsAdapterError> {
    if payload.trim().is_empty() {
        return Err(ChatOpsAdapterError::new(
            "Discord interaction payload must not be empty",
        ));
    }

    let interaction: DiscordInteractionPayload =
        serde_json::from_str(payload).map_err(|error| {
            ChatOpsAdapterError::new(format!("invalid Discord interaction payload: {error}"))
        })?;
    match interaction.interaction_type {
        1 => Ok(DiscordInteractionIngressPayload::Ping),
        2 => {
            let user_id = interaction
                .resolve_user_id()
                .ok_or_else(|| ChatOpsAdapterError::new("Discord interaction missing user id"))?;
            let channel_id = interaction.channel_id.ok_or_else(|| {
                ChatOpsAdapterError::new("Discord interaction missing channel_id")
            })?;
            let data = interaction.data.ok_or_else(|| {
                ChatOpsAdapterError::new("Discord interaction missing command data")
            })?;
            let message_text = data
                .into_message_text()
                .ok_or_else(|| ChatOpsAdapterError::new("Discord interaction command is empty"))?;
            Ok(DiscordInteractionIngressPayload::Command(
                ChatOpsCommandEnvelope::new(
                    ChatOpsPlatform::Discord,
                    channel_id,
                    user_id,
                    message_text,
                    current_unix_timestamp_ms(),
                )
                .with_transport(IngressTransport::DiscordInteractions),
            ))
        }
        kind => Err(ChatOpsAdapterError::new(format!(
            "unsupported Discord interaction type: {kind}"
        ))),
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_rate_limit_strategy(value: &str) -> Option<ChatOpsRateLimitStrategy> {
    match value.trim().to_ascii_lowercase().as_str() {
        "fixed_window" | "fixed" | "window" => Some(ChatOpsRateLimitStrategy::FixedWindow),
        "token_bucket" | "token-bucket" | "bucket" => Some(ChatOpsRateLimitStrategy::TokenBucket),
        _ => None,
    }
}

fn parse_rate_limit_auto_tune_profile(value: &str) -> Option<ChatOpsRateLimitAutoTuneProfile> {
    match value.trim().to_ascii_lowercase().as_str() {
        "disabled" | "disable" | "off" | "none" => Some(ChatOpsRateLimitAutoTuneProfile::Disabled),
        "conservative" => Some(ChatOpsRateLimitAutoTuneProfile::Conservative),
        "balanced" => Some(ChatOpsRateLimitAutoTuneProfile::Balanced),
        "aggressive" => Some(ChatOpsRateLimitAutoTuneProfile::Aggressive),
        _ => None,
    }
}

fn parse_list(value: &str) -> Vec<String> {
    value
        .split([',', ';'])
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn current_unix_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

fn check_counter_budget(
    registry: &mut HashMap<String, ChatOpsRateCounter>,
    identifier: &str,
    limit: usize,
    now_unix_ms: u64,
    window: Duration,
    scope: &'static str,
) -> Result<(), ChatOpsRateLimitViolation> {
    let key = identifier.trim().to_string();
    let counter = registry.entry(key.clone()).or_insert(ChatOpsRateCounter {
        window_started_unix_ms: now_unix_ms,
        used: 0,
    });

    let window_unix_ms = u64::try_from(window.as_millis()).unwrap_or(u64::MAX);
    if now_unix_ms.saturating_sub(counter.window_started_unix_ms) >= window_unix_ms {
        counter.window_started_unix_ms = now_unix_ms;
        counter.used = 0;
    }

    if counter.used >= limit {
        let elapsed = now_unix_ms.saturating_sub(counter.window_started_unix_ms);
        let retry_after_ms = window_unix_ms.saturating_sub(elapsed);
        return Err(ChatOpsRateLimitViolation {
            scope,
            identifier: key,
            limit,
            retry_after: Duration::from_millis(retry_after_ms.max(1)),
        });
    }

    counter.used += 1;
    Ok(())
}

fn check_token_bucket_budget(
    registry: &mut HashMap<String, ChatOpsTokenBucket>,
    identifier: &str,
    limit: usize,
    burst_limit: Option<usize>,
    now_unix_ms: u64,
    window: Duration,
    scope: &'static str,
) -> Result<(), ChatOpsRateLimitViolation> {
    let key = identifier.trim().to_string();
    let window_unix_ms = u64::try_from(window.as_millis()).unwrap_or(u64::MAX).max(1);
    let capacity = burst_limit.unwrap_or(limit).max(1) as f64;
    let refill_per_ms = (limit.max(1) as f64) / (window_unix_ms as f64);
    let bucket = registry.entry(key.clone()).or_insert(ChatOpsTokenBucket {
        capacity,
        refill_per_ms,
        available_tokens: capacity,
        last_refill_unix_ms: now_unix_ms,
    });

    let elapsed_ms = now_unix_ms.saturating_sub(bucket.last_refill_unix_ms);
    if elapsed_ms > 0 {
        bucket.available_tokens = (bucket.available_tokens
            + (elapsed_ms as f64 * bucket.refill_per_ms))
            .min(bucket.capacity);
        bucket.last_refill_unix_ms = now_unix_ms;
    }

    if bucket.available_tokens < 1.0 {
        let needed_tokens = 1.0 - bucket.available_tokens;
        let retry_after_ms = if bucket.refill_per_ms > 0.0 {
            (needed_tokens / bucket.refill_per_ms).ceil() as u64
        } else {
            window_unix_ms
        };
        return Err(ChatOpsRateLimitViolation {
            scope,
            identifier: key,
            limit,
            retry_after: Duration::from_millis(retry_after_ms.max(1)),
        });
    }

    bucket.available_tokens = (bucket.available_tokens - 1.0).max(0.0);
    Ok(())
}

fn append_rate_limit_audit(
    store: Option<&ChatOpsLocalAuditStore>,
    envelope: &ChatOpsCommandEnvelope,
    message: &str,
) {
    append_runtime_audit(store, envelope, ChatOpsAuditKind::CommandRejected, message);
}

fn append_notification_audit(
    store: Option<&ChatOpsLocalAuditStore>,
    envelope: &ChatOpsCommandEnvelope,
    message: &str,
) {
    append_runtime_audit(store, envelope, ChatOpsAuditKind::NotificationSent, message);
}

fn append_runtime_audit(
    store: Option<&ChatOpsLocalAuditStore>,
    envelope: &ChatOpsCommandEnvelope,
    kind: ChatOpsAuditKind,
    message: &str,
) {
    let Some(store) = store else {
        return;
    };
    let entry = ChatOpsAuditEntry {
        kind,
        platform: envelope.platform,
        channel_id: envelope.channel_id.clone(),
        user_id: envelope.user_id.clone(),
        message_text: envelope.message_text.clone(),
        internal_command: None,
        request_id: Some(envelope.request_id.clone()),
        correlation_id: envelope.correlation_id.clone(),
        operator_id: None,
        session_id: None,
        transport: Some(envelope.transport),
        task_id: None,
        exit_status: Some("error".to_string()),
        note: message.to_string(),
        timestamp_unix_ms: current_unix_timestamp_ms(),
    };
    let _ = store.append(&entry);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::Mutex as AsyncMutex;

    static ENV_LOCK: std::sync::OnceLock<AsyncMutex<()>> = std::sync::OnceLock::new();

    async fn env_guard() -> tokio::sync::MutexGuard<'static, ()> {
        ENV_LOCK.get_or_init(|| AsyncMutex::new(())).lock().await
    }

    #[test]
    fn runtime_config_defaults_to_disabled_mode() {
        let config = ChatOpsRuntimeConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.max_batch_size, DEFAULT_CHATOPS_MAX_BATCH_SIZE);
    }

    #[tokio::test]
    async fn runtime_config_from_env_parses_lists_and_limits() {
        let _guard = env_guard().await;
        std::env::set_var(NTK_CHATOPS_ENABLED_ENV, "true");
        std::env::set_var(NTK_CHATOPS_ALLOWED_USERS_ENV, "u1,u2");
        std::env::set_var(NTK_CHATOPS_ALLOWED_CHANNELS_ENV, "c1;c2");
        std::env::set_var(NTK_CHATOPS_ALLOWED_COMMANDS_ENV, "submit:ai-plan,list");
        std::env::set_var(NTK_CHATOPS_MAX_BATCH_ENV, "25");
        std::env::set_var(NTK_CHATOPS_POLL_INTERVAL_MS_ENV, "4500");
        std::env::set_var(NTK_CHATOPS_RATE_LIMIT_PER_USER_ENV, "3");
        std::env::set_var(NTK_CHATOPS_RATE_LIMIT_PER_CHANNEL_ENV, "5");
        std::env::set_var(NTK_CHATOPS_RATE_LIMIT_STRATEGY_ENV, "token_bucket");
        std::env::set_var(NTK_CHATOPS_RATE_LIMIT_BURST_PER_USER_ENV, "9");
        std::env::set_var(NTK_CHATOPS_RATE_LIMIT_BURST_PER_CHANNEL_ENV, "11");
        std::env::set_var(NTK_CHATOPS_RATE_LIMIT_AUTOTUNE_PROFILE_ENV, "balanced");
        std::env::set_var(NTK_CHATOPS_RATE_LIMIT_WINDOW_SECONDS_ENV, "90");
        std::env::set_var(NTK_CHATOPS_TELEGRAM_WEBHOOK_ENABLED_ENV, "true");
        std::env::set_var(NTK_CHATOPS_DISCORD_INTERACTIONS_ENABLED_ENV, "true");

        let config = ChatOpsRuntimeConfig::from_env();

        std::env::remove_var(NTK_CHATOPS_ENABLED_ENV);
        std::env::remove_var(NTK_CHATOPS_ALLOWED_USERS_ENV);
        std::env::remove_var(NTK_CHATOPS_ALLOWED_CHANNELS_ENV);
        std::env::remove_var(NTK_CHATOPS_ALLOWED_COMMANDS_ENV);
        std::env::remove_var(NTK_CHATOPS_MAX_BATCH_ENV);
        std::env::remove_var(NTK_CHATOPS_POLL_INTERVAL_MS_ENV);
        std::env::remove_var(NTK_CHATOPS_RATE_LIMIT_PER_USER_ENV);
        std::env::remove_var(NTK_CHATOPS_RATE_LIMIT_PER_CHANNEL_ENV);
        std::env::remove_var(NTK_CHATOPS_RATE_LIMIT_STRATEGY_ENV);
        std::env::remove_var(NTK_CHATOPS_RATE_LIMIT_BURST_PER_USER_ENV);
        std::env::remove_var(NTK_CHATOPS_RATE_LIMIT_BURST_PER_CHANNEL_ENV);
        std::env::remove_var(NTK_CHATOPS_RATE_LIMIT_AUTOTUNE_PROFILE_ENV);
        std::env::remove_var(NTK_CHATOPS_RATE_LIMIT_WINDOW_SECONDS_ENV);
        std::env::remove_var(NTK_CHATOPS_TELEGRAM_WEBHOOK_ENABLED_ENV);
        std::env::remove_var(NTK_CHATOPS_DISCORD_INTERACTIONS_ENABLED_ENV);

        assert!(config.enabled);
        assert_eq!(
            config.allowed_user_ids,
            vec!["u1".to_string(), "u2".to_string()]
        );
        assert_eq!(
            config.allowed_channel_ids,
            vec!["c1".to_string(), "c2".to_string()]
        );
        assert_eq!(
            config.allowed_command_scopes,
            vec!["submit:ai-plan".to_string(), "list".to_string()]
        );
        assert_eq!(config.max_batch_size, 25);
        assert_eq!(config.poll_interval, Duration::from_millis(4_500));
        assert_eq!(config.rate_limit_per_user, Some(3));
        assert_eq!(config.rate_limit_per_channel, Some(5));
        assert_eq!(
            config.rate_limit_strategy,
            ChatOpsRateLimitStrategy::TokenBucket
        );
        assert_eq!(config.rate_limit_burst_per_user, Some(9));
        assert_eq!(config.rate_limit_burst_per_channel, Some(11));
        assert_eq!(
            config.rate_limit_auto_tune_profile,
            ChatOpsRateLimitAutoTuneProfile::Balanced
        );
        assert_eq!(config.rate_limit_window, Duration::from_secs(90));
        assert!(config.telegram_webhook_enabled);
        assert!(config.discord_interactions_enabled);
    }

    #[tokio::test]
    async fn build_runtime_from_env_returns_none_when_disabled() {
        let _guard = env_guard().await;
        std::env::remove_var(NTK_CHATOPS_ENABLED_ENV);
        let runtime = build_chatops_runtime_from_env().expect("disabled runtime should not fail");
        assert!(runtime.is_none());
    }

    #[tokio::test]
    async fn build_runtime_from_env_errors_when_enabled_without_tokens() {
        let _guard = env_guard().await;
        std::env::set_var(NTK_CHATOPS_ENABLED_ENV, "true");

        let runtime = build_chatops_runtime_from_env();

        std::env::remove_var(NTK_CHATOPS_ENABLED_ENV);
        assert!(runtime.is_err());
    }

    #[test]
    fn parse_list_handles_comma_and_semicolon() {
        let parsed = parse_list("a,b; c ;");
        assert_eq!(
            parsed,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn rate_limiter_blocks_after_budget_is_exhausted() {
        let policy = ChatOpsRateLimitPolicy {
            strategy: ChatOpsRateLimitStrategy::FixedWindow,
            per_user_limit: Some(1),
            per_channel_limit: Some(2),
            burst_per_user: None,
            burst_per_channel: None,
            auto_tune_profile: ChatOpsRateLimitAutoTuneProfile::Disabled,
            window: Duration::from_secs(60),
        };
        let mut limiter = ChatOpsRateLimiter::new(policy);
        let first = ChatOpsCommandEnvelope::new(
            ChatOpsPlatform::Telegram,
            "channel-1",
            "user-1",
            "list",
            1,
        );
        let second = ChatOpsCommandEnvelope::new(
            ChatOpsPlatform::Telegram,
            "channel-1",
            "user-1",
            "list",
            2,
        );

        assert!(limiter.check_and_record(&first).is_ok());
        let blocked = limiter.check_and_record(&second);
        assert!(blocked.is_err());
    }

    #[test]
    fn rate_limiter_resets_window_and_allows_after_expiration() {
        let policy = ChatOpsRateLimitPolicy {
            strategy: ChatOpsRateLimitStrategy::FixedWindow,
            per_user_limit: Some(1),
            per_channel_limit: None,
            burst_per_user: None,
            burst_per_channel: None,
            auto_tune_profile: ChatOpsRateLimitAutoTuneProfile::Disabled,
            window: Duration::from_secs(1),
        };
        let mut limiter = ChatOpsRateLimiter::new(policy);
        let first =
            ChatOpsCommandEnvelope::new(ChatOpsPlatform::Discord, "ops", "user-1", "list", 10);
        let expired_window =
            ChatOpsCommandEnvelope::new(ChatOpsPlatform::Discord, "ops", "user-1", "list", 2_000);

        assert!(limiter.check_and_record(&first).is_ok());
        assert!(limiter.check_and_record(&expired_window).is_ok());
    }

    #[test]
    fn parse_rate_limit_strategy_supports_fixed_and_token_bucket() {
        assert_eq!(
            parse_rate_limit_strategy("fixed_window"),
            Some(ChatOpsRateLimitStrategy::FixedWindow)
        );
        assert_eq!(
            parse_rate_limit_strategy("token_bucket"),
            Some(ChatOpsRateLimitStrategy::TokenBucket)
        );
        assert_eq!(
            parse_rate_limit_strategy("bucket"),
            Some(ChatOpsRateLimitStrategy::TokenBucket)
        );
        assert_eq!(parse_rate_limit_strategy("unknown"), None);
    }

    #[test]
    fn parse_rate_limit_auto_tune_profile_supports_expected_values() {
        assert_eq!(
            parse_rate_limit_auto_tune_profile("disabled"),
            Some(ChatOpsRateLimitAutoTuneProfile::Disabled)
        );
        assert_eq!(
            parse_rate_limit_auto_tune_profile("balanced"),
            Some(ChatOpsRateLimitAutoTuneProfile::Balanced)
        );
        assert_eq!(
            parse_rate_limit_auto_tune_profile("aggressive"),
            Some(ChatOpsRateLimitAutoTuneProfile::Aggressive)
        );
        assert_eq!(parse_rate_limit_auto_tune_profile("invalid"), None);
    }

    #[test]
    fn auto_tune_switches_to_token_bucket_under_sustained_high_traffic() {
        let policy = ChatOpsRateLimitPolicy {
            strategy: ChatOpsRateLimitStrategy::FixedWindow,
            per_user_limit: Some(5),
            per_channel_limit: None,
            burst_per_user: Some(10),
            burst_per_channel: None,
            auto_tune_profile: ChatOpsRateLimitAutoTuneProfile::Aggressive,
            window: Duration::from_secs(1),
        };
        let mut limiter = ChatOpsRateLimiter::new(policy);

        for index in 0..5 {
            let envelope = ChatOpsCommandEnvelope::new(
                ChatOpsPlatform::Telegram,
                "ops",
                format!("user-{index}"),
                "list",
                1_000,
            );
            assert!(limiter.check_and_record(&envelope).is_ok());
        }
        assert_eq!(
            limiter.effective_strategy,
            ChatOpsRateLimitStrategy::FixedWindow
        );

        let rollover = ChatOpsCommandEnvelope::new(
            ChatOpsPlatform::Telegram,
            "ops",
            "user-next",
            "list",
            2_100,
        );
        assert!(limiter.check_and_record(&rollover).is_ok());
        assert_eq!(
            limiter.effective_strategy,
            ChatOpsRateLimitStrategy::TokenBucket
        );
        let switch = limiter
            .take_strategy_switch()
            .expect("auto-tune switch should be reported");
        assert_eq!(switch.from, ChatOpsRateLimitStrategy::FixedWindow);
        assert_eq!(switch.to, ChatOpsRateLimitStrategy::TokenBucket);
    }

    #[test]
    fn auto_tune_switches_back_to_fixed_window_after_low_traffic() {
        let policy = ChatOpsRateLimitPolicy {
            strategy: ChatOpsRateLimitStrategy::TokenBucket,
            per_user_limit: Some(6),
            per_channel_limit: None,
            burst_per_user: Some(12),
            burst_per_channel: None,
            auto_tune_profile: ChatOpsRateLimitAutoTuneProfile::Aggressive,
            window: Duration::from_secs(1),
        };
        let mut limiter = ChatOpsRateLimiter::new(policy);
        assert_eq!(
            limiter.effective_strategy,
            ChatOpsRateLimitStrategy::TokenBucket
        );

        let first_low_window =
            ChatOpsCommandEnvelope::new(ChatOpsPlatform::Discord, "ops", "user-1", "list", 1_000);
        assert!(limiter.check_and_record(&first_low_window).is_ok());
        assert_eq!(
            limiter.effective_strategy,
            ChatOpsRateLimitStrategy::TokenBucket
        );

        let second_low_window =
            ChatOpsCommandEnvelope::new(ChatOpsPlatform::Discord, "ops", "user-2", "list", 2_100);
        assert!(limiter.check_and_record(&second_low_window).is_ok());
        assert_eq!(
            limiter.effective_strategy,
            ChatOpsRateLimitStrategy::TokenBucket
        );

        let third_low_window =
            ChatOpsCommandEnvelope::new(ChatOpsPlatform::Discord, "ops", "user-3", "list", 3_200);
        assert!(limiter.check_and_record(&third_low_window).is_ok());
        assert_eq!(
            limiter.effective_strategy,
            ChatOpsRateLimitStrategy::FixedWindow
        );
        let switch = limiter
            .take_strategy_switch()
            .expect("auto-tune switch should be reported");
        assert_eq!(switch.from, ChatOpsRateLimitStrategy::TokenBucket);
        assert_eq!(switch.to, ChatOpsRateLimitStrategy::FixedWindow);
    }

    #[test]
    fn token_bucket_allows_burst_and_refills_over_time() {
        let policy = ChatOpsRateLimitPolicy {
            strategy: ChatOpsRateLimitStrategy::TokenBucket,
            per_user_limit: Some(2),
            per_channel_limit: None,
            burst_per_user: Some(4),
            burst_per_channel: None,
            auto_tune_profile: ChatOpsRateLimitAutoTuneProfile::Disabled,
            window: Duration::from_secs(60),
        };
        let mut limiter = ChatOpsRateLimiter::new(policy);

        for index in 0..4 {
            let envelope = ChatOpsCommandEnvelope::new(
                ChatOpsPlatform::Discord,
                "ops",
                "user-1",
                "list",
                1_000,
            );
            assert!(
                limiter.check_and_record(&envelope).is_ok(),
                "expected burst request #{index} to pass"
            );
        }

        let blocked =
            ChatOpsCommandEnvelope::new(ChatOpsPlatform::Discord, "ops", "user-1", "list", 1_000);
        assert!(limiter.check_and_record(&blocked).is_err());

        let refilled =
            ChatOpsCommandEnvelope::new(ChatOpsPlatform::Discord, "ops", "user-1", "list", 31_000);
        assert!(limiter.check_and_record(&refilled).is_ok());

        let blocked_again =
            ChatOpsCommandEnvelope::new(ChatOpsPlatform::Discord, "ops", "user-1", "list", 31_000);
        assert!(limiter.check_and_record(&blocked_again).is_err());
    }

    #[test]
    fn telegram_webhook_payload_parses_text_message_to_envelope() {
        let payload = r#"{"update_id":10,"message":{"date":1737200000,"text":"list","chat":{"id":555},"from":{"id":777}}}"#;
        let envelopes =
            parse_telegram_webhook_payload(payload).expect("valid payload should parse");
        assert_eq!(envelopes.len(), 1);
        assert_eq!(envelopes[0].platform, ChatOpsPlatform::Telegram);
        assert_eq!(envelopes[0].channel_id, "555");
        assert_eq!(envelopes[0].user_id, "777");
        assert_eq!(envelopes[0].message_text, "list");
        assert_eq!(envelopes[0].transport, IngressTransport::TelegramPolling);
    }

    #[test]
    fn telegram_webhook_payload_rejects_invalid_json() {
        let parsed = parse_telegram_webhook_payload("{invalid");
        assert!(parsed.is_err());
    }

    #[test]
    fn discord_interaction_payload_parses_ping() {
        let parsed = parse_discord_interaction_payload(r#"{"type":1}"#)
            .expect("ping interaction should parse");
        assert!(matches!(parsed, DiscordInteractionIngressPayload::Ping));
    }

    #[test]
    fn discord_interaction_payload_parses_command_with_options() {
        let payload = r#"{"type":2,"channel_id":"555","member":{"user":{"id":"777"}},"data":{"name":"submit","options":[{"type":3,"name":"intent","value":"ai-plan"},{"type":3,"name":"payload","value":"review pipeline"}]}}"#;
        let parsed =
            parse_discord_interaction_payload(payload).expect("command interaction should parse");
        let DiscordInteractionIngressPayload::Command(envelope) = parsed else {
            panic!("expected command payload");
        };

        assert_eq!(envelope.platform, ChatOpsPlatform::Discord);
        assert_eq!(envelope.channel_id, "555");
        assert_eq!(envelope.user_id, "777");
        assert_eq!(envelope.message_text, "submit ai-plan review pipeline");
        assert_eq!(envelope.transport, IngressTransport::DiscordInteractions);
    }

    #[test]
    fn discord_interaction_payload_rejects_invalid_json() {
        let parsed = parse_discord_interaction_payload("{invalid");
        assert!(parsed.is_err());
    }

    #[tokio::test]
    async fn webhook_ingress_queue_drains_in_fifo_order() {
        let ingress = TelegramWebhookIngressAdapter::default();
        let first = r#"{"update_id":10,"message":{"date":1737200000,"text":"list","chat":{"id":100},"from":{"id":200}}}"#;
        let second = r#"{"update_id":11,"message":{"date":1737200001,"text":"help","chat":{"id":101},"from":{"id":201}}}"#;

        let added_first = ingress
            .enqueue_payload(first)
            .expect("first payload should enqueue");
        let added_second = ingress
            .enqueue_payload(second)
            .expect("second payload should enqueue");
        assert_eq!(added_first, 1);
        assert_eq!(added_second, 1);

        let first_batch = ingress.pull_pending(1).await.expect("pull should succeed");
        let second_batch = ingress.pull_pending(8).await.expect("pull should succeed");

        assert_eq!(first_batch.len(), 1);
        assert_eq!(first_batch[0].message_text, "list");
        assert_eq!(second_batch.len(), 1);
        assert_eq!(second_batch[0].message_text, "help");
    }

    #[tokio::test]
    async fn discord_interaction_ingress_handles_ping_and_command_queue() {
        let ingress = DiscordInteractionIngressAdapter::default();
        let ping = ingress
            .enqueue_payload(r#"{"type":1}"#)
            .expect("ping payload should parse");
        assert!(ping.ping);
        assert_eq!(ping.queued, 0);

        let command = ingress
            .enqueue_payload(
                r#"{"type":2,"channel_id":"555","member":{"user":{"id":"777"}},"data":{"name":"list"}}"#,
            )
            .expect("command payload should parse");
        assert!(!command.ping);
        assert_eq!(command.queued, 1);

        let batch = ingress.pull_pending(8).await.expect("pull should succeed");
        assert_eq!(batch.len(), 1);
        assert_eq!(batch[0].platform, ChatOpsPlatform::Discord);
        assert_eq!(batch[0].message_text, "list");
        assert_eq!(batch[0].transport, IngressTransport::DiscordInteractions);
    }

    #[test]
    fn runtime_rejects_webhook_enqueue_when_mode_is_disabled() {
        let runtime = build_chatops_runtime(ChatOpsRuntimeConfig {
            enabled: true,
            allowed_user_ids: vec!["777".to_string()],
            allowed_channel_ids: vec!["555".to_string()],
            telegram_bot_token: Some("token".to_string()),
            telegram_api_base: "http://127.0.0.1".to_string(),
            telegram_webhook_enabled: false,
            ..ChatOpsRuntimeConfig::default()
        })
        .expect("runtime should build")
        .expect("enabled runtime should be present");

        let result = runtime.enqueue_telegram_webhook_payload(
            r#"{"update_id":10,"message":{"date":1737200000,"text":"list","chat":{"id":555},"from":{"id":777}}}"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn runtime_accepts_webhook_enqueue_when_mode_is_enabled() {
        let runtime = build_chatops_runtime(ChatOpsRuntimeConfig {
            enabled: true,
            allowed_user_ids: vec!["777".to_string()],
            allowed_channel_ids: vec!["555".to_string()],
            telegram_bot_token: Some("token".to_string()),
            telegram_api_base: "http://127.0.0.1".to_string(),
            telegram_webhook_enabled: true,
            ..ChatOpsRuntimeConfig::default()
        })
        .expect("runtime should build")
        .expect("enabled runtime should be present");

        let added = runtime
            .enqueue_telegram_webhook_payload(
                r#"{"update_id":10,"message":{"date":1737200000,"text":"list","chat":{"id":555},"from":{"id":777}}}"#,
            )
            .expect("payload should enqueue");
        assert_eq!(added, 1);
        assert!(runtime.is_telegram_webhook_enabled());
    }

    #[test]
    fn runtime_rejects_discord_interaction_enqueue_when_mode_is_disabled() {
        let runtime = build_chatops_runtime(ChatOpsRuntimeConfig {
            enabled: true,
            allowed_user_ids: vec!["777".to_string()],
            allowed_channel_ids: vec!["555".to_string()],
            discord_bot_token: Some("token".to_string()),
            discord_api_base: "http://127.0.0.1".to_string(),
            discord_channel_ids: vec!["555".to_string()],
            discord_interactions_enabled: false,
            ..ChatOpsRuntimeConfig::default()
        })
        .expect("runtime should build")
        .expect("enabled runtime should be present");

        let result = runtime.enqueue_discord_interaction_payload(
            r#"{"type":2,"channel_id":"555","member":{"user":{"id":"777"}},"data":{"name":"list"}}"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn runtime_accepts_discord_interaction_enqueue_when_mode_is_enabled() {
        let runtime = build_chatops_runtime(ChatOpsRuntimeConfig {
            enabled: true,
            allowed_user_ids: vec!["777".to_string()],
            allowed_channel_ids: vec!["555".to_string()],
            discord_bot_token: Some("token".to_string()),
            discord_api_base: "http://127.0.0.1".to_string(),
            discord_interactions_enabled: true,
            ..ChatOpsRuntimeConfig::default()
        })
        .expect("runtime should build")
        .expect("enabled runtime should be present");

        let outcome = runtime
            .enqueue_discord_interaction_payload(
                r#"{"type":2,"channel_id":"555","member":{"user":{"id":"777"}},"data":{"name":"list"}}"#,
            )
            .expect("payload should enqueue");
        assert!(!outcome.ping);
        assert_eq!(outcome.queued, 1);
        assert!(runtime.is_discord_interactions_enabled());
    }
}
