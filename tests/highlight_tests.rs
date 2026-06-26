use reading_cli::highlight::analyzer::{
    AnalyzerKind, TextAnalyzer, align_words_to_offsets, create_analyzer,
};
use reading_cli::highlight::annotation::Annotation;
use reading_cli::highlight::annotation::AnnotationKind;
use reading_cli::highlight::jieba_analyzer::{JiebaAnalyzer, map_jieba_tag};
use reading_cli::highlight::ltp_legacy_analyzer::LtpLegacyAnalyzer;
use reading_cli::highlight::ltp_legacy_analyzer::{
    LtpLegacyModelPaths, map_ltp_pos_tag, ner_annotations_from_offsets,
};
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
fn maps_ltp_pos_tags_to_highlight_kinds() {
    assert_eq!(map_ltp_pos_tag("n"), Some(AnnotationKind::Noun));
    assert_eq!(map_ltp_pos_tag("nh"), Some(AnnotationKind::Noun));
    assert_eq!(map_ltp_pos_tag("v"), Some(AnnotationKind::Verb));
    assert_eq!(map_ltp_pos_tag("r"), Some(AnnotationKind::Pronoun));
    assert_eq!(map_ltp_pos_tag("d"), Some(AnnotationKind::Adverb));
    assert_eq!(map_ltp_pos_tag("a"), Some(AnnotationKind::Adjective));
    assert_eq!(map_ltp_pos_tag("wp"), None);
}

#[test]
fn ner_annotations_map_single_token_entities() {
    let offsets = vec![(0, 6), (6, 12)];
    let annotations = ner_annotations_from_offsets(&offsets, &["S-Nh", "O"]);

    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].start_offset, 0);
    assert_eq!(annotations[0].end_offset, 6);
    assert_eq!(annotations[0].kind, AnnotationKind::Person);
}

#[test]
fn ner_annotations_merge_begin_inside_end_entities() {
    let offsets = vec![(0, 3), (3, 6), (6, 9), (9, 12)];
    let annotations = ner_annotations_from_offsets(&offsets, &["B-Ns", "I-Ns", "E-Ns", "O"]);

    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].start_offset, 0);
    assert_eq!(annotations[0].end_offset, 9);
    assert_eq!(annotations[0].kind, AnnotationKind::Location);
}

#[test]
fn ltp_legacy_model_paths_are_derived_from_model_dir() {
    let paths = LtpLegacyModelPaths::from_dir(std::path::Path::new(".reading/models/ltp/legacy"));

    assert!(paths.cws.ends_with("cws_model.bin"));
    assert!(paths.pos.ends_with("pos_model.bin"));
    assert!(paths.ner.ends_with("ner_model.bin"));
}

#[test]
fn analyzer_factory_creates_jieba_analyzer() -> anyhow::Result<()> {
    let analyzer = create_analyzer(AnalyzerKind::Jieba)?;

    assert_eq!(analyzer.analyzer_id(), "jieba");

    Ok(())
}

#[test]
fn analyzer_kind_exposes_ltp_legacy_id() {
    assert_eq!(AnalyzerKind::LtpLegacyPos.analyzer_id(), "ltp-pos");
    assert_eq!(AnalyzerKind::LtpLegacyNer.analyzer_id(), "ltp-ner");
}

#[test]
fn analyzer_kind_cycles_between_available_analyzers() {
    assert_eq!(AnalyzerKind::Jieba.next(), AnalyzerKind::LtpLegacyPos);
    assert_eq!(
        AnalyzerKind::LtpLegacyPos.next(),
        AnalyzerKind::LtpLegacyNer
    );
    assert_eq!(AnalyzerKind::LtpLegacyNer.next(), AnalyzerKind::Jieba);
}

