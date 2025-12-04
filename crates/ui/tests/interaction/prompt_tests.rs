use nettoolskit_ui::{get_prompt_symbol, get_prompt_string};

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
