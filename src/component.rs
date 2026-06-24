use leptos::prelude::*;

use crate::command::{filter_commands, CommandPalettePosition};
use crate::context::{use_command_palette, CommandPaletteContext};
use crate::shortcut::{Modifier, Shortcut};
use crate::theme::*;

/// Provides the command palette context and renders children.
///
/// Place this near the root of your app. All descendants can then use
/// [`use_command_palette`] to register/unregister commands and open/close
/// the palette.
#[component]
pub fn CommandPaletteProvider(children: Children) -> impl IntoView {
    let ctx = CommandPaletteContext::new();
    provide_context(ctx.clone());

    let handle = window_event_listener(leptos::ev::keydown, move |ev| {
        let ev: web_sys::KeyboardEvent = ev.into();

        // Main+K toggles the palette (Cmd+K on Mac, Ctrl+K on Windows/Linux)
        let toggle_shortcut = Shortcut::new(vec![Modifier::Main], "k");
        if toggle_shortcut.matches(&ev) {
            ev.prevent_default();
            ctx.toggle();
            return;
        }

        // Escape goes up one submenu level, or closes the palette at the root
        if ev.key() == "Escape" {
            ctx.back_or_close();
            return;
        }

        // Don't match shortcuts while the palette is open (input has focus)
        if ctx.is_open().get_untracked() {
            return;
        }

        // Check registered command shortcuts
        let cmds = ctx.commands().get_untracked();
        for cmd in &cmds {
            if let Some(ref shortcut) = cmd.shortcut {
                if shortcut.matches(&ev) {
                    ev.prevent_default();
                    cmd.execute();
                    return;
                }
            }
        }
    });

    on_cleanup(move || {
        handle.remove();
    });

    children()
}

