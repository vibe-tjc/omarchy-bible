//! The projection control desk. Audience output is a separate native
//! window/display, never a mirror of this page.
use crate::store::{Store, store};
use crate::ui::controls::{chip, label, panel, panel_header};
use crate::ui::theme::theme;
use nsh_core::prelude::*;

pub fn projection_panel() -> View {
    let s = store();
    panel().test_id("projection-panel")
        .child(panel_header("投影", "projection-close", move || s.close_overlay()))
        .child(label("點選經文指定單節；選擇模式可選多節。預設每張一節。"))
        .child(chip("projection-multiple", move || {
            if s.projection.with(|p| p.multiple) { "多節：開啟" } else { "多節：關閉（單節）" }
        }, move || s.projection.with(|p| p.multiple), move || s.projection_multiple()))
        .child(view().flex_row().flex_wrap().gap_2()
            .child(chip("projection-selection", "使用目前經文／選取", || false, move || s.prepare_projection(false)))
            .child(chip("projection-chapter", "使用本章", || false, move || s.prepare_projection(true))))
        .child(label(move || format!("輸出螢幕：{}", s.projection.with(|p| {
            if p.screen<0 { "自動選擇外接螢幕".into() } else if cfg!(target_os="android") { format!("外接螢幕 {}",p.screen+1) } else { format!("螢幕 {}（0 為預覽）",p.screen) }
        }))))
        .child(view().flex_row().gap_2()
            .child(chip("projection-display-prev", "上一個螢幕", || false, move || s.projection_screen(-1)))
            .child(chip("projection-display-next", "下一個螢幕", || false, move || s.projection_screen(1))))
        .child(label("自動模式不會佔用主螢幕。外接顯示需系統支援 HDMI／USB-C 或無線延伸顯示；單純鏡像不算第二螢幕。"))
        .child(view().flex_row().flex_wrap().gap_2()
            .child(chip("projection-start", "開始投影", || false, move || s.start_projection()))
            .child(chip("projection-stop", "停止投影", || false, move || s.stop_projection()))
            .child(chip("projection-blank", move || if s.projection.with(|p|p.blank) {"恢復畫面"} else {"黑屏"}, move || s.projection.with(|p|p.blank), move || s.projection_blank())))
        .child(view().flex_row().gap_2()
            .child(chip("projection-prev", "上一張", || false, move || s.projection_step(-1)))
            .child(label(move || s.projection.with(|p|format!("{} / {}", if p.slides.is_empty(){0}else{p.index+1},p.slides.len()))))
            .child(chip("projection-next", "下一張", || false, move || s.projection_step(1))))
        .child(label(move || s.projection_status.get()).test_id("projection-status"))
        .child(scroll().min_h(px(100.0)).max_h(px(240.0)).child(
            view().p_3().gap_2().bg(move || theme().background)
                .child(text(move || s.projection.with(|p|p.current().map(|s|s.reference.clone()).unwrap_or_default())).text_lg())
                .child(text(move || s.projection.with(|p|p.current().map(|s|s.text.clone()).unwrap_or_else(||"尚未選取投影內容".into()))).text_base())))
}

/// Called on an explicit user press, not when the root mounts.
pub fn open(s: Store) {
    s.prepare_projection(false);
    s.open_overlay(crate::store::Overlay::Projection);
}
