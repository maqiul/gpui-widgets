use gpui_widgets::feedback::notification_center::{
    notification_center_row_height, notification_center_target, unread_count,
};
use gpui_widgets::theme::{Appearance, Seed, Tokens, derive, metric::notification_metric};
use gpui_widgets::{NotificationItem, NotificationVariant};

fn tokens(appearance: Appearance) -> Tokens {
    derive(&Seed::default(), appearance)
}

fn items() -> Vec<NotificationItem> {
    vec![
        NotificationItem::new("构建完成")
            .success()
            .description("main 分支已生成新产物")
            .timestamp("10:42")
            .read(false),
        NotificationItem::new("有新评论")
            .description("林墨回复了你的评审")
            .timestamp("10:35")
            .read(false),
        NotificationItem::new("磁盘空间不足")
            .warning()
            .description("剩余空间低于 10%")
            .timestamp("09:58")
            .read(true),
        NotificationItem::new("同步失败")
            .danger()
            .description("请检查网络后重试")
            .timestamp("09:12")
            .read(false),
        NotificationItem::new("欢迎使用")
            .description("通知会保留在这里")
            .read(true),
    ]
}

#[test]
fn notification_center_unread_count_tracks_item_state() {
    let mut items = items();
    assert_eq!(unread_count(&items), 3);
    assert_eq!(unread_count(&[]), 0);

    items[0].set_read(true);
    assert_eq!(unread_count(&items), 2);
    items[0].set_read(false);
    assert_eq!(unread_count(&items), 3);
}

#[test]
fn notification_center_item_accessors_preserve_data() {
    let item = &items()[0];
    assert_eq!(item.title().as_ref(), "构建完成");
    assert_eq!(
        item.description_text().unwrap().as_ref(),
        "main 分支已生成新产物"
    );
    assert_eq!(item.timestamp_text().unwrap().as_ref(), "10:42");
    assert_eq!(item.variant(), NotificationVariant::Success);
    assert!(!item.is_read());
    assert!(NotificationItem::new("纯标题").description_text().is_none());
}

#[test]
fn notification_center_navigation_wraps_and_clamps() {
    assert_eq!(notification_center_target(None, "down", 4), Some(0));
    assert_eq!(notification_center_target(None, "up", 4), Some(3));
    assert_eq!(notification_center_target(Some(0), "up", 4), Some(3));
    assert_eq!(notification_center_target(Some(3), "down", 4), Some(0));
    assert_eq!(notification_center_target(Some(2), "home", 4), Some(0));
    assert_eq!(notification_center_target(Some(2), "end", 4), Some(3));
    assert_eq!(notification_center_target(Some(0), "enter", 4), None);
    assert_eq!(notification_center_target(None, "down", 0), None);
    assert_eq!(notification_center_target(Some(0), "up", 1), Some(0));
}

#[test]
fn notification_center_row_height_fits_title_and_description() {
    for appearance in [Appearance::Light, Appearance::Dark] {
        let tokens = tokens(appearance);
        let metric = notification_metric(&tokens);
        let height = notification_center_row_height(&tokens);
        assert!(height > 0.);
        assert!((height - (metric.title_line + metric.line + tokens.padding_sm)).abs() < 1e-6);
        assert!(height >= metric.title_line + metric.line);
    }
}
