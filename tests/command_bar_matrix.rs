//! CommandBar 状态矩阵：度量常量 + 两种外观 + 四类动作按钮三态色。
//!
//! 命令条没有 antd 参照（自研控件），故截图基线不适用；这里锁定「渲染层不得再出现
//! 魔法 px / 硬编码色值」：几何取 `command_bar_metric`，颜色取 `command_bar_style` /
//! `command_bar_action_style`，两者都必须是 `Tokens` 的纯函数。

use gpui::Hsla;
use gpui_widgets::theme::metric::{
    CommandBarAction, command_bar_action_style, command_bar_metric, command_bar_style,
};
use gpui_widgets::theme::{Appearance, Color, Seed, Tokens, derive};

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

/// 描边 / 底色只分「画 / 不画」两态：`Some` 的色值另用 `assert_color` 校。
fn assert_frame(actual: &Option<Hsla>, expected: Option<Hsla>, label: &str) {
    assert_eq!(
        actual.map(channels),
        expected.map(channels),
        "{label}: 描边态不符"
    );
}

#[test]
fn command_bar_metric_matrix() {
    let m = command_bar_metric(&tokens_for("light"));
    // framed（控件自带整条外框）
    assert_f32(m.bar_padding_block, 7.0, "bar_padding_block");
    assert_f32(m.bar_padding_inline, 12.0, "bar_padding_inline");
    assert_f32(m.bar_gap, 8.0, "bar_gap");
    assert_f32(m.bar_radius, 6.0, "bar_radius");
    // bare（宿主 chrome 条 + 输入框自描边）
    assert_f32(m.bar_height, 34.0, "bar_height");
    assert_f32(m.bare_padding_inline, 10.0, "bare_padding_inline");
    assert_f32(m.bare_gap, 6.0, "bare_gap");
    assert_f32(m.bare_radius, 6.0, "bare_radius");
    assert_f32(m.input_height, 24.0, "input_height");
    assert_f32(m.input_padding_inline, 8.0, "input_padding_inline");
    assert_f32(m.prompt_font_size, 12.0, "prompt_font_size");
    // 动作按钮
    assert_f32(m.action_height, 22.0, "action_height");
    assert_f32(m.action_padding_inline, 8.0, "action_padding_inline");
    assert_f32(m.action_font_size, 11.0, "action_font_size");
    // 浮层
    assert_f32(m.pop_min_width, 220.0, "pop_min_width");
    assert_f32(m.pop_max_height, 240.0, "pop_max_height");
    assert_f32(m.pop_padding, 4.0, "pop_padding");
    assert_f32(m.pop_radius, 8.0, "pop_radius");
    assert_f32(m.pop_row_padding_block, 5.0, "pop_row_padding_block");
    assert_f32(m.pop_row_padding_inline, 8.0, "pop_row_padding_inline");
    assert_f32(m.pop_row_font_size, 11.0, "pop_row_font_size");
    assert_f32(m.pop_empty_padding, 8.0, "pop_empty_padding");
    assert_f32(m.pop_gap, 2.0, "pop_gap");
    // 片段芯片
    assert_f32(m.chip_gap, 6.0, "chip_gap");
    assert_f32(m.chip_padding_block, 2.0, "chip_padding_block");
    assert_f32(m.chip_radius, 4.0, "chip_radius");
    // 片段面板（原版 SnippetPanel 形态）
    assert_f32(m.input_font_size, 12.0, "input_font_size");
    assert_f32(m.snip_width, 320.0, "snip_width");
    assert_f32(m.snip_max_height, 340.0, "snip_max_height");
    assert_f32(m.snip_padding, 6.0, "snip_padding");
    assert_f32(m.snip_head_padding_top, 2.0, "snip_head_padding_top");
    assert_f32(m.snip_head_padding_inline, 4.0, "snip_head_padding_inline");
    assert_f32(m.snip_head_padding_bottom, 6.0, "snip_head_padding_bottom");
    assert_f32(m.snip_title_font_size, 11.0, "snip_title_font_size");
    assert_f32(m.snip_new_height, 20.0, "snip_new_height");
    assert_f32(m.snip_new_padding_inline, 6.0, "snip_new_padding_inline");
    assert_f32(m.snip_new_gap, 3.0, "snip_new_gap");
    assert_f32(m.snip_group_gap, 4.0, "snip_group_gap");
    assert_f32(
        m.snip_group_name_padding_top,
        4.0,
        "snip_group_name_padding_top",
    );
    assert_f32(
        m.snip_group_name_padding_inline,
        6.0,
        "snip_group_name_padding_inline",
    );
    assert_f32(
        m.snip_group_name_padding_bottom,
        2.0,
        "snip_group_name_padding_bottom",
    );
    assert_f32(
        m.snip_group_name_font_size,
        10.0,
        "snip_group_name_font_size",
    );
    assert_f32(m.snip_row_padding_block, 5.0, "snip_row_padding_block");
    assert_f32(m.snip_row_padding_inline, 6.0, "snip_row_padding_inline");
    assert_f32(m.snip_row_gap, 6.0, "snip_row_gap");
    assert_f32(m.snip_name_font_size, 11.0, "snip_name_font_size");
    assert_f32(m.snip_cmd_font_size, 10.0, "snip_cmd_font_size");
    assert_f32(m.snip_empty_padding_block, 12.0, "snip_empty_padding_block");
    assert_f32(
        m.snip_empty_padding_inline,
        8.0,
        "snip_empty_padding_inline",
    );
    // 片段面板比历史列表更宽更高：两者共用外壳但尺寸不同
    assert!(
        m.snip_width > m.pop_min_width && m.snip_max_height > m.pop_max_height,
        "片段面板尺寸不该退化到历史列表"
    );
}

