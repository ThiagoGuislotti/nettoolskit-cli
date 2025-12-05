///! Template mapping definition

use serde::Deserialize;

/// Template mapping definition
#[derive(Debug, Deserialize, Clone)]
pub struct TemplateMapping {
    pub artifact: String,
    pub template: String,
    pub dst: String,
}