#[test]
fn ltp_legacy_analyzer_reports_missing_model_files() {
    let temp_dir = tempdir().expect("tempdir should be created");
    let error = match LtpLegacyAnalyzer::load_from_dir(temp_dir.path()) {
        Ok(_) => panic!("missing model files should fail loading"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("cws_model.bin"));
}

#[test]
#[ignore = "requires local LTP legacy model files in .reading/models/ltp/legacy"]
fn ltp_legacy_analyzer_loads_local_models_and_analyzes_text() -> anyhow::Result<()> {
    let analyzer =
        LtpLegacyAnalyzer::load_from_dir(std::path::Path::new(".reading/models/ltp/legacy"))?;
    let annotations = analyzer.analyze("他看见海。", 0)?;

    assert!(!annotations.is_empty());

    Ok(())
}

#[test]
#[ignore = "requires local LTP legacy model files in .reading/models/ltp/legacy"]
fn ltp_legacy_ner_analyzer_loads_local_models_and_analyzes_text() -> anyhow::Result<()> {
    let analyzer =
        LtpLegacyAnalyzer::load_ner_from_dir(std::path::Path::new(".reading/models/ltp/legacy"))?;
    let _annotations = analyzer.analyze("米切尔在巴黎。", 0)?;

    Ok(())
}

#[test]
fn analyzer_returns_valid_utf8_byte_offsets() -> anyhow::Result<()> {
    let analyzer = JiebaAnalyzer::new();
    let text = "\u{4ed6}\u{6162}\u{6162}\u{5730}\u{6253}\u{5f00}\u{95e8}";
    let annotations = analyzer.analyze(text, 100)?;

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

    Ok(())
}

#[test]
fn annotation_path_is_stable_for_same_book_path() {
    let left = annotation_path_for_book(std::path::Path::new("E:/books/novel.txt"), "jieba");
    let right = annotation_path_for_book(std::path::Path::new("E:/books/novel.txt"), "jieba");

    assert_eq!(left, right);
}

#[test]
fn annotation_path_is_separated_by_analyzer_id() {
    let book_path = std::path::Path::new("E:/books/novel.txt");
    let jieba_path = annotation_path_for_book(book_path, "jieba");
    let ltp_path = annotation_path_for_book(book_path, "ltp-legacy");

    assert_ne!(jieba_path, ltp_path);
    assert!(jieba_path.ends_with("jieba"));
    assert!(ltp_path.ends_with("ltp-legacy"));
}

#[test]
fn align_words_to_offsets_tracks_utf8_byte_offsets() -> anyhow::Result<()> {
    let text = "他看见海";
    let offsets = align_words_to_offsets(text, &["他", "看见", "海"], 100)?;

    assert_eq!(offsets, vec![(100, 103), (103, 109), (109, 112)]);

    Ok(())
}

#[test]
fn align_words_to_offsets_handles_repeated_words_with_cursor() -> anyhow::Result<()> {
    let text = "他看见他";
    let offsets = align_words_to_offsets(text, &["他", "看见", "他"], 0)?;

    assert_eq!(offsets, vec![(0, 3), (3, 9), (9, 12)]);

    Ok(())
}

#[test]
fn align_words_to_offsets_reports_unmatched_words() {
    let error = align_words_to_offsets("他看见海", &["他", "不存在"], 0)
        .expect_err("missing word should fail alignment");

    assert!(error.to_string().contains("cannot align word"));
}

#[test]
fn annotation_cache_loads_chunks_on_demand() -> anyhow::Result<()> {
    let temp_dir = tempdir()?;
    let book_path = temp_dir.path().join("novel.txt");
    let cache_path = temp_dir.path().join("annotations");
    std::fs::write(&book_path, "他打开门。她跑步。他看见海。")?;
    let text_source = TextSource::new(book_path)?;
    let analyzer = JiebaAnalyzer::new();

    let mut built_cache = AnnotationCache::load_or_build(&cache_path, &text_source, 12, &analyzer)?;

    assert!(built_cache.chunk_count() > 1);
    assert_eq!(built_cache.loaded_chunk_count(), 0);

    let annotations = built_cache.query(0, 12)?;

    assert!(!annotations.is_empty());
    assert_eq!(built_cache.loaded_chunk_count(), 1);

    let loaded_cache = AnnotationCache::load_or_build(&cache_path, &text_source, 12, &analyzer)?;

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
