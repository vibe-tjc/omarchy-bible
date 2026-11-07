//! The reader's look, as NSH theme tokens.
//!
//! The palette (Tokyo Night, the light paper palette, or the live Omarchy
//! `colors.toml`) is the only place colors are written: [`tokens_for`]
//! turns it into shadcn-style tokens that the platform applies with
//! `set_tokens`, and every screen reads roles through [`theme()`] in
//! bindings, so a theme switch restyles the app with no rebuild.
//!
//! Role mapping from the GPUI palette:
//!
//! | Palette      | Token                         | Used for                          |
//! |--------------|-------------------------------|-----------------------------------|
//! | `bg`         | `background`                  | the window                        |
//! | `bg_sidebar` | `card`, `popover`             | the header, panels and pickers    |
//! | `bg_hover`   | `accent`, `muted`             | hovered, selected, highlighted    |
//! | `fg`         | `foreground`                  | body and English text             |
//! | `fg_primary` | `card-foreground`, `reader`   | titles and Chinese text           |
//! | `fg_muted`   | `muted-foreground`            | labels and hints                  |
//! | `accent`     | `primary`, `ring`             | verse numbers, active chips, focus|
//! | `border`     | `border`, `input`             | hairlines and fields              |

use nsh_base::Theme;
use nsh_core::element::{AdaptiveColor, Tokens, rgb};

use crate::settings::Palette;

/// The app's own token for the emphasised reading text (Chinese lanes and
/// titles): Omarchy's bright foreground.
pub const READER: &str = "reader";
/// The app's token for the dimmed overlay behind pickers and panels.
pub const SCRIM: &str = "scrim";

fn same(hex: u32) -> AdaptiveColor {
    let c = rgb(hex).to_hsla();
    AdaptiveColor { light: c, dark: c }
}

/// The tokens that draw the app in `palette`. The palette is already the
/// resolved one (light or dark), so each color is the same in both
/// appearances; the appearance is set alongside for the platform's own
/// controls.
pub fn tokens_for(palette: Palette) -> Tokens {
    let mut theme = Theme::default();
    let set = |theme: &mut Theme, name: &str, hex: u32| theme.set_color(name, same(hex));
    set(&mut theme, "background", palette.bg);
    set(&mut theme, "foreground", palette.fg);
    set(&mut theme, "card", palette.bg_sidebar);
    set(&mut theme, "card-foreground", palette.fg_primary);
    set(&mut theme, "popover", palette.bg_sidebar);
    set(&mut theme, "popover-foreground", palette.fg_primary);
    set(&mut theme, "primary", palette.accent);
    set(&mut theme, "primary-foreground", palette.bg);
    set(&mut theme, "secondary", palette.bg_hover);
    set(&mut theme, "secondary-foreground", palette.accent);
    set(&mut theme, "muted", palette.bg_hover);
    set(&mut theme, "muted-foreground", palette.fg_muted);
    set(&mut theme, "accent", palette.bg_hover);
    set(&mut theme, "accent-foreground", palette.accent);
    set(&mut theme, "border", palette.border);
    set(&mut theme, "input", palette.border);
    set(&mut theme, "ring", palette.accent);
    set(&mut theme, READER, palette.fg_primary);
    // 40% black, as the GPUI overlays' rgba(0x00000066).
    let scrim = nsh_core::element::rgba(0x0000_0066).to_hsla();
    theme.set_color(
        SCRIM,
        AdaptiveColor {
            light: scrim,
            dark: scrim,
        },
    );
    theme.set_number("radius", 6.0);
    theme.into_tokens()
}

/// The Chinese reading font, as the GPUI app chose it: Traditional Chinese
/// serif faces first, each platform's own, then the system's sans-serif.
/// The platform takes the first one installed and falls back per glyph.
#[cfg(target_os = "macos")]
pub const READING_FONT_ZH: &str =
    "\"Songti TC\", \"PingFang TC\", \"Noto Serif CJK TC\", \"Source Han Serif TC\", sans-serif";
#[cfg(not(target_os = "macos"))]
pub const READING_FONT_ZH: &str = "\"Noto Serif CJK TC\", \"Source Han Serif TC\", \"Noto Sans CJK TC\", \"Source Han Sans TC\", sans-serif";

/// The theme the app is drawn with. Read it in a binding.
pub fn theme() -> std::sync::Arc<Theme> {
    nsh_base::theme()
}

/// The emphasised reading color ([`READER`]), falling back to the card
/// foreground so a missing token is visible in review, not a panic.
pub fn reader() -> AdaptiveColor {
    let t = theme();
    t.color(READER).unwrap_or(t.card_foreground)
}

/// The overlay behind pickers and panels ([`SCRIM`]).
pub fn scrim() -> AdaptiveColor {
    let t = theme();
    t.color(SCRIM).unwrap_or(t.muted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_carry_every_palette_role() {
        let palette = Palette::tokyo_night();
        let tokens = tokens_for(palette);
        let hex = |name: &str| tokens.color(name).expect(name).dark;
        assert_eq!(hex("background"), rgb(palette.bg).to_hsla());
        assert_eq!(hex("primary"), rgb(palette.accent).to_hsla());
        assert_eq!(hex("muted-foreground"), rgb(palette.fg_muted).to_hsla());
        assert_eq!(hex(READER), rgb(palette.fg_primary).to_hsla());
        assert!(tokens.color(SCRIM).is_some());
        // A resolved palette looks the same in both appearances.
        let bg = tokens.color("background").unwrap();
        assert_eq!(bg.light, bg.dark);
    }
}
