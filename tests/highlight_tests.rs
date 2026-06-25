use reading_cli::highlight::annotation::Annotation;
use reading_cli::highlight::annotation::AnnotationKind;
use reading_cli::highlight::jieba_analyzer::{JiebaAnalyzer, map_jieba_tag};
use reading_cli::highlight::render::render_highlighted_page;
use reading_cli::highlight::store::{AnnotationCache, annotation_path_for_book};
use reading_cli::text_source::TextSource;
use tempfile::tempdir;

#[test]
fn maps_jieba_tags_to_highlight_kinds() {
    assert_eq!(map_jieba_tag("n"), Some(AnnotationKind::Noun));
    assert_eq!(map_jieba_tag("nr"), Some(AnnotationKind::Noun));
    assert_eq!(map_jieba_tag("v"), Some(AnnotationKind::Verb));
    assert_eq!(map_jieba_tag("r"), Some(AnnotationKind::Pronoun));
    assert_eq!(map_jieba_tag("d"), Some(AnnotationKind::Adverb));
    assert_eq!(map_jieba_tag("a"), Some(AnnotationKind::Adjective));
    assert_eq!(map_jieba_tag("ul"), None);
}

#[test]
fn analyzer_returns_valid_utf8_byte_offsets() {
    let analyzer = JiebaAnalyzer::new();
    let text = "\u{4ed6}\u{6162}\u{6162}\u{5730}\u{6253}\u{5f00}\u{95e8}";
    let annotations = analyzer.analyze(text, 100);

    assert!(!annotations.is_empty());

    for annotation in annotations {
        assert!(annotation.start_offset >= 100);
        assert!(annotation.end_offset <= 100 + text.len() as u64);
        assert!(annotation.start_offset < annotation.end_offset);

        let local_start = (annotation.start_offset - 100) as usize;
        let local_end = (annotation.end_offset - 100) as usize;
        assert!(text.is_char_boundary(local_start));
        assert!(text.is_char_boundary(local_end));
    }
}

#[test]
fn annotation_path_is_stable_for_same_book_path() {
    let left = annotation_path_for_book(std::path::Path::new("E:/books/novel.txt"));
    let right = annotation_path_for_book(std::path::Path::new("E:/books/novel.txt"));

    assert_eq!(left, right);
}

#[test]
fn annotation_cache_loads_chunks_on_demand() -> anyhow::Result<()> {
    let temp_dir = tempdir()?;
    let book_path = temp_dir.path().join("novel.txt");
    let cache_path = temp_dir.path().join("annotations");
    std::fs::write(&book_path, "他打开门。她跑步。他看见海。")?;
    let text_source = TextSource::new(book_path)?;

    let mut built_cache = AnnotationCache::load_or_build(&cache_path, &text_source, 12)?;

    assert!(built_cache.chunk_count() > 1);
    assert_eq!(built_cache.loaded_chunk_count(), 0);

    let annotations = built_cache.query(0, 12)?;

    assert!(!annotations.is_empty());
    assert_eq!(built_cache.loaded_chunk_count(), 1);

    let loaded_cache = AnnotationCache::load_or_build(&cache_path, &text_source, 12)?;

    assert_eq!(loaded_cache.chunk_count(), built_cache.chunk_count());
    assert_eq!(loaded_cache.loaded_chunk_count(), 0);

    Ok(())
}

#[test]
fn highlighted_render_keeps_all_visible_text() {
    let text = "\u{4ed6}\u{6253}\u{5f00}\u{95e8}";
    let annotations = vec![Annotation {
        start_offset: 3,
        end_offset: 9,
        kind: AnnotationKind::Verb,
    }];

    let page = render_highlighted_page(text, 0, 20, 2, &annotations);
    let rendered = page
        .lines
        .iter()
        .flat_map(|line| line.spans.iter())
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert_eq!(rendered, text);
    assert_eq!(page.end_offset, text.len() as u64);
}

#[test]
fn highlighted_render_does_not_embed_newline_inside_ratatui_lines() {
    let page = render_highlighted_page("第一行\n第二行\n第三行", 0, 20, 2, &[]);

    assert_eq!(page.lines.len(), 2);
    assert!(
        page.lines
            .iter()
            .flat_map(|line| line.spans.iter())
            .all(|span| !span.content.contains('\n'))
    );
}

#[test]
fn highlighted_render_does_not_embed_carriage_return_inside_ratatui_lines() {
    let page = render_highlighted_page("第一行\r\n第二行\r\n第三行", 0, 20, 2, &[]);

    assert_eq!(page.lines.len(), 2);
    assert!(
        page.lines
            .iter()
            .flat_map(|line| line.spans.iter())
            .all(|span| !span.content.contains('\r'))
    );
}

#[test]
fn highlighted_render_treats_crlf_as_a_single_line_break() {
    let page = render_highlighted_page("第一行\r\n第二行", 0, 20, 3, &[]);
    let rendered_lines = page
        .lines
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>()
        })
        .collect::<Vec<_>>();

    assert_eq!(rendered_lines, vec!["第一行", "第二行"]);
}

#[test]
fn highlighted_render_merges_adjacent_text_with_same_style() {
    let page = render_highlighted_page("abcdef", 0, 20, 2, &[]);

    assert_eq!(page.lines.len(), 1);
    assert_eq!(page.lines[0].spans.len(), 1);
    assert_eq!(page.lines[0].spans[0].content.as_ref(), "abcdef");
}
