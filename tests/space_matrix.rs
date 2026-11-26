//! Space 预设间距：small 8 / middle 16 / large 24。
//!
//! 基准由 `harness/antd_baseline/gen_space_matrix.mjs` 依据 antdv-next
//! `space/style/index.js` 的 mergeToken 生成。

use gpui_widgets::space::{SpaceSize, space_gap};
use gpui_widgets::theme::{Appearance, Color, Seed, Tokens, derive, metric::SpaceMetric};

const BASELINE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/harness/antd_baseline/");

fn check(tokens: &Tokens, suffix: &str) {
    let text = std::fs::read_to_string(format!("{BASELINE_DIR}space_matrix_{suffix}.txt")).unwrap();
    let line = text.lines().next().unwrap();
    let p: Vec<f32> = line.split('|').map(|v| v.parse().unwrap()).collect();
    assert_eq!(p.len(), 3, "坏行: {line}");
    let m = SpaceMetric::resolve(tokens);
    assert_eq!(m.small, p[0], "{suffix} small");
    assert_eq!(m.middle, p[1], "{suffix} middle");
    assert_eq!(m.large, p[2], "{suffix} large");
    assert_eq!(space_gap(tokens, SpaceSize::Small), p[0]);
    assert_eq!(space_gap(tokens, SpaceSize::Middle), p[1]);
    assert_eq!(space_gap(tokens, SpaceSize::Large), p[2]);
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn space_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn space_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn space_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn space_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = SpaceMetric::resolve(&tokens);
    assert_eq!((m.small, m.middle, m.large), (8.0, 16.0, 24.0));
}
