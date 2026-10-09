//! Badges default to inline contextual labels. Use
//! `CommandBadge::new("Ready", "blue").trailing()` for separate right-hand metadata.
//! Both placements remain searchable and update reactively with the command.
//! The renderer exposes `data-command-palette-command`, `data-command-palette-badges`
//! and `data-command-palette-badge` attributes for integration/accessibility tests.

pub mod command;
pub mod component;
pub mod context;
pub mod shortcut;
pub mod theme;

pub use command::{
    Command, CommandBadge, CommandBadgePlacement, CommandId, CommandPalettePosition,
};
pub use component::{CommandPalette, CommandPaletteProvider};
pub use context::{use_command_palette, CommandPaletteContext, NavLevel};
pub use shortcut::{Modifier, Shortcut};
pub use theme::{
    CommandPaletteBackdropTheme, CommandPaletteEmptyTheme, CommandPaletteInputTheme,
    CommandPaletteItemTheme, CommandPaletteTheme,
};
