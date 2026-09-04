//! Persisted reader settings and theme palettes.

use bible_core::ViewMode;
use gpui::{App, Window, WindowAppearance};
use serde::{Deserialize, Serialize};
#[cfg(any(test, target_os = "linux"))]
use std::collections::HashMap;
use std::path::{Path, PathBuf};
#[cfg(target_os = "linux")]
use std::time::SystemTime;

pub const FONT_MIN: u32 = 14;
pub const FONT_MAX: u32 = 28;
pub const FONT_DEFAULT: u32 = 18;
pub const FONT_SMALL: u32 = 14;
pub const FONT_MEDIUM: u32 = 18;
pub const FONT_LARGE: u32 = 22;

/// User preference. `System` follows the OS / Omarchy appearance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreference {
    #[default]
    Dark,
    Light,
    System,
}

impl ThemePreference {
    pub fn label_zh(self) -> &'static str {
        match self {
            ThemePreference::Dark => "深色",
            ThemePreference::Light => "淺色",
            ThemePreference::System => "跟隨系統",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub theme: ThemePreference,
    #[serde(default = "default_font_size")]
    pub font_size: u32,
    #[serde(default)]
    pub view_mode: ViewMode,
    /// When true, sidebar shows Traditional Chinese abbreviations (創／太／羅).
    #[serde(default)]
    pub sidebar_abbrev: bool,
}

fn default_font_size() -> u32 {
    FONT_DEFAULT
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: ThemePreference::Dark,
            font_size: FONT_DEFAULT,
            view_mode: ViewMode::default(),
            sidebar_abbrev: false,
        }
    }
}

impl AppSettings {
    pub fn clamp(mut self) -> Self {
        self.font_size = self.font_size.clamp(FONT_MIN, FONT_MAX);
        self.view_mode = self.view_mode.sanitized();
        self
    }

    pub fn config_path() -> PathBuf {
        config_dir().join("settings.json")
    }

    pub fn load() -> Self {
        Self::load_from(&Self::config_path())
    }

    pub fn load_from(path: &Path) -> Self {
        let Ok(raw) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        serde_json::from_str::<AppSettings>(&raw)
            .unwrap_or_default()
            .clamp()
    }

    pub fn save(&self) {
        if let Err(err) = self.save_to(&Self::config_path()) {
            eprintln!("omarchy-bible: failed to save settings: {err}");
        }
    }

    pub fn save_to(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let json = serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".into());
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, json)?;
        std::fs::rename(&tmp, path)
    }

    pub fn chinese_px(&self) -> f32 {
        self.font_size as f32
    }

    pub fn english_px(&self) -> f32 {
        (self.font_size as f32 * 0.85).round().max(12.0)
    }

    pub fn number_px(&self) -> f32 {
        (self.font_size as f32 * 0.72).round().max(11.0)
    }
}

fn config_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        let base = std::env::var_os("HOME")
            .map(|h| PathBuf::from(h).join("Library/Application Support"))
            .unwrap_or_else(|| PathBuf::from("Library/Application Support"));
        return base.join("omarchy-bible");
    }
    #[cfg(not(target_os = "macos"))]
    {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
            .unwrap_or_else(|| PathBuf::from(".config"));
        base.join("omarchy-bible")
    }
}

/// Short Chinese note shown in the settings panel.
pub fn settings_location_note_zh() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "設定儲存於 ~/Library/Application Support/omarchy-bible/settings.json"
    }
    #[cfg(not(target_os = "macos"))]
    {
        "設定儲存於 ~/.config/omarchy-bible/settings.json"
    }
}

/// App chrome + verse colours. Dark is Tokyo Night; light is a paper/latte palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub bg: u32,
    pub bg_sidebar: u32,
    pub bg_hover: u32,
    pub fg: u32,
    pub fg_muted: u32,
    pub accent: u32,
    pub fg_primary: u32,
    pub border: u32,
}

