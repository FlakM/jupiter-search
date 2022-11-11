use std::io::ErrorKind;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use rubato::{InterpolationParameters, InterpolationType, Resampler, SincFixedIn, WindowFunction};

// This should baiscally return the 
// ffmpeg -i input.mp3 -ar 16000 -ac 1 -c:a pcm_s16le output.wav
pub fn read_file(audio_file_path: String) -> Vec<f32> {
    let src = std::fs::File::open(audio_file_path).expect("failed to open media");
    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    let mut hint = Hint::new();
    hint.with_extension("mp3");

    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .expect("unsupported format");

    let mut format = probed.format;
    let tracks = format.tracks();

    if tracks.len() != 1 {
        panic!("invalid number of tracks");
    }

    // Find the first audio track with a known (decodeable) codec.
    let track = tracks
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .unwrap();

    // Use the default options for the decoder.
    let dec_opts: DecoderOptions = Default::default();

    // Create a decoder for the track.
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &dec_opts)
        .expect("unsupported codec");

    // Store the track identifier, it will be used to filter packets.
    let track_id = track.id;
    let mut sample_buf = None;

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(Error::ResetRequired) => {
                // The track list has been changed. Re-examine it and create a new set of decoders,
                // then restart the decode loop. This is an advanced feature and it is not
                // unreasonable to consider this "the end." As of v0.5.0, the only usage of this is
                // for chained OGG physical streams.
                unimplemented!();
            }
            Err(Error::IoError(error)) => match error.kind() {
                ErrorKind::UnexpectedEof => {
                    continue;
                    //break;
                }
                _ => panic!("{}", error),
            },
            Err(err) => {
                // A unrecoverable error occured, halt decoding.
                panic!("{}", err);
            }
        };

        // If the packet does not belong to the selected track, skip it.
        if packet.track_id() != track_id {
            continue;
        }

        // Decode the packet into audio samples, ignoring any decode errors.
        match decoder.decode(&packet) {
            Ok(audio_buf) => {
                if sample_buf.is_none() {
                    let spec = *audio_buf.spec();
                    let duration = audio_buf.capacity() as u64;
                    sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
                }

                if let Some(buf) = &mut sample_buf {
                    buf.copy_planar_ref(audio_buf);
                    //buf.copy_interleaved_ref(audio_buf);
                }
            }
            Err(Error::DecodeError(_)) => (),
            Err(_) => break,
        }
    }

    let input_params = decoder.codec_params();
    let params = InterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: InterpolationType::Linear,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };

    println!(">>>> {}", input_params.sample_rate.unwrap());
    let source_samples = sample_buf.unwrap();
    let mut resampler = SincFixedIn::<f32>::new(
        16000 as f64 / input_params.sample_rate.unwrap() as f64,
        5.0,
        params,
        source_samples.len(),
        1,
    )
    .unwrap();

    let waves_in = vec![source_samples.samples()];
    let mut waves_out = resampler.process(&waves_in, None).unwrap();
    waves_out.pop().unwrap()
}
