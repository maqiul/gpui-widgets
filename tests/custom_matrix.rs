//! 自研控件（antd 无对应）的度量与状态色矩阵。
//!
//! 这些控件没有 antd 参照，数值是本库自定；测试锁定 metric 常量并将状态色
//! 与当前主题令牌逐通道对齐，防止渲染层重新出现魔法 px / 硬编码色值。

use gpui::Hsla;
use gpui_widgets::theme::metric::*;
use gpui_widgets::theme::{Appearance, Color, Seed, Tokens, derive};
use gpui_widgets::{ThoughtStatus, TransferStatus, UploadStatus};

fn tokens_for(name: &str) -> Tokens {
    let mut seed = Seed::default();
    let appearance = match name {
        "light" => Appearance::Light,
        "dark" => Appearance::Dark,
        "accent-violet" => {
            seed.color_primary = Color::from_hex(0x8b5cf6);
            Appearance::Light
        }
        other => panic!("unknown theme {other}"),
    };
    derive(&seed, appearance)
}

fn channels(c: Hsla) -> [u8; 4] {
    let c = Color::from(c);
    [
        c.r.round().clamp(0.0, 255.0) as u8,
        c.g.round().clamp(0.0, 255.0) as u8,
        c.b.round().clamp(0.0, 255.0) as u8,
        (c.a * 255.0).round().clamp(0.0, 255.0) as u8,
    ]
}

fn assert_color(actual: Hsla, expected: Hsla, label: &str) {
    let a = channels(actual);
    let e = channels(expected);
    let diff = |x: u8, y: u8| (x as i16 - y as i16).abs();
    assert!(
        (0..4).all(|i| diff(a[i], e[i]) <= 1),
        "{label}: actual {a:?}, expected {e:?}"
    );
}

fn assert_f32(actual: f32, expected: f32, label: &str) {
    assert!(
        (actual - expected).abs() <= 0.01,
        "{label}: actual {actual}, expected {expected}"
    );
}

#[test]
fn bubble_custom_metric() {
    let m = bubble_metric(&tokens_for("light"));
    assert_f32(m.avatar_size, 32.0, "avatar_size");
    assert_f32(m.max_width, 420.0, "max_width");
    assert_f32(m.spinner_size, 16.0, "spinner_size");
}

#[test]
fn code_block_custom_metric() {
    let m = code_block_metric(&tokens_for("light"));
    assert_f32(m.font_size, 13.0, "font_size");
    assert_f32(m.line_height, 20.0, "line_height");
    assert_f32(m.action_gap, 4.0, "action_gap");
}

#[test]
fn markdown_custom_metric() {
    let tokens = tokens_for("light");
    let m = markdown_metric(&tokens);
    assert_f32(m.font_size, tokens.font_size, "font_size");
    assert_f32(m.line_height, tokens.font_height, "line_height");
    assert_eq!(m.heading_sizes.len(), 6);
    assert_f32(m.heading_sizes[0], tokens.font_size_heading1, "heading1");
    assert_f32(m.heading_sizes[4], tokens.font_size_heading5, "heading5");
    assert_f32(m.heading_sizes[5], tokens.font_size, "heading6");
    let extra = tokens.font_height - tokens.font_size;
    assert_f32(
        m.heading_line_heights[0],
        tokens.font_size_heading1 + extra,
        "heading1_line_height",
    );
    assert_f32(m.block_gap, tokens.margin_sm, "block_gap");
    assert_f32(m.bullet_width, tokens.size_ms, "bullet_width");
    assert_f32(m.quote_padding, tokens.padding_sm, "quote_padding");
    assert_f32(m.quote_border, tokens.line_width_bold, "quote_border");
    assert_f32(m.rule_thickness, tokens.line_width, "rule_thickness");
    assert_f32(m.link_thickness, tokens.line_width, "link_thickness");
    assert_f32(m.table_border, tokens.line_width, "table_border");
    assert_f32(m.task_box_size, tokens.font_size_lg, "task_box_size");
    assert_f32(m.task_check_size, tokens.font_size_sm, "task_check_size");
}

#[test]
fn markdown_compact_custom_metric() {
    let tokens = tokens_for("light");
    let m = markdown_compact_metric(&tokens);
    assert_f32(m.font_size, tokens.font_size, "font_size");
    assert_f32(
        m.heading_sizes[0],
        tokens.font_size * 1.45,
        "compact_heading1",
    );
    assert_f32(
        m.heading_sizes[2],
        tokens.font_size * 1.18,
        "compact_heading3",
    );
    assert_f32(m.heading_sizes[5], tokens.font_size, "compact_heading6");
    assert!(m.heading_sizes[2] < tokens.font_size_heading3);
    let extra = tokens.font_height - tokens.font_size;
    assert_f32(
        m.heading_line_heights[2],
        tokens.font_size * 1.18 + extra,
        "compact_heading3_line_height",
    );
    assert_f32(m.block_gap, tokens.margin_xs, "compact_block_gap");
    assert_f32(
        m.table_cell_padding,
        tokens.padding_xs,
        "compact_table_padding",
    );
}

