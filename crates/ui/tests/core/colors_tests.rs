use nettoolskit_ui::Color;

#[test]
fn test_color_constants_exist() {
    // Test that all colors are accessible
    let _ = Color::WHITE;
    let _ = Color::BLACK;
    let _ = Color::GRAY;
    let _ = Color::GRAY_LIGHT;
    let _ = Color::GRAY_DARK;

    let _ = Color::PURPLE;
    let _ = Color::PURPLE_LIGHT;
    let _ = Color::PURPLE_DARK;

    let _ = Color::BLUE;
    let _ = Color::BLUE_LIGHT;
    let _ = Color::BLUE_DARK;

    let _ = Color::GREEN;
    let _ = Color::GREEN_LIGHT;
    let _ = Color::GREEN_DARK;

    let _ = Color::YELLOW;
    let _ = Color::YELLOW_LIGHT;
    let _ = Color::YELLOW_DARK;

    let _ = Color::RED;
    let _ = Color::RED_LIGHT;
    let _ = Color::RED_DARK;
}

#[test]
fn test_rgb_values() {
    // Test some specific RGB values
    assert_eq!(Color::WHITE, owo_colors::Rgb(255, 255, 255));
    assert_eq!(Color::BLACK, owo_colors::Rgb(0, 0, 0));
    assert_eq!(Color::PURPLE, owo_colors::Rgb(155, 114, 255));
}
