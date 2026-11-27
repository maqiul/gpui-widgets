use gpui_widgets::{DiffKind, diff_lines, diff_stats};

fn kinds(lines: &[gpui_widgets::DiffLine]) -> Vec<(DiffKind, Option<usize>, Option<usize>, &str)> {
    lines
        .iter()
        .map(|line| (line.kind, line.old_line, line.new_line, line.text.as_str()))
        .collect()
}

fn assert_reconstructs(old: &str, new: &str) {
    let lines = diff_lines(old, new);
    let old_source: Vec<&str> = old.lines().collect();
    let new_source: Vec<&str> = new.lines().collect();
    let mut old_result = Vec::new();
    let mut new_result = Vec::new();
    let mut previous_old = 0;
    let mut previous_new = 0;
    for line in &lines {
        if let Some(number) = line.old_line {
            assert!(number > previous_old);
            assert_eq!(
                old_source[number - 1],
                line.text,
                "old mismatch at {number}: old={old:?} new={new:?} lines={lines:?}"
            );
            previous_old = number;
            old_result.push(line.text.as_str());
        }
        if let Some(number) = line.new_line {
            assert!(number > previous_new);
            assert_eq!(
                new_source[number - 1],
                line.text,
                "new mismatch at {number}: old={old:?} new={new:?} lines={lines:?}"
            );
            previous_new = number;
            new_result.push(line.text.as_str());
        }
        match line.kind {
            DiffKind::Equal => assert!(line.old_line.is_some() && line.new_line.is_some()),
            DiffKind::Added => assert!(line.old_line.is_none() && line.new_line.is_some()),
            DiffKind::Removed => assert!(line.old_line.is_some() && line.new_line.is_none()),
        }
    }
    assert_eq!(old_result, old_source);
    assert_eq!(new_result, new_source);
}

#[test]
fn diff_identical_and_empty() {
    assert!(diff_lines("", "").is_empty());
    assert_eq!(
        kinds(&diff_lines("same\n", "same")),
        vec![(DiffKind::Equal, Some(1), Some(1), "same")]
    );
    assert_eq!(
        kinds(&diff_lines("a\nb", "a\nb")),
        vec![
            (DiffKind::Equal, Some(1), Some(1), "a"),
            (DiffKind::Equal, Some(2), Some(2), "b"),
        ]
    );
}

#[test]
fn diff_insertion_before_first_line() {
    assert_eq!(
        kinds(&diff_lines("a", "b\na")),
        vec![
            (DiffKind::Added, None, Some(1), "b"),
            (DiffKind::Equal, Some(1), Some(2), "a"),
        ]
    );
}

#[test]
fn diff_insertion_before_changed_region() {
    assert_eq!(
        kinds(&diff_lines("a\nx", "b\na\ny")),
        vec![
            (DiffKind::Added, None, Some(1), "b"),
            (DiffKind::Equal, Some(1), Some(2), "a"),
            (DiffKind::Removed, Some(2), None, "x"),
            (DiffKind::Added, None, Some(3), "y"),
        ]
    );
}

#[test]
fn diff_insert_delete_replace() {
    assert_eq!(
        kinds(&diff_lines("a\nc", "a\nb\nc")),
        vec![
            (DiffKind::Equal, Some(1), Some(1), "a"),
            (DiffKind::Added, None, Some(2), "b"),
            (DiffKind::Equal, Some(2), Some(3), "c"),
        ]
    );
    assert_eq!(
        kinds(&diff_lines("a\nb\nc", "a\nc")),
        vec![
            (DiffKind::Equal, Some(1), Some(1), "a"),
            (DiffKind::Removed, Some(2), None, "b"),
            (DiffKind::Equal, Some(3), Some(2), "c"),
        ]
    );
    assert_eq!(
        kinds(&diff_lines("a\nold\nc", "a\nnew\nc")),
        vec![
            (DiffKind::Equal, Some(1), Some(1), "a"),
            (DiffKind::Removed, Some(2), None, "old"),
            (DiffKind::Added, None, Some(2), "new"),
            (DiffKind::Equal, Some(3), Some(3), "c"),
        ]
    );
}

#[test]
fn diff_blank_lines_unicode_and_stats() {
    let old = "标题\n\n旧文本\n";
    let new = "标题\n\n新文本\n👩‍💻\n";
    let lines = diff_lines(old, new);
    assert_eq!(
        diff_stats(&lines),
        gpui_widgets::DiffStats {
            added: 2,
            removed: 1
        }
    );
    assert_reconstructs(old, new);
    assert!(diff_lines("\n", "").iter().any(|line| line.text.is_empty()));
}

#[test]
fn diff_randomized_reconstruction() {
    let mut seed = 0x4d595df4d0f33173u64;
    let alphabet = ["a", "b", "c", "d", "", "中文", "👩‍💻"];
    for _ in 0..600 {
        let mut source = String::new();
        for _ in 0..(seed as usize % 45) {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            source.push_str(alphabet[seed as usize % alphabet.len()]);
            source.push('\n');
        }
        let mut target = String::new();
        for _ in 0..((seed >> 11) as usize % 45) {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            target.push_str(alphabet[seed as usize % alphabet.len()]);
            target.push('\n');
        }
        assert_reconstructs(&source, &target);
    }
}

#[test]
fn diff_large_common_regions_and_disjoint_replacement() {
    let shared = (0..4000).map(|i| format!("line {i}\n")).collect::<String>();
    let old = format!("old head\n{shared}old tail\n");
    let new = format!("new head\n{shared}new tail\n");
    let lines = diff_lines(&old, &new);
    assert_reconstructs(&old, &new);
    assert_eq!(diff_stats(&lines).added, 2);
    assert_eq!(diff_stats(&lines).removed, 2);

    let old = (0..3000).map(|i| format!("old {i}\n")).collect::<String>();
    let new = (0..3000).map(|i| format!("new {i}\n")).collect::<String>();
    let lines = diff_lines(&old, &new);
    assert_eq!(diff_stats(&lines).added, 3000);
    assert_eq!(diff_stats(&lines).removed, 3000);
}