#[test]
fn command_bar_custom_metric() {
    let m = command_bar_metric(&tokens_for("light"));
    assert_f32(m.bar_padding_block, 7.0, "bar_padding_block");
    assert_f32(m.chip_gap, 6.0, "chip_gap");
    assert_f32(m.chip_padding_block, 2.0, "chip_padding_block");
    assert_f32(m.chip_radius, 4.0, "chip_radius");
}

#[test]
fn command_palette_custom_metric() {
    for theme in ["light", "dark", "accent-violet"] {
        let t = tokens_for(theme);
        let m = command_palette_metric(&t);
        assert_f32(m.width, 480.0, "width");
        assert_f32(m.margin_top, 80.0, "margin_top");
        assert_f32(m.header_padding_block, 12.0, "header_padding_block");
        assert_f32(m.list_max_height, 320.0, "list_max_height");
        assert_f32(m.item_padding_block, 8.0, "item_padding_block");
        assert_color(m.mask, t.color_bg_mask, "mask");
    }
}

#[test]
fn drop_zone_custom_metric() {
    let m = drop_zone_metric(&tokens_for("light"));
    assert_f32(m.icon_size, 24.0, "icon_size");
}

#[test]
fn fps_custom_metric() {
    let m = fps_metric(&tokens_for("light"));
    assert_f32(m.value_font_size, 30.0, "value_font_size");
    assert_f32(m.unit_padding_bottom, 4.0, "unit_padding_bottom");
    assert_f32(m.graph_width, 180.0, "graph_width");
    assert_f32(m.graph_height, 44.0, "graph_height");
    assert_f32(m.dot_size, 8.0, "dot_size");
}

#[test]
fn prompts_custom_metric() {
    let m = prompts_metric(&tokens_for("light"));
    assert_f32(m.card_width, 200.0, "card_width");
}

#[test]
fn sender_custom_metric() {
    let m = sender_metric(&tokens_for("light"));
    assert_f32(m.input_height, 56.0, "input_height");
}

#[test]
fn sender_send_state_colors() {
    for name in ["light", "dark", "accent-violet"] {
        let t = tokens_for(name);
        let idle = sender_send_style(&t, false);
        assert_color(idle.bg, t.color_primary, "idle bg");
        assert_color(idle.fg, t.color_text_light_solid, "idle fg");
        assert_color(idle.hover_bg.unwrap(), t.color_primary_hover, "hover bg");

        let loading = sender_send_style(&t, true);
        assert_color(loading.bg, t.control_item_bg_hover, "loading bg");
        assert_color(loading.fg, t.color_text_disabled, "loading fg");
        assert!(loading.hover_bg.is_none(), "loading has no hover");
    }
}

#[test]
fn thought_chain_custom_metric() {
    let t = tokens_for("light");
    let m = thought_chain_metric(&t);
    assert_f32(m.icon_size, 18.0, "icon_size");
    assert_f32(m.icon_font_size, 11.0, "icon_font_size");
    assert_f32(m.indicator_width, 20.0, "indicator_width");
    assert_f32(m.line_width, 2.0, "line_width");

    let pending = thought_status_style(&t, ThoughtStatus::Pending);
    assert_color(pending.border.unwrap(), t.color_primary, "pending border");
    assert!(pending.fill.is_none(), "pending should not be filled");
    assert!(pending.glyph.is_none(), "pending has no glyph");

    let success = thought_status_style(&t, ThoughtStatus::Success);
    assert!(success.border.is_none(), "success has no ring");
    assert_color(success.fill.unwrap(), t.color_success, "success fill");
    assert_color(success.fg, t.color_text_light_solid, "success fg");
    assert_eq!(success.glyph, Some("✓"));

    let error = thought_status_style(&t, ThoughtStatus::Error);
    assert_color(error.fill.unwrap(), t.color_error, "error fill");
    assert_color(error.fg, t.color_text_light_solid, "error fg");
    assert_eq!(error.glyph, Some("✕"));
}

#[test]
fn title_bar_custom_metric() {
    let m = title_bar_metric(&tokens_for("light"));
    assert_f32(m.control_glyph_size, 14.0, "control_glyph_size");
}

