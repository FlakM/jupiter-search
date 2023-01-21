use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use episode_full::EpisodeFull;
use futures::stream;
use futures_util::StreamExt;
use jupiter_common::{AllEpisodes, Episode};
use log::{debug, info};
use metadata::Metadata;
use reqwest::Client;
use rss::Enclosure;
use stt::SttContext;
use tokio::sync::Mutex;

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

pub struct DownloadParams<'a> {
    pub rss_url: &'a str,
    pub worker_count: usize,
    pub model_file_path: &'a Path,
    pub output_dir: &'a Path,
    pub n_elements: Option<usize>,
    pub debug: bool,
    pub download_directory: &'a Path,
    pub threads_per_worker: usize,
    pub name_filter: Option<&'a str>,
}

impl Downloader {
    pub fn new() -> Downloader {
        Downloader {
            client: Client::new(),
        }
    }

    pub async fn download_rss<'a>(&self, params: DownloadParams<'a>) -> Result<()> {
        let client = self.client.clone();

        let rss_content = client.get(params.rss_url).send().await?.text().await?;

        let mut episodes: Vec<Episode> = AllEpisodes::try_from(rss_content)?.episodes;
        episodes.sort_by(|a, b| b.pub_date.cmp(&a.pub_date)); // we want download the latest first

        if let Some(filter) = params.name_filter {
            episodes.retain(|e| e.title.to_lowercase().contains(filter))
        };

        let num = params.n_elements.ok_or(episodes.len()).unwrap_or_default();
        let episodes: Vec<Episode> = episodes.into_iter().take(num).collect();

        let workers = if episodes.len() < params.worker_count {
            episodes.len()
        } else {
            params.worker_count
        };

        let workers = 0..workers;
        let episodes = Arc::new(Mutex::new(episodes));

        let tasks = stream::iter(workers).map(|worker| {
            let client = client.clone();
            let dir = params.output_dir.to_path_buf();
            let model_file_path = params.model_file_path.to_path_buf();
            let episodes = Arc::clone(&episodes);
            let download_dir = params.download_directory.to_path_buf();

            tokio::spawn(async move {
                debug!("#{} - starting new task for worker ", worker);
                let mut context = SttContext::try_new(&model_file_path)?;
                let mut downloaded = vec![];
                loop {
                    let episode = {
                        let mut episodes = episodes.lock().await;
                        if let Some(episode) = episodes.pop() {
                            episode
                        } else {
                            break;
                        }
                    };
                    let file_target = dir.join(format!("{}.json", episode.title));
                    debug!(
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
                            params.threads_per_worker,
                            params.debug,
                            &download_dir,
                            episode.title.clone()
                        )
                        .await?;
                        serde_json::to_writer_pretty(&File::create(&file_target)?, &full)?;
                        downloaded.push(TranscriptionResult::Downloaded {
                            title: episode.title,
                        });
                        info!("#{} - transcription done and saved to a file {:?}", worker, &file_target);
                    } else {
                        info!(
                        "#{} - skipping downloading a file for episode {} since it seems to be already present!",
                        worker,episode.title
                    );
                        downloaded.push(TranscriptionResult::Skipped {
                            title: episode.title,
                        });
                    }
                }
                Ok::<Vec<TranscriptionResult>, anyhow::Error>(downloaded)
            })
        }).buffer_unordered(params.worker_count);

        tasks
            .for_each(|b| async move {
                info!("{:?}", b);
            })
            .await;
        Ok(())
    }
}

pub async fn process_episode(
    client: Client,
    episode: Episode,
    whisper_context: &mut SttContext,
    threads_per_worker: usize,
    debug: bool,
    dir: &Path,
    title: String,
) -> Result<EpisodeFull> {
    let path = download_episode(client, &episode.enclosure, dir, title).await?;
    let metadata: Metadata = path.as_path().try_into()?;
    let transcript =
        whisper_context.get_transcript_file(path.as_path(), debug, threads_per_worker as u8)?;
    let speedup = metadata.duration.as_secs_f32() / transcript.processing_time.as_secs_f32();
    let model_info = whisper_context.model_data.clone().into();
    Ok(EpisodeFull {
        transcript,
        metadata,
        episode,
        speedup,
        podcast2text_git_commit: env!("GIT_HASH").to_string(),
        model_info,
    })
}

pub async fn download_episode(
    client: Client,
    data: &Enclosure,
    dir: &Path,
    title: String,
) -> Result<PathBuf> {
    use std::io::Write;
    let path = dir.join(format!("{}.mp3", title));
    if !path.exists() {
        let mut file = std::fs::File::create(&path)?;
        let mut stream = client.get(data.url()).send().await?.bytes_stream();
        while let Some(bytes) = stream.next().await {
            file.write_all(bytes?.as_ref())?;
        }
        info!("Downloaded file {}", path.as_path().to_string_lossy());
    } else {
        info!(
            "Skipped downloading file {}",
            path.as_path().to_string_lossy()
        );
    }
    Ok(path)
}
