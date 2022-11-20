use jupiter_common::Episode;
use serde::{Deserialize, Serialize};
use stt::Transcript;

use crate::metadata::Metadata;

#[derive(Debug, Serialize, Deserialize)]
pub struct EpisodeFull {
    pub transcript: Transcript,
    pub metadata: Metadata,
    pub episode: Episode,
}
