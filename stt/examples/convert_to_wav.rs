use anyhow::{Context, Result};
use core::slice;
use std::fs::OpenOptions;
use std::io::Write;
use std::{env::args, mem};

/// Converts provided audio file to wav
fn main() -> Result<()> {
    let audio_file_path = args().nth(1).context("Please specify an audio file")?;
    let output_path = args()
        .nth(2)
        .context("Please specify an output audio file")?;
    let chunks = stt::ffmpeg_decoder::read_file(audio_file_path)?;
    let slice_u32 = &chunks[..];

    let slice_u8: &[u8] = unsafe {
        slice::from_raw_parts(
            slice_u32.as_ptr() as *const u8,
            slice_u32.len() * mem::size_of::<u16>(),
        )
    };

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(output_path)?;

    file.write_all(slice_u8)?;

    Ok(())
}
