//! Color constants for NetToolsKit UI
//!
//! Provides standard color palette used throughout the UI components.

use owo_colors::Rgb;

/// Color palette with light and dark variants
pub struct Color;

impl Color {
    // Neutral colors

    /// Pure white `(255, 255, 255)`
    pub const WHITE: Rgb = Rgb(255, 255, 255);
    /// Light gray `(180, 180, 180)` — subtle backgrounds and borders
    pub const GRAY_LIGHT: Rgb = Rgb(180, 180, 180);
    /// Medium gray `(128, 128, 128)` — secondary text and hints
    pub const GRAY: Rgb = Rgb(128, 128, 128);
    /// Dark gray `(80, 80, 80)` — muted accents
    pub const GRAY_DARK: Rgb = Rgb(80, 80, 80);
    /// Pure black `(0, 0, 0)`
    pub const BLACK: Rgb = Rgb(0, 0, 0);

    // Purple (Brand)

    /// Light purple `(204, 185, 254)` — soft brand accent
    pub const PURPLE_LIGHT: Rgb = Rgb(204, 185, 254);
    /// Primary brand purple `(155, 114, 255)`
    pub const PURPLE: Rgb = Rgb(155, 114, 255);
    /// Dark purple `(120, 80, 200)` — emphasis brand accent
    pub const PURPLE_DARK: Rgb = Rgb(120, 80, 200);

    // Blue

    /// Light blue `(135, 206, 250)` — informational highlights
    pub const BLUE_LIGHT: Rgb = Rgb(135, 206, 250);
    /// Standard blue `(65, 105, 225)` — links and actions
    pub const BLUE: Rgb = Rgb(65, 105, 225);
    /// Dark blue `(25, 60, 150)` — deep accent
    pub const BLUE_DARK: Rgb = Rgb(25, 60, 150);

    // Green

    /// Light green `(144, 238, 144)` — soft success indicator
    pub const GREEN_LIGHT: Rgb = Rgb(144, 238, 144);
    /// Standard green `(50, 205, 50)` — success and confirmation
    pub const GREEN: Rgb = Rgb(50, 205, 50);
    /// Dark green `(34, 139, 34)` — emphasis success
    pub const GREEN_DARK: Rgb = Rgb(34, 139, 34);

    // Yellow

    /// Light yellow `(255, 255, 153)` — soft warning indicator
    pub const YELLOW_LIGHT: Rgb = Rgb(255, 255, 153);
    /// Standard yellow `(255, 215, 0)` — warnings and caution
    pub const YELLOW: Rgb = Rgb(255, 215, 0);
    /// Dark yellow `(204, 153, 0)` — emphasis warning
    pub const YELLOW_DARK: Rgb = Rgb(204, 153, 0);

    // Red

    /// Light red `(255, 182, 193)` — soft error indicator
    pub const RED_LIGHT: Rgb = Rgb(255, 182, 193);
    /// Standard red `(220, 20, 60)` — errors and critical alerts
    pub const RED: Rgb = Rgb(220, 20, 60);
    /// Dark red `(139, 0, 0)` — emphasis error
    pub const RED_DARK: Rgb = Rgb(139, 0, 0);

    // Orange

    /// Light orange `(255, 200, 124)` — soft attention indicator
    pub const ORANGE_LIGHT: Rgb = Rgb(255, 200, 124);
    /// Standard orange `(255, 140, 0)` — attention and highlights
    pub const ORANGE: Rgb = Rgb(255, 140, 0);
    /// Dark orange `(204, 85, 0)` — emphasis attention
    pub const ORANGE_DARK: Rgb = Rgb(204, 85, 0);

    // Cyan

    /// Light cyan `(175, 238, 238)` — soft info accent
    pub const CYAN_LIGHT: Rgb = Rgb(175, 238, 238);
    /// Standard cyan `(0, 206, 209)` — headings and accents
    pub const CYAN: Rgb = Rgb(0, 206, 209);
    /// Dark cyan `(0, 139, 139)` — emphasis info accent
    pub const CYAN_DARK: Rgb = Rgb(0, 139, 139);
}
