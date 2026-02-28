//! Tests for file search functionality
//!
//! Validates synchronous and asynchronous file search, glob filtering,
//! exclude patterns, depth limits, hidden files, template/manifest discovery,
//! and concurrent search across multiple directories.
//!
//! ## Test Coverage
//! - `search_files` with default config
//! - Include/exclude glob patterns
//! - Max depth limits
//! - Hidden files toggle
//! - `search_files_async` (async wrapper)
//! - `search_files_concurrent` (multi-root)
//! - `find_templates` (template discovery)
//! - `find_manifests` (manifest discovery)
//! - `find_templates_async` (async template discovery)
//! - `SearchConfig::default` validation

use nettoolskit_core::file_search::search::{
    find_manifests, find_templates, find_templates_async, search_files, search_files_async,
    search_files_concurrent, SearchConfig,
};
use std::fs;
use tempfile::TempDir;

// Helper to create a temp directory with test files

fn setup_test_dir() -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    let root = dir.path();

    // Create files at root
    fs::write(root.join("main.rs"), "fn main() {}").unwrap();
    fs::write(root.join("lib.rs"), "pub mod utils;").unwrap();
    fs::write(root.join("README.md"), "# Readme").unwrap();
    fs::write(root.join("config.yaml"), "key: value").unwrap();

    // Create subdirectory with files
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src").join("utils.rs"), "pub fn add() {}").unwrap();
    fs::write(root.join("src").join("helper.ts"), "export {}").unwrap();

    // Create nested subdirectory
    fs::create_dir_all(root.join("src").join("deep")).unwrap();
    fs::write(
        root.join("src").join("deep").join("nested.rs"),
        "mod inner;",
    )
    .unwrap();

    // Create template files
    fs::write(root.join("template.hbs"), "{{name}}").unwrap();
    fs::write(root.join("layout.template"), "layout").unwrap();

    // Create manifest-like files
    fs::write(root.join("ntk-manifest.yml"), "name: test").unwrap();
    fs::write(root.join("service.yaml"), "service: true").unwrap();

    dir
}

fn setup_test_dir_with_hidden() -> TempDir {
    let dir = setup_test_dir();
    let root = dir.path();

    // Create hidden file (dot-prefixed)
    fs::write(root.join(".hidden"), "secret").unwrap();
    fs::create_dir_all(root.join(".config")).unwrap();
    fs::write(root.join(".config").join("settings.json"), "{}").unwrap();

    dir
}

// SearchConfig Default Tests

#[test]
fn test_search_config_default() {
    let config = SearchConfig::default();

    assert_eq!(config.include_patterns, vec!["*".to_string()]);
    assert!(config.exclude_patterns.is_empty());
    assert!(config.max_depth.is_none());
    assert!(!config.follow_links);
    assert!(!config.include_hidden);
}

#[test]
fn test_search_config_clone() {
    let config = SearchConfig {
        include_patterns: vec!["*.rs".to_string()],
        exclude_patterns: vec!["**/target/**".to_string()],
        max_depth: Some(5),
        follow_links: true,
        include_hidden: true,
    };

    let cloned = config.clone();
    assert_eq!(cloned.include_patterns, config.include_patterns);
    assert_eq!(cloned.exclude_patterns, config.exclude_patterns);
    assert_eq!(cloned.max_depth, config.max_depth);
    assert_eq!(cloned.follow_links, config.follow_links);
    assert_eq!(cloned.include_hidden, config.include_hidden);
}

#[test]
fn test_search_config_debug() {
    let config = SearchConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("SearchConfig"));
    assert!(debug.contains("include_patterns"));
}

// search_files — Basic Tests

#[test]
fn test_search_files_default_config() {
    let dir = setup_test_dir();
    let config = SearchConfig::default();

    let results = search_files(dir.path(), &config).unwrap();

    // Should find all non-hidden files
    assert!(
        results.len() >= 10,
        "Expected at least 10 files, got {}",
        results.len()
    );
}

