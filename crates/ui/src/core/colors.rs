//! Color constants for NetToolsKit UI
//!
//! Provides standard color palette used throughout the UI components.

use owo_colors::Rgb;

/// Color palette with light and dark variants
pub struct Color;

impl Color {
    // Neutral colors
    pub const WHITE: Rgb = Rgb(255, 255, 255);
    pub const GRAY_LIGHT: Rgb = Rgb(180, 180, 180);
    pub const GRAY: Rgb = Rgb(128, 128, 128);
    pub const GRAY_DARK: Rgb = Rgb(80, 80, 80);
    pub const BLACK: Rgb = Rgb(0, 0, 0);

    // Purple (Brand)
    pub const PURPLE_LIGHT: Rgb = Rgb(204, 185, 254);
    pub const PURPLE: Rgb = Rgb(155, 114, 255);
    pub const PURPLE_DARK: Rgb = Rgb(120, 80, 200);

    // Blue
    pub const BLUE_LIGHT: Rgb = Rgb(135, 206, 250);
    pub const BLUE: Rgb = Rgb(65, 105, 225);
    pub const BLUE_DARK: Rgb = Rgb(25, 60, 150);

    // Green
    pub const GREEN_LIGHT: Rgb = Rgb(144, 238, 144);
    pub const GREEN: Rgb = Rgb(50, 205, 50);
    pub const GREEN_DARK: Rgb = Rgb(34, 139, 34);

    // Yellow
    pub const YELLOW_LIGHT: Rgb = Rgb(255, 255, 153);
    pub const YELLOW: Rgb = Rgb(255, 215, 0);
    pub const YELLOW_DARK: Rgb = Rgb(204, 153, 0);

    // Red
    pub const RED_LIGHT: Rgb = Rgb(255, 182, 193);
    pub const RED: Rgb = Rgb(220, 20, 60);
    pub const RED_DARK: Rgb = Rgb(139, 0, 0);

    // Orange
    pub const ORANGE_LIGHT: Rgb = Rgb(255, 200, 124);
    pub const ORANGE: Rgb = Rgb(255, 140, 0);
    pub const ORANGE_DARK: Rgb = Rgb(204, 85, 0);

    // Cyan
    pub const CYAN_LIGHT: Rgb = Rgb(175, 238, 238);
    pub const CYAN: Rgb = Rgb(0, 206, 209);
    pub const CYAN_DARK: Rgb = Rgb(0, 139, 139);
}

// Legacy aliases for backward compatibility (to be removed)
#[deprecated(since = "0.1.0", note = "Use Color::PURPLE instead")]
pub const PRIMARY_COLOR: Rgb = Color::PURPLE;

#[deprecated(since = "0.1.0", note = "Use Color::PURPLE_LIGHT instead")]
pub const SECONDARY_COLOR: Rgb = Color::PURPLE_LIGHT;

#[deprecated(since = "0.1.0", note = "Use Color::WHITE instead")]
pub const WHITE_COLOR: Rgb = Color::WHITE;

#[deprecated(since = "0.1.0", note = "Use Color::GRAY instead")]
pub const GRAY_COLOR: Rgb = Color::GRAY;