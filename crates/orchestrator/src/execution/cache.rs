//! Runtime command-result cache with LRU, TTL, and memory budget controls.
//!
//! The cache is intentionally small and lightweight because it is used by
//! interactive command flows where predictable latency matters more than
//! unlimited retention.

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Supported cache partitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum CacheKind {
    /// Cache entry for help markdown content.
    Help,
    /// Cache entry for manifest discovery results in a specific root.
    ManifestList,
    /// Cache entry for AI response reuse.
    AiResponse,
}

/// Cache key composed of command partition + discriminator.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct CacheKey {
    kind: CacheKind,
    discriminator: String,
}

impl CacheKey {
    /// Build key for global help markdown.
    pub(crate) fn help() -> Self {
        Self {
            kind: CacheKind::Help,
            discriminator: "global".to_string(),
        }
    }

    /// Build key for manifest discovery scoped by root path.
    pub(crate) fn manifest_list(root: &Path) -> Self {
        let normalized = root
            .to_string_lossy()
            .replace('\\', "/")
            .to_ascii_lowercase();
        Self {
            kind: CacheKind::ManifestList,
            discriminator: normalized,
        }
    }

    /// Build key for AI cache reuse entries.
    pub(crate) fn ai_response(signature: &str) -> Self {
        Self {
            kind: CacheKind::AiResponse,
            discriminator: signature.trim().to_string(),
        }
    }

    /// Return partition for this cache key.
    pub(crate) fn kind(&self) -> CacheKind {
        self.kind
    }
}

/// Value variants supported by command-result cache.
#[derive(Debug, Clone)]
pub(crate) enum CacheValue {
    /// Render-ready help markdown.
    HelpMarkdown(String),
    /// Discovered manifest paths for `manifest list`.
    ManifestListEntries(Vec<PathBuf>),
    /// Cached AI response text.
    AiResponseText(String),
}

impl CacheValue {
    fn estimate_size_bytes(&self) -> usize {
        match self {
            Self::HelpMarkdown(markdown) => 32 + markdown.len(),
            Self::ManifestListEntries(entries) => {
                64 + entries
                    .iter()
                    .map(|path| 24 + path.to_string_lossy().len())
                    .sum::<usize>()
            }
            Self::AiResponseText(text) => 32 + text.len(),
        }
    }
}

/// TTLs by command partition.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CacheTtl {
    /// TTL for help markdown cache entries.
    pub(crate) help: Duration,
    /// TTL for manifest list cache entries.
    pub(crate) manifest_list: Duration,
    /// TTL for AI response cache entries.
    pub(crate) ai_response: Duration,
}

impl CacheTtl {
    fn for_kind(&self, kind: CacheKind) -> Duration {
        match kind {
            CacheKind::Help => self.help,
            CacheKind::ManifestList => self.manifest_list,
            CacheKind::AiResponse => self.ai_response,
        }
    }
}

impl Default for CacheTtl {
    fn default() -> Self {
        Self {
            help: Duration::from_secs(300),
            manifest_list: Duration::from_secs(20),
            ai_response: Duration::from_secs(180),
        }
    }
}

#[derive(Debug, Clone)]
struct CacheEntry {
    value: CacheValue,
    expires_at: Instant,
    size_bytes: usize,
}

/// Observable cache state and behavior counters.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct CacheStats {
    /// Number of successful lookups.
    pub(crate) hits: u64,
    /// Number of cache misses.
    pub(crate) misses: u64,
    /// Number of evictions caused by LRU/TTL/memory pressure.
    pub(crate) evictions: u64,
    /// Number of active entries.
    pub(crate) entries: usize,
    /// Current approximate memory footprint.
    pub(crate) size_bytes: usize,
}

/// In-memory LRU cache used by orchestrator command flows.
pub(crate) struct CommandResultCache {
    entries: HashMap<CacheKey, CacheEntry>,
    lru_order: VecDeque<CacheKey>,
    max_entries: usize,
    max_size_bytes: usize,
    current_size_bytes: usize,
    ttl: CacheTtl,
    hits: u64,
    misses: u64,
    evictions: u64,
}

impl CommandResultCache {
    /// Create cache with bounded entry count and memory budget.
    pub(crate) fn new(max_entries: usize, max_size_bytes: usize, ttl: CacheTtl) -> Self {
        Self {
            entries: HashMap::new(),
            lru_order: VecDeque::new(),
            max_entries: max_entries.max(1),
            max_size_bytes: max_size_bytes.max(1),
            current_size_bytes: 0,
            ttl,
            hits: 0,
            misses: 0,
            evictions: 0,
        }
    }

    /// Fetch a cache entry and refresh its recency on hit.
    pub(crate) fn get(&mut self, key: &CacheKey) -> Option<CacheValue> {
        self.evict_expired();

        let value = self
            .entries
            .get(key)
            .and_then(|entry| (entry.expires_at > Instant::now()).then(|| entry.value.clone()));

        if let Some(value) = value {
            self.hits = self.hits.saturating_add(1);
            self.touch(key);
            return Some(value);
        }

        self.misses = self.misses.saturating_add(1);
        None
    }

    /// Insert or replace a cache entry.
    ///
    /// Returns `false` when value is larger than cache memory budget.
    pub(crate) fn insert(&mut self, key: CacheKey, value: CacheValue) -> bool {
        self.evict_expired();

        let size_bytes = value.estimate_size_bytes();
        if size_bytes > self.max_size_bytes {
            return false;
        }

        self.remove_entry(&key);

        let expires_at = Instant::now() + self.ttl.for_kind(key.kind());
        let entry = CacheEntry {
            value,
            expires_at,
            size_bytes,
        };

        self.current_size_bytes = self.current_size_bytes.saturating_add(size_bytes);
        self.entries.insert(key.clone(), entry);
        self.lru_order.push_back(key);

        self.evict_to_budget();
        true
    }

