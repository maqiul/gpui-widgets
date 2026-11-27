use gpui::SharedString;
use gpui_widgets::{
    TreeTableNode, tree_table_cascade_check, tree_table_check_state, tree_table_navigation_target,
    tree_table_toggle_expanded, tree_table_visible_ids,
};

fn id(value: &str) -> SharedString {
    value.into()
}

fn nodes() -> Vec<TreeTableNode> {
    vec![TreeTableNode::new("root", ["项目"]).children([
        TreeTableNode::new("src", ["src", "Rust"]).children([
            TreeTableNode::new("main", ["main.rs", "入口"]),
            TreeTableNode::new("lib", ["lib.rs", "库"]),
        ]),
        TreeTableNode::new("docs", ["docs", "文档"]),
    ])]
}

#[test]
fn tree_table_visible_ids_follow_depth_first_expansion_order() {
    let nodes = nodes();
    assert_eq!(
        tree_table_visible_ids(&nodes, &[id("root")]),
        vec![id("root"), id("src"), id("docs")]
    );
    assert_eq!(
        tree_table_visible_ids(&nodes, &[id("root"), id("src")]),
        vec![id("root"), id("src"), id("main"), id("lib"), id("docs")]
    );
    assert_eq!(
        tree_table_visible_ids(&nodes, &[id("src")]),
        vec![id("root")]
    );
}

#[test]
fn tree_table_expansion_toggle_is_stable_and_removes_duplicates() {
    let mut expanded = vec![id("src"), id("root")];
    expanded = tree_table_toggle_expanded(&expanded, &id("root"));
    assert_eq!(expanded, vec![id("src")]);
    expanded = tree_table_toggle_expanded(&expanded, &id("docs"));
    assert_eq!(expanded, vec![id("docs"), id("src")]);
}

#[test]
fn tree_table_keyboard_navigation_clamps_at_edges() {
    assert_eq!(tree_table_navigation_target(None, "down", 3), Some(0));
    assert_eq!(tree_table_navigation_target(None, "up", 3), Some(2));
    assert_eq!(tree_table_navigation_target(Some(0), "up", 3), Some(0));
    assert_eq!(tree_table_navigation_target(Some(2), "down", 3), Some(2));
    assert_eq!(tree_table_navigation_target(Some(1), "home", 3), Some(0));
    assert_eq!(tree_table_navigation_target(Some(1), "end", 3), Some(2));
    assert_eq!(tree_table_navigation_target(Some(1), "left", 3), None);
    assert_eq!(tree_table_navigation_target(Some(1), "down", 0), None);
}

#[test]
fn tree_table_checkbox_cascades_descendants_and_ancestors() {
    let nodes = nodes();
    let all = tree_table_cascade_check(&nodes, &[], &id("root"), true);
    assert_eq!(all.len(), 5);
    assert!(all.contains(&id("main")));
    assert!(all.contains(&id("docs")));

    let empty = tree_table_cascade_check(&nodes, &all, &id("root"), false);
    assert!(empty.is_empty());

    let main = tree_table_cascade_check(&nodes, &[], &id("main"), true);
    assert_eq!(
        tree_table_check_state(&nodes, &main, &id("src")),
        (false, true)
    );
    assert_eq!(
        tree_table_check_state(&nodes, &main, &id("root")),
        (false, true)
    );

    let main_and_lib = tree_table_cascade_check(&nodes, &main, &id("lib"), true);
    assert_eq!(
        tree_table_check_state(&nodes, &main_and_lib, &id("src")),
        (true, false)
    );
    assert_eq!(
        tree_table_check_state(&nodes, &main_and_lib, &id("root")),
        (false, true)
    );
}

#[test]
fn tree_table_unknown_node_leaves_check_state_unchanged() {
    let nodes = nodes();
    let checked = vec![id("main")];
    assert_eq!(
        tree_table_cascade_check(&nodes, &checked, &id("missing"), true),
        checked
    );
    assert_eq!(
        tree_table_check_state(&nodes, &checked, &id("missing")),
        (false, false)
    );
}
