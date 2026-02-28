//! Template mapping definition

use serde::Deserialize;

/// Template mapping definition
#[derive(Debug, Deserialize, Clone)]
pub struct TemplateMapping {
    /// Artifact kind identifier.
    pub artifact: String,
    /// Handlebars template file path.
    pub template: String,
    /// Destination path pattern.
    pub dst: String,
}