    /// Current cache counters and footprint.
    pub(crate) fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits,
            misses: self.misses,
            evictions: self.evictions,
            entries: self.entries.len(),
            size_bytes: self.current_size_bytes,
        }
    }

    fn touch(&mut self, key: &CacheKey) {
        if let Some(position) = self.lru_order.iter().position(|stored| stored == key) {
            self.lru_order.remove(position);
        }
        self.lru_order.push_back(key.clone());
    }

    fn evict_to_budget(&mut self) {
        while self.entries.len() > self.max_entries || self.current_size_bytes > self.max_size_bytes
        {
            let Some(key) = self.lru_order.pop_front() else {
                break;
            };
            self.remove_entry_with_evict_count(&key);
        }
    }

    fn evict_expired(&mut self) {
        let now = Instant::now();
        let expired: Vec<CacheKey> = self
            .entries
            .iter()
            .filter_map(|(key, entry)| (entry.expires_at <= now).then_some(key.clone()))
            .collect();

        for key in expired {
            self.remove_entry_with_evict_count(&key);
        }
    }

    fn remove_entry_with_evict_count(&mut self, key: &CacheKey) {
        if self.remove_entry(key) {
            self.evictions = self.evictions.saturating_add(1);
        }
    }

    fn remove_entry(&mut self, key: &CacheKey) -> bool {
        if let Some(entry) = self.entries.remove(key) {
            self.current_size_bytes = self.current_size_bytes.saturating_sub(entry.size_bytes);
            if let Some(position) = self.lru_order.iter().position(|stored| stored == key) {
                self.lru_order.remove(position);
            }
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    fn short_ttl() -> CacheTtl {
        CacheTtl {
            help: Duration::from_millis(20),
            manifest_list: Duration::from_millis(80),
            ai_response: Duration::from_millis(80),
        }
    }

    #[test]
    fn lru_eviction_respects_recent_usage() {
        let mut cache = CommandResultCache::new(2, 1024 * 1024, CacheTtl::default());
        let k1 = CacheKey::help();
        let k2 = CacheKey::manifest_list(Path::new("c:/workspace/a"));
        let k3 = CacheKey::manifest_list(Path::new("c:/workspace/b"));

        assert!(cache.insert(k1.clone(), CacheValue::HelpMarkdown("help".to_string())));
        assert!(cache.insert(
            k2.clone(),
            CacheValue::ManifestListEntries(vec![PathBuf::from("a.manifest.yaml")])
        ));

        let _ = cache.get(&k1);
        assert!(cache.insert(
            k3.clone(),
            CacheValue::ManifestListEntries(vec![PathBuf::from("b.manifest.yaml")])
        ));

        assert!(
            cache.get(&k1).is_some(),
            "Most recently used entry should stay"
        );
        assert!(
            cache.get(&k2).is_none(),
            "Least recently used entry should be evicted"
        );
        assert!(cache.get(&k3).is_some(), "Newest entry should stay");
    }

    #[test]
    fn ttl_is_applied_per_cache_kind() {
        let mut cache = CommandResultCache::new(4, 1024 * 1024, short_ttl());
        let help_key = CacheKey::help();
        let manifest_key = CacheKey::manifest_list(Path::new("c:/workspace"));

        assert!(cache.insert(
            help_key.clone(),
            CacheValue::HelpMarkdown("help markdown".to_string())
        ));
        assert!(cache.insert(
            manifest_key.clone(),
            CacheValue::ManifestListEntries(vec![PathBuf::from("service.manifest.yaml")])
        ));

        sleep(Duration::from_millis(30));

        assert!(
            cache.get(&help_key).is_none(),
            "Help entry should expire first"
        );
        assert!(
            cache.get(&manifest_key).is_some(),
            "Manifest list entry should remain cached longer"
        );
    }

    #[test]
    fn memory_budget_evicts_entries_to_fit_limit() {
        let mut cache = CommandResultCache::new(10, 120, CacheTtl::default());

        assert!(cache.insert(
            CacheKey::manifest_list(Path::new("c:/workspace/a")),
            CacheValue::HelpMarkdown("123456789012345678901234567890".to_string())
        ));
        assert!(cache.insert(
            CacheKey::manifest_list(Path::new("c:/workspace/b")),
            CacheValue::HelpMarkdown("abcdefghijklmnopqrstuvwxyz012345".to_string())
        ));
        assert!(cache.insert(
            CacheKey::manifest_list(Path::new("c:/workspace/c")),
            CacheValue::HelpMarkdown("cache-memory-pressure-third-entry".to_string())
        ));

        let stats = cache.stats();
        assert!(
            stats.size_bytes <= 120,
            "Cache should respect memory budget"
        );
        assert!(stats.entries <= 2, "At least one entry should be evicted");
        assert!(
            stats.evictions >= 1,
            "Eviction counter should be incremented"
        );
    }

    #[test]
    fn oversized_entry_is_rejected() {
        let mut cache = CommandResultCache::new(4, 64, CacheTtl::default());
        let inserted = cache.insert(CacheKey::help(), CacheValue::HelpMarkdown("x".repeat(256)));
        assert!(
            !inserted,
            "Entry bigger than memory budget must be rejected"
        );
        assert_eq!(cache.stats().entries, 0);
    }
}
