use std::convert::TryFrom;

use anyhow::anyhow;
use anyhow::Result;
use chrono::offset::FixedOffset;
use chrono::DateTime;
use log::info;
use rss::Enclosure;
use rss::{Channel, Item};
use serde::{Deserialize, Serialize};

impl TryFrom<String> for AllEpisodes {
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let channel = Channel::read_from(value.as_bytes())?;
        info!("deserialized rss items: {}", channel.items.len());

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
    pub title: String,
    pub description: Option<String>,
    pub content: Option<String>,
    pub link: String,
    pub enclosure: Enclosure,
    pub pub_date: DateTime<FixedOffset>,
}

impl TryFrom<Item> for Episode {
    type Error = anyhow::Error;
    fn try_from(episode: Item) -> Result<Self, Self::Error> {
        let title = episode.title.ok_or_else(|| anyhow!("missing title"))?;
        let episode = Episode {
            title,
            description: episode.description,
            content: episode.content,
            link: episode.link.ok_or_else(|| anyhow!("missing link!"))?,
            enclosure: episode
                .enclosure
                .ok_or_else(|| anyhow!("missing enclosure url"))?,
            pub_date: DateTime::parse_from_rfc2822(
                &episode
                    .pub_date
                    .ok_or_else(|| anyhow!("missing pubValue"))?,
            )?,
        };

        Ok(episode)
    }
}