/// The command palette UI component.
///
/// Renders nothing when closed. When open, displays a searchable list of
/// all registered commands.
#[component]
pub fn CommandPalette(
    #[prop(optional, into)] position: Option<CommandPalettePosition>,
) -> impl IntoView {
    let ctx = use_command_palette();
    let is_open = ctx.is_open();
    let position = position.unwrap_or_default();

    let backdrop_theme = use_context::<CommandPaletteBackdropTheme>().unwrap_or_default();
    let panel_theme = use_context::<CommandPaletteTheme>().unwrap_or_default();
    let input_theme = use_context::<CommandPaletteInputTheme>().unwrap_or_default();
    let item_theme = use_context::<CommandPaletteItemTheme>().unwrap_or_default();
    let empty_theme = use_context::<CommandPaletteEmptyTheme>().unwrap_or_default();

    let (query, set_query) = signal(String::new());
    let (selected_id, set_selected_id) = signal(Option::<String>::None);
    let input_ref = NodeRef::<leptos::html::Input>::new();

    let position_css = position.to_css();

    // The commands visible at the current depth: root registrations when not in
    // a submenu, otherwise the snapshot captured when the current branch was
    // entered. Search filters this level only.
    let current_items = Memo::new(move |_| match ctx.nav_stack().get().last() {
        Some(level) => level.items.clone(),
        None => ctx.commands().get(),
    });

    // Filter the current level by the query. For searchable branches this also
    // surfaces matching children inline (promoted with the branch name as
    // context), so a sub-option can be reached without entering the submenu.
    let filtered_commands = Memo::new(move |_| filter_commands(&current_items.get(), &query.get()));

    let selected_index_in_list = move || {
        let cmds = filtered_commands.get();
        let sel = selected_id.get();
        match sel {
            Some(id) => cmds.iter().position(|c| c.id == id).unwrap_or(0),
            None => 0,
        }
    };

    let select_at_index = move |idx: usize| {
        let cmds = filtered_commands.get();
        if let Some(cmd) = cmds.get(idx) {
            set_selected_id.set(Some(cmd.id.clone()));
        }
    };

    Effect::new(move || {
        let _ = query.get();
        let cmds = filtered_commands.get();
        set_selected_id.set(cmds.first().map(|c| c.id.clone()));
    });

    Effect::new(move || {
        if is_open.get() {
            set_query.set(String::new());
            set_selected_id.set(None);
            request_animation_frame(move || {
                if let Some(input) = input_ref.get_untracked() {
                    let _ = input.focus();
                }
            });
        }
    });

    // Clear the search box whenever the drill-down depth changes (entering or
    // leaving a submenu), so each level starts with a fresh, unfiltered list.
    // Also refocus the input, since drilling in via mouse click moves focus off
    // it — without this the search box would be unusable after a click-drill.
    Effect::new(move || {
        let _depth = ctx.nav_stack().get().len();
        set_query.set(String::new());
        request_animation_frame(move || {
            if let Some(input) = input_ref.get_untracked() {
                let _ = input.focus();
            }
        });
    });

    // Build all style strings from theme values
    let backdrop_style = format!(
        "position:fixed;top:0;left:0;right:0;bottom:0;background:{bg};z-index:{z}",
        bg = backdrop_theme.background,
        z = backdrop_theme.z_index,
    );

    let panel_style = format!(
        "position:absolute;{pos};background:{bg};color:{color};border:{border};border-radius:{br};width:{w};max-height:{mh};box-shadow:{shadow};font-size:{fs};padding:{pad};overflow:hidden;display:flex;flex-direction:column",
        pos = position_css,
        bg = panel_theme.background,
        color = panel_theme.color,
        border = panel_theme.border,
        br = panel_theme.border_radius,
        w = panel_theme.width,
        mh = panel_theme.max_height,
        shadow = panel_theme.shadow,
        fs = panel_theme.font_size,
        pad = panel_theme.padding,
    );

    let input_style = format!(
        "width:100%;box-sizing:border-box;background:{bg};color:{color};border:{border};border-radius:{br};font-size:{fs};padding:{pad};outline:none;margin-bottom:{mb}",
        bg = input_theme.background,
        color = input_theme.color,
        border = input_theme.border,
        br = input_theme.border_radius,
        fs = input_theme.font_size,
        pad = input_theme.padding,
        mb = input_theme.margin_bottom,
    );

    let empty_style = format!(
        "padding:{pad};text-align:{ta};color:{color};opacity:{op};font-size:{fs}",
        pad = empty_theme.padding,
        ta = empty_theme.text_align,
        color = empty_theme.color,
        op = empty_theme.opacity,
        fs = empty_theme.font_size,
    );

    let input_ph_color = StoredValue::new(input_theme.placeholder_color.clone());
    let panel_color = StoredValue::new(panel_theme.color.clone());
    let item_pad = StoredValue::new(item_theme.padding.clone());
    let item_br = StoredValue::new(item_theme.border_radius.clone());
    let item_sel_bg = StoredValue::new(item_theme.selected_background.clone());
    let item_sel_color = StoredValue::new(item_theme.selected_color.clone());
    let item_desc_color = StoredValue::new(item_theme.description_color.clone());
    let item_desc_fs = StoredValue::new(item_theme.description_font_size.clone());
    let item_desc_mt = StoredValue::new(item_theme.description_margin_top.clone());
    let item_sc_color = StoredValue::new(item_theme.shortcut_color.clone());
    let item_sc_fs = StoredValue::new(item_theme.shortcut_font_size.clone());
    let item_sc_opacity = StoredValue::new(item_theme.shortcut_opacity.clone());
    let item_sc_ml = StoredValue::new(item_theme.shortcut_margin_left.clone());
    let backdrop_style = StoredValue::new(backdrop_style);
    let panel_style = StoredValue::new(panel_style);
    let input_style = StoredValue::new(input_style);
    let empty_style = StoredValue::new(empty_style);

    view! {
        <Show when=move || is_open.get()>
            <style>
                {format!(
                    ".command-palette-input::placeholder {{ color: {}; opacity: 1; }}",
                    input_ph_color.get_value()
                )}
            </style>
            <div
                style=move || backdrop_style.get_value()
                on:click=move |_| ctx.close()
            >
                <div
                    style=move || panel_style.get_value()
                    on:click=move |ev: web_sys::MouseEvent| ev.stop_propagation()
                >
                    <Show when=move || !ctx.nav_stack().get().is_empty()>
                        <div style="display:flex;align-items:center;flex-wrap:wrap;gap:4px;margin-bottom:8px;font-size:13px">
                            <span
                                style=move || format!(
                                    "cursor:pointer;color:{};opacity:0.7;padding-right:2px",
                                    panel_color.get_value(),
                                )
                                on:mousedown=move |ev: web_sys::MouseEvent| ev.prevent_default()
                                on:click=move |_| ctx.back()
                            >
                                "‹"
                            </span>
                            {move || {
                                let stack = ctx.nav_stack().get();
                                stack
                                    .into_iter()
                                    .enumerate()
                                    .map(|(i, level)| {
                                        let target = i + 1;
                                        let crumb_style = format!(
                                            "cursor:pointer;color:{}",
                                            panel_color.get_value(),
                                        );
                                        let sep_style = format!(
                                            "color:{};opacity:0.5",
                                            item_sc_color.get_value(),
                                        );
                                        view! {
                                            <span
                                                style=crumb_style
                                                on:mousedown=move |ev: web_sys::MouseEvent| ev.prevent_default()
                                                on:click=move |_| ctx.pop_to(target)
                                            >
                                                {level.label}
                                            </span>
                                            <span style=sep_style>"›"</span>
                                        }
                                    })
                                    .collect_view()
                            }}
                        </div>
                    </Show>
                    <input
                        class="command-palette-input"
                        style=move || input_style.get_value()
                        placeholder="Type a command..."
                        prop:value=move || query.get()
                        on:input=move |ev| {
                            set_query.set(event_target_value(&ev));
                        }
                        on:keydown=move |ev: web_sys::KeyboardEvent| {
                            let key = ev.key();
                            // Backspace on an empty query pops one submenu level.
                            // Handled before the empty-list guard so it works
                            // even when the current level has no items.
                            if key == "Backspace" && query.get().is_empty() && ctx.depth() > 0 {
                                ev.prevent_default();
                                ctx.back();
                                return;
                            }
                            let cmds = filtered_commands.get();
                            let count = cmds.len();
                            if count == 0 {
                                return;
                            }
                            let cur = selected_index_in_list();
                            match key.as_str() {
                                "ArrowDown" => {
                                    ev.prevent_default();
                                    select_at_index((cur + 1).min(count - 1));
                                }
                                "ArrowUp" => {
                                    ev.prevent_default();
                                    select_at_index(cur.saturating_sub(1));
                                }
                                "Enter" => {
                                    ev.prevent_default();
                                    if let Some(cmd) = cmds.get(cur) {
                                        // Branch: drill in. Leaf: run + close.
                                        if cmd.is_branch() {
                                            ctx.enter(cmd);
                                        } else {
                                            cmd.execute();
                                            ctx.close();
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        node_ref=input_ref
                    />
                    <div style="overflow-y:auto;flex:1">
                        <For
                            each=move || filtered_commands.get()
                            key=|cmd| cmd.id.clone()
                            children=move |cmd| {
                                let cmd_id_hover = cmd.id.clone();
                                let cmd_id_style = cmd.id.clone();
                                let is_branch = cmd.is_branch();
                                let chevron_style = format!(
                                    "color:{};opacity:{};flex-shrink:0;margin-left:{};font-size:16px;line-height:1",
                                    item_sc_color.get_value(),
                                    item_sc_opacity.get_value(),
                                    item_sc_ml.get_value(),
                                );
                                let desc_style = format!(
                                    "color:{};font-size:{};margin-top:{}",
                                    item_desc_color.get_value(),
                                    item_desc_fs.get_value(),
                                    item_desc_mt.get_value(),
                                );
                                let shortcut_style = format!(
                                    "color:{};font-size:{};opacity:{};flex-shrink:0;margin-left:{}",
                                    item_sc_color.get_value(),
                                    item_sc_fs.get_value(),
                                    item_sc_opacity.get_value(),
                                    item_sc_ml.get_value(),
                                );
                                let cmd_for_click = cmd.clone();
                                view! {
                                    <div
                                        style=move || {
                                            let is_sel = selected_id.get().as_deref() == Some(&cmd_id_style);
                                            let bg = if is_sel { item_sel_bg.get_value() } else { "transparent".into() };
                                            let color = if is_sel { item_sel_color.get_value() } else { "inherit".into() };
                                            format!(
                                                "padding:{};border-radius:{};background:{};color:{};cursor:pointer;display:flex;justify-content:space-between;align-items:center",
                                                item_pad.get_value(), item_br.get_value(), bg, color,
                                            )
                                        }
                                        // Keep the search input focused when a
                                        // row is clicked (so typing/Backspace
                                        // keep working after a mouse drill-in).
                                        on:mousedown=move |ev: web_sys::MouseEvent| ev.prevent_default()
                                        on:click=move |_| {
                                            // Branch: drill in. Leaf: run + close.
                                            if cmd_for_click.is_branch() {
                                                ctx.enter(&cmd_for_click);
                                            } else {
                                                cmd_for_click.execute();
                                                ctx.close();
                                            }
                                        }
                                        on:mouseenter=move |_| {
                                            set_selected_id.set(Some(cmd_id_hover.clone()));
                                        }
                                    >
                                        <div>
                                            <div>{cmd.name.clone()}</div>
                                            {cmd.description.as_ref().map(|d| {
                                                view! {
                                                    <div style={desc_style.clone()}>{d.clone()}</div>
                                                }
                                            })}
                                        </div>
                                        <div style="display:flex;align-items:center">
                                            {cmd.shortcut.as_ref().map(|s| {
                                                view! {
                                                    <div style={shortcut_style.clone()}>{s.to_string()}</div>
                                                }
                                            })}
                                            {is_branch.then(|| view! {
                                                <div style={chevron_style.clone()}>"›"</div>
                                            })}
                                        </div>
                                    </div>
                                }
                            }
                        />
                        <Show when=move || filtered_commands.get().is_empty()>
                            <div style=move || empty_style.get_value()>
                                "No commands found"
                            </div>
                        </Show>
                    </div>
                </div>
            </div>
        </Show>
    }
}
