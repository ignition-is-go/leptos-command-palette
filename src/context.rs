use leptos::prelude::*;

use crate::command::Command;

/// One level of the submenu navigation stack.
///
/// Created when a branch command is entered. `items` is the snapshot of child
/// commands produced at entry time; `label` is the branch's name, shown in the
/// breadcrumb.
#[derive(Clone)]
pub struct NavLevel {
    /// Display label for this level (the entered branch's name).
    pub label: String,
    /// The child commands resolved when this level was entered.
    pub items: Vec<Command>,
}

/// Reactive context for managing command palette state.
///
/// Provided to the component tree by [`CommandPaletteProvider`]. Use
/// [`use_command_palette`] to access it from any descendant component.
#[derive(Clone, Copy)]
pub struct CommandPaletteContext {
    /// All currently registered commands (the root level).
    commands: RwSignal<Vec<Command>>,
    /// Whether the palette is currently open.
    open: RwSignal<bool>,
    /// Submenu navigation stack. Empty means we're at the root level; each
    /// entry is a level the user has drilled into.
    nav_stack: RwSignal<Vec<NavLevel>>,
}

impl CommandPaletteContext {
    pub fn new() -> Self {
        Self {
            commands: RwSignal::new(Vec::new()),
            open: RwSignal::new(false),
            nav_stack: RwSignal::new(Vec::new()),
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

    /// Close the command palette. Resets submenu navigation back to the root,
    /// so reopening always starts at the top level.
    pub fn close(&self) {
        self.open.set(false);
        self.nav_stack.set(Vec::new());
    }

    /// Toggle the command palette open/closed.
    pub fn toggle(&self) {
        if self.open.get_untracked() {
            self.close();
        } else {
            self.open.set(true);
        }
    }

    /// Whether the palette is currently open.
    pub fn is_open(&self) -> RwSignal<bool> {
        self.open
    }

    /// The reactive submenu navigation stack. Empty at the root level.
    pub fn nav_stack(&self) -> RwSignal<Vec<NavLevel>> {
        self.nav_stack
    }

    /// Current drill-down depth (0 at the root level).
    pub fn depth(&self) -> usize {
        self.nav_stack.get().len()
    }

    /// Enter a branch command: resolve its children and push a new level.
    ///
    /// No-op for leaf commands. The child list is snapshotted now, so it
    /// reflects live data at the moment of entry.
    pub fn enter(&self, command: &Command) {
        if let Some(items) = command.resolve_children() {
            let level = NavLevel {
                label: command.name.clone(),
                items,
            };
            self.nav_stack.update(|s| s.push(level));
        }
    }

    /// Pop one level off the navigation stack (go up one menu). No-op at root.
    pub fn back(&self) {
        self.nav_stack.update(|s| {
            s.pop();
        });
    }

    /// Truncate the navigation stack to `depth` levels (used by breadcrumb
    /// clicks). `pop_to(0)` returns to the root level.
    pub fn pop_to(&self, depth: usize) {
        self.nav_stack.update(|s| s.truncate(depth));
    }

    /// Go up one level if inside a submenu, otherwise close the palette.
    /// Wired to the Escape key.
    pub fn back_or_close(&self) {
        if self.depth() > 0 {
            self.back();
        } else {
            self.close();
        }
    }
}

/// Retrieve the [`CommandPaletteContext`] from Leptos context.
///
/// Panics if called outside a [`CommandPaletteProvider`].
pub fn use_command_palette() -> CommandPaletteContext {
    use_context::<CommandPaletteContext>()
        .expect("use_command_palette must be called within a CommandPaletteProvider")
}
