use core::slice;
use std::fs::OpenOptions;
use std::io::Write; // bring trait into scope
use std::{env::args, mem};

fn main() {
    let audio_file_path = args().nth(1).expect("Please specify an audio file");
    let output_path = args().nth(2).expect("Please specify an output audio file");
    let chunks = jupiter_search::decoder::read_file(audio_file_path);
    let slice_u32 = &chunks[..];

    let slice_u8: &[u8] = unsafe {
        slice::from_raw_parts(
            slice_u32.as_ptr() as *const u8,
            slice_u32.len() * mem::size_of::<u16>(),
        )
    };

    println!("u8s: {:?}", slice_u8);

    // ... later in code
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        // either use ? or unwrap since it returns a Result
        .open(output_path)
        .unwrap();

    file.write_all(slice_u8).unwrap();
}
