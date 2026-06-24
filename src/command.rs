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
    /// The action to execute when this command is selected (no-op for branches).
    action: Arc<dyn Fn() + Send + Sync>,
    /// When present, this command is a *branch*: selecting it drills into a
    /// child list instead of executing. The closure is invoked at the moment
    /// the branch is entered, so it can snapshot live data.
    children: Option<Arc<dyn Fn() -> Vec<Command> + Send + Sync>>,
    /// When `true` (and this is a branch), the branch's children are surfaced
    /// directly in search results — see [`Command::searchable_children`].
    search_children: bool,
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
            children: None,
            search_children: false,
        }
    }

    /// Create a *submenu* (branch) command. Selecting it drills into the list
    /// produced by `children` rather than executing an action.
    ///
    /// `children` is invoked each time the branch is entered, so it can read
    /// live application state and return an up-to-date child list. Children may
    /// themselves be submenus, giving arbitrary-depth drill-down flows.
    ///
    /// ```ignore
    /// use leptos_command_palette::Command;
    ///
    /// Command::submenu("scenes", "Open Scene", || {
    ///     load_scenes()
    ///         .into_iter()
    ///         .map(|s| Command::new(format!("scene.{}", s.id), s.name, move || open(s.id)))
    ///         .collect()
    /// });
    /// ```
    pub fn submenu(
        id: impl Into<String>,
        name: impl Into<String>,
        children: impl Fn() -> Vec<Command> + Send + Sync + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            group: None,
            shortcut: None,
            action: Arc::new(|| {}),
            children: Some(Arc::new(children)),
            search_children: false,
        }
    }

    /// Turn this command into a *branch* by attaching a child-list producer.
    ///
    /// Useful for adding drill-down to a command built with [`Command::new`].
    /// Any existing action is preserved but ignored while the command is a
    /// branch (selecting it drills in rather than executing).
    pub fn children(
        mut self,
        children: impl Fn() -> Vec<Command> + Send + Sync + 'static,
    ) -> Self {
        self.children = Some(Arc::new(children));
        self
    }

    /// Opt this branch's children into top-level search.
    ///
    /// While the query is non-empty, matching children are surfaced inline
    /// alongside this command (shown with this command's name as context), so
    /// the user can jump straight to a sub-option without entering the submenu
    /// first — e.g. typing a scene name surfaces it directly instead of
    /// requiring "Open Scene" to be selected first. With an empty query the
    /// submenu still shows as a normal drill-in; this has no effect on a leaf.
    ///
    /// ```ignore
    /// use leptos_command_palette::Command;
    ///
    /// Command::submenu("scenes", "Open Scene", load_scene_commands)
    ///     .searchable_children();
    /// ```
    pub fn searchable_children(mut self) -> Self {
        self.search_children = true;
        self
    }

    /// Whether this branch's children are surfaced in top-level search
    /// (see [`Command::searchable_children`]).
    pub fn searches_children(&self) -> bool {
        self.search_children
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

    /// Whether this command drills into a submenu (rather than executing).
    pub fn is_branch(&self) -> bool {
        self.children.is_some()
    }

    /// Resolve this branch's child commands by invoking its producer closure.
    ///
    /// Returns `None` for leaf commands. Called when the branch is entered, so
    /// the returned list reflects live data at that moment.
    pub(crate) fn resolve_children(&self) -> Option<Vec<Command>> {
        self.children.as_ref().map(|f| f())
    }

    /// A clone of this child shown with `parent_label` as context, used when a
    /// child is promoted into top-level search. Keeps any existing description.
    fn promoted_under(&self, parent_label: &str) -> Command {
        let mut child = self.clone();
        if child.description.is_none() {
            child.description = Some(parent_label.to_string());
        }
        child
    }
}

/// Case-insensitive substring match of `q` against a command's name,
/// description, or group.
fn matches_query(cmd: &Command, q: &str) -> bool {
    cmd.name.to_lowercase().contains(q)
        || cmd
            .description
            .as_ref()
            .map(|d| d.to_lowercase().contains(q))
            .unwrap_or(false)
        || cmd
            .group
            .as_ref()
            .map(|g| g.to_lowercase().contains(q))
            .unwrap_or(false)
}

