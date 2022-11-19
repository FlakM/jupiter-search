use std::{
    collections::VecDeque,
    env::temp_dir,
    fs::File,
    io::Cursor,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use common::{AllEpisodes, Episode};
use episode_full::EpisodeFull;
use futures::future::join_all;
use metadata::Metadata;
use reqwest::Client;
use rss::Enclosure;
use stt::SttContext;

pub mod episode_full;
pub mod metadata;

#[derive(Debug)]
pub enum TranscriptionResult {
    Downloaded { title: String },
    Skipped { title: String },
}

pub struct Downloader {
    client: Client,
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new()
    }
}

impl Downloader {
    pub fn new() -> Downloader {
        Downloader {
            client: Client::new(),
        }
    }

    pub async fn download_rss<'a>(
        &self,
        rss_url: &'a str,
        worker_count: usize,
        model_file_path: &Path,
        output_dir: &Path,
        n_elements: Option<usize>,
        debug: bool,
        threads_per_worker: usize,
    ) -> Result<Vec<Result<Vec<TranscriptionResult>>>> {
        let client = self.client.clone();

        let rss_content = client.get(rss_url).send().await?.text().await?;

        let mut episodes: Vec<Episode> = AllEpisodes::try_from(rss_content)?.episodes;
        episodes.sort_by(|a, b| b.pub_date.cmp(&a.pub_date)); // we want download the latest first

        let num = n_elements.ok_or(episodes.len()).unwrap_or_default();

        //let chunk_length = episodes.len() / worker_count;
        let chunk_length = num / worker_count;
        let mut chunks: VecDeque<Vec<Episode>> =
            episodes
                .into_iter()
                .take(num)
                .fold(VecDeque::new(), |mut acc, a| {
                    if acc.is_empty() {
                        acc.push_back(vec![a])
                    } else {
                        let i = acc.len() - 1;
                        let last: &mut Vec<_> = acc[i].as_mut();
                        if last.len() >= chunk_length {
                            acc.push_back(vec![a])
                        } else {
                            last.push(a);
                        }
                    };
                    acc
                });

        let mut handles = Vec::new();
        for worker in 0..worker_count {
            let client = client.clone();
            let chunk = chunks.pop_front().unwrap();
            let dir = output_dir.to_path_buf();
            let model_file_path = model_file_path.to_path_buf();
            let handle = tokio::spawn(async move {
                println!("#{} - starting new task for worker ", worker);
                let mut context = SttContext::try_new(&model_file_path)?;

                let mut downloaded = vec![];
                for episode in chunk {
                    let file_target = dir.join(format!("{}.json", episode.id));
                    println!(
                        "#{} - transcribing episode {} in file {}",
                        worker,
                        episode.title,
                        file_target.to_string_lossy()
                    );
                    if !file_target.exists() {
                        let full = process_episode(
                            client.clone(),
                            episode.clone(),
                            &mut context,
                            threads_per_worker,
                            debug,
                        )
                        .await?;
                        serde_json::to_writer_pretty(&File::create(file_target)?, &full)?;
                        downloaded.push(TranscriptionResult::Downloaded {
                            title: episode.title,
                        });
                    } else {
                        eprintln!(
                        "#{} - skipping downloading a file for episode {} since it seems to be already present!",
                        worker,episode.title
                    );
                        downloaded.push(TranscriptionResult::Skipped {
                            title: episode.title,
                        });
                    }
                }
                Ok::<Vec<TranscriptionResult>, anyhow::Error>(downloaded)
            });
            handles.push(handle);
        }

        let results: Result<Vec<_>> = join_all(handles)
            .await
            .into_iter()
            .map(|a| a.map_err(|e| anyhow!(e)))
            .collect();
        results
    }
}

pub async fn process_episode(
    client: Client,
    episode: Episode,
    whisper_context: &mut SttContext,
    threads_per_worker: usize,
    debug: bool,
) -> Result<EpisodeFull> {
    let path = download_episode(client, &episode.enclosure).await?;
    let metadata: Metadata = path.as_path().try_into()?;
    let transcript =
        whisper_context.get_transcript_file(path.as_path(), debug, threads_per_worker as u8)?;
    Ok(EpisodeFull {
        transcript,
        metadata,
        episode,
    })
}

pub async fn download_episode(client: Client, data: &Enclosure) -> Result<PathBuf> {
    let temp_file = temp_dir().join(format!("{}.mp3", uuid::Uuid::new_v4()));
    let mut file = std::fs::File::create(&temp_file)?;
    let bytes = client.get(data.url()).send().await?.bytes().await?;
    let mut content = Cursor::new(bytes);
    std::io::copy(&mut content, &mut file)?;
    eprintln!("Downloaded file {}", temp_file.as_path().to_string_lossy());
    Ok(temp_file)
}
