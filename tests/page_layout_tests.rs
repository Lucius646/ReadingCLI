use reading_cli::page_layout::layout_page;

#[test]
fn layout_page_wraps_wide_characters_by_termianl_columns() {
    let page = layout_page("一二三ABC", 0, 4, 2);

    assert_eq!(page.text, "一二\n三AB");
    assert_eq!(page.end_offset, "一二三AB".len() as u64);
}