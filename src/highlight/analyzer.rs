use std::path::Path;

use anyhow::{Context, Result, anyhow};

use crate::highlight::annotation::Annotation;
use crate::highlight::jieba_analyzer::JiebaAnalyzer;
use crate::highlight::ltp_legacy_analyzer::LtpLegacyAnalyzer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalyzerKind {
    Jieba,
    LtpLegacyPos,
    LtpLegacyNer,
}

/// TextAnalyzer 是高亮分析器的统一接口。
///
/// jieba、LTP 或未来其他 NLP 后端都应该实现这个 trait，
/// 这样缓存和渲染层不需要关心具体分析器来自哪里。
pub trait TextAnalyzer {
    fn analyzer_id(&self) -> &'static str;

    fn analyze(&self, text: &str, base_offset: u64) -> Result<Vec<Annotation>>;
}

impl AnalyzerKind {
    pub fn analyzer_id(self) -> &'static str {
        match self {
            AnalyzerKind::Jieba => "jieba",
            AnalyzerKind::LtpLegacyPos => "ltp-pos",
            AnalyzerKind::LtpLegacyNer => "ltp-ner",
        }
    }

    pub fn next(self) -> Self {
        match self {
            AnalyzerKind::Jieba => AnalyzerKind::LtpLegacyPos,
            AnalyzerKind::LtpLegacyPos => AnalyzerKind::LtpLegacyNer,
            AnalyzerKind::LtpLegacyNer => AnalyzerKind::Jieba,
        }
    }
}

pub fn create_analyzer(kind: AnalyzerKind) -> Result<Box<dyn TextAnalyzer>> {
    match kind {
        AnalyzerKind::Jieba => Ok(Box::new(JiebaAnalyzer::new())),
        AnalyzerKind::LtpLegacyPos => Ok(Box::new(
            LtpLegacyAnalyzer::load_pos_from_dir(Path::new(".reading/models/ltp/legacy"))
                .context("failed to load ltp legacy analyzer")?,
        )),
        AnalyzerKind::LtpLegacyNer => Ok(Box::new(
            LtpLegacyAnalyzer::load_ner_from_dir(Path::new(".reading/models/ltp/legacy"))
                .context("failed to load ltp legacy analyzer")?,
        )),
    }
}

pub fn align_words_to_offsets(
    text: &str,
    words: &[&str],
    base_offset: u64,
) -> Result<Vec<(u64, u64)>> {
    let mut cursor = 0usize;
    let mut offsets = Vec::with_capacity(words.len());

    for word in words {
        let Some(relative_start) = text[cursor..].find(word) else {
            return Err(anyhow!("cannot align word: {word}"));
        };

        let start = cursor + relative_start;
        let end = start + word.len();

        offsets.push((base_offset + start as u64, base_offset + end as u64));
        cursor = end;
    }

    Ok(offsets)
}
