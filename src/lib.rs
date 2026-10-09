//! Commands accept caller-owned second-row views via `Command::second_row`.
//! Supply their searchable labels separately with `Command::search_terms`.
//! Search never renders custom views or extracts their text. Factories run
//! under the row's Leptos owner and can return arbitrary reactive content.
//!
//! ```ignore
//! use leptos::prelude::*;
//! use leptos_command_palette::Command;
//! Command::new("scene", "Scene", || {})
//!     .search_terms(["Exterior", "Active"])
//!     .second_row(|| view! { <div>"Application-owned content and layout"</div> });
//! ```
//! `data-command-palette-command` and `data-command-palette-second-row`
//! identify generic row containers for integration tests.

pub mod command;
pub mod component;
pub mod context;
pub mod shortcut;
pub mod theme;

pub use command::{Command, CommandId, CommandPalettePosition};
pub use component::{CommandPalette, CommandPaletteProvider};
pub use context::{use_command_palette, CommandPaletteContext, NavLevel};
pub use shortcut::{Modifier, Shortcut};
pub use theme::{
    CommandPaletteBackdropTheme, CommandPaletteEmptyTheme, CommandPaletteInputTheme,
    CommandPaletteItemTheme, CommandPaletteTheme,
};
