//! Manifest discovery handler

use crate::models::ManifestInfo;
use nettoolskit_manifest::ManifestParser;
use nettoolskit_otel::Metrics;
use owo_colors::OwoColorize;
use std::path::{Path, PathBuf};
use tracing::{debug, info};
use walkdir::WalkDir;

/// Discover manifest files in the workspace
///
/// Searches for `.manifest.yaml`, `.manifest.yml`, `manifest.yaml`, or `manifest.yml` files
/// in the specified directory (or current directory if none provided).
///
/// # Arguments
/// * `root` - Optional root directory to search. Defaults to current directory.
///
/// # Returns
/// Vector of discovered and parsed manifest information
pub async fn discover_manifests(root: Option<PathBuf>) -> Vec<ManifestInfo> {
    let metrics = Metrics::new();
    let search_root = root.unwrap_or_else(|| PathBuf::from("."));

    info!("Searching for manifest files in {:?}", search_root);

    let manifest_paths = find_manifest_files(&search_root).await.unwrap_or_default();
    debug!("Found {} potential manifest files", manifest_paths.len());

    let mut manifests = Vec::new();
    for path in manifest_paths {
        debug!("Attempting to parse manifest: {:?}", path);
        match ManifestParser::from_file(&path) {
            Ok(doc) => {
                debug!("Successfully parsed manifest: {}", doc.meta.name);
                manifests.push(ManifestInfo {
                    path: path.clone(),
                    project_name: doc.meta.name.clone(),
                    language: doc.conventions.target_framework.clone(),
                    context_count: doc.contexts.len(),
                });
            }
            Err(e) => {
                debug!("Failed to parse manifest at {:?}: {}", path, e);
            }
        }
    }

    metrics.increment_counter("manifests_discovered");
    info!("Discovered {} manifest files", manifests.len());

    manifests
}

/// Display discovered manifests in a formatted list
///
/// # Arguments
/// * `manifests` - Slice of manifest information to display
pub fn display_manifests(manifests: &[ManifestInfo]) {
    use nettoolskit_ui::Color;
    if manifests.is_empty() {
        println!(
            "{}",
            "No manifest files found in workspace.".color(Color::YELLOW)
        );
        return;
    }

    println!("\n{}", "📋 Discovered Manifests:".color(Color::CYAN).bold());
    println!();

    for (i, manifest) in manifests.iter().enumerate() {
        println!(
            "{}. {} ({})",
            (i + 1).to_string().color(Color::GREEN),
            manifest.project_name.color(Color::CYAN).bold(),
            manifest.language.color(Color::YELLOW)
        );
        println!("   📁 Path: {}", manifest.path.display());
        println!(
            "   🔢 Contexts: {}",
            manifest.context_count.to_string().color(Color::GREEN)
        );

        if i < manifests.len() - 1 {
            println!();
        }
    }

    println!();
    println!(
        "{} {}",
        "Total:".bold(),
        manifests.len().to_string().color(Color::GREEN).bold()
    );
}

/// Find manifest files in directory tree
async fn find_manifest_files(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    use tokio::task;

    let root = root.to_path_buf();
    task::spawn_blocking(move || {
        let mut manifests = Vec::new();

        for entry_result in WalkDir::new(&root)
            .max_depth(10)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| !entry.file_type().is_dir() || !should_skip_dir(entry.path()))
        {
            let entry = match entry_result {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name() {
                    let name = file_name.to_string_lossy();
                    if is_manifest_file_name(&name) {
                        debug!("Found manifest: {:?}", path);
                        manifests.push(path.to_path_buf());
                    }
                }
            }
        }

        manifests.sort_unstable();
        manifests.dedup();
        Ok(manifests)
    })
    .await?
}

fn is_manifest_file_name(name: &str) -> bool {
    name.ends_with(".manifest.yaml")
        || name.ends_with(".manifest.yml")
        || name == "manifest.yaml"
        || name == "manifest.yml"
}

fn should_skip_dir(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(".git" | "target" | "node_modules" | ".idea" | ".vscode")
    )
}

#[cfg(test)]
mod tests {
    use super::{is_manifest_file_name, should_skip_dir};
    use std::path::Path;

    #[test]
    fn is_manifest_file_name_matches_supported_patterns() {
        assert!(is_manifest_file_name("manifest.yaml"));
        assert!(is_manifest_file_name("manifest.yml"));
        assert!(is_manifest_file_name("feature.manifest.yaml"));
        assert!(is_manifest_file_name("feature.manifest.yml"));
        assert!(!is_manifest_file_name("manifest.json"));
    }

    #[test]
    fn should_skip_dir_filters_known_heavy_directories() {
        assert!(should_skip_dir(Path::new("target")));
        assert!(should_skip_dir(Path::new(".git")));
        assert!(should_skip_dir(Path::new("node_modules")));
        assert!(should_skip_dir(Path::new(".idea")));
        assert!(should_skip_dir(Path::new(".vscode")));
        assert!(!should_skip_dir(Path::new("src")));
    }
}