#[test]
fn command_bar_style_matrix() {
    for name in ["light", "dark", "accent-violet"] {
        let t = tokens_for(name);
        let framed = command_bar_style(&t, true, false);
        assert_frame(&framed.bar_bg, Some(t.color_bg_container), "framed bar_bg");
        assert_frame(
            &framed.bar_border,
            Some(t.color_border),
            "framed bar_border",
        );
        assert_frame(&framed.input_border, None, "framed input_border");
        assert_color(framed.prompt_fg, t.color_primary, "framed prompt_fg");
        assert_color(framed.hint_fg, t.color_text_secondary, "framed hint_fg");

        let framed_focus = command_bar_style(&t, true, true);
        assert_frame(
            &framed_focus.bar_border,
            Some(t.color_primary),
            "framed focused bar_border",
        );

        let bare = command_bar_style(&t, false, false);
        // 裸条不画底：宿主铺自己的 chrome，控件不抢它的色。
        assert_frame(&bare.bar_bg, None, "bare bar_bg");
        assert_frame(&bare.bar_border, None, "bare bar_border");
        assert_frame(
            &bare.input_border,
            Some(t.color_border),
            "bare input_border",
        );
        assert_color(bare.input_bg, t.color_bg_container, "bare input_bg");

        let bare_focus = command_bar_style(&t, false, true);
        assert_frame(
            &bare_focus.input_border,
            Some(t.color_primary),
            "bare focused input_border",
        );

        // 浮层配色两种外观一致
        assert_color(
            bare.pop_row_hover_bg,
            t.color_primary_bg,
            "pop_row_hover_bg",
        );
        assert_color(bare.pop_row_hover_fg, t.color_primary, "pop_row_hover_fg");
        assert_color(
            bare.pop_danger_hover_fg,
            t.color_error,
            "pop_danger_hover_fg",
        );
        assert_color(bare.pop_empty_fg, t.color_text_tertiary, "pop_empty_fg");
        // 片段面板：标题与组名降到三级字，名称用正文色，命令列三级字
        assert_color(bare.snip_title_fg, t.color_text_tertiary, "snip_title_fg");
        assert_color(bare.snip_group_fg, t.color_text_tertiary, "snip_group_fg");
        assert_color(bare.snip_name_fg, t.color_text, "snip_name_fg");
        assert_color(bare.snip_cmd_fg, t.color_text_tertiary, "snip_cmd_fg");
        // 两种外观的描边必须真的落在不同元素上
        assert!(
            framed.bar_border.is_some() && framed.input_border.is_none(),
            "{name}: framed 描边位置不符"
        );
        assert!(
            bare.bar_border.is_none() && bare.input_border.is_some(),
            "{name}: bare 描边位置不符"
        );
    }
}

