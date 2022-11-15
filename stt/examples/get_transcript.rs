use anyhow::Result;
use std::env::args;

use stt::{SttContext, Utternace};

/// Prints transcript for a given path using specified model
fn main() -> Result<()> {
    let model_file_path = args().nth(1).expect("Please model path");
    let audio_file_path = args()
        .nth(2)
        .expect("Please specify an audio file to run STT on");

    let mut stt = SttContext::try_new(&model_file_path)?;

    let transcipt = stt.get_transcript_file(audio_file_path, true, 12)?;

    for Utternace { start, stop, text } in transcipt.utterances {
        println!("{start} - {stop}\t\t{text}");
    }
    Ok(())
}
