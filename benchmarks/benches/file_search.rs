//! Benchmarks for file search operations (WalkBuilder + GlobSet).
#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nettoolskit_core::file_search::{search_files, SearchConfig};
use std::fs;
use tempfile::TempDir;

/// Creates a temporary directory tree with `n` total files across a nested structure.
fn create_test_tree(n: usize) -> TempDir {
    let dir = TempDir::new().expect("create temp dir");
    let root = dir.path();

    let folders = ["src", "src/core", "src/domain", "tests", "docs", "config"];
    for folder in &folders {
        fs::create_dir_all(root.join(folder)).expect("create subfolder");
    }

    let extensions = ["rs", "toml", "md", "yaml", "json", "txt"];
    for i in 0..n {
        let folder_idx = i % folders.len();
        let ext_idx = i % extensions.len();
        let file_name = format!("file_{i}.{}", extensions[ext_idx]);
        let path = root.join(folders[folder_idx]).join(file_name);
        fs::write(&path, format!("// content {i}")).expect("write file");
    }

    dir
}

fn bench_search_files_100_no_filter(c: &mut Criterion) {
    let dir = create_test_tree(100);
    let config = SearchConfig::default();

    c.bench_function("search_files_100_no_filter", |b| {
        b.iter(|| search_files(black_box(dir.path()), black_box(&config)).expect("search"));
    });
}

fn bench_search_files_100_glob_filter(c: &mut Criterion) {
    let dir = create_test_tree(100);
    let config = SearchConfig {
        include_patterns: vec!["*.rs".to_string()],
        ..SearchConfig::default()
    };

    c.bench_function("search_files_100_glob_rs", |b| {
        b.iter(|| search_files(black_box(dir.path()), black_box(&config)).expect("search"));
    });
}

fn bench_search_files_500_no_filter(c: &mut Criterion) {
    let dir = create_test_tree(500);
    let config = SearchConfig::default();

    c.bench_function("search_files_500_no_filter", |b| {
        b.iter(|| search_files(black_box(dir.path()), black_box(&config)).expect("search"));
    });
}

fn bench_search_files_500_glob_filter(c: &mut Criterion) {
    let dir = create_test_tree(500);
    let config = SearchConfig {
        include_patterns: vec!["*.rs".to_string(), "*.toml".to_string()],
        exclude_patterns: vec!["docs/*".to_string()],
        ..SearchConfig::default()
    };

    c.bench_function("search_files_500_glob_multi", |b| {
        b.iter(|| search_files(black_box(dir.path()), black_box(&config)).expect("search"));
    });
}

fn bench_search_files_1000_no_filter(c: &mut Criterion) {
    let dir = create_test_tree(1000);
    let config = SearchConfig::default();

    c.bench_function("search_files_1000_no_filter", |b| {
        b.iter(|| search_files(black_box(dir.path()), black_box(&config)).expect("search"));
    });
}

criterion_group!(
    benches,
    bench_search_files_100_no_filter,
    bench_search_files_100_glob_filter,
    bench_search_files_500_no_filter,
    bench_search_files_500_glob_filter,
    bench_search_files_1000_no_filter,
);
criterion_main!(benches);
