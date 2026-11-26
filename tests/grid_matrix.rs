//! Grid 矩阵：24 列占比 / offset / gutter 拆半几何。
//!
//! 基准由 `harness/antd_baseline/gen_layout_grid_matrix.mjs` 依据 antdv-next
//! `grid/style/index.js` + `grid/Row.js` gutter 处理生成。
use gpui_widgets::theme::metric::{
    GRID_COLUMNS, grid_col_basis, grid_col_gutter_padding, grid_offset_margin,
    grid_row_gutter_margin,
};
use gpui_widgets::theme::{Appearance, Color, Seed, Tokens, derive};

const BASELINE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/harness/antd_baseline/");

fn assert_f32(actual: f32, expected: &str, what: &str, row: &str) {
    let expected = expected.parse::<f32>().unwrap();
    assert!(
        (actual - expected).abs() < 1e-4,
        "{row} {what}: got {actual}, want {expected}"
    );
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

fn baseline(name: &str, suffix: &str) -> Vec<String> {
    let text =
        std::fs::read_to_string(format!("{BASELINE_DIR}{name}_matrix_{suffix}.txt")).unwrap();
    text.lines()
        .next()
        .unwrap()
        .split('|')
        .map(str::to_string)
        .collect()
}

fn check(_tokens: &Tokens, suffix: &str) {
    let p = baseline("grid", suffix);
    assert_eq!(p.len(), 6, "坏行: {p:?}");
    let row = format!("{p:?}");

    assert_f32(GRID_COLUMNS as f32, &p[0], "列数", &row);
    assert_f32(grid_col_basis(12) * 100.0, &p[1], "span12 占比", &row);
    assert_f32(grid_col_basis(8) * 100.0, &p[2], "span8 占比", &row);
    assert_f32(grid_offset_margin(6) * 100.0, &p[3], "offset6 占比", &row);
    assert_f32(
        grid_row_gutter_margin(16.0),
        &p[4],
        "gutter 行负外边距",
        &row,
    );
    assert_f32(
        grid_col_gutter_padding(16.0),
        &p[5],
        "gutter 列内边距",
        &row,
    );
}

#[test]
fn grid_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn grid_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn grid_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn grid_gutter_matrix() {
    // 满行 span 和为 24；gutter 下两栏内容宽 = (W+g)/2 - g
    let sum: f32 = (0..=24).map(|s| grid_col_basis(s)).sum();
    assert_f32(sum, "12.5", "0..24 占比和", "gutter");
    assert_f32(grid_col_basis(24), "1", "满行", "gutter");
    assert_f32(
        grid_row_gutter_margin(0.0),
        "0",
        "无 gutter 无负边距",
        "gutter",
    );
    assert_f32(
        grid_col_gutter_padding(0.0),
        "0",
        "无 gutter 无内边距",
        "gutter",
    );
    let w = 240.0_f32;
    let g = 16.0_f32;
    let content = (w + g) * grid_col_basis(12) - 2.0 * grid_col_gutter_padding(g);
    assert_f32(content, "112", "gutter16 半栏内容宽", "gutter");
}
