use std::{convert::{TryFrom, TryInto}, env::{args, temp_dir}, fs::File, io::Cursor, path::{Path, PathBuf}};

use anyhow::{Context, Result};
use common::{AllEpisodes, Episode};
use reqwest::Client;
use rss::Enclosure;

#[tokio::main]
async fn main() -> Result<()> {
    let rss_url = args()
        .nth(1)
        .context("provide url to rss as first argument")?;

    let client = Client::new();

    let rss_content = client.get(rss_url).send().await?.text().await?;

    let mut episodes: Vec<Episode> = AllEpisodes::try_from(rss_content)?.episodes;
    episodes.sort_by(|a, b| b.pub_date.cmp(&a.pub_date)); // we want download the latest first
    
    let first = episodes.pop().unwrap();

    println!("episode: {:?}", first);
    let mp3 = download_episode(client, first.enclosure).await?;
    println!("downloaded to {}", mp3.as_path().to_str().unwrap());

    Ok(())
}

async fn download_episode(client: Client, data: Enclosure) -> Result<PathBuf> {
    let temp_file = temp_dir().join(format!("{}.mp3", uuid::Uuid::new_v4()));
    let mut file = std::fs::File::create(&temp_file)?;
    let  bytes = client.get(data.url()).send().await?.bytes().await?;
    let  mut content =  Cursor::new(bytes); 
    std::io::copy(&mut content, &mut file)?;
    Ok(temp_file)
}
