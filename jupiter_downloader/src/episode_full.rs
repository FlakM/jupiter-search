use jupiter_common::Episode;
use serde::{Deserialize, Serialize};
use stt::Transcript;

use crate::metadata::Metadata;

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelData {
    pub model_file_name: String,
    pub model_size: usize,
    pub model_sha256: String,
}

impl From<stt::ModelInfo> for ModelData {
    fn from(info: stt::ModelInfo) -> Self {
        Self {
            model_file_name: info.file_name,
            model_size: info.size,
            model_sha256: info.sha256,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EpisodeFull {
    pub transcript: Transcript,
    pub metadata: Option<Metadata>,
    pub episode: Episode,
    pub speedup: Option<f32>,
    pub podcast2text_git_commit: String,
    pub model_info: ModelData,
}
