use nettoolskit_file_search::{SearchConfig, FileFilters};

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

#[test]
fn test_file_extension_matching() {
    // Test .NET project extensions
    assert!(FileFilters::is_dotnet_project("App.csproj"));
    assert!(FileFilters::is_dotnet_project("Library.vbproj"));
    assert!(FileFilters::is_dotnet_project("Functional.fsproj"));

    // Case sensitivity test - current implementation is case sensitive
    assert!(!FileFilters::is_dotnet_project("Project.CSPROJ"));
    assert!(!FileFilters::is_dotnet_project("project.CsProj"));
}#[test]
fn test_template_extensions() {
    // Handlebars templates
    assert!(FileFilters::is_template("component.hbs"));
    assert!(!FileFilters::is_template("layout.handlebars")); // .handlebars not supported, only .hbs

    // Generic templates
    assert!(FileFilters::is_template("readme.template"));
    assert!(!FileFilters::is_template("config.tmpl")); // .tmpl not supported, only .template

    // Should not match regular files
    assert!(!FileFilters::is_template("readme.md"));
    assert!(!FileFilters::is_template("config.json"));
}

#[test]
fn test_manifest_naming_convention() {
    // Standard NetToolsKit manifests
    assert!(FileFilters::is_manifest("ntk-manifest.yml"));
    assert!(FileFilters::is_manifest("ntk-manifest.yaml"));
    assert!(FileFilters::is_manifest("ntk-config.yml"));
    assert!(FileFilters::is_manifest("ntk-config.yaml"));

    // Should not match other YAML files
    assert!(!FileFilters::is_manifest("docker-compose.yml"));
    assert!(!FileFilters::is_manifest("config.yaml"));
    assert!(!FileFilters::is_manifest("manifest.yml"));
}

#[test]
fn test_ignore_patterns() {
    // Build outputs
    assert!(FileFilters::should_ignore_directory("bin"));
    assert!(FileFilters::should_ignore_directory("obj"));
    assert!(FileFilters::should_ignore_directory("target"));
    assert!(!FileFilters::should_ignore_directory("dist")); // dist not in current implementation
    assert!(!FileFilters::should_ignore_directory("build")); // build not in current implementation

    // Package managers
    assert!(FileFilters::should_ignore_directory("node_modules"));
    assert!(!FileFilters::should_ignore_directory("packages")); // packages not in current implementation

    // Version control
    assert!(FileFilters::should_ignore_directory(".git"));
    assert!(!FileFilters::should_ignore_directory(".svn")); // .svn not in current implementation

    // IDE files
    assert!(FileFilters::should_ignore_directory(".vs"));
    assert!(FileFilters::should_ignore_directory(".vscode"));

    // Should not ignore source directories
    assert!(!FileFilters::should_ignore_directory("src"));
    assert!(!FileFilters::should_ignore_directory("lib"));
    assert!(!FileFilters::should_ignore_directory("tests"));
}