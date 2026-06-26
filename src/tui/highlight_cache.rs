use std::path::Path;

use anyhow::Result;

use crate::highlight::analyzer::{AnalyzerKind, create_analyzer};
use crate::highlight::store::{AnnotationCache, annotation_path_for_book};
use crate::text_source::TextSource;

const ANNOTATION_CHUNK_SIZE: usize = 64 * 1024;

/// 为当前书籍加载或构建高亮标注缓存。
pub(super) fn load_or_build_annotation_cache(
    book_path: &Path,
    text_source: &TextSource,
    analyzer_kind: AnalyzerKind,
) -> Result<AnnotationCache> {
    let analyzer = create_analyzer(analyzer_kind)?;
    let annotation_path = annotation_path_for_book(book_path, analyzer.analyzer_id());
    AnnotationCache::load_or_build(
        &annotation_path,
        text_source,
        ANNOTATION_CHUNK_SIZE,
        analyzer.as_ref(),
    )
}
