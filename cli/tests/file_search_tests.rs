use nettoolskit_file_search::{SearchConfig, search_files, FileFilters};
use std::path::Path;

#[test]
fn test_dotnet_project_detection() {
    assert!(FileFilters::is_dotnet_project("MyProject.csproj"));
    assert!(FileFilters::is_dotnet_project("MyProject.vbproj"));
    assert!(FileFilters::is_dotnet_project("MyProject.fsproj"));
    assert!(!FileFilters::is_dotnet_project("MyProject.txt"));
}

#[test]
fn test_solution_detection() {
    assert!(FileFilters::is_solution("MySolution.sln"));
    assert!(!FileFilters::is_solution("MySolution.csproj"));
}

#[test]
fn test_template_detection() {
    assert!(FileFilters::is_template("template.hbs"));
    assert!(FileFilters::is_template("file.template"));
    assert!(!FileFilters::is_template("file.txt"));
}

#[test]
fn test_manifest_detection() {
    assert!(FileFilters::is_manifest("ntk-manifest.yml"));
    assert!(FileFilters::is_manifest("ntk-config.yaml"));
    assert!(!FileFilters::is_manifest("config.yml"));
    assert!(!FileFilters::is_manifest("ntk-config.json"));
}

#[test]
fn test_directory_ignore() {
    assert!(FileFilters::should_ignore_directory("target"));
    assert!(FileFilters::should_ignore_directory("node_modules"));
    assert!(FileFilters::should_ignore_directory(".git"));
    assert!(FileFilters::should_ignore_directory("bin"));
    assert!(FileFilters::should_ignore_directory("obj"));
    assert!(!FileFilters::should_ignore_directory("src"));
}

#[test]
fn test_search_config_default() {
    let config = SearchConfig::default();
    assert_eq!(config.include_patterns, vec!["*"]);
    assert!(config.exclude_patterns.is_empty());
    assert!(config.max_depth.is_none());
    assert!(!config.follow_links);
    assert!(!config.include_hidden);
}