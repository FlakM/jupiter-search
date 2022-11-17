use std::convert::TryFrom;

use anyhow::anyhow;
use anyhow::Result;
use regex::Regex;
use rss::Enclosure;
use rss::{Channel, Item};
use serde::{Deserialize, Serialize};

impl TryFrom<String> for AllEpisodes {
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let channel = Channel::read_from(value.as_bytes())?;
        println!("deserialized items: {}", channel.items.len());

        let episodes: Vec<Episode> = channel
            .items
            .into_iter()
            .map(|i| i.try_into())
            .collect::<Result<_>>()?;
        Ok(AllEpisodes { episodes })
    }

    type Error = anyhow::Error;
}

pub struct AllEpisodes {
    pub episodes: Vec<Episode>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Episode {
    /// id has to contain only letters: a-zA-Z0-9 and has to be unique
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub content: Option<String>,
    pub link: String,
    pub enclosure: Enclosure,
    pub pub_date: String,
}

impl TryFrom<Item> for Episode {
    type Error = anyhow::Error;
    fn try_from(episode: Item) -> Result<Self, Self::Error> {
        //todo come up with better cleaning up... at least move it to lazy static
        let re = Regex::new(r"[a-zA-Z0-9]").unwrap();
        let title = episode.title.ok_or_else(|| anyhow!("missing title"))?;
        let episode = Episode {
            id: re
                .find_iter(&title)
                .map(|a| a.as_str())
                .collect::<Vec<_>>()
                .join(""),
            title,
            description: episode.description,
            content: episode.content,
            link: episode.link.ok_or_else(|| anyhow!("missing link!"))?,
            enclosure: episode
                .enclosure
                .ok_or_else(|| anyhow!("missing enclosure url"))?,
            pub_date: episode
                .pub_date
                .ok_or_else(|| anyhow!("missing pubValue"))?,
        };

        Ok(episode)
    }
}
