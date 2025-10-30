use std::path::Path;

/// Common file filters for NetToolsKit
pub struct FileFilters;

impl FileFilters {
    /// Check if file is a .NET project file
    pub fn is_dotnet_project<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| matches!(ext, "csproj" | "vbproj" | "fsproj"))
            .unwrap_or(false)
    }

    /// Check if file is a solution file
    pub fn is_solution<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| matches!(ext, "sln"))
            .unwrap_or(false)
    }

    /// Check if file is a template file
    pub fn is_template<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| matches!(ext, "hbs" | "template"))
            .unwrap_or(false)
    }

    /// Check if file is a manifest file
    pub fn is_manifest<P: AsRef<Path>>(path: P) -> bool {
        let path = path.as_ref();

        // Check extension
        let has_yaml_ext = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| matches!(ext, "yml" | "yaml"))
            .unwrap_or(false);

        if !has_yaml_ext {
            return false;
        }

        // Check filename pattern
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.starts_with("ntk-"))
            .unwrap_or(false)
    }

    /// Check if directory should be ignored
    pub fn should_ignore_directory<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref()
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| {
                matches!(
                    name,
                    "target" | "node_modules" | ".git" | "bin" | "obj" | ".vs" | ".vscode"
                )
            })
            .unwrap_or(false)
    }
}