use std::{
    path::Path,
    time::{Duration, Instant},
};

use anyhow::{anyhow, bail, Result};
use log::{debug, info};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext};

use serde::{Deserialize, Serialize};

pub mod decoder;
pub mod ffmpeg_decoder;

#[derive(Debug, Serialize, Deserialize)]
pub struct Utternace {
    pub start: i64,
    pub stop: i64,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transcript {
    pub utterances: Vec<Utternace>,
    pub processing_time: Duration,
}

#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub file_name: String,
    pub sha256: String,
    pub size: usize,
}

pub struct SttContext {
    whisper_context: WhisperContext,
    pub model_data: ModelInfo,
}

impl SttContext {
    pub fn try_new<P: AsRef<Path>>(path: P) -> Result<SttContext> {
        let stringy_path = path
            .as_ref()
            .to_str()
            .ok_or_else(|| anyhow!("invalid utf in path {:?}", path.as_ref()))?;
        info!("Created model from path {}", stringy_path);
        let ctx = WhisperContext::new(stringy_path).map_err(|e| {
            anyhow!(
                "failed to load model at path {:?} due to {:?}",
                path.as_ref(),
                e
            )
        })?;

        let model_path = Path::new(stringy_path);
        let hash = sha256::try_digest(model_path)?;

        Ok(SttContext {
            whisper_context: ctx,
            model_data: ModelInfo {
                file_name: stringy_path.to_string(),
                sha256: hash,
                size: std::fs::metadata(path)?.len() as usize,
            },
        })
    }

    pub fn get_transcript_file<P: AsRef<Path>>(
        &mut self,
        path: P,
        debug: bool,
        threads: u8,
    ) -> Result<Transcript> {
        let decoded = ffmpeg_decoder::read_file(path)?;
        self.get_transcript(&decoded, debug, threads)
    }

    fn get_transcript(
        &mut self,
        input_bytes: &[f32],
        debug: bool,
        threads: u8,
    ) -> Result<Transcript> {
        let mut params = FullParams::new(SamplingStrategy::Greedy { n_past: 0 });

        params.set_n_threads(threads as i32);
        params.set_print_special(debug);
        params.set_print_progress(debug);
        params.set_print_timestamps(debug);

        let ctx = &mut self.whisper_context;
        let st = Instant::now();

        debug!("Starting inference");

        ctx.full(params, input_bytes)
            .map_err(|_| anyhow!("failed to run model"))?;

        let num_segments = ctx.full_n_segments();

        let mut utterances = vec![];

        if num_segments == 0 {
            bail!("failed to get any segments");
        }

        for i in 0..num_segments {
            let segment = ctx
                .full_get_segment_text(i)
                .map_err(|e| anyhow!("failed to get segment due to {:?}", e))?;
            let start_timestamp = ctx.full_get_segment_t0(i);
            let end_timestamp = ctx.full_get_segment_t1(i);
            utterances.push(Utternace {
                start: start_timestamp,
                stop: end_timestamp,
                text: segment,
            });
        }
        let et = Instant::now();
        let tt = et.duration_since(st);

        Ok(Transcript {
            utterances,
            processing_time: tt,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    // TODO understand why it doesn't work!?
    //#[test]
    fn stt_works() {
        let mut ctx = SttContext::try_new("resources/ggml-tiny.en.bin").unwrap();
        let t = ctx
            .get_transcript_file("resources/super_short.mp3", true, 12)
            .unwrap();
        println!("{:?}", t);
        assert!(!t.utterances.is_empty())
    }
}
