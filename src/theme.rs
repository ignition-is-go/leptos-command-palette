/// Theme for the command palette overlay backdrop.
#[derive(Clone, Debug)]
pub struct CommandPaletteBackdropTheme {
    /// Background color of the overlay behind the palette.
    pub background: String,
    /// Stacking order of the overlay. Should be high enough to sit above the
    /// host app's positioned content, since the palette is an app-modal overlay.
    pub z_index: u32,
}

impl Default for CommandPaletteBackdropTheme {
    fn default() -> Self {
        Self {
            background: "rgba(0,0,0,0.5)".into(),
            z_index: 9999,
        }
    }
}

/// Theme for the command palette panel.
#[derive(Clone, Debug)]
pub struct CommandPaletteTheme {
    /// Background color of the palette.
    pub background: String,
    /// Text color.
    pub color: String,
    /// Border (CSS shorthand).
    pub border: String,
    /// Border radius.
    pub border_radius: String,
    /// Width of the palette (e.g. "500px").
    pub width: String,
    /// Max height before scrolling.
    pub max_height: String,
    /// Box shadow.
    pub shadow: String,
    /// Font size.
    pub font_size: String,
    /// Padding inside the palette container.
    pub padding: String,
}

impl Default for CommandPaletteTheme {
    fn default() -> Self {
        Self {
            background: "#1e1e1e".into(),
            color: "#cccccc".into(),
            border: "1px solid #3c3c3c".into(),
            border_radius: "8px".into(),
            width: "500px".into(),
            max_height: "400px".into(),
            shadow: "0 8px 30px rgba(0,0,0,0.5)".into(),
            font_size: "14px".into(),
            padding: "8px".into(),
        }
    }
}

/// Theme for the search input inside the palette.
#[derive(Clone, Debug)]
pub struct CommandPaletteInputTheme {
    /// Background color of the input.
    pub background: String,
    /// Text color of the input.
    pub color: String,
    /// Border.
    pub border: String,
    /// Border radius.
    pub border_radius: String,
    /// Font size.
    pub font_size: String,
    /// Padding inside the input.
    pub padding: String,
    /// Placeholder text color.
    pub placeholder_color: String,
    /// Margin below the input (space between input and results list).
    pub margin_bottom: String,
}

impl Default for CommandPaletteInputTheme {
    fn default() -> Self {
        Self {
            background: "#2a2a2a".into(),
            color: "#cccccc".into(),
            border: "1px solid #3c3c3c".into(),
            border_radius: "4px".into(),
            font_size: "14px".into(),
            padding: "8px 12px".into(),
            placeholder_color: "#666666".into(),
            margin_bottom: "8px".into(),
        }
    }
}

/// Theme for individual command items in the list.
#[derive(Clone, Debug)]
pub struct CommandPaletteItemTheme {
    /// Padding for each item.
    pub padding: String,
    /// Border radius for each item.
    pub border_radius: String,
    /// Background color when selected/highlighted.
    pub selected_background: String,
    /// Text color when selected.
    pub selected_color: String,
    /// Color for the command description text.
    pub description_color: String,
    /// Font size for the description.
    pub description_font_size: String,
    /// Margin above the description line.
    pub description_margin_top: String,
    /// Color for the shortcut hint text.
    pub shortcut_color: String,
    /// Font size for the shortcut hint.
    pub shortcut_font_size: String,
    /// Opacity for the shortcut hint.
    pub shortcut_opacity: String,
    /// Left margin for the shortcut hint (space between name and shortcut).
    pub shortcut_margin_left: String,
}

impl Default for CommandPaletteItemTheme {
    fn default() -> Self {
        Self {
            padding: "8px 12px".into(),
            border_radius: "4px".into(),
            selected_background: "#094771".into(),
            selected_color: "#ffffff".into(),
            description_color: "#888888".into(),
            description_font_size: "12px".into(),
            description_margin_top: "2px".into(),
            shortcut_color: "#888888".into(),
            shortcut_font_size: "12px".into(),
            shortcut_opacity: "0.7".into(),
            shortcut_margin_left: "12px".into(),
        }
    }
}

/// Theme for the empty state shown when no commands match the query.
#[derive(Clone, Debug)]
pub struct CommandPaletteEmptyTheme {
    /// Padding around the empty message.
    pub padding: String,
    /// Text alignment.
    pub text_align: String,
    /// Text color.
    pub color: String,
    /// Text opacity.
    pub opacity: String,
    /// Font size.
    pub font_size: String,
}

impl Default for CommandPaletteEmptyTheme {
    fn default() -> Self {
        Self {
            padding: "12px".into(),
            text_align: "center".into(),
            color: "inherit".into(),
            opacity: "0.5".into(),
            font_size: "inherit".into(),
        }
    }
}
