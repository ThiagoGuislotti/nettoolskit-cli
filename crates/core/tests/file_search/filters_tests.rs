//! Tests for file filtering functionality
//!
//! Validates file type detection (dotnet projects, solutions, templates, manifests),
//! directory ignore rules, search configuration, and pattern matching.
//!
//! ## Test Coverage
//! - Dotnet project detection (.csproj, .vbproj, .fsproj)
//! - Solution detection (.sln)
//! - Template detection (.hbs, .template)
//! - Manifest detection (ntk-manifest.yml, ntk-config.yaml)
//! - Directory ignore rules (target, `node_modules`, .git, bin, obj)
//! - Search configuration (default, custom patterns)

use nettoolskit_core::file_search::{FileFilters, SearchConfig};

// File Type Detection Tests

#[test]
fn test_dotnet_project_detection() {
    // Assert
    assert!(FileFilters::is_dotnet_project("MyProject.csproj"));
    assert!(FileFilters::is_dotnet_project("MyProject.vbproj"));
    assert!(FileFilters::is_dotnet_project("MyProject.fsproj"));
    assert!(!FileFilters::is_dotnet_project("MyProject.txt"));
}

#[test]
fn test_solution_detection() {
    // Assert
    assert!(FileFilters::is_solution("MySolution.sln"));
    assert!(!FileFilters::is_solution("MySolution.csproj"));
}

#[test]
fn test_template_detection() {
    // Assert
    assert!(FileFilters::is_template("template.hbs"));
    assert!(FileFilters::is_template("file.template"));
    assert!(!FileFilters::is_template("file.txt"));
}

#[test]
fn test_manifest_detection() {
    // Assert
    assert!(FileFilters::is_manifest("ntk-manifest.yml"));
    assert!(FileFilters::is_manifest("ntk-config.yaml"));
    assert!(!FileFilters::is_manifest("config.yml"));
    assert!(!FileFilters::is_manifest("ntk-config.json"));
}

// Directory Ignore Rules Tests

#[test]
fn test_directory_ignore() {
    // Assert
    assert!(FileFilters::should_ignore_directory("target"));
    assert!(FileFilters::should_ignore_directory("node_modules"));
    assert!(FileFilters::should_ignore_directory(".git"));
    assert!(FileFilters::should_ignore_directory("bin"));
    assert!(FileFilters::should_ignore_directory("obj"));
    assert!(!FileFilters::should_ignore_directory("src"));
}

// Search Configuration Tests

#[test]
fn test_search_config_default() {
    // Act
    let config = SearchConfig::default();

    // Assert
    assert_eq!(config.include_patterns, vec!["*"]);
    assert!(config.exclude_patterns.is_empty());
    assert!(config.max_depth.is_none());
    assert!(!config.follow_links);
    assert!(!config.include_hidden);
}

#[test]
fn test_file_extension_matching() {
    // Assert - .NET project extensions
    assert!(FileFilters::is_dotnet_project("App.csproj"));
    assert!(FileFilters::is_dotnet_project("Library.vbproj"));
    assert!(FileFilters::is_dotnet_project("Functional.fsproj"));

    // Assert - Case sensitivity
    assert!(!FileFilters::is_dotnet_project("Project.CSPROJ"));
    assert!(!FileFilters::is_dotnet_project("project.CsProj"));
}

#[test]
fn test_template_extensions() {
    // Assert - Handlebars templates
    assert!(FileFilters::is_template("component.hbs"));
    assert!(!FileFilters::is_template("layout.handlebars"));

    // Assert - Generic templates
    assert!(FileFilters::is_template("readme.template"));
    assert!(!FileFilters::is_template("config.tmpl"));

    // Assert - Regular files
    assert!(!FileFilters::is_template("readme.md"));
    assert!(!FileFilters::is_template("config.json"));
}

#[test]
fn test_manifest_naming_convention() {
    // Assert - Standard NetToolsKit manifests
    assert!(FileFilters::is_manifest("ntk-manifest.yml"));
    assert!(FileFilters::is_manifest("ntk-manifest.yaml"));
    assert!(FileFilters::is_manifest("ntk-config.yml"));
    assert!(FileFilters::is_manifest("ntk-config.yaml"));

    // Assert - Other YAML files
    assert!(!FileFilters::is_manifest("docker-compose.yml"));
    assert!(!FileFilters::is_manifest("config.yaml"));
    assert!(!FileFilters::is_manifest("manifest.yml"));
}

#[test]
fn test_ignore_patterns() {
    // Assert - Build outputs
    assert!(FileFilters::should_ignore_directory("bin"));
    assert!(FileFilters::should_ignore_directory("obj"));
    assert!(FileFilters::should_ignore_directory("target"));
    assert!(!FileFilters::should_ignore_directory("dist"));
    assert!(!FileFilters::should_ignore_directory("build"));

    // Assert - Package managers
    assert!(FileFilters::should_ignore_directory("node_modules"));
    assert!(!FileFilters::should_ignore_directory("packages"));

    // Assert - Version control
    assert!(FileFilters::should_ignore_directory(".git"));
    assert!(!FileFilters::should_ignore_directory(".svn"));

    // Assert - IDE files
    assert!(FileFilters::should_ignore_directory(".vs"));
    assert!(FileFilters::should_ignore_directory(".vscode"));

    // Assert - Source directories
    assert!(!FileFilters::should_ignore_directory("src"));
    assert!(!FileFilters::should_ignore_directory("lib"));
    assert!(!FileFilters::should_ignore_directory("tests"));
}
