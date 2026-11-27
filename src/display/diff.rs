//! 行级代码差异：Myers 最短编辑路径、统一/分栏视图。

use std::collections::HashSet;

use gpui::{
    Div, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div, prelude::*, px,
};

use crate::{a11y::Semantics, theme};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffKind {
    Equal,
    Added,
    Removed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: DiffKind,
    pub old_line: Option<usize>,
    pub new_line: Option<usize>,
    pub text: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DiffStats {
    pub added: usize,
    pub removed: usize,
}

pub fn diff_stats(lines: &[DiffLine]) -> DiffStats {
    lines.iter().fold(DiffStats::default(), |mut stats, line| {
        match line.kind {
            DiffKind::Added => stats.added += 1,
            DiffKind::Removed => stats.removed += 1,
            DiffKind::Equal => {}
        }
        stats
    })
}

fn removed(text: &str, old_line: usize) -> DiffLine {
    DiffLine {
        kind: DiffKind::Removed,
        old_line: Some(old_line),
        new_line: None,
        text: text.to_string(),
    }
}

fn added(text: &str, new_line: usize) -> DiffLine {
    DiffLine {
        kind: DiffKind::Added,
        old_line: None,
        new_line: Some(new_line),
        text: text.to_string(),
    }
}

fn equal(text: &str, old_line: usize, new_line: usize) -> DiffLine {
    DiffLine {
        kind: DiffKind::Equal,
        old_line: Some(old_line),
        new_line: Some(new_line),
        text: text.to_string(),
    }
}

/// 计算两段文本的最短行级差异；行号从 1 开始。
///
/// 先移除公共前后缀，再执行 Myers。完全不共享行的替换直接返回，避免 O(D²)
/// 轨迹内存；共享行很少的极端输入保留 Myers 的最短路径结果。
pub fn diff_lines(old: &str, new: &str) -> Vec<DiffLine> {
    let old: Vec<&str> = old.lines().collect();
    let new: Vec<&str> = new.lines().collect();
    let prefix = old.iter().zip(&new).take_while(|(a, b)| a == b).count();
    let suffix = old[prefix..]
        .iter()
        .rev()
        .zip(new[prefix..].iter().rev())
        .take_while(|(a, b)| a == b)
        .count();
    let old_end = old.len() - suffix;
    let new_end = new.len() - suffix;
    let old_mid = &old[prefix..old_end];
    let new_mid = &new[prefix..new_end];

    let new_set: HashSet<&str> = new_mid.iter().copied().collect();
    let disjoint = !old_mid.iter().any(|line| new_set.contains(line));

    let mut out = Vec::with_capacity(old.len().max(new.len()) + prefix + suffix);
    for (index, line) in old[..prefix].iter().enumerate() {
        out.push(equal(line, index + 1, index + 1));
    }
    if disjoint {
        out.extend(
            old_mid
                .iter()
                .enumerate()
                .map(|(index, line)| removed(line, prefix + index + 1)),
        );
        out.extend(
            new_mid
                .iter()
                .enumerate()
                .map(|(index, line)| added(line, prefix + index + 1)),
        );
    } else {
        let middle = myers(old_mid, new_mid);
        out.extend(middle.into_iter().map(|mut line| {
            if let Some(number) = &mut line.old_line {
                *number += prefix;
            }
            if let Some(number) = &mut line.new_line {
                *number += prefix;
            }
            line
        }));
    }
    for (index, line) in old[old_end..].iter().enumerate() {
        out.push(equal(line, old_end + index + 1, new_end + index + 1));
    }
    out
}

fn myers(old: &[&str], new: &[&str]) -> Vec<DiffLine> {
    if old.is_empty() {
        return new
            .iter()
            .enumerate()
            .map(|(index, line)| added(line, index + 1))
            .collect();
    }
    if new.is_empty() {
        return old
            .iter()
            .enumerate()
            .map(|(index, line)| removed(line, index + 1))
            .collect();
    }
    let n = old.len() as isize;
    let m = new.len() as isize;
    let max = (n + m) as usize;
    let offset = max + 1;
    let mut frontier = vec![0isize; 2 * max + 3];
    let mut trace = Vec::with_capacity(max + 1);

    for depth in 0..=max {
        let previous = frontier.clone();
        let mut parents = vec![0isize; 2 * max + 3];
        for diagonal in (-(depth as isize)..=(depth as isize)).step_by(2) {
            let index = (diagonal + offset as isize) as usize;
            let down = diagonal == -(depth as isize)
                || (diagonal != depth as isize && previous[index - 1] < previous[index + 1]);
            parents[index] = if down { diagonal + 1 } else { diagonal - 1 };
            frontier[index] = if down {
                previous[index + 1]
            } else {
                previous[index - 1] + 1
            };
            let mut x = frontier[index];
            let mut y = x - diagonal;
            while x < n && y < m && old[x as usize] == new[y as usize] {
                x += 1;
                y += 1;
            }
            frontier[index] = x;
        }
        trace.push((frontier.clone(), parents));
        let diagonal = n - m;
        if frontier[(diagonal + offset as isize) as usize] >= n {
            return backtrack(old, new, &trace);
        }
    }
    unreachable!()
}

fn backtrack(old: &[&str], new: &[&str], trace: &[(Vec<isize>, Vec<isize>)]) -> Vec<DiffLine> {
    let offset = trace[0].0.len() / 2;
    let mut x = old.len() as isize;
    let mut y = new.len() as isize;
    let mut out = Vec::with_capacity(old.len() + new.len());

    for depth in (0..trace.len()).rev() {
        let diagonal = x - y;
        let previous = trace[depth].1[(diagonal + offset as isize) as usize];
        let previous_x = if depth == 0 {
            0
        } else {
            trace[depth - 1].0[(previous + offset as isize) as usize]
        };
        let previous_y = previous_x - previous;
        while x > previous_x && y > previous_y {
            x -= 1;
            y -= 1;
            out.push(equal(old[x as usize], x as usize + 1, y as usize + 1));
        }
        if depth == 0 {
            break;
        }
        if previous == diagonal + 1 {
            y -= 1;
            out.push(added(new[y as usize], y as usize + 1));
        } else {
            x -= 1;
            out.push(removed(old[x as usize], x as usize + 1));
        }
    }
    out.reverse();
    out
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DiffMode {
    #[default]
    Split,
    Unified,
}

#[derive(Clone, Copy)]
struct DiffColors {
    added_bg: gpui::Hsla,
    removed_bg: gpui::Hsla,
    added_text: gpui::Hsla,
    removed_text: gpui::Hsla,
    border: gpui::Hsla,
    muted: gpui::Hsla,
    text: gpui::Hsla,
    surface: gpui::Hsla,
}

impl DiffColors {
    fn resolve() -> Self {
        Self {
            added_bg: theme::success_bg(),
            removed_bg: theme::danger_bg(),
            added_text: theme::success(),
            removed_text: theme::danger(),
            border: theme::border_secondary(),
            muted: theme::text_tertiary(),
            text: theme::text(),
            surface: theme::bg_container(),
        }
    }

    fn for_kind(self, kind: DiffKind) -> (gpui::Hsla, gpui::Hsla) {
        match kind {
            DiffKind::Added => (self.added_bg, self.added_text),
            DiffKind::Removed => (self.removed_bg, self.removed_text),
            DiffKind::Equal => (self.surface, self.text),
        }
    }
}

#[derive(IntoElement)]
pub struct DiffView {
    id: SharedString,
    old: SharedString,
    new: SharedString,
    title: SharedString,
    mode: DiffMode,
    line_numbers: bool,
}

impl DiffView {
    #[track_caller]
    pub fn new(old: impl Into<SharedString>, new: impl Into<SharedString>) -> Self {
        let location = core::panic::Location::caller();
        Self {
            id: format!(
                "diff-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            old: old.into(),
            new: new.into(),
            title: "代码差异".into(),
            mode: DiffMode::default(),
            line_numbers: true,
        }
    }

    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = id.into();
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }

    pub fn mode(mut self, mode: DiffMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn line_numbers(mut self, show: bool) -> Self {
        self.line_numbers = show;
        self
    }

    fn cell(&self, line: Option<&DiffLine>, side: &str, colors: DiffColors) -> Div {
        let Some(line) = line else {
            return div().flex_1().h(px(24.)).bg(colors.surface);
        };
        let (background, foreground) = colors.for_kind(line.kind);
        let number = if side == "old" {
            line.old_line
        } else {
            line.new_line
        };
        let marker = match line.kind {
            DiffKind::Added => "+",
            DiffKind::Removed => "−",
            DiffKind::Equal => " ",
        };
        let label = if line.text.is_empty() {
            " "
        } else {
            &line.text
        };
        div()
            .flex()
            .flex_1()
            .min_w_0()
            .h(px(24.))
            .items_center()
            .bg(background)
            .border_color(colors.border)
            .child(
                div()
                    .w(px(20.))
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(foreground)
                    .child(marker),
            )
            .when(self.line_numbers, |cell| {
                cell.child(
                    div()
                        .w(px(42.))
                        .h_full()
                        .flex()
                        .items_center()
                        .justify_end()
                        .pr(px(6.))
                        .text_color(colors.muted)
                        .child(number.map(|n| n.to_string()).unwrap_or_default()),
                )
            })
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_color(foreground)
                    .child(label.to_string()),
            )
    }
}

impl RenderOnce for DiffView {
    fn render(self, _window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        let lines = diff_lines(&self.old, &self.new);
        let stats = diff_stats(&lines);
        let colors = DiffColors::resolve();
        let root_id = self.id.clone();
        let mut body = div().flex().flex_col().w_full();
        let title = self.title.clone();
        let label = format!(
            "{title}，新增 {} 行，删除 {} 行",
            stats.added, stats.removed
        );

        if self.mode == DiffMode::Unified {
            for (row, line) in lines.iter().enumerate() {
                let kind = match line.kind {
                    DiffKind::Added => "新增",
                    DiffKind::Removed => "删除",
                    DiffKind::Equal => "未变",
                };
                let semantics =
                    Semantics::new(gpui::Role::ListItem, format!("{kind}：{}", line.text))
                        .id(format!("{root_id}-row-{row}"));
                body = body.child(
                    semantics.apply(
                        div()
                            .id(SharedString::from(format!("{root_id}-row-{row}")))
                            .flex()
                            .w_full()
                            .child(self.cell(Some(line), "new", colors)),
                    ),
                );
            }
        } else {
            let mut index = 0;
            let mut row = 0;
            while index < lines.len() {
                if lines[index].kind == DiffKind::Equal {
                    let line = &lines[index];
                    let semantics = Semantics::new(gpui::Role::ListItem, line.text.clone())
                        .id(format!("{root_id}-row-{row}"));
                    body = body.child(
                        semantics.apply(
                            div()
                                .id(SharedString::from(format!("{root_id}-row-{row}")))
                                .flex()
                                .w_full()
                                .child(self.cell(Some(line), "old", colors))
                                .child(self.cell(Some(line), "new", colors)),
                        ),
                    );
                    index += 1;
                } else {
                    let start = index;
                    while index < lines.len() && lines[index].kind != DiffKind::Equal {
                        index += 1;
                    }
                    let changed = &lines[start..index];
                    let removed_lines: Vec<&DiffLine> = changed
                        .iter()
                        .filter(|line| line.kind == DiffKind::Removed)
                        .collect();
                    let added_lines: Vec<&DiffLine> = changed
                        .iter()
                        .filter(|line| line.kind == DiffKind::Added)
                        .collect();
                    for offset in 0..removed_lines.len().max(added_lines.len()) {
                        let left = removed_lines.get(offset).copied();
                        let right = added_lines.get(offset).copied();
                        let description = match (left, right) {
                            (Some(left), Some(right)) => {
                                format!("- {}\n+ {}", left.text, right.text)
                            }
                            (Some(left), None) => format!("- {}", left.text),
                            (None, Some(right)) => format!("+ {}", right.text),
                            (None, None) => unreachable!(),
                        };
                        let semantics = Semantics::new(gpui::Role::ListItem, description)
                            .id(format!("{root_id}-row-{row}"));
                        body = body.child(
                            semantics.apply(
                                div()
                                    .id(SharedString::from(format!("{root_id}-row-{row}")))
                                    .flex()
                                    .w_full()
                                    .child(self.cell(left, "old", colors))
                                    .child(self.cell(right, "new", colors)),
                            ),
                        );
                        row += 1;
                    }
                    continue;
                }
                row += 1;
            }
        }

        Semantics::new(gpui::Role::Group, label)
            .id(root_id.clone())
            .apply(
                div()
                    .id(root_id)
                    .flex()
                    .flex_col()
                    .w_full()
                    .overflow_hidden()
                    .rounded_lg()
                    .border_1()
                    .border_color(colors.border)
                    .bg(colors.surface)
                    .font_family(theme::mono_family_or("monospace"))
                    .text_size(px(12.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .h(px(34.))
                            .px_3()
                            .border_b_1()
                            .border_color(colors.border)
                            .bg(theme::bg_elevated())
                            .child(title)
                            .child(
                                div()
                                    .text_color(colors.added_text)
                                    .child(format!("+{}", stats.added)),
                            )
                            .child(
                                div()
                                    .text_color(colors.removed_text)
                                    .child(format!("−{}", stats.removed)),
                            ),
                    )
                    .child(body),
            )
    }
}