#[test]
fn video_custom_metric() {
    let m = video_metric(&tokens_for("light"));
    assert_f32(m.height, 220.0, "height");
    assert_f32(m.radius, 8.0, "radius");
    assert_f32(m.spinner_size, 24.0, "spinner_size");
    assert_color(m.backdrop, gpui::rgb(0x14141a).into(), "backdrop");
}

#[test]
fn welcome_custom_metric() {
    let m = welcome_metric(&tokens_for("light"));
    assert_f32(m.icon_size, 56.0, "icon_size");
    assert_f32(m.icon_font_size, 28.0, "icon_font_size");
}

#[test]
fn qrcode_custom_metric() {
    for theme in ["light", "dark"] {
        let t = tokens_for(theme);
        let m = qrcode_metric(&t);
        assert_f32(m.default_size, 160.0, "default_size");
        assert_color(m.bg, t.color_white, "bg");
        assert_color(m.fg, t.color_text, "fg");
    }
}

#[test]
fn upload_list_custom_metric() {
    let m = upload_list_metric(&tokens_for("light"));
    assert_f32(m.bar_height, 3.0, "bar_height");
    assert_f32(m.row_padding_block, 6.0, "row_padding_block");

    for theme in ["light", "dark"] {
        let t = tokens_for(theme);
        let uploading = upload_status_style(&t, UploadStatus::Uploading);
        assert_color(uploading.color, t.color_primary, "uploading color");
        assert_color(uploading.name, t.color_text, "uploading name");
        let done = upload_status_style(&t, UploadStatus::Done);
        assert_color(done.color, t.color_success, "done color");
        let error = upload_status_style(&t, UploadStatus::Error);
        assert_color(error.color, t.color_error, "error color");
        assert_color(error.name, t.color_error, "error name");
        assert_eq!(error.label, "失败");
    }
}

#[test]
fn list_custom_metric() {
    let t = tokens_for("light");
    let m = list_metric(&t);
    assert_f32(m.avatar_size, 32.0, "avatar_size");
    assert_f32(m.footer_padding_block, 10.0, "footer_padding_block");
    // antd List：条目内边距 paddingSM / padding
    assert_f32(t.padding_sm, 12.0, "item padding block");
    assert_f32(t.padding, 16.0, "item padding inline");
}

#[test]
fn watermark_custom_metric() {
    for theme in ["light", "dark"] {
        let t = tokens_for(theme);
        let m = watermark_metric(&t);
        assert_f32(m.font_size, t.font_size_lg, "font_size");
        assert_color(m.color, t.color_fill, "color");
        assert_f32(m.gap_x, 100.0, "gap_x");
        assert_f32(m.gap_y, 100.0, "gap_y");
    }
}

#[test]
fn transfer_manager_custom_metric() {
    let m = transfer_manager_metric(&tokens_for("light"));
    assert_f32(m.meta_gap, 2.0, "meta_gap");
    assert_f32(m.empty_padding_block, 28.0, "empty_padding_block");
    assert_f32(m.tag_padding_inline, 6.0, "tag_padding_inline");
    assert_f32(m.tag_padding_block, 1.0, "tag_padding_block");
    assert_f32(m.actions_padding_top, 2.0, "actions_padding_top");
    assert_f32(m.direction_icon_size, 26.0, "direction_icon_size");
    assert_f32(m.action_button_size, 22.0, "action_button_size");
    assert_f32(m.bar_height, 4.0, "bar_height");
}

#[test]
fn transfer_manager_status_style() {
    for theme in ["light", "dark"] {
        let t = tokens_for(theme);
        let queued = transfer_status_style(&t, TransferStatus::Queued);
        assert_color(queued.fg, t.color_text_secondary, "queued fg");
        assert_color(queued.bg, t.control_item_bg_hover, "queued bg");
        let running = transfer_status_style(&t, TransferStatus::Transferring);
        assert_color(running.fg, t.color_primary, "transferring fg");
        assert_color(running.bg, t.color_primary_bg, "transferring bg");
        let paused = transfer_status_style(&t, TransferStatus::Paused);
        assert_color(paused.fg, t.color_warning, "paused fg");
        assert_color(paused.bg, t.color_warning_bg, "paused bg");
        let done = transfer_status_style(&t, TransferStatus::Done);
        assert_color(done.fg, t.color_success, "done fg");
        let failed = transfer_status_style(&t, TransferStatus::Failed);
        assert_color(failed.fg, t.color_error, "failed fg");
        let canceled = transfer_status_style(&t, TransferStatus::Canceled);
        assert_color(canceled.fg, t.color_text_disabled, "canceled fg");
    }
}
