/// A keyboard modifier key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Modifier {
    /// The primary system modifier: Cmd on Mac, Ctrl on Windows/Linux.
    Main,
    /// Alt / Option key.
    Alt,
    /// Shift key.
    Shift,
}

fn is_mac() -> bool {
    web_sys::window()
        .map(|w| {
            w.navigator()
                .platform()
                .unwrap_or_default()
                .contains("Mac")
        })
        .unwrap_or(false)
}

impl Modifier {
    /// Display name for the current platform.
    pub fn display_name(self) -> &'static str {
        match self {
            Modifier::Main => {
                if is_mac() { "Cmd" } else { "Ctrl" }
            }
            Modifier::Alt => {
                if is_mac() { "Opt" } else { "Alt" }
            }
            Modifier::Shift => "Shift",
        }
    }
}

impl std::fmt::Display for Modifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// A keyboard shortcut: a set of modifiers plus a key.
#[derive(Clone, Debug, PartialEq)]
pub struct Shortcut {
    pub modifiers: Vec<Modifier>,
    pub key: String,
}

impl Shortcut {
    pub fn new(modifiers: Vec<Modifier>, key: impl Into<String>) -> Self {
        Self {
            modifiers,
            key: key.into(),
        }
    }

    /// Convert the user-provided key to the expected `KeyboardEvent.code` value.
    ///
    /// - Single letter `"s"` → `"KeyS"`
    /// - Single digit `"2"` → `"Digit2"`
    /// - Other keys passed through as-is (e.g. `"Escape"`, `"ArrowUp"`)
    fn key_to_code(key: &str) -> String {
        if key.len() == 1 {
            let ch = key.chars().next().unwrap();
            if ch.is_ascii_alphabetic() {
                format!("Key{}", ch.to_ascii_uppercase())
            } else if ch.is_ascii_digit() {
                format!("Digit{ch}")
            } else {
                key.to_string()
            }
        } else {
            key.to_string()
        }
    }

    /// Check if this shortcut matches a KeyboardEvent.
    ///
    /// `Modifier::Main` matches meta (Cmd) on Mac, ctrl on Windows/Linux.
    /// Uses `ev.code()` (physical key) rather than `ev.key()` (produced character)
    /// so that modifier combos like Alt+2 work regardless of platform character mapping.
    pub fn matches(&self, ev: &web_sys::KeyboardEvent) -> bool {
        let has = |m: Modifier| self.modifiers.contains(&m);
        let mac = is_mac();

        // Main maps to meta on Mac, ctrl on Windows/Linux
        let main_pressed = if mac { ev.meta_key() } else { ev.ctrl_key() };
        let main_expected = has(Modifier::Main);

        // The "other" modifier (ctrl on Mac, meta on Windows) should not be pressed
        let other_pressed = if mac { ev.ctrl_key() } else { ev.meta_key() };

        let code_matches = ev.code().eq_ignore_ascii_case(&Self::key_to_code(&self.key));

        main_pressed == main_expected
            && !other_pressed
            && ev.alt_key() == has(Modifier::Alt)
            && ev.shift_key() == has(Modifier::Shift)
            && code_matches
    }
}

impl std::fmt::Display for Shortcut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, m) in self.modifiers.iter().enumerate() {
            if i > 0 {
                write!(f, "+")?;
            }
            write!(f, "{m}")?;
        }
        if !self.modifiers.is_empty() {
            write!(f, "+")?;
        }
        write!(f, "{}", self.key.to_uppercase())
    }
}
