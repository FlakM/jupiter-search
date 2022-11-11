use anyhow::{anyhow, Result};
use audrey::Reader;
use std::env::temp_dir;
use std::{fs::File, process::Command};

// this is a dirty workaround because I suck at audio processing
// and i could not for the love of god get the decoder.rs to work
//
// ffmpeg -i input.mp3 -ar 16000 -ac 1 -c:a pcm_s16le output.wav
fn use_ffmpeg(input_path: &str) -> Result<Vec<i16>> {
    let temp_file = temp_dir().join(format!("{}.wav", uuid::Uuid::new_v4()));
    let mut pid = Command::new("ffmpeg")
        .arg("-i")
        .arg(input_path)
        .arg("-ar")
        .arg("16000")
        .arg(&temp_file)
        .spawn()?;

    if pid.wait()?.success() {
        let output = File::open(&temp_file)?;
        let mut reader = Reader::new(output)?;
        let samples: Result<Vec<i16>, _> = reader.samples().map(|s| s).collect();
        std::fs::remove_file(temp_file)?;
        samples.map_err(|e| e.into())
    } else {
        Err(anyhow!("unable to convert file"))
    }
}

pub fn read_file(audio_file_path: String) -> Vec<f32> {
    let audio_buf = use_ffmpeg(&audio_file_path).unwrap();
    let audio_data = whisper_rs::convert_stereo_to_mono_audio(
        &whisper_rs::convert_integer_to_float_audio(&audio_buf),
    );
    audio_data
}
