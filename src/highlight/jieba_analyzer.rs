use jieba_rs::Jieba;

use crate::highlight::annotation::{Annotation, AnnotationKind};

pub struct JiebaAnalyzer {
    jieba: Jieba,
}

impl JiebaAnalyzer {
    // Create a jieba analyzer with the default embedded dictionary.
    pub fn new() -> Self {
        Self {
            jieba: Jieba::new(),
        }
    }

    // Convert jieba POS tags into byte-offset annotations for later rendering.
    pub fn analyze(&self, text: &str, base_offset: u64) -> Vec<Annotation> {
        self.jieba
            .tag(text, true)
            .into_iter()
            .filter_map(|tag| {
                let kind = map_jieba_tag(tag.tag)?;

                if tag.byte_start >= tag.byte_end {
                    return None;
                }

                Some(Annotation {
                    start_offset: base_offset + tag.byte_start as u64,
                    end_offset: base_offset + tag.byte_end as u64,
                    kind,
                })
            })
            .collect()
    }
}

impl Default for JiebaAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

pub fn map_jieba_tag(tag: &str) -> Option<AnnotationKind> {
    match tag {
        "n" | "nr" | "ns" | "nt" | "nz" => Some(AnnotationKind::Noun),
        "v" | "vd" | "vn" => Some(AnnotationKind::Verb),
        "r" => Some(AnnotationKind::Pronoun),
        "d" => Some(AnnotationKind::Adverb),
        "a" | "ad" | "an" => Some(AnnotationKind::Adjective),
        _ => None,
    }
}
