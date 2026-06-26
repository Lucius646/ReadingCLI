use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ltp::{CWSModel, Codec, Format, ModelSerde, NERModel, POSModel};

use crate::highlight::analyzer::{TextAnalyzer, align_words_to_offsets};
use crate::highlight::annotation::Annotation;
use crate::highlight::annotation::AnnotationKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LtpLegacyModelPaths {
    pub cws: PathBuf,
    pub pos: PathBuf,
    pub ner: PathBuf,
}

impl LtpLegacyModelPaths {
    pub fn from_dir(model_dir: &Path) -> Self {
        Self {
            cws: model_dir.join("cws_model.bin"),
            pos: model_dir.join("pos_model.bin"),
            ner: model_dir.join("ner_model.bin"),
        }
    }
}

pub fn map_ltp_pos_tag(tag: &str) -> Option<AnnotationKind> {
    match tag {
        "n" | "nh" | "ni" | "nl" | "ns" | "nz" => Some(AnnotationKind::Noun),
        "v" => Some(AnnotationKind::Verb),
        "r" => Some(AnnotationKind::Pronoun),
        "d" => Some(AnnotationKind::Adverb),
        "a" => Some(AnnotationKind::Adjective),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LtpLegacyMode {
    Pos,
    Ner,
}

pub struct LtpLegacyAnalyzer {
    cws: CWSModel,
    pos: POSModel,
    ner: Option<NERModel>,
    mode: LtpLegacyMode,
}

impl LtpLegacyAnalyzer {
    pub fn load_from_dir(model_dir: &Path) -> Result<Self> {
        Self::load_pos_from_dir(model_dir)
    }

    pub fn load_pos_from_dir(model_dir: &Path) -> Result<Self> {
        let paths = LtpLegacyModelPaths::from_dir(model_dir);
        Self::load_pos_from_paths(&paths)
    }

    pub fn load_ner_from_dir(model_dir: &Path) -> Result<Self> {
        let paths = LtpLegacyModelPaths::from_dir(model_dir);
        Self::load_ner_from_paths(&paths)
    }

    pub fn load_pos_from_paths(paths: &LtpLegacyModelPaths) -> Result<Self> {
        let (cws, pos) = load_cws_and_pos(paths)?;
        Ok(Self {
            cws,
            pos,
            ner: None,
            mode: LtpLegacyMode::Pos,
        })
    }

    pub fn load_ner_from_paths(paths: &LtpLegacyModelPaths) -> Result<Self> {
        let (cws, pos) = load_cws_and_pos(paths)?;
        let ner_file = File::open(&paths.ner)
            .with_context(|| format!("failed to open {}", paths.ner.display()))?;
        let ner: NERModel = ModelSerde::load(ner_file, Format::AVRO(Codec::Deflate))
            .with_context(|| format!("failed to load {}", paths.ner.display()))?;

        Ok(Self {
            cws,
            pos,
            ner: Some(ner),
            mode: LtpLegacyMode::Ner,
        })
    }
}

fn load_cws_and_pos(paths: &LtpLegacyModelPaths) -> Result<(CWSModel, POSModel)> {
    let cws_file = File::open(&paths.cws)
        .with_context(|| format!("failed to open {}", paths.cws.display()))?;
    let cws: CWSModel = ModelSerde::load(cws_file, Format::AVRO(Codec::Deflate))
        .with_context(|| format!("failed to load {}", paths.cws.display()))?;

    let pos_file = File::open(&paths.pos)
        .with_context(|| format!("failed to open {}", paths.pos.display()))?;
    let pos: POSModel = ModelSerde::load(pos_file, Format::AVRO(Codec::Deflate))
        .with_context(|| format!("failed to load {}", paths.pos.display()))?;

    Ok((cws, pos))
}

impl TextAnalyzer for LtpLegacyAnalyzer {
    fn analyzer_id(&self) -> &'static str {
        match self.mode {
            LtpLegacyMode::Pos => "ltp-pos",
            LtpLegacyMode::Ner => "ltp-ner",
        }
    }

    fn analyze(&self, text: &str, base_offset: u64) -> Result<Vec<Annotation>> {
        let words = self.cws.predict(text)?;
        let pos_tags = self.pos.predict(&words)?;
        let offsets = align_words_to_offsets(text, &words, base_offset)?;

        let annotations = match self.mode {
            LtpLegacyMode::Pos => pos_annotations_from_offsets(&offsets, &pos_tags),
            LtpLegacyMode::Ner => {
                let ner = self.ner.as_ref().context("ltp ner model is not loaded")?;
                let ner_tags = ner.predict((&words, &pos_tags))?;
                ner_annotations_from_offsets(&offsets, &ner_tags)
            }
        };

        Ok(annotations)
    }
}

fn pos_annotations_from_offsets(offsets: &[(u64, u64)], pos_tags: &[&str]) -> Vec<Annotation> {
    offsets
        .iter()
        .zip(pos_tags)
        .filter_map(|((start_offset, end_offset), tag)| {
            let kind = map_ltp_pos_tag(tag)?;
            Some(Annotation {
                start_offset: *start_offset,
                end_offset: *end_offset,
                kind,
            })
        })
        .collect()
}

pub fn ner_annotations_from_offsets(offsets: &[(u64, u64)], ner_tags: &[&str]) -> Vec<Annotation> {
    let mut annotations = Vec::new();
    let mut active_start: Option<u64> = None;
    let mut active_kind: Option<AnnotationKind> = None;

    for ((start_offset, end_offset), tag) in offsets.iter().zip(ner_tags) {
        let Some((position, kind)) = parse_ner_tag(tag) else {
            active_start = None;
            active_kind = None;
            continue;
        };

        match position {
            NerPosition::Single => {
                annotations.push(Annotation {
                    start_offset: *start_offset,
                    end_offset: *end_offset,
                    kind,
                });
                active_start = None;
                active_kind = None;
            }
            NerPosition::Begin => {
                active_start = Some(*start_offset);
                active_kind = Some(kind);
            }
            NerPosition::Inside => {}
            NerPosition::End => {
                if let (Some(entity_start), Some(entity_kind)) = (active_start, active_kind) {
                    annotations.push(Annotation {
                        start_offset: entity_start,
                        end_offset: *end_offset,
                        kind: entity_kind,
                    });
                }
                active_start = None;
                active_kind = None;
            }
        }
    }

    annotations
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NerPosition {
    Single,
    Begin,
    Inside,
    End,
}

fn parse_ner_tag(tag: &str) -> Option<(NerPosition, AnnotationKind)> {
    if tag == "O" || tag == "o" {
        return None;
    }

    let (position, entity_type) = tag.split_once('-')?;
    let position = match position {
        "S" | "s" => NerPosition::Single,
        "B" | "b" => NerPosition::Begin,
        "I" | "i" => NerPosition::Inside,
        "E" | "e" => NerPosition::End,
        _ => return None,
    };

    let kind = match entity_type.to_ascii_lowercase().as_str() {
        "nh" => AnnotationKind::Person,
        "ns" => AnnotationKind::Location,
        "ni" => AnnotationKind::Organization,
        _ => return None,
    };

    Some((position, kind))
}
