#[cfg(test)]
mod tests {
    use nettoolskit_ui::{GRAY_COLOR, PRIMARY_COLOR, SECONDARY_COLOR, WHITE_COLOR};

    #[test]
    fn test_color_constants() {
        // Test that color constants are accessible and have expected RGB values
        assert_eq!(PRIMARY_COLOR.0, 155);
        assert_eq!(PRIMARY_COLOR.1, 114);
        assert_eq!(PRIMARY_COLOR.2, 255);

        assert_eq!(WHITE_COLOR.0, 255);
        assert_eq!(WHITE_COLOR.1, 255);
        assert_eq!(WHITE_COLOR.2, 255);

        assert_eq!(GRAY_COLOR.0, 128);
        assert_eq!(GRAY_COLOR.1, 128);
        assert_eq!(GRAY_COLOR.2, 128);

        assert_eq!(SECONDARY_COLOR.0, 204);
        assert_eq!(SECONDARY_COLOR.1, 185);
        assert_eq!(SECONDARY_COLOR.2, 254);
    }

    #[test]
    fn test_ui_modules_exist() {
        // Test that we can access the UI modules
        use nettoolskit_ui::{clear_terminal, print_logo};

        // These functions should be available (even if we don't call them in tests)
        let _ = clear_terminal;
        let _ = print_logo;
    }
}