/// Filter `items` by `query` for display in the palette.
///
/// An empty query returns `items` unchanged (the menu, with branches shown as
/// drill-ins). Otherwise each matching item is included, and for any *searchable
/// branch* (see [`Command::searchable_children`]) its matching children are
/// surfaced inline too — promoted with the branch's name as context — so a
/// sub-option can be reached without entering the submenu first. Results are
/// de-duplicated by id (first occurrence wins) to keep row keys unique.
pub(crate) fn filter_commands(items: &[Command], query: &str) -> Vec<Command> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return items.to_vec();
    }
    let mut out: Vec<Command> = Vec::new();
    for cmd in items {
        if matches_query(cmd, &q) {
            out.push(cmd.clone());
        }
        if cmd.search_children {
            if let Some(children) = cmd.resolve_children() {
                for child in children {
                    if matches_query(&child, &q) {
                        out.push(child.promoted_under(&cmd.name));
                    }
                }
            }
        }
    }
    let mut seen = std::collections::HashSet::new();
    out.retain(|c| seen.insert(c.id.clone()));
    out
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
            .field("is_branch", &self.is_branch())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn new_command_is_a_leaf() {
        let cmd = Command::new("save", "Save", || {});
        assert!(!cmd.is_branch());
        assert!(cmd.resolve_children().is_none());
    }

    #[test]
    fn submenu_is_a_branch_and_resolves_children() {
        let cmd = Command::submenu("scenes", "Open Scene", || {
            vec![
                Command::new("a", "Scene A", || {}),
                Command::new("b", "Scene B", || {}),
            ]
        });
        assert!(cmd.is_branch());
        let children = cmd.resolve_children().expect("branch resolves children");
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].name, "Scene A");
    }

    #[test]
    fn children_builder_turns_a_leaf_into_a_branch() {
        let cmd = Command::new("more", "More", || {})
            .children(|| vec![Command::new("x", "X", || {})]);
        assert!(cmd.is_branch());
        assert_eq!(cmd.resolve_children().unwrap().len(), 1);
    }

    #[test]
    fn children_are_resolved_each_time_from_live_data() {
        // The producer reads shared state every time it runs, so the child list
        // reflects whatever is current at the moment the branch is entered.
        static COUNT: AtomicU32 = AtomicU32::new(0);
        let cmd = Command::submenu("dyn", "Dynamic", || {
            let n = COUNT.fetch_add(1, Ordering::SeqCst) + 1;
            (0..n)
                .map(|i| Command::new(format!("i{i}"), format!("Item {i}"), || {}))
                .collect()
        });
        assert_eq!(cmd.resolve_children().unwrap().len(), 1);
        assert_eq!(cmd.resolve_children().unwrap().len(), 2);
        assert_eq!(cmd.resolve_children().unwrap().len(), 3);
    }

    #[test]
    fn searchable_children_is_opt_in() {
        let leaf = Command::new("x", "X", || {});
        assert!(!leaf.searches_children());

        let plain_branch = Command::submenu("s", "Open Scene", Vec::new);
        assert!(!plain_branch.searches_children());

        let searchable = Command::submenu("s", "Open Scene", Vec::new).searchable_children();
        assert!(searchable.searches_children());
    }

    fn scene_branch(searchable: bool) -> Command {
        let b = Command::submenu("scenes", "Open Scene", || {
            vec![
                Command::new("scene.a", "Sunset", || {}),
                Command::new("scene.b", "Dawn", || {}),
            ]
        });
        if searchable {
            b.searchable_children()
        } else {
            b
        }
    }

    #[test]
    fn empty_query_returns_items_unchanged() {
        let items = vec![scene_branch(true), Command::new("save", "Save", || {})];
        let out = filter_commands(&items, "");
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].id, "scenes");
        // The branch is still a branch (drill-in), not flattened.
        assert!(out[0].is_branch());
    }

    #[test]
    fn searchable_branch_promotes_matching_children() {
        let items = vec![scene_branch(true), Command::new("save", "Save", || {})];
        // Typing a child's name surfaces it directly, without entering the menu.
        let out = filter_commands(&items, "sunset");
        let promoted = out
            .iter()
            .find(|c| c.id == "scene.a")
            .expect("matching child is promoted to the top level");
        // ...and carries the branch name as context.
        assert_eq!(promoted.description.as_deref(), Some("Open Scene"));
        // The non-matching sibling is not surfaced.
        assert!(out.iter().all(|c| c.id != "scene.b"));
    }

    #[test]
    fn non_searchable_branch_does_not_promote_children() {
        let items = vec![scene_branch(false)];
        let out = filter_commands(&items, "sunset");
        // The branch label doesn't match and children aren't opted in → nothing.
        assert!(out.iter().all(|c| c.id != "scene.a"));
    }

    #[test]
    fn branch_label_match_keeps_branch_and_can_coexist_with_promotions() {
        let items = vec![scene_branch(true)];
        // "scene" matches the branch label AND is a substring of neither child
        // name — so we get just the branch (still a drill-in).
        let out = filter_commands(&items, "open");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].id, "scenes");
        assert!(out[0].is_branch());
    }
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