#[test]
fn test_search_files_include_pattern_rs() {
    let dir = setup_test_dir();
    let config = SearchConfig {
        include_patterns: vec!["*.rs".to_string()],
        ..SearchConfig::default()
    };

    let results = search_files(dir.path(), &config).unwrap();

    // Should find: main.rs, lib.rs, src/utils.rs, src/deep/nested.rs = 4 .rs files
    assert!(
        results
            .iter()
            .all(|p| p.extension().is_some_and(|e| e == "rs")),
        "All results should be .rs files"
    );
    assert!(
        results.len() >= 4,
        "Expected at least 4 .rs files, got {}",
        results.len()
    );
}

#[test]
fn test_search_files_exclude_pattern() {
    let dir = setup_test_dir();
    let config = SearchConfig {
        include_patterns: vec!["*".to_string()],
        exclude_patterns: vec!["*.rs".to_string()],
        ..SearchConfig::default()
    };

    let results = search_files(dir.path(), &config).unwrap();

    assert!(
        results
            .iter()
            .all(|p| p.extension().is_none_or(|e| e != "rs")),
        "No .rs files should be in results"
    );
}

#[test]
fn test_search_files_multiple_include_patterns() {
    let dir = setup_test_dir();
    let config = SearchConfig {
        include_patterns: vec!["*.rs".to_string(), "*.ts".to_string()],
        ..SearchConfig::default()
    };

    let results = search_files(dir.path(), &config).unwrap();

    assert!(
        results
            .iter()
            .all(|p| { p.extension().is_some_and(|e| e == "rs" || e == "ts") }),
        "All results should be .rs or .ts files"
    );
    // Should include .rs and .ts files
    assert!(results.len() >= 5);
}

#[test]
fn test_search_files_max_depth() {
    let dir = setup_test_dir(); // has files at depth 0, 1, 2

    let config = SearchConfig {
        include_patterns: vec!["*.rs".to_string()],
        max_depth: Some(1),
        ..SearchConfig::default()
    };

    let results = search_files(dir.path(), &config).unwrap();

    // max_depth=1 means only root level files (main.rs, lib.rs)
    // src/utils.rs is at depth 2, src/deep/nested.rs at depth 3
    assert!(!results.is_empty());
    for path in &results {
        // Files should be directly in root (not in subdirectories deeper than 1)
        let relative = path.strip_prefix(dir.path()).unwrap();
        let components: Vec<_> = relative.components().collect();
        assert!(
            components.len() <= 1,
            "File {:?} exceeds max_depth=1",
            relative
        );
    }
}

#[test]
fn test_search_files_hidden_excluded_by_default() {
    let dir = setup_test_dir_with_hidden();
    let config = SearchConfig::default();

    let results = search_files(dir.path(), &config).unwrap();

    // .hidden and .config/settings.json should NOT be in results
    let has_hidden = results.iter().any(|p| {
        p.file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with('.'))
    });
    assert!(
        !has_hidden,
        "Hidden files should not appear with default config"
    );
}

#[test]
fn test_search_files_include_hidden() {
    let dir = setup_test_dir_with_hidden();
    let config = SearchConfig {
        include_hidden: true,
        ..SearchConfig::default()
    };

    let results = search_files(dir.path(), &config).unwrap();

    // Should include more files than default (hidden file + .config/settings.json)
    let config_default = SearchConfig::default();
    let results_default = search_files(dir.path(), &config_default).unwrap();

    assert!(
        results.len() > results_default.len(),
        "Including hidden files should return more results ({} vs {})",
        results.len(),
        results_default.len()
    );
}

#[test]
fn test_search_files_empty_directory() {
    let dir = TempDir::new().unwrap();
    let config = SearchConfig::default();

    let results = search_files(dir.path(), &config).unwrap();
    assert!(results.is_empty());
}

// find_templates Tests

