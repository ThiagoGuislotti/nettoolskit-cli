use std::path::Path;

/// Common file filters for `NetToolsKit`
pub struct FileFilters;

impl FileFilters {
    /// Check if file is a .NET project file
    pub fn is_dotnet_project<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| matches!(ext, "csproj" | "vbproj" | "fsproj"))
    }

    /// Check if file is a solution file
    pub fn is_solution<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| matches!(ext, "sln"))
    }

    /// Check if file is a template file
    pub fn is_template<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| matches!(ext, "hbs" | "template"))
    }

    /// Check if file is a manifest file
    pub fn is_manifest<P: AsRef<Path>>(path: P) -> bool {
        let path = path.as_ref();

        // Check extension
        let has_yaml_ext = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| matches!(ext, "yml" | "yaml"));

        if !has_yaml_ext {
            return false;
        }

        // Check filename pattern
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("ntk-"))
    }

    /// Check if directory should be ignored
    pub fn should_ignore_directory<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref()
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                matches!(
                    name,
                    "target" | "node_modules" | ".git" | "bin" | "obj" | ".vs" | ".vscode"
                )
            })
    }
}
