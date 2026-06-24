use leptos::prelude::*;
use leptos_command_palette::*;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

/// Shared app state available via context. Commands mutate this directly
/// without needing signals or callbacks passed through props.
#[derive(Clone, Copy)]
struct AppState {
    current_page: RwSignal<String>,
    format_count: RwSignal<u32>,
    saved: RwSignal<bool>,
    dark_mode: RwSignal<bool>,
}

impl AppState {
    fn new() -> Self {
        Self {
            current_page: RwSignal::new("home".into()),
            format_count: RwSignal::new(0),
            saved: RwSignal::new(false),
            dark_mode: RwSignal::new(true),
        }
    }
}

#[component]
fn App() -> impl IntoView {
    let state = AppState::new();
    provide_context(state);

    provide_context(CommandPaletteBackdropTheme {
        background: "rgba(0,0,0,0.6)".into(),
        ..Default::default()
    });
    provide_context(CommandPaletteTheme {
        background: "#1a1a1a".into(),
        color: "#eee".into(),
        border: "1px solid #333".into(),
        border_radius: "6px".into(),
        shadow: "0 8px 30px rgba(0,0,0,0.7)".into(),
        font_size: "13px".into(),
        padding: "6px".into(),
        max_height: "340px".into(),
        ..Default::default()
    });
    provide_context(CommandPaletteInputTheme {
        background: "#111111".into(),
        color: "#eee".into(),
        border: "1px solid #333".into(),
        border_radius: "4px".into(),
        placeholder_color: "#666".into(),
        font_size: "13px".into(),
        padding: "6px 10px".into(),
        margin_bottom: "6px".into(),
        ..Default::default()
    });
    provide_context(CommandPaletteItemTheme {
        padding: "5px 10px".into(),
        border_radius: "3px".into(),
        selected_background: "#2a2a2a".into(),
        selected_color: "#fff".into(),
        description_color: "#777".into(),
        description_font_size: "11px".into(),
        description_margin_top: "1px".into(),
        shortcut_font_size: "11px".into(),
        ..Default::default()
    });

    view! {
        <CommandPaletteProvider>
            <CommandPalette />
            <RegisterGlobalCommands />

            <div style="padding:40px">
                <h1 style="margin:0 0 8px 0;color:#eee;font-size:20px;font-weight:500">"Command Palette Demo"</h1>
                <p style="margin:0 0 24px 0;color:#777;font-size:13px">
                    "Press " <kbd style="background:#222;padding:2px 6px;border-radius:3px;border:1px solid #333;color:#bbb;font-size:12px">"Cmd+K"</kbd>
                    " (or Ctrl+K) to open the command palette"
                </p>

                <div style="display:flex;gap:8px;margin-bottom:24px">
                    <PageTab label="Home" page="home" />
                    <PageTab label="Editor" page="editor" />
                    <PageTab label="Settings" page="settings" />
                </div>

                <div style="background:#111111;border:1px solid #222;border-radius:8px;padding:24px;min-height:200px">
                    {move || match state.current_page.get().as_str() {
                        "editor" => view! { <EditorPage /> }.into_any(),
                        "settings" => view! { <SettingsPage /> }.into_any(),
                        _ => view! { <HomePage /> }.into_any(),
                    }}
                </div>
            </div>
        </CommandPaletteProvider>
    }
}

/// Registers commands that are always available. Each handler just
/// grabs AppState from context — no props needed.
#[component]
fn RegisterGlobalCommands() -> impl IntoView {
    let palette = use_command_palette();
    let state = use_context::<AppState>().unwrap();

    Effect::new(move |_| {
        palette.register_many(vec![
            Command::new("nav.home", "Go to Home", move || {
                state.current_page.set("home".into());
            })
                .group("Navigation")
                .shortcut(vec![Modifier::Alt], "1"),
            Command::new("nav.editor", "Go to Editor", move || {
                state.current_page.set("editor".into());
            })
                .group("Navigation")
                .shortcut(vec![Modifier::Alt], "2"),
            Command::new("nav.settings", "Go to Settings", move || {
                state.current_page.set("settings".into());
            })
                .group("Navigation")
                .shortcut(vec![Modifier::Alt], "3"),
            Command::new("app.reload", "Reload Application", || {
                web_sys::window().unwrap().location().reload().ok();
            })
                .description("Refresh the current page")
                .group("Application"),
            // Drill-down submenu: children are generated from live data each
            // time the branch is entered. Picking a scene runs its action and
            // closes; Esc / Backspace-on-empty / the breadcrumb go back up.
            Command::submenu("scenes", "Open Scene", move || {
                (1..=90)
                    .map(|n| {
                        Command::new(
                            format!("scene.{n}"),
                            format!("Scene {n}"),
                            move || {
                                state.current_page.set(format!("scene {n}"));
                            },
                        )
                        .group("Scenes")
                    })
                    .collect()
            })
                .description("Browse all scenes")
                .group("Navigation"),
        ]);
    });

    view! {}
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <h2 style="margin:0 0 12px 0;font-size:14px;font-weight:500;text-transform:uppercase;letter-spacing:1px;color:#bbb">"Home"</h2>
        <p style="color:#999;font-size:13px;line-height:1.6">"This is the home page. Open the command palette to see global navigation commands."</p>
        <p style="color:#666;font-size:13px;line-height:1.6;margin-top:8px">"Try switching to the Editor or Settings page to see context-specific commands appear."</p>
    }
}

