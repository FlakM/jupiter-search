use std::env::args;
use std::time::Instant;

use whisper_rs::{FullParams, SamplingStrategy, WhisperContext};
// this is copied from https://github.com/tazz4843/coqui-stt/blob/master/examples/basic_usage.rs
// and run using https://coqui.ai/english/coqui/v0.9.3 model
// mp3 had to be first converted using ffmpeg to
// ffmpeg -i action.mp3 -ar 16000 action.wav
fn main() {
    let audio_file_path = args()
        .nth(2)
        .expect("Please specify an audio file to run STT on");
    let model_file_path = args().nth(1).expect("Please model path");

    let mut ctx = WhisperContext::new(&model_file_path).expect("failed to load model");

    let st = Instant::now();
    let  audio_data = jupiter_search::ffmpeg_decoder::read_file(audio_file_path);

    println!("len: {}", &audio_data.len());

    let mut params = FullParams::new(SamplingStrategy::Greedy { n_past: 0 });

    // limit the number of threads
    params.set_n_threads(1);
    params.set_print_special_tokens(false);
    params.set_print_progress(true);

    // now we can run the model
    ctx.full(params, &audio_data[..])
        .expect("failed to run model");

    let num_segments = ctx.full_n_segments();

    assert!(num_segments > 0);
    for i in 0..num_segments {
        let segment = ctx.full_get_segment_text(i).expect("failed to get segment");
        let start_timestamp = ctx.full_get_segment_t0(i);
        let end_timestamp = ctx.full_get_segment_t1(i);
        println!("[{} - {}]: {}", start_timestamp, end_timestamp, segment);
    }

    // Output the result
    let et = Instant::now();
    let tt = et.duration_since(st);
    println!("took {:?}", tt);
}
