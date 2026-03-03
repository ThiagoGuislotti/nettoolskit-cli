use nettoolskit_ui::HistoryViewer;

#[test]
fn history_viewer_filter_supports_case_insensitive_search() {
    let mut viewer = HistoryViewer::new(vec![
        "/help".to_string(),
        "/manifest list".to_string(),
        "plain text".to_string(),
    ]);
    viewer.set_query(Some("MANIFEST".to_string()));
    let filtered = viewer.filtered_entries();
    assert_eq!(filtered, vec!["/manifest list".to_string()]);
}

#[test]
fn history_viewer_rendered_page_is_limited_by_page_size() {
    let viewer =
        HistoryViewer::new((0..10).map(|idx| format!("entry-{idx}")).collect()).with_page_size(4);
    let rendered = viewer.rendered_page_entries();
    assert_eq!(rendered.len(), 4);
}

#[test]
fn history_viewer_scroll_next_page_changes_first_entry_index() {
    let mut viewer =
        HistoryViewer::new((0..10).map(|idx| format!("entry-{idx}")).collect()).with_page_size(3);
    let first_before = viewer.rendered_page_entries()[0].clone();
    viewer.scroll_next_page();
    let first_after = viewer.rendered_page_entries()[0].clone();

    assert_ne!(first_before, first_after);
    assert!(first_after.starts_with("0004"));
}
