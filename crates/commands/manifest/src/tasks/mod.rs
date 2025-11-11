/// Task generation modules
mod api;
mod application;
mod artifact;
mod domain;

pub use api::append_api_tasks;
pub use application::append_application_tasks;
pub use artifact::append_artifact_tasks;
pub use domain::append_domain_tasks;
