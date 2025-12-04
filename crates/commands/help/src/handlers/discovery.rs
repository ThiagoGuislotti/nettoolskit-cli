//! Manifest discovery handler

use crate::models::ManifestInfo;
use nettoolskit_manifest::parser::ManifestParser;
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
    if manifests.is_empty() {
        println!("{}", "No manifest files found in workspace.".yellow());
        return;
    }

    println!("\n{}", "📋 Discovered Manifests:".cyan().bold());
    println!("{}", "─".repeat(80).dimmed());

    for (i, manifest) in manifests.iter().enumerate() {
        println!(
            "{}. {} ({})",
            (i + 1).to_string().green(),
            manifest.project_name.cyan().bold(),
            manifest.language.yellow()
        );
        println!("   📁 Path: {}", manifest.path.display().to_string().dimmed());
        println!("   🔢 Contexts: {}", manifest.context_count.to_string().green());

        if i < manifests.len() - 1 {
            println!();
        }
    }

    println!("{}", "─".repeat(80).dimmed());
    println!("\n{} {}", "Total:".bold(), manifests.len().to_string().green().bold());
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
        {
            let entry = match entry_result {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name() {
                    let name = file_name.to_string_lossy();

                    // Support both manifest.yaml and *.manifest.yaml patterns
                    if name.ends_with(".manifest.yaml")
                        || name.ends_with(".manifest.yml")
                        || name == "manifest.yaml"
                        || name == "manifest.yml"
                    {
                        debug!("Found manifest: {:?}", path);
                        manifests.push(path.to_path_buf());
                    }
                }
            }
        }

        Ok(manifests)
    })
    .await?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discover_manifests_empty_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manifests = discover_manifests(Some(temp_dir.path().to_path_buf())).await;
        assert_eq!(manifests.len(), 0);
    }

    #[test]
    fn test_display_manifests_empty() {
        let manifests: Vec<ManifestInfo> = vec![];
        // Should not panic
        display_manifests(&manifests);
    }
}
