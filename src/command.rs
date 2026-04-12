use std::sync::Arc;

use crate::shortcut::{Modifier, Shortcut};

/// A unique identifier for a command.
pub type CommandId = String;

/// A command that can appear in the palette.
#[derive(Clone)]
pub struct Command {
    /// Unique identifier for this command.
    pub id: CommandId,
    /// Display name shown in the palette.
    pub name: String,
    /// Optional description shown below the name.
    pub description: Option<String>,
    /// Optional group/category for visual grouping.
    pub group: Option<String>,
    /// Optional keyboard shortcut — both the keybinding and the display hint.
    pub shortcut: Option<Shortcut>,
    /// The action to execute when this command is selected.
    action: Arc<dyn Fn() + Send + Sync>,
}

impl Command {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        action: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            group: None,
            shortcut: None,
            action: Arc::new(action),
        }
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }

    /// Set the keyboard shortcut for this command.
    ///
    /// ```ignore
    /// use leptos_command_palette::{Command, Modifier};
    ///
    /// Command::new("save", "Save", || {})
    ///     .shortcut(vec![Modifier::Cmd], "s");
    ///
    /// Command::new("format", "Format", || {})
    ///     .shortcut(vec![Modifier::Shift, Modifier::Alt], "f");
    /// ```
    pub fn shortcut(mut self, modifiers: Vec<Modifier>, key: impl Into<String>) -> Self {
        self.shortcut = Some(Shortcut::new(modifiers, key));
        self
    }

    pub fn execute(&self) {
        (self.action)();
    }
}

impl PartialEq for Command {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl std::fmt::Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("description", &self.description)
            .field("group", &self.group)
            .field("shortcut", &self.shortcut)
            .finish()
    }
}

/// Position of the command palette on screen.
#[derive(Clone, Debug, Default)]
pub enum CommandPalettePosition {
    /// Centered horizontally at the top of the window (default).
    #[default]
    TopCenter,
    /// Centered both horizontally and vertically.
    Center,
    /// Custom CSS positioning. Values are applied directly as CSS properties.
    Custom {
        top: Option<String>,
        right: Option<String>,
        bottom: Option<String>,
        left: Option<String>,
        transform: Option<String>,
    },
}

impl CommandPalettePosition {
    pub(crate) fn to_css(&self) -> String {
        match self {
            CommandPalettePosition::TopCenter => {
                "top:20%;left:50%;transform:translateX(-50%)".into()
            }
            CommandPalettePosition::Center => {
                "top:50%;left:50%;transform:translate(-50%,-50%)".into()
            }
            CommandPalettePosition::Custom {
                top,
                right,
                bottom,
                left,
                transform,
            } => {
                let mut parts = Vec::new();
                if let Some(v) = top {
                    parts.push(format!("top:{v}"));
                }
                if let Some(v) = right {
                    parts.push(format!("right:{v}"));
                }
                if let Some(v) = bottom {
                    parts.push(format!("bottom:{v}"));
                }
                if let Some(v) = left {
                    parts.push(format!("left:{v}"));
                }
                if let Some(v) = transform {
                    parts.push(format!("transform:{v}"));
                }
                parts.join(";")
            }
        }
    }
}