impl Palette {
    /// Omarchy-like Tokyo Night (existing Phase 1 look).
    pub fn tokyo_night() -> Self {
        Self {
            bg: 0x1a1b26,
            bg_sidebar: 0x16161e,
            bg_hover: 0x24283b,
            fg: 0xa9b1d6,
            fg_muted: 0x565f89,
            accent: 0x7aa2f7,
            fg_primary: 0xc0caf5,
            border: 0x292e42,
        }
    }

    /// Readable light palette (Catppuccin Latte-inspired, not an invert).
    pub fn light() -> Self {
        Self {
            bg: 0xeff1f5,
            bg_sidebar: 0xe6e9ef,
            bg_hover: 0xdce0e8,
            fg: 0x4c4f69,
            fg_muted: 0x8c8fa1,
            accent: 0x1e66f5,
            fg_primary: 0x4c4f69,
            border: 0xccd0da,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeSource {
    BuiltinDark,
    BuiltinLight,
    Omarchy,
    GpuiAppearance,
    FallbackDark,
}

impl ThemeSource {
    pub fn note_zh(self, dark: bool) -> String {
        let tone = if dark { "深色" } else { "淺色" };
        match self {
            ThemeSource::BuiltinDark | ThemeSource::BuiltinLight => String::new(),
            ThemeSource::Omarchy => format!("系統外觀：{tone}（Omarchy）"),
            ThemeSource::GpuiAppearance => format!("系統外觀：{tone}（視窗外觀）"),
            ThemeSource::FallbackDark => "系統外觀無法偵測，已回退為深色".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedTheme {
    pub dark: bool,
    pub palette: Palette,
    pub source: ThemeSource,
}

pub fn resolve_theme(
    pref: ThemePreference,
    window: Option<&Window>,
    cx: Option<&App>,
) -> ResolvedTheme {
    match pref {
        ThemePreference::Dark => ResolvedTheme {
            dark: true,
            palette: Palette::tokyo_night(),
            source: ThemeSource::BuiltinDark,
        },
        ThemePreference::Light => ResolvedTheme {
            dark: false,
            palette: Palette::light(),
            source: ThemeSource::BuiltinLight,
        },
        ThemePreference::System => resolve_system(window, cx),
    }
}

fn resolve_system(window: Option<&Window>, cx: Option<&App>) -> ResolvedTheme {
    if let Some(omarchy) = detect_omarchy() {
        return ResolvedTheme {
            dark: omarchy.dark,
            palette: omarchy.palette.unwrap_or(if omarchy.dark {
                Palette::tokyo_night()
            } else {
                Palette::light()
            }),
            source: ThemeSource::Omarchy,
        };
    }

    if let Some(appearance) = gpui_appearance(window, cx) {
        let dark = matches!(
            appearance,
            WindowAppearance::Dark | WindowAppearance::VibrantDark
        );
        return ResolvedTheme {
            dark,
            palette: if dark {
                Palette::tokyo_night()
            } else {
                Palette::light()
            },
            source: ThemeSource::GpuiAppearance,
        };
    }

    ResolvedTheme {
        dark: true,
        palette: Palette::tokyo_night(),
        source: ThemeSource::FallbackDark,
    }
}

/// Prefer `window.appearance()`; skip `App::window_appearance` on Linux without a
/// window (gpui-component#104).
fn gpui_appearance(window: Option<&Window>, cx: Option<&App>) -> Option<WindowAppearance> {
    if let Some(window) = window {
        return Some(window.appearance());
    }
    #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
    if let Some(cx) = cx {
        return Some(cx.window_appearance());
    }
    let _ = cx;
    None
}

struct OmarchyTheme {
    dark: bool,
    palette: Option<Palette>,
}

#[cfg(target_os = "linux")]
fn omarchy_theme_dir() -> Option<PathBuf> {
    let state = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/state")))?;
    let dir = state.join("omarchy").join("current").join("theme");
    dir.exists().then_some(dir)
}

/// Omarchy Quattro: `light.mode` marker in the current theme dir, else `colors.toml`
/// `mode = "light"|"dark"`. Live colours from `colors.toml` when present.
/// Linux only; macOS never inspects Omarchy paths.
fn detect_omarchy() -> Option<OmarchyTheme> {
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
    #[cfg(target_os = "linux")]
    {
        let dir = omarchy_theme_dir()?;
        let light_marker = dir.join("light.mode").is_file();
        let colors_path = dir.join("colors.toml");
        let colors = std::fs::read_to_string(&colors_path).ok();
        let toml_mode = colors.as_deref().and_then(colors_toml_mode);

        let dark = if light_marker {
            false
        } else if let Some(is_light) = toml_mode {
            !is_light
        } else {
            // Theme dir exists but no light.mode and no mode key — treat as dark
            // only when we have nothing else; still an Omarchy signal.
            true
        };

        let palette = colors.as_deref().and_then(palette_from_colors_toml);
        Some(OmarchyTheme { dark, palette })
    }
}

#[cfg(any(test, target_os = "linux"))]
fn colors_toml_mode(text: &str) -> Option<bool> {
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() != "mode" {
            continue;
        }
        let value = value.trim().trim_matches('"').trim_matches('\'');
        return match value {
            "light" => Some(true),
            "dark" => Some(false),
            _ => None,
        };
    }
    None
}

#[cfg(any(test, target_os = "linux"))]
fn parse_hex_rgb(value: &str) -> Option<u32> {
    let value = value.trim().trim_matches('"').trim_matches('\'');
    let hex = value.strip_prefix('#').unwrap_or(value);
    if hex.len() != 6 {
        return None;
    }
    u32::from_str_radix(hex, 16).ok()
}

#[cfg(any(test, target_os = "linux"))]
fn palette_from_colors_toml(text: &str) -> Option<Palette> {
    let mut map: HashMap<&str, u32> = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if let Some(hex) = parse_hex_rgb(value) {
            map.insert(key, hex);
        }
    }
    let bg = *map.get("background")?;
    let fg = *map.get("foreground")?;
    let accent = map
        .get("accent")
        .copied()
        .or_else(|| map.get("blue").copied())?;
    Some(Palette {
        bg,
        bg_sidebar: map.get("dark_background").copied().unwrap_or(bg),
        bg_hover: map
            .get("darker_background")
            .copied()
            .or_else(|| map.get("lighter_background").copied())
            .unwrap_or(bg),
        fg,
        fg_muted: map.get("dark_foreground").copied().unwrap_or(fg),
        accent,
        fg_primary: map
            .get("bright_foreground")
            .copied()
            .or_else(|| map.get("light_foreground").copied())
            .unwrap_or(fg),
        border: map
            .get("darker_background")
            .copied()
            .or_else(|| map.get("lighter_background").copied())
            .unwrap_or(bg),
    })
}

/// Cheap stamp so the UI can poll Omarchy theme switches (Linux). No-op elsewhere.
pub fn omarchy_stamp() -> Option<u128> {
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
    #[cfg(target_os = "linux")]
    {
        let dir = omarchy_theme_dir()?;
        let mut stamp = file_mtime(&dir).unwrap_or(0);
        if let Some(m) = file_mtime(&dir.join("colors.toml")) {
            stamp = stamp.saturating_add(m);
        }
        if dir.join("light.mode").is_file() {
            stamp = stamp.saturating_add(1);
        }
        Some(stamp)
    }
}

#[cfg(target_os = "linux")]
fn file_mtime(path: &Path) -> Option<u128> {
    let meta = std::fs::metadata(path).ok()?;
    let elapsed = meta
        .modified()
        .ok()?
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()?;
    Some(elapsed.as_millis())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bible_core::TranslationId;

    #[test]
    fn settings_json_roundtrip() {
        let settings = AppSettings {
            theme: ThemePreference::System,
            font_size: 20,
            view_mode: ViewMode::single(TranslationId::Kjv),
            sidebar_abbrev: false,
        };
        let json = serde_json::to_string(&settings).expect("serialize");
        assert!(json.contains("system"), "{json}");
        assert!(json.contains("kjv"), "{json}");
        let back: AppSettings = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, settings);
    }

    #[test]
    fn settings_defaults_and_clamp() {
        let parsed: AppSettings = serde_json::from_str("{}").unwrap();
        assert_eq!(parsed.theme, ThemePreference::Dark);
        assert_eq!(parsed.font_size, FONT_DEFAULT);
        assert!(parsed.view_mode.is_compare());
        assert_eq!(parsed.view_mode.lane_count(), 2);

        let huge = AppSettings {
            theme: ThemePreference::Light,
            font_size: 99,
            view_mode: ViewMode::Compare(vec![]),
            sidebar_abbrev: false,
        }
        .clamp();
        assert_eq!(huge.font_size, FONT_MAX);
        assert_eq!(huge.view_mode.lane_count(), 2);

        let tiny = AppSettings {
            theme: ThemePreference::Dark,
            font_size: 1,
            view_mode: ViewMode::single(TranslationId::Cuv1919),
            sidebar_abbrev: false,
        }
        .clamp();
        assert_eq!(tiny.font_size, FONT_MIN);
    }

    #[test]
    fn settings_save_load_file() {
        let dir = std::env::temp_dir().join(format!(
            "omarchy-bible-settings-test-{}",
            std::process::id()
        ));
        let path = dir.join("settings.json");
        let settings = AppSettings {
            theme: ThemePreference::Light,
            font_size: 22,
            view_mode: ViewMode::single(TranslationId::Cuv1919),
            sidebar_abbrev: false,
        };
        settings.save_to(&path).expect("save");
        let loaded = AppSettings::load_from(&path);
        assert_eq!(loaded, settings);
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(raw.contains("light"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn colors_toml_mode_and_palette() {
        let light = "mode = \"light\"\nbackground = \"#eff1f5\"\nforeground = \"#4c4f69\"\naccent = \"#1e66f5\"\n";
        assert_eq!(colors_toml_mode(light), Some(true));
        let pal = palette_from_colors_toml(light).expect("palette");
        assert_eq!(pal.bg, 0xeff1f5);
        assert_eq!(pal.accent, 0x1e66f5);
        assert_eq!(colors_toml_mode("mode = \"dark\"\n"), Some(false));
        assert_eq!(colors_toml_mode("accent = \"#fff\"\n"), None);
    }

    #[test]
    fn config_path_is_platform_specific() {
        let path = AppSettings::config_path();
        let s = path.to_string_lossy();
        assert!(s.contains("omarchy-bible"), "{s}");
        assert!(s.ends_with("settings.json"), "{s}");
        #[cfg(target_os = "macos")]
        assert!(
            s.contains("Library/Application Support"),
            "macOS settings should live under Application Support, got {s}"
        );
        #[cfg(not(target_os = "macos"))]
        assert!(
            s.contains(".config") || std::env::var_os("XDG_CONFIG_HOME").is_some(),
            "Linux settings should use XDG config, got {s}"
        );
        assert!(
            !s.contains(".local/state/omarchy"),
            "settings path must not be an Omarchy theme path: {s}"
        );
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn detect_omarchy_is_none_off_linux() {
        assert!(detect_omarchy().is_none());
        assert!(omarchy_stamp().is_none());
    }

    #[test]
    fn view_mode_persists_in_settings() {
        let compare = AppSettings::default();
        let json = serde_json::to_string(&compare).unwrap();
        let back: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.view_mode.lane_count(), 2);
        assert_eq!(back.view_mode.header_label(), "和合本 1919 神版 · KJV");
    }
}