#[test]
fn command_bar_action_matrix() {
    use CommandBarAction as A;
    for name in ["light", "dark", "accent-violet"] {
        let t = tokens_for(name);
        let plain = command_bar_action_style(&t, A::Default);
        assert_frame(&plain.bg, None, "default bg");
        assert_color(plain.border, t.color_border, "default border");
        assert_color(plain.fg, t.color_text_secondary, "default fg");
        assert_frame(
            &plain.hover_bg,
            Some(t.color_primary_bg),
            "default hover_bg",
        );
        assert_color(plain.hover_border, t.color_primary, "default hover_border");
        assert_color(plain.hover_fg, t.color_primary, "default hover_fg");

        let active = command_bar_action_style(&t, A::Active);
        assert_frame(&active.bg, Some(t.color_primary_bg), "active bg");
        assert_color(active.border, t.color_primary, "active border");
        assert_color(active.fg, t.color_primary, "active fg");

        let go = command_bar_action_style(&t, A::Go);
        assert_frame(&go.bg, Some(t.color_primary), "go bg");
        assert_color(go.fg, t.color_text_light_solid, "go fg");
        // 「执行」的悬停往深走，和其余描边按钮相反
        assert_color(go.hover_bg.unwrap(), t.color_primary_active, "go hover_bg");
        assert!(
            luminance(go.bg.unwrap()) > luminance(go.hover_bg.unwrap()),
            "{name}: go 悬停没有变深"
        );

        for kind in [A::Disabled, A::GoDisabled] {
            let d = command_bar_action_style(&t, kind);
            assert_frame(&d.bg, None, "disabled bg");
            assert_frame(&d.hover_bg, None, "disabled hover_bg");
            assert_color(d.border, t.color_border, "disabled border");
            assert_color(d.fg, t.color_text_tertiary, "disabled fg");
            assert!(
                channels(d.fg) == channels(d.hover_fg),
                "disabled 悬停不应改色：{kind:?}"
            );
        }

        // 换主色必须带到主动作上
        let violet = tokens_for("accent-violet");
        assert!(
            channels(command_bar_action_style(&violet, A::Go).bg.unwrap())
                != channels(command_bar_action_style(&t, A::Go).bg.unwrap())
                || name == "accent-violet",
            "{name}: go 底色不随主色"
        );
    }
}

/// 粗略感知亮度，只用于比较同族色的深浅方向。
fn luminance(c: Hsla) -> f32 {
    let rgb = Color::from(c);
    (0.2126 * rgb.r + 0.7152 * rgb.g + 0.0722 * rgb.b) / 255.0
}

#[test]
fn command_bar_snippet_matrix() {
    use gpui_widgets::{CommandSnippet, group_snippets};

    let snip = |name: &str, command: &str, group: &str| -> CommandSnippet {
        CommandSnippet::new(name.to_string(), command.to_string(), group.to_string())
    };
    assert!(
        group_snippets(&[], "未分组").is_empty(),
        "空列表不该造出分组"
    );

    // 分组按首次出现顺序，组内保持输入顺序（原版 computed `groups` 的行为）
    let items = [
        snip("看目录", "ls -la", "文件"),
        snip("状态", "git status", "git"),
        snip("发布", "cargo build --release", "rust"),
        snip("提交", "git commit", "git"),
    ];
    let groups = group_snippets(&items, "未分组");
    let names: Vec<&str> = groups.iter().map(|(g, _)| g.as_ref()).collect();
    assert_eq!(names, ["文件", "git", "rust"], "分组顺序不符");
    assert_eq!(groups[1].1.len(), 2, "同组条目应合并");
    assert_eq!(groups[1].1[0].name, "状态");
    assert_eq!(groups[1].1[1].name, "提交");
    assert_eq!(
        groups[2].1[0].command, "cargo build --release",
        "命令内容不该丢"
    );

    // 空分组统一落到兜底标签，且只出现一次
    let mixed = [
        snip("a", "echo a", ""),
        snip("b", "echo b", "工具"),
        snip("c", "echo c", ""),
    ];
    let groups = group_snippets(&mixed, "未分组");
    let names: Vec<&str> = groups.iter().map(|(g, _)| g.as_ref()).collect();
    assert_eq!(
        names,
        ["未分组", "工具"],
        "空分组应归入兜底标签且按首次出现排序"
    );
    assert_eq!(groups[0].1.len(), 2);
}

