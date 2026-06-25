use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum AnnotationKind {
    Noun,
    Verb,
    Pronoun,
    Adverb,
    Adjective,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Annotation {
    pub start_offset: u64,
    pub end_offset: u64,
    pub kind: AnnotationKind,
}
