use std::fs;
use tempfile::TempDir;

/// Creates a test template structure for .NET
pub fn create_dotnet_test_structure() -> TempDir {
    let temp = TempDir::new().unwrap();
    let templates = temp.path().join("templates/dotnet");

    fs::create_dir_all(templates.join("src/Domain")).unwrap();
    fs::create_dir_all(templates.join("tests")).unwrap();

    fs::write(
        templates.join("src/Domain/Entity.cs.hbs"),
        "template content",
    )
    .unwrap();

    fs::write(templates.join("tests/Test.cs.hbs"), "test content").unwrap();

    temp
}

/// Creates a test template structure for batch rendering
pub fn create_batch_test_templates() -> TempDir {
    let temp = TempDir::new().unwrap();
    let templates = temp.path().join("templates/dotnet/Domain");

    fs::create_dir_all(&templates).unwrap();
    fs::write(templates.join("Entity.cs.hbs"), "public class {{name}} { }").unwrap();

    temp
}
