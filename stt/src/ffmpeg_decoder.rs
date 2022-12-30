use anyhow::{anyhow, Result};
use audrey::Reader;
use std::env::temp_dir;
use std::path::Path;
use std::process::Stdio;
use std::{fs::File, process::Command};

// this is a dirty workaround because I suck at audio processing
// and i could not for the love of god get the decoder.rs to work
//
// ffmpeg -i input.mp3 -ar 16000 output.wav
fn use_ffmpeg<P: AsRef<Path>>(input_path: P) -> Result<Vec<i16>> {
    let temp_file = temp_dir().join(format!("{}.wav", uuid::Uuid::new_v4()));
    eprintln!(
        "Starting converting to wav file {}",
        temp_file.as_path().to_string_lossy()
    );
    let mut pid = Command::new("ffmpeg")
        .args([
            "-i",
            input_path
                .as_ref()
                .to_str()
                .ok_or_else(|| anyhow!("invalid path"))?,
            "-ar",
            "16000",
            "-ac",
            "1",
            "-c:a",
            "pcm_s16le",
            (temp_file.to_str().unwrap()),
            "-hide_banner",
            "-y",
            "-loglevel",
            "error",
        ])
        .stdin(Stdio::null())
        .spawn()?;

    if pid.wait()?.success() {
        let output = File::open(&temp_file)?;
        let mut reader = Reader::new(output)?;
        let samples: Result<Vec<i16>, _> = reader.samples().collect();
        eprintln!(
            "Finished converting to wav file {}",
            temp_file.as_path().to_string_lossy()
        );
        std::fs::remove_file(temp_file)?;

        samples.map_err(|e| e.into())
    } else {
        Err(anyhow!("unable to convert file"))
    }
}

pub fn read_file<P: AsRef<Path>>(audio_file_path: P) -> Result<Vec<f32>> {
    let audio_buf = use_ffmpeg(&audio_file_path)?;
    Ok(whisper_rs::convert_integer_to_float_audio(&audio_buf))
}

pub fn convert_stereo_to_mono_audio(samples: &[f32]) -> Vec<f32> {
    let mut mono = Vec::with_capacity(samples.len() / 2);
    for i in (0..samples.len()).step_by(2) {
        mono.push((samples[i] + samples[i + 1]) / 2.0);
    }
    mono
}