/// Editor page — registers context-specific commands on mount, removes on cleanup.
/// Handlers reference AppState from context directly.
#[component]
fn EditorPage() -> impl IntoView {
    let palette = use_command_palette();
    let state = use_context::<AppState>().unwrap();

    Effect::new(move |_| {
        palette.register_many(vec![
            Command::new("editor.format", "Format Document", move || {
                state.format_count.update(|n| *n += 1);
            })
                .description("Auto-format the current document")
                .group("Editor")
                .shortcut(vec![Modifier::Shift, Modifier::Alt], "f"),
            Command::new("editor.save", "Save Document", move || {
                state.saved.set(true);
            })
                .description("Save the current file")
                .group("Editor")
                .shortcut(vec![Modifier::Main], "s"),
            Command::new("editor.find", "Find in Document", || {})
                .description("Search within the current file")
                .group("Editor")
                .shortcut(vec![Modifier::Main], "f"),
            Command::new("editor.replace", "Find and Replace", || {})
                .description("Search and replace text")
                .group("Editor")
                .shortcut(vec![Modifier::Main], "h"),
        ]);
    });

    on_cleanup(move || {
        palette.unregister_many(&[
            "editor.format",
            "editor.save",
            "editor.find",
            "editor.replace",
        ]);
    });

    view! {
        <h2 style="margin:0 0 12px 0;font-size:14px;font-weight:500;text-transform:uppercase;letter-spacing:1px;color:#bbb">"Editor"</h2>
        <p style="color:#999;font-size:13px;line-height:1.6">"Editor-specific commands are now available in the palette."</p>
        <div style="margin-top:16px;padding:12px;background:#1a1a1a;border:1px solid #222;border-radius:4px;font-family:monospace;font-size:13px;color:#bbb">
            <div>"Format count: " {move || state.format_count.get()}</div>
            <div style="margin-top:4px">"Saved: " {move || if state.saved.get() { "yes" } else { "no" }}</div>
        </div>
    }
}

/// Settings page — same pattern, commands reference shared state.
#[component]
fn SettingsPage() -> impl IntoView {
    let palette = use_command_palette();
    let state = use_context::<AppState>().unwrap();

    Effect::new(move |_| {
        palette.register_many(vec![
            Command::new("settings.toggle_dark", "Toggle Dark Mode", move || {
                state.dark_mode.update(|v| *v = !*v);
            })
                .description("Switch between dark and light theme")
                .group("Settings"),
            Command::new("settings.reset", "Reset All Settings", || {})
                .description("Restore default settings")
                .group("Settings"),
            Command::new("settings.export", "Export Settings", || {})
                .description("Download settings as JSON")
                .group("Settings"),
        ]);
    });

    on_cleanup(move || {
        palette.unregister_many(&[
            "settings.toggle_dark",
            "settings.reset",
            "settings.export",
        ]);
    });

    view! {
        <h2 style="margin:0 0 12px 0;font-size:14px;font-weight:500;text-transform:uppercase;letter-spacing:1px;color:#bbb">"Settings"</h2>
        <p style="color:#999;font-size:13px;line-height:1.6">"Settings-specific commands are now available in the palette."</p>
        <div style="margin-top:16px;padding:12px;background:#1a1a1a;border:1px solid #222;border-radius:4px;font-size:13px;color:#bbb">
            <div>"Dark mode: " {move || if state.dark_mode.get() { "on" } else { "off" }}</div>
        </div>
    }
}

#[component]
fn PageTab(label: &'static str, page: &'static str) -> impl IntoView {
    let state = use_context::<AppState>().unwrap();
    let page_str = page.to_string();
    let is_active = move || state.current_page.get() == page;
    view! {
        <button
            style=move || format!(
                "padding:6px 14px;border:1px solid {};border-radius:4px;cursor:pointer;font-size:13px;background:{};color:{}",
                if is_active() { "#333" } else { "#222" },
                if is_active() { "#1a1a1a" } else { "transparent" },
                if is_active() { "#eee" } else { "#777" },
            )
            on:click=move |_| state.current_page.set(page_str.clone())
        >
            {label}
        </button>
    }
}
