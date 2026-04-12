use leptos::prelude::*;

use crate::command::Command;

/// Reactive context for managing command palette state.
///
/// Provided to the component tree by [`CommandPaletteProvider`]. Use
/// [`use_command_palette`] to access it from any descendant component.
#[derive(Clone, Copy)]
pub struct CommandPaletteContext {
    /// All currently registered commands.
    commands: RwSignal<Vec<Command>>,
    /// Whether the palette is currently open.
    open: RwSignal<bool>,
}

impl CommandPaletteContext {
    pub fn new() -> Self {
        Self {
            commands: RwSignal::new(Vec::new()),
            open: RwSignal::new(false),
        }
    }

    /// Register a command. If a command with the same ID exists, it is replaced.
    pub fn register(&self, command: Command) {
        self.commands.update(|cmds| {
            cmds.retain(|c| c.id != command.id);
            cmds.push(command);
        });
    }

    /// Register multiple commands at once.
    pub fn register_many(&self, new_commands: Vec<Command>) {
        self.commands.update(|cmds| {
            for command in new_commands {
                cmds.retain(|c| c.id != command.id);
                cmds.push(command);
            }
        });
    }

    /// Remove a command by ID.
    pub fn unregister(&self, id: &str) {
        self.commands.update(|cmds| {
            cmds.retain(|c| c.id != id);
        });
    }

    /// Remove multiple commands by ID.
    pub fn unregister_many(&self, ids: &[&str]) {
        self.commands.update(|cmds| {
            cmds.retain(|c| !ids.contains(&c.id.as_str()));
        });
    }

    /// Get the reactive signal of all registered commands.
    pub fn commands(&self) -> RwSignal<Vec<Command>> {
        self.commands
    }

    /// Open the command palette.
    pub fn open(&self) {
        self.open.set(true);
    }

    /// Close the command palette.
    pub fn close(&self) {
        self.open.set(false);
    }

    /// Toggle the command palette open/closed.
    pub fn toggle(&self) {
        self.open.update(|v| *v = !*v);
    }

    /// Whether the palette is currently open.
    pub fn is_open(&self) -> RwSignal<bool> {
        self.open
    }
}

/// Retrieve the [`CommandPaletteContext`] from Leptos context.
///
/// Panics if called outside a [`CommandPaletteProvider`].
pub fn use_command_palette() -> CommandPaletteContext {
    use_context::<CommandPaletteContext>()
        .expect("use_command_palette must be called within a CommandPaletteProvider")
}
