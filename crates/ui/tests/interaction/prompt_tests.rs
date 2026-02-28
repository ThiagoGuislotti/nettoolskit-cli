use nettoolskit_ui::{get_prompt_string, get_prompt_symbol};

#[test]
fn test_prompt_symbol_constant() {
    assert_eq!(get_prompt_symbol(), "> ");
}

#[test]
fn test_prompt_string_not_empty() {
    let prompt = get_prompt_string();
    assert!(!prompt.is_empty());
    assert!(prompt.contains(">"));
}

#[test]
fn test_prompt_symbol_length() {
    assert_eq!(get_prompt_symbol().len(), 2);
}

#[test]
fn test_prompt_string_contains_symbol() {
    let formatted = get_prompt_string();
    // The formatted string must contain the raw symbol content
    assert!(formatted.contains('>'));
}

#[test]
fn test_prompt_symbol_starts_with_angle() {
    assert!(get_prompt_symbol().starts_with('>'));
}

#[test]
fn test_prompt_string_is_deterministic() {
    let a = get_prompt_string();
    let b = get_prompt_string();
    assert_eq!(a, b);
}