#[test]
fn command_bar_snippet_manage_matrix() {
    use gpui_widgets::{CommandSnippet, panel_groups, snippet_can_save, sorted_snippets};

    // 必填校验：去空白后两者都非空（原版 canSave）
    assert!(snippet_can_save("看目录", "ls -la"));
    assert!(snippet_can_save("  a  ", " echo a "));
    assert!(!snippet_can_save("", "ls"));
    assert!(!snippet_can_save("  ", "ls"));
    assert!(!snippet_can_save("a", "   "));
    assert!(!snippet_can_save("a", ""));

    // 默认值：sendEnter 勾选、排序号 0、id 自增不重
    let a = CommandSnippet::new("a", "echo a", "");
    let b = CommandSnippet::new("b", "echo b", "");
    assert!(a.send_enter, "原版 formSendEnter 默认 true");
    assert_eq!(a.sort, 0);
    assert_ne!(a.id, b.id);
    assert!(!a.clone().send_enter(false).send_enter);
    assert_eq!(a.clone().sort(7).sort, 7);

    // 排序：排序号升序，同号按名称字典序（原版 localeCompare 的近似）
    let items = vec![
        CommandSnippet::new("c", "echo c", "g").sort(2),
        CommandSnippet::new("b", "echo b", "g").sort(1),
        CommandSnippet::new("a", "echo a", "g").sort(1),
    ];
    let sorted = sorted_snippets(&items);
    let names: Vec<&str> = sorted.iter().map(|s| s.name.as_ref()).collect();
    assert_eq!(names, ["a", "b", "c"], "排序号升序且同号按名称");

    // 面板分组：未分组永远垫底（哪怕首现最早），组序按排序后的首现
    let items = vec![
        CommandSnippet::new("u1", "echo u1", "").sort(0),
        CommandSnippet::new("g1", "echo g1", "git").sort(1),
        CommandSnippet::new("u2", "echo u2", "").sort(5),
        CommandSnippet::new("f1", "echo f1", "文件").sort(2),
    ];
    let groups = panel_groups(&items, "未分组");
    let names: Vec<&str> = groups.iter().map(|(g, _)| g.as_ref()).collect();
    assert_eq!(names, ["git", "文件", "未分组"], "未分组垫底、其余按首现");
    assert_eq!(groups[2].1.len(), 2, "空分组合并到兜底组");

    // 排序先于分组：排序号改变组的首现顺序
    let items = vec![
        CommandSnippet::new("late", "echo late", "z").sort(9),
        CommandSnippet::new("early", "echo early", "a").sort(0),
    ];
    let groups = panel_groups(&items, "未分组");
    let names: Vec<&str> = groups.iter().map(|(g, _)| g.as_ref()).collect();
    assert_eq!(names, ["a", "z"]);

    // 真有个组叫「未分组」时不被挪底（原版按空串判，不按标签）
    let items = vec![
        CommandSnippet::new("n", "echo n", "未分组").sort(0),
        CommandSnippet::new("u", "echo u", "").sort(1),
    ];
    let groups = panel_groups(&items, "未分组");
    let names: Vec<&str> = groups.iter().map(|(g, _)| g.as_ref()).collect();
    assert_eq!(names, ["未分组", "未分组"], "同名真组保持首现位置");
    assert_eq!(groups[0].1[0].name, "n");
    assert_eq!(groups[1].1[0].name, "u");

    // 空列表不造组
    assert!(panel_groups(&[], "未分组").is_empty());
}
