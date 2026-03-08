//! Benchmarks for orchestrator runtime command cache operations.
#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use std::path::Path;
use std::time::Duration;

#[path = "../../crates/orchestrator/src/execution/cache.rs"]
#[allow(dead_code, unused_imports)]
mod cache_impl;

use cache_impl::{CacheKey, CacheTtl, CacheValue, CommandResultCache};

const HELP_MARKDOWN_SAMPLE: &str = r#"# NetToolsKit CLI - Help

## Available Commands
- `/help` - Display help information and available commands
- `/manifest` - Manage and apply manifests (submenu)
- `/ai` - AI assistant commands (ask, plan, explain, apply)
- `/config` - View and edit user configuration
- `/clear` - Clear and redraw terminal layout
- `/quit` - Exit NetToolsKit CLI
"#;

fn build_cache() -> CommandResultCache {
    CommandResultCache::new(
        256,
        4 * 1024 * 1024,
        CacheTtl {
            help: Duration::from_secs(300),
            manifest_list: Duration::from_secs(60),
            ai_response: Duration::from_secs(180),
        },
    )
}

fn bench_cache_insert_help(c: &mut Criterion) {
    c.bench_function("command_cache_insert_help", |b| {
        let mut cache = build_cache();
        b.iter(|| {
            let payload = black_box(HELP_MARKDOWN_SAMPLE.to_string());
            let inserted = cache.insert(CacheKey::help(), CacheValue::HelpMarkdown(payload));
            black_box(inserted);
        });
    });
}

fn bench_cache_get_help_hit(c: &mut Criterion) {
    c.bench_function("command_cache_get_help_hit", |b| {
        let mut cache = build_cache();
        let key = CacheKey::help();
        let _ = cache.insert(
            key.clone(),
            CacheValue::HelpMarkdown(HELP_MARKDOWN_SAMPLE.to_string()),
        );

        b.iter(|| {
            let value = cache.get(black_box(&key));
            black_box(value);
        });
    });
}

fn bench_cache_get_manifest_miss(c: &mut Criterion) {
    c.bench_function("command_cache_get_manifest_miss", |b| {
        let mut cache = build_cache();
        let mut sequence = 0usize;

        b.iter(|| {
            sequence = sequence.saturating_add(1);
            let path = format!("c:/bench/miss-{sequence}.manifest.yaml");
            let key = CacheKey::manifest_list(Path::new(&path));
            let value = cache.get(black_box(&key));
            black_box(value);
        });
    });
}

fn bench_cache_eviction_pressure(c: &mut Criterion) {
    c.bench_function("command_cache_eviction_pressure", |b| {
        b.iter_batched(
            build_cache,
            |mut cache| {
                for i in 0..2048usize {
                    let path = format!("c:/bench/evict-{i}.manifest.yaml");
                    let key = CacheKey::manifest_list(Path::new(&path));
                    let value = CacheValue::ManifestListEntries(vec![
                        format!("service-{i}.manifest.yaml").into(),
                        format!("api-{i}.manifest.yaml").into(),
                        format!("infra-{i}.manifest.yaml").into(),
                    ]);
                    let inserted = cache.insert(key, value);
                    black_box(inserted);
                }
                black_box(cache.stats());
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    benches,
    bench_cache_insert_help,
    bench_cache_get_help_hit,
    bench_cache_get_manifest_miss,
    bench_cache_eviction_pressure
);
criterion_main!(benches);