#[test]
fn test_find_templates() {
    let dir = setup_test_dir();

    let results = find_templates(dir.path()).unwrap();

    // Should find template.hbs and layout.template
    assert_eq!(
        results.len(),
        2,
        "Expected 2 template files, got {}",
        results.len()
    );
    let names: Vec<String> = results
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .collect();
    assert!(names.contains(&"template.hbs".to_string()));
    assert!(names.contains(&"layout.template".to_string()));
}

#[test]
fn test_find_templates_no_templates() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();

    let results = find_templates(dir.path()).unwrap();
    assert!(results.is_empty());
}

// find_manifests Tests

#[test]
fn test_find_manifests() {
    let dir = setup_test_dir();

    let results = find_manifests(dir.path()).unwrap();

    // Should find config.yaml, ntk-manifest.yml, service.yaml
    assert!(
        results.len() >= 2,
        "Expected at least 2 manifest files, got {}",
        results.len()
    );
    let extensions: Vec<String> = results
        .iter()
        .filter_map(|p| p.extension().map(|e| e.to_string_lossy().to_string()))
        .collect();
    assert!(
        extensions.iter().all(|e| e == "yml" || e == "yaml"),
        "All manifest results should be .yml or .yaml files"
    );
}

#[test]
fn test_find_manifests_no_manifests() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("code.rs"), "fn main() {}").unwrap();

    let results = find_manifests(dir.path()).unwrap();
    assert!(results.is_empty());
}

// Async Tests

#[tokio::test]
async fn test_search_files_async_basic() {
    let dir = setup_test_dir();
    let config = SearchConfig::default();

    let results = search_files_async(dir.path(), &config).await.unwrap();

    assert!(
        results.len() >= 10,
        "Expected at least 10 files, got {}",
        results.len()
    );
}

#[tokio::test]
async fn test_search_files_async_with_filter() {
    let dir = setup_test_dir();
    let config = SearchConfig {
        include_patterns: vec!["*.rs".to_string()],
        ..SearchConfig::default()
    };

    let results = search_files_async(dir.path(), &config).await.unwrap();

    assert!(results.len() >= 4);
    assert!(results
        .iter()
        .all(|p| p.extension().is_some_and(|e| e == "rs")));
}

#[tokio::test]
async fn test_find_templates_async() {
    let dir = setup_test_dir();

    let results = find_templates_async(dir.path()).await.unwrap();

    assert_eq!(
        results.len(),
        2,
        "Expected 2 template files, got {}",
        results.len()
    );
}

// Concurrent Tests

#[tokio::test]
async fn test_search_files_concurrent_multiple_roots() {
    let dir1 = TempDir::new().unwrap();
    let dir2 = TempDir::new().unwrap();

    fs::write(dir1.path().join("a.rs"), "a").unwrap();
    fs::write(dir1.path().join("b.rs"), "b").unwrap();
    fs::write(dir2.path().join("c.rs"), "c").unwrap();

    let config = SearchConfig {
        include_patterns: vec!["*.rs".to_string()],
        ..SearchConfig::default()
    };

    let roots = vec![dir1.path().to_path_buf(), dir2.path().to_path_buf()];
    let results = search_files_concurrent(roots, &config).await.unwrap();

    assert_eq!(results.len(), 3, "Expected 3 files across 2 directories");
}

#[tokio::test]
async fn test_search_files_concurrent_single_root() {
    let dir = setup_test_dir();
    let config = SearchConfig::default();

    let roots = vec![dir.path().to_path_buf()];
    let results = search_files_concurrent(roots, &config).await.unwrap();

    assert!(results.len() >= 10);
}

#[tokio::test]
async fn test_search_files_concurrent_empty_roots() {
    let config = SearchConfig::default();
    let roots: Vec<std::path::PathBuf> = Vec::new();
    let results = search_files_concurrent(roots, &config).await.unwrap();

    assert!(results.is_empty());
}
