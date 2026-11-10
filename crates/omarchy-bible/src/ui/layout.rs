//! Which layout the window gets: the compact phone layout or the regular one
//! (tablets and desktops).
//!
//! The width decides, not the platform: a phone in landscape, a split-screen
//! tablet, or a narrow desktop window all get what fits. Android's window
//! classes put the compact/medium boundary at 600 dp, the same as here.

use nsh_core::element::Os;
use nsh_core::prelude::*;

/// Below this width (points) the compact layout is used.
pub const COMPACT_MAX_WIDTH: f32 = 600.0;

/// True when the window is narrower than [`COMPACT_MAX_WIDTH`]. Reading it
/// subscribes to window size changes (rotation, split screen, resizing).
pub fn compact() -> bool {
    window().width.0 < COMPACT_MAX_WIDTH
}

/// True on a touch-first platform: bigger targets, no hover, no tooltips.
pub fn touch() -> bool {
    matches!(os(), Os::Android | Os::Ios)
}

/// Touch target size (points): 44 on touch platforms, as the design guide
/// asks; the desktop keeps its denser controls.
pub fn target() -> f32 {
    if touch() { 44.0 } else { 0.0 }
}
