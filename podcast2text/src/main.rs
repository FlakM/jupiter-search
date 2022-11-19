use std::env::args;

use anyhow::{Context, Result};
use downloader::Downloader;

// cargo run --release https://feed.jupiter.zone/allshows ../stt/resources/ggml-tiny.en.bin .
#[tokio::main]
async fn main() -> Result<()> {
    let rss_url = args()
        .nth(1)
        .context("provide url to rss as first argument")?;

    let model_file_path = args().nth(2).expect("Please model path as param 2");
    let output_dir = args().nth(3).expect("Please provide output dir as param 3");

    let worker_count = args()
        .nth(4)
        .expect("Please provide worker count as param 4")
        .parse::<usize>()?;

    let downloader = Downloader::default();

    let results = downloader
        .download_rss(rss_url, worker_count, model_file_path, output_dir)
        .await?;

    println!("{:?}", results);

    Ok(())
}
