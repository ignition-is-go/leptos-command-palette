use std::{collections::HashSet, sync::Arc};

use crate::shortcut::{Modifier, Shortcut};

/// A unique identifier for a command.
pub type CommandId = String;

/// A compact colored label displayed with a command.
///
/// Applications can use badges for contextual metadata such as tags, status,
/// environment, or ownership without encoding presentation into descriptions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandBadge {
    /// Text displayed inside the badge and included in palette search.
    pub label: String,
    /// CSS color used for the badge dot, text, border, and tinted background.
    pub color: String,
}

impl CommandBadge {
    pub fn new(label: impl Into<String>, color: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            color: color.into(),
        }
    }
}

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
    /// Compact contextual labels rendered below the command name.
    pub badges: Vec<CommandBadge>,
    /// The action to execute when this command is selected (no-op for branches).
    action: Arc<dyn Fn() + Send + Sync>,
    /// When present, this command is a *branch*: selecting it drills into a
    /// child list instead of executing. The closure is invoked at the moment
    /// the branch is entered, so it can snapshot live data.
    children: Option<Arc<dyn Fn() -> Vec<Command> + Send + Sync>>,
    /// When `true` (and this is a branch), the branch's children are surfaced
    /// directly in search results — see [`Command::searchable_children`].
    search_children: bool,
    /// Parent submenu shown as the sticky section heading when this command is
    /// promoted into search results.
    search_parent: Option<String>,
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
            badges: Vec::new(),
            action: Arc::new(action),
            children: None,
            search_children: false,
            search_parent: None,
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
            badges: Vec::new(),
            action: Arc::new(|| {}),
            children: Some(Arc::new(children)),
            search_children: false,
            search_parent: None,
        }
    }

    /// Turn this command into a *branch* by attaching a child-list producer.
    ///
    /// Useful for adding drill-down to a command built with [`Command::new`].
    /// Any existing action is preserved but ignored while the command is a
    /// branch (selecting it drills in rather than executing).
    pub fn children(mut self, children: impl Fn() -> Vec<Command> + Send + Sync + 'static) -> Self {
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

    /// Add one contextual badge to this command.
    pub fn badge(mut self, badge: CommandBadge) -> Self {
        self.badges.push(badge);
        self
    }

    /// Add contextual badges to this command in display order.
    pub fn badges(mut self, badges: impl IntoIterator<Item = CommandBadge>) -> Self {
        self.badges.extend(badges);
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

    /// A clone of this child grouped beneath `parent_label`, used when a child
    /// is promoted into top-level search. Keeps any existing description.
    fn promoted_under(&self, parent_label: &str) -> Command {
        let mut child = self.clone();
        child.search_parent = Some(parent_label.to_string());
        child
    }

    pub(crate) fn search_parent(&self) -> Option<&str> {
        self.search_parent.as_deref()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SearchScore {
    order_penalty: usize,
    boundary_penalty: usize,
    field_penalty: usize,
    span: usize,
    start: usize,
}

struct SearchField {
    text: String,
    field_penalty: usize,
    is_parent: bool,
}

#[derive(Clone, Copy, Debug)]
struct SearchOccurrence {
    start: usize,
    end: usize,
    boundary_penalty: usize,
    field_penalty: usize,
    is_parent: bool,
}

#[derive(Clone, Copy, Debug)]
struct OrderedState {
    first_start: usize,
    end: usize,
    boundary_penalty: usize,
    field_penalty: usize,
    has_non_parent: bool,
}

fn query_terms(query: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    query
        .split_whitespace()
        .map(str::to_lowercase)
        .filter(|term| seen.insert(term.clone()))
        .collect()
}

fn search_fields(cmd: &Command) -> Vec<SearchField> {
    let mut fields = Vec::new();
    if let Some(parent) = cmd.search_parent() {
        fields.push(SearchField {
            text: parent.to_lowercase(),
            field_penalty: 1,
            is_parent: true,
        });
    }
    fields.push(SearchField {
        text: cmd.name.to_lowercase(),
        field_penalty: 0,
        is_parent: false,
    });
    if let Some(description) = &cmd.description {
        fields.push(SearchField {
            text: description.to_lowercase(),
            field_penalty: 2,
            is_parent: false,
        });
    }
    for badge in &cmd.badges {
        fields.push(SearchField {
            text: badge.label.to_lowercase(),
            field_penalty: 1,
            is_parent: false,
        });
    }
    if let Some(group) = &cmd.group {
        fields.push(SearchField {
            text: group.to_lowercase(),
            field_penalty: 2,
            is_parent: false,
        });
    }
    fields
}

fn boundary_penalty(text: &str, start: usize, end: usize) -> usize {
    let starts_word = text
        .get(..start)
        .and_then(|prefix| prefix.chars().next_back())
        .map(|character| !character.is_alphanumeric())
        .unwrap_or(true);
    let ends_word = text
        .get(end..)
        .and_then(|suffix| suffix.chars().next())
        .map(|character| !character.is_alphanumeric())
        .unwrap_or(true);
    usize::from(!starts_word) + usize::from(!ends_word)
}

fn term_occurrences(fields: &[SearchField], term: &str) -> Vec<SearchOccurrence> {
    let mut occurrences = Vec::new();
    let mut field_offset = 0;
    for field in fields {
        for (local_start, matched) in field.text.match_indices(term) {
            let local_end = local_start + matched.len();
            occurrences.push(SearchOccurrence {
                start: field_offset + local_start,
                end: field_offset + local_end,
                boundary_penalty: boundary_penalty(&field.text, local_start, local_end),
                field_penalty: field.field_penalty,
                is_parent: field.is_parent,
            });
        }
        field_offset += field.text.len() + 1;
    }
    occurrences
}

fn ordered_state_key(state: &OrderedState) -> (usize, usize, usize, usize) {
    (
        state.boundary_penalty,
        state.field_penalty,
        state.end.saturating_sub(state.first_start),
        state.first_start,
    )
}

fn keep_better_state(slot: &mut Option<OrderedState>, candidate: OrderedState) {
    if slot
        .as_ref()
        .is_none_or(|existing| ordered_state_key(&candidate) < ordered_state_key(existing))
    {
        *slot = Some(candidate);
    }
}

fn best_ordered_state(
    matches_by_term: &[Vec<SearchOccurrence>],
    require_non_parent: bool,
) -> Option<OrderedState> {
    let first_matches = matches_by_term.first()?;
    let mut states = vec![[None; 2]; first_matches.len()];
    for (index, occurrence) in first_matches.iter().enumerate() {
        let state = OrderedState {
            first_start: occurrence.start,
            end: occurrence.end,
            boundary_penalty: occurrence.boundary_penalty,
            field_penalty: occurrence.field_penalty,
            has_non_parent: !occurrence.is_parent,
        };
        states[index][usize::from(state.has_non_parent)] = Some(state);
    }

    for term_index in 1..matches_by_term.len() {
        let previous_matches = &matches_by_term[term_index - 1];
        let current_matches = &matches_by_term[term_index];
        let mut next_states = vec![[None; 2]; current_matches.len()];
        for (current_index, current) in current_matches.iter().enumerate() {
            for (previous_index, previous) in previous_matches.iter().enumerate() {
                if previous.end > current.start {
                    continue;
                }
                for previous_state in states[previous_index].iter().flatten() {
                    let candidate = OrderedState {
                        first_start: previous_state.first_start,
                        end: current.end,
                        boundary_penalty: previous_state.boundary_penalty
                            + current.boundary_penalty,
                        field_penalty: previous_state.field_penalty + current.field_penalty,
                        has_non_parent: previous_state.has_non_parent || !current.is_parent,
                    };
                    keep_better_state(
                        &mut next_states[current_index][usize::from(candidate.has_non_parent)],
                        candidate,
                    );
                }
            }
        }
        states = next_states;
    }

    states
        .iter()
        .flat_map(|state| {
            if require_non_parent {
                state[1].into_iter().collect::<Vec<_>>()
            } else {
                state.iter().flatten().copied().collect()
            }
        })
        .min_by_key(ordered_state_key)
}

fn search_score(cmd: &Command, terms: &[String]) -> Option<SearchScore> {
    let fields = search_fields(cmd);
    let matches_by_term = terms
        .iter()
        .map(|term| term_occurrences(&fields, term))
        .collect::<Vec<_>>();
    if matches_by_term.iter().any(Vec::is_empty) {
        return None;
    }

    let require_non_parent = cmd.search_parent().is_some();
    if require_non_parent
        && !matches_by_term
            .iter()
            .flatten()
            .any(|occurrence| !occurrence.is_parent)
    {
        // Matching only the submenu title should keep the submenu itself as the
        // result instead of expanding every one of its children.
        return None;
    }

    if let Some(state) = best_ordered_state(&matches_by_term, require_non_parent) {
        return Some(SearchScore {
            order_penalty: 0,
            boundary_penalty: state.boundary_penalty,
            field_penalty: state.field_penalty,
            span: state.end.saturating_sub(state.first_start),
            start: state.first_start,
        });
    }

    let mut chosen = matches_by_term
        .iter()
        .map(|matches| {
            matches
                .iter()
                .min_by_key(|occurrence| {
                    (
                        occurrence.boundary_penalty,
                        occurrence.field_penalty,
                        occurrence.start,
                    )
                })
                .copied()
        })
        .collect::<Option<Vec<_>>>()?;
    if require_non_parent && chosen.iter().all(|occurrence| occurrence.is_parent) {
        let (term_index, replacement) = matches_by_term
            .iter()
            .enumerate()
            .flat_map(|(term_index, matches)| {
                matches
                    .iter()
                    .filter(|occurrence| !occurrence.is_parent)
                    .map(move |occurrence| (term_index, *occurrence))
            })
            .min_by_key(|(_, occurrence)| {
                (
                    occurrence.boundary_penalty,
                    occurrence.field_penalty,
                    occurrence.start,
                )
            })?;
        chosen[term_index] = replacement;
    }

    let mut inversions = 0;
    for left in 0..chosen.len() {
        for right in (left + 1)..chosen.len() {
            inversions += usize::from(chosen[left].start > chosen[right].start);
        }
    }
    let start = chosen.iter().map(|occurrence| occurrence.start).min()?;
    let end = chosen.iter().map(|occurrence| occurrence.end).max()?;
    Some(SearchScore {
        order_penalty: 1 + inversions,
        boundary_penalty: chosen
            .iter()
            .map(|occurrence| occurrence.boundary_penalty)
            .sum(),
        field_penalty: chosen
            .iter()
            .map(|occurrence| occurrence.field_penalty)
            .sum(),
        span: end.saturating_sub(start),
        start,
    })
}

/// Filter `items` by `query` for display in the palette.
///
/// An empty query returns `items` unchanged (the menu, with branches shown as
/// drill-ins). Otherwise every whitespace-separated term must match somewhere
/// in a command's name, description, group, badges, or promoted submenu parent.
/// Matches are ranked by query order, word boundaries, field relevance, and
/// proximity. Searchable branch children are surfaced inline and results are
/// de-duplicated by id after ranking.
pub(crate) fn filter_commands(items: &[Command], query: &str) -> Vec<Command> {
    let terms = query_terms(query);
    if terms.is_empty() {
        return items.to_vec();
    }
    let mut ranked: Vec<(SearchScore, usize, Command)> = Vec::new();
    let mut ordinal = 0;
    for cmd in items {
        if let Some(score) = search_score(cmd, &terms) {
            ranked.push((score, ordinal, cmd.clone()));
            ordinal += 1;
        }
        if cmd.search_children {
            if let Some(children) = cmd.resolve_children() {
                for child in children {
                    let child = child.promoted_under(&cmd.name);
                    if let Some(score) = search_score(&child, &terms) {
                        ranked.push((score, ordinal, child));
                        ordinal += 1;
                    }
                }
            }
        }
    }
    ranked.sort_by_key(|(score, order, _)| (*score, *order));
    let mut seen = HashSet::new();
    ranked
        .into_iter()
        .filter_map(|(_, _, command)| seen.insert(command.id.clone()).then_some(command))
        .collect()
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
            .field("badges", &self.badges)
            .field("is_branch", &self.is_branch())
            .field("search_parent", &self.search_parent)
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
        assert!(cmd.badges.is_empty());
    }

    #[test]
    fn badge_builders_preserve_display_order() {
        let cmd = Command::new("scene", "Scene", || {})
            .badge(CommandBadge::new("Active", "green"))
            .badges([
                CommandBadge::new("Exterior", "orange"),
                CommandBadge::new("Night", "blue"),
            ]);
        assert_eq!(
            cmd.badges
                .iter()
                .map(|badge| badge.label.as_str())
                .collect::<Vec<_>>(),
            ["Active", "Exterior", "Night"]
        );
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
        let cmd =
            Command::new("more", "More", || {}).children(|| vec![Command::new("x", "X", || {})]);
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
                Command::new("scene.a", "Sunset", || {})
                    .badge(CommandBadge::new("Exterior", "orange")),
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
        // ...and carries the branch name as its search-result section.
        assert_eq!(promoted.search_parent(), Some("Open Scene"));
        assert_eq!(promoted.description, None);
        // The non-matching sibling is not surfaced.
        assert!(out.iter().all(|c| c.id != "scene.b"));
    }

    #[test]
    fn searchable_branch_promotes_children_matching_badges() {
        let items = vec![scene_branch(true)];
        let out = filter_commands(&items, "exterior");
        let promoted = out
            .iter()
            .find(|command| command.id == "scene.a")
            .expect("a matching badge promotes its command");
        assert_eq!(promoted.badges[0].label, "Exterior");
    }

    #[test]
    fn search_terms_intersect_across_names_and_badges() {
        let items = vec![
            Command::new("scene.uds", "The UDS", || {}).badges([
                CommandBadge::new("_CORE", "lime"),
                CommandBadge::new("Active", "green"),
            ]),
            Command::new("scene.other", "Other UDS", || {})
                .badge(CommandBadge::new("Archive", "gray")),
        ];

        let out = filter_commands(&items, "uds core active");

        assert_eq!(out.len(), 1);
        assert_eq!(out[0].id, "scene.uds");
    }

    #[test]
    fn every_search_term_is_required() {
        let items =
            vec![Command::new("scene.uds", "The UDS", || {})
                .badge(CommandBadge::new("Core", "lime"))];

        assert!(filter_commands(&items, "uds missing").is_empty());
    }

    #[test]
    fn matches_in_query_order_rank_first() {
        let items = vec![
            Command::new("reversed", "Beta Alpha", || {}),
            Command::new("ordered", "Alpha Beta", || {}),
        ];

        let out = filter_commands(&items, "alpha beta");

        assert_eq!(
            out.iter()
                .map(|command| command.id.as_str())
                .collect::<Vec<_>>(),
            ["ordered", "reversed"]
        );
    }

    #[test]
    fn closer_matches_rank_first() {
        let items = vec![
            Command::new("far", "Alpha Something Far Away Beta", || {}),
            Command::new("close", "Alpha Beta", || {}),
        ];

        let out = filter_commands(&items, "alpha beta");

        assert_eq!(
            out.iter()
                .map(|command| command.id.as_str())
                .collect::<Vec<_>>(),
            ["close", "far"]
        );
    }

    #[test]
    fn word_boundary_matches_rank_before_embedded_substrings() {
        let items = vec![
            Command::new("embedded", "Xalpha Beta", || {}),
            Command::new("boundary", "Alpha Beta", || {}),
        ];

        let out = filter_commands(&items, "alpha beta");

        assert_eq!(
            out.iter()
                .map(|command| command.id.as_str())
                .collect::<Vec<_>>(),
            ["boundary", "embedded"]
        );
    }

    #[test]
    fn promoted_parent_can_supply_a_search_term() {
        let items = vec![scene_branch(true)];

        let out = filter_commands(&items, "open sunset");

        assert_eq!(out.len(), 1);
        assert_eq!(out[0].id, "scene.a");
        assert_eq!(out[0].search_parent(), Some("Open Scene"));
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
