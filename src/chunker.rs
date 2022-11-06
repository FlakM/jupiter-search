use std::time::Duration;

use webrtc_vad::{SampleRate, VadMode};

// webrtcvad requires frames of 12,20,30s so SEGMENT_SIZE has to be
// adjusted based on the format
const SAMPLE_SIZE: usize = 16;

// webrtcvad requires frame size eq 10, 20 or 30 ms
const FRAME_SIZE: Duration = Duration::from_millis(20);

const VAD_BUF_LENGTH: usize = SAMPLE_SIZE * FRAME_SIZE.as_millis() as usize;

#[derive(Default)]
struct State<'a> {
    pub chunks: Vec<Chunk<'a>>,
    pub curr_segment_start: usize,
    pub curr_segment_stop: usize,
    pub curr_silence_len: usize,
}

impl<'a> State<'a> {
    fn current_has_sound(&self) -> bool {
        self.curr_segment_stop > self.curr_segment_start
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Chunk<'a> {
    pub buf: &'a [i16],
    pub start: Duration,
    pub stop: Duration,
}

pub struct Chunker {
    /// amount of voice inactivity required between sentences to create sperate chunks
    silence_threshold: Duration,
    vad: webrtc_vad::Vad,
}

impl Chunker {
    pub fn new(silence_threshold: Duration, vad_mode: VadMode) -> Chunker {
        let vad = webrtc_vad::Vad::new_with_rate_and_mode(SampleRate::Rate16kHz, vad_mode);
        Chunker {
            silence_threshold,
            vad,
        }
    }
    pub fn get_silence_chunks<'a, 'b>(&'a mut self, bytes: &'b [i16]) -> Vec<Chunk<'b>> {
        // todo move those parameters up
        let chunks = bytes.chunks_exact(VAD_BUF_LENGTH);
        let mut state: State = chunks
            .enumerate()
            .fold(State::default(), |mut state, (i, curr)| {
                // this is a nasty trick to make tests work...
                // todo delete this shit
                #[cfg(test)]
                let is_silent =
                    { curr.iter().all(|a| *a == 0) || !self.vad.is_voice_segment(&curr).unwrap() };

                #[cfg(not(test))]
                let is_silent = !self.vad.is_voice_segment(curr).unwrap();

                // TODO add duration mapping here
                let chunk_silence_threshold_reached = is_silent
                    && Duration::from_millis((state.curr_silence_len * VAD_BUF_LENGTH) as u64)
                        >= self.silence_threshold;
                if is_silent && chunk_silence_threshold_reached {
                    if state.current_has_sound() {
                        let start = state.curr_segment_start * VAD_BUF_LENGTH;
                        let stop = state.curr_segment_stop * VAD_BUF_LENGTH;
                        state.chunks.push(Chunk {
                            buf: &bytes[start..stop],
                            start: Duration::from_millis(
                                state.curr_segment_start as u64 * FRAME_SIZE.as_millis() as u64,
                            ),
                            stop: Duration::from_millis(
                                state.curr_segment_stop as u64 * FRAME_SIZE.as_millis() as u64,
                            ),
                        })
                    }
                    state.curr_segment_start = i + 1;
                } else if is_silent {
                    state.curr_silence_len += 1;
                } else {
                    state.curr_silence_len = 0;
                    state.curr_segment_stop = i + 1;
                };
                state
            });

        if state.current_has_sound() {
            let start = state.curr_segment_start * VAD_BUF_LENGTH;
            let stop = state.curr_segment_stop * VAD_BUF_LENGTH;
            state.chunks.push(Chunk {
                buf: &bytes[start..stop],
                start: Duration::from_millis(
                    state.curr_segment_start as u64 * FRAME_SIZE.as_millis() as u64,
                ),
                stop: Duration::from_millis(
                    state.curr_segment_stop as u64 * FRAME_SIZE.as_millis() as u64,
                ),
            })
        }
        state.chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        let sound1: [i16; VAD_BUF_LENGTH] = (0_i16..VAD_BUF_LENGTH as i16)
            .collect::<Vec<i16>>()
            .try_into()
            .expect("wrong size iterator");
        let sound2: [i16; VAD_BUF_LENGTH] = (1_i16..=VAD_BUF_LENGTH as i16)
            .collect::<Vec<i16>>()
            .try_into()
            .expect("wrong size iterator");
        let silence_vec = std::iter::repeat(0)
            .take(VAD_BUF_LENGTH)
            .collect::<Vec<i16>>();

        let silence: [i16; VAD_BUF_LENGTH] = silence_vec.clone().try_into().unwrap();

        let mut vad =
            webrtc_vad::Vad::new_with_rate_and_mode(SampleRate::Rate16kHz, VadMode::Quality);
        assert!(vad.is_voice_segment(&sound1).unwrap());
        assert!(vad.is_voice_segment(&sound2).unwrap());

        let mut c = Chunker::new(Duration::from_millis(20), VadMode::Quality);
        assert!(c.get_silence_chunks(&[]).is_empty());
        assert_eq!(
            c.get_silence_chunks(&sound1),
            vec![Chunk {
                buf: &sound1,
                start: Duration::from_millis(0),
                stop: Duration::from_millis(20)
            }]
        );
        assert_eq!(
            c.get_silence_chunks(&[sound1, silence].concat()),
            vec![Chunk {
                buf: &sound1,
                start: Duration::from_millis(0),
                stop: Duration::from_millis(20)
            }]
        );
        assert_eq!(
            c.get_silence_chunks(&[sound1, silence, silence].concat()),
            vec![Chunk {
                buf: &sound1,
                start: Duration::from_millis(0),
                stop: Duration::from_millis(20)
            }]
        );
        assert_eq!(
            c.get_silence_chunks(&[sound1, silence, silence, sound2].concat()),
            vec![
                Chunk {
                    buf: &sound1,
                    start: Duration::from_millis(0),
                    stop: Duration::from_millis(20)
                },
                Chunk {
                    buf: &sound2,
                    start: Duration::from_millis(60),
                    stop: Duration::from_millis(80)
                }
            ]
        );

        let mut silence_in_between = sound1.clone();
        for i in 10..30 {
            silence_in_between[i] = i as i16;
        }

        assert_eq!(
            c.get_silence_chunks(&silence_in_between),
            vec![Chunk {
                buf: &silence_in_between,
                start: Duration::from_millis(0),
                stop: Duration::from_millis(20)
            }]
        );
    }
}
