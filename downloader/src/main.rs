use std::{
    convert::{TryFrom, TryInto},
    env::{args, temp_dir},
    path::PathBuf, fs::File, str::FromStr,
};

use anyhow::{Context, Result};
use common::{AllEpisodes, Episode};
use downloader::{metadata::Metadata, episode_full::EpisodeFull};
use reqwest::Client;
use rss::Enclosure;
use stt::SttContext;


// cargo run --release https://feed.jupiter.zone/allshows ../stt/resources/ggml-tiny.en.bin .
#[tokio::main]
async fn main() -> Result<()> {
    let rss_url = args()
        .nth(1)
        .context("provide url to rss as first argument")?;

    // TODO: There are some situation when `args()` will return all arguments only once and will
    // so output_dir will fail, this may require using crate like `gumdrop`, `structopt` or
    // `clap`. Be aware that newest `clap` fail on nix
    let model_file_path = args().nth(2).expect("Please model path as param 2");
    let output_dir = args().nth(3).expect("Please provide output dir as param 3");

    let client = Client::new();

    let rss_content = client.get(rss_url).send().await?.text().await?;

    let mut episodes: Vec<Episode> = AllEpisodes::try_from(rss_content)?.episodes;
    episodes.sort_by(|a, b| a.pub_date.cmp(&b.pub_date)); // we want download the latest first

    for episode in episodes.into_iter().take(1) {
        let file_target = PathBuf::from_str(&output_dir)?.join(format!("{}.json", episode.id));

        if !file_target.exists() {
            let mut context =  SttContext::try_new(&model_file_path)?;
            let full = process_episode(client.clone(), episode, &mut context).await?;
            serde_json::to_writer_pretty(&File::create(file_target)?, &full)?;
        } else {
            eprintln!("Skipping downloading a file for episode {} since it seems to be already present!", episode.title);
        }
    }

    Ok(())
}

async fn process_episode(
    client: Client,
    episode: Episode,
    whisper_context: &mut SttContext,
) -> Result<EpisodeFull> {
    let path = download_episode(client, &episode.enclosure).await?;
    let metadata: Metadata = path.as_path().try_into()?;
    let transcript = whisper_context.get_transcript_file(path.as_path(), true, 12)?;
    Ok(EpisodeFull {
        transcript,
        metadata,
        episode,
    })
}

/// Download remote resource without allocating too much memory
async fn download_episode(client: Client, data: &Enclosure) -> Result<PathBuf> {
    use std::io::Write;
    use futures_util::StreamExt;

    let temp_file = temp_dir().join(format!("{}.mp3", uuid::Uuid::new_v4()));
    let mut file = std::fs::File::create(&temp_file)?;
    let mut stream = client.get(data.url()).send().await?.bytes_stream();
    while let Some(bytes) = stream.next().await {
        file.write_all(bytes?.as_ref())?;
    }
    Ok(temp_file)
}
