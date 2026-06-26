use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::highlight::analyzer::TextAnalyzer;
use crate::highlight::annotation::Annotation;
use crate::text_source::TextSource;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnnotationManifest {
    pub source_file_len: u64,
    pub chunk_size: usize,
    pub chunks: Vec<AnnotationChunkMeta>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnnotationChunkMeta {
    pub index: usize,
    pub start_offset: u64,
    pub end_offset: u64,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct AnnotationChunk {
    annotations: Vec<Annotation>,
}

pub struct AnnotationCache {
    root_path: PathBuf,
    manifest: AnnotationManifest,
    loaded_chunks: HashMap<usize, Vec<Annotation>>,
}

impl AnnotationCache {
    pub fn load_or_build(
        root_path: &Path,
        text_source: &TextSource,
        chunk_size: usize,
        analyzer: &dyn TextAnalyzer,
    ) -> Result<Self> {
        let source_file_len = text_source.file_len();

        if let Some(cache) = Self::load_if_fresh(root_path, source_file_len)? {
            return Ok(cache);
        }

        Self::build(root_path, text_source, chunk_size, analyzer)
    }

    pub fn query(&mut self, start_offset: u64, end_offset: u64) -> Result<Vec<Annotation>> {
        let chunk_indexes = self
            .manifest
            .chunks
            .iter()
            .filter(|chunk| chunk.start_offset < end_offset && chunk.end_offset > start_offset)
            .map(|chunk| chunk.index)
            .collect::<Vec<_>>();

        let mut annotations = Vec::new();

        for chunk_index in chunk_indexes {
            self.load_chunk_if_needed(chunk_index)?;

            if let Some(chunk_annotations) = self.loaded_chunks.get(&chunk_index) {
                annotations.extend(
                    chunk_annotations
                        .iter()
                        .filter(|annotation| {
                            annotation.start_offset < end_offset
                                && annotation.end_offset > start_offset
                        })
                        .cloned(),
                );
            }
        }

        Ok(annotations)
    }

    pub fn chunk_count(&self) -> usize {
        self.manifest.chunks.len()
    }

    pub fn loaded_chunk_count(&self) -> usize {
        self.loaded_chunks.len()
    }

    fn load_if_fresh(root_path: &Path, source_file_len: u64) -> Result<Option<Self>> {
        let manifest_path = manifest_path(root_path);

        if !manifest_path.exists() {
            return Ok(None);
        }

        let manifest = load_manifest(&manifest_path)?;

        if manifest.source_file_len != source_file_len {
            return Ok(None);
        }

        Ok(Some(Self {
            root_path: root_path.to_path_buf(),
            manifest,
            loaded_chunks: HashMap::new(),
        }))
    }

    fn build(
        root_path: &Path,
        text_source: &TextSource,
        chunk_size: usize,
        analyzer: &dyn TextAnalyzer,
    ) -> Result<Self> {
        if root_path.exists() {
            fs::remove_dir_all(root_path)?;
        }
        fs::create_dir_all(root_path)?;

        let source_file_len = text_source.file_len();
        let mut chunks = Vec::new();
        let mut current_offset = 0u64;
        let mut chunk_index = 0usize;

        while current_offset < source_file_len {
            let text = text_source.read_from_offset(current_offset, chunk_size)?;

            if text.is_empty() {
                break;
            }

            let end_offset = current_offset + text.len() as u64;
            let annotations = analyzer.analyze(&text, current_offset)?;
            let chunk = AnnotationChunk { annotations };
            save_chunk(root_path, chunk_index, &chunk)?;

            chunks.push(AnnotationChunkMeta {
                index: chunk_index,
                start_offset: current_offset,
                end_offset,
            });

            current_offset = end_offset;
            chunk_index += 1;
        }

        let manifest = AnnotationManifest {
            source_file_len,
            chunk_size,
            chunks,
        };
        save_manifest(&manifest_path(root_path), &manifest)?;

        Ok(Self {
            root_path: root_path.to_path_buf(),
            manifest,
            loaded_chunks: HashMap::new(),
        })
    }

    fn load_chunk_if_needed(&mut self, chunk_index: usize) -> Result<()> {
        if self.loaded_chunks.contains_key(&chunk_index) {
            return Ok(());
        }

        let chunk = load_chunk(&self.root_path, chunk_index)?;
        self.loaded_chunks.insert(chunk_index, chunk.annotations);

        Ok(())
    }
}

pub fn annotation_path_for_book(book_path: &Path, analyzer_id: &str) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    book_path.to_string_lossy().hash(&mut hasher);
    let book_hash = hasher.finish();

    PathBuf::from(".reading")
        .join("annotations")
        .join(format!("{book_hash:016x}"))
        .join(analyzer_id)
}

fn manifest_path(root_path: &Path) -> PathBuf {
    root_path.join("manifest.json")
}

fn chunk_path(root_path: &Path, chunk_index: usize) -> PathBuf {
    root_path.join(format!("chunk-{chunk_index:06}.json"))
}

fn load_manifest(path: &Path) -> Result<AnnotationManifest> {
    let json = fs::read_to_string(path)?;
    let manifest = serde_json::from_str(&json)?;
    Ok(manifest)
}

fn save_manifest(path: &Path, manifest: &AnnotationManifest) -> Result<()> {
    let json = serde_json::to_string(manifest)?;
    fs::write(path, json)?;
    Ok(())
}

fn load_chunk(root_path: &Path, chunk_index: usize) -> Result<AnnotationChunk> {
    let json = fs::read_to_string(chunk_path(root_path, chunk_index))?;
    let chunk = serde_json::from_str(&json)?;
    Ok(chunk)
}

fn save_chunk(root_path: &Path, chunk_index: usize, chunk: &AnnotationChunk) -> Result<()> {
    let json = serde_json::to_string(chunk)?;
    fs::write(chunk_path(root_path, chunk_index), json)?;
    Ok(())
}
