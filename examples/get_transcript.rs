use audrey::Reader;
use coqui_stt::Model;
use dasp_interpolate::linear::Linear;
use dasp_signal::interpolate::Converter;
use dasp_signal::{from_iter, Signal};
use std::env::args;
use std::fs::File;
use std::path::Path;
use std::time::Instant;


// this is copied from https://github.com/tazz4843/coqui-stt/blob/master/examples/basic_usage.rs
// and run using https://coqui.ai/english/coqui/v0.9.3 model
// mp3 had to be first converted using ffmpeg to 
// ffmpeg -i action.mp3 -ar 16000 action.wav
fn main() {
    let model_dir_str = args().nth(1).expect("Please specify model dir");
    let audio_file_path = args()
        .nth(2)
        .expect("Please specify an audio file to run STT on");
    let dir_path = Path::new(&model_dir_str);
    let mut model_name: Box<Path> = dir_path.join("output_graph.pb").into_boxed_path();
    let mut scorer_name: Option<Box<Path>> = None;
    // search for model in model directory
    for file in dir_path
        .read_dir()
        .expect("Specified model dir is not a dir")
    {
        if let Ok(f) = file {
            let file_path = f.path();
            if file_path.is_file() {
                if let Some(ext) = file_path.extension() {
                    if ext == "pb" || ext == "pbmm" || ext == "tflite" {
                        model_name = file_path.into_boxed_path();
                    } else if ext == "scorer" {
                        scorer_name = Some(file_path.into_boxed_path());
                    }
                }
            }
        }
    }

    let mut m = Model::new(model_name.to_str().expect("invalid utf-8 found in path")).unwrap();
    // enable external scorer if found in the model folder
    if let Some(scorer) = scorer_name {
        let scorer = scorer.to_str().expect("invalid utf-8 found in path");
        println!("Using external scorer `{}`", scorer);
        m.enable_external_scorer(scorer).unwrap();
    }

    let audio_file = File::open(audio_file_path).unwrap();
    let mut reader = Reader::new(audio_file).unwrap();
    let desc = reader.description();
    // input audio must be mono and usually at 16KHz, but this depends on the model
    let channel_count = desc.channel_count();

    let src_sample_rate = desc.sample_rate();
    let dest_sample_rate = m.get_sample_rate() as u32;
    // Obtain the buffer of samples
    let mut audio_buf: Vec<_> = if src_sample_rate == dest_sample_rate {
        reader.samples().map(|s| s.unwrap()).collect()
    } else {
        // We need to interpolate to the target sample rate
        let interpolator = Linear::new([0i16], [0]);
        let conv = Converter::from_hz_to_hz(
            from_iter(reader.samples::<i16>().map(|s| [s.unwrap()])),
            interpolator,
            src_sample_rate as f64,
            dest_sample_rate as f64,
        );
        conv.until_exhausted().map(|v| v[0]).collect()
    };
    // Convert to mono if required
    if channel_count == 2 {
        audio_buf = stereo_to_mono(&audio_buf);
    } else if channel_count != 1 {
        panic!(
            "unknown number of channels: got {}, expected 1 or 2",
            channel_count
        );
    }

    let st = Instant::now();

    // Run the speech to text algorithm
    let result = m.speech_to_text(&audio_buf).unwrap();

    let et = Instant::now();
    let tt = et.duration_since(st);

    // Output the result
    println!("{}", result);
    println!("took {:?}", tt);
}

fn stereo_to_mono(samples: &[i16]) -> Vec<i16> {
    // converting stereo to mono audio is relatively simple
    // just take the average of the two channels
    samples.chunks(2).map(|c| ((c[0] as i32 + c[1] as i32) / 2) as i16).collect()
}
