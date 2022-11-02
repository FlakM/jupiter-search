
use anyhow::Result;
use meilisearch_sdk::{settings::Settings, Client, task_info::TaskInfo};
use rss::{Channel, Item};
use serde::{Deserialize, Serialize};
use anyhow::anyhow;
use regex::Regex;

#[derive(Serialize, Deserialize)]
struct Episode {
    /// id has to contain only letters: a-zA-Z0-9 and has to be unique
    id: String,
    title: String,
    description: Option<String>,
    link: String,
    pub_date: String,
}

impl TryFrom<Item> for Episode {
    type Error = anyhow::Error;
    fn try_from(value: Item) -> Result<Self, Self::Error> {
        //todo come up with better cleaning up... at least move it to lazy static
        let re = Regex::new(r"[a-zA-Z0-9]").unwrap();
        let title = value.title.ok_or_else(|| anyhow!("missing title"))?;
        let episode = Episode {
            id: re.find_iter(&title).map(|a| a.as_str()).collect::<Vec<_>>().join(""),
            title,
            description: value.description,
            link: value.link.ok_or_else(|| anyhow!("missing link!"))?,
            pub_date: value.pub_date.ok_or_else(|| anyhow!("missing pubValue"))?,
        };
        Ok(episode)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let content = include_str!("../all_shows.xml");
    let channel = Channel::read_from(content.as_bytes())?;
    println!("deserialized items: {}", channel.items.len());

    let episodes: Vec<Episode> = channel
        .items
        .into_iter()
        .map(|i| i.try_into())
        .collect::<Result<_>>()?;
    let client = Client::new("http://139.144.189.54:7700", "MASTER_KEY");

    let indexes = client.list_all_indexes().await?;

    if !indexes
        .results
        .iter()
        .find(|i| i.uid == "podcasts")
        .is_none()
    {
        let settings = Settings::new()
            .with_ranking_rules([
                "words",
                "typo",
                "proximity",
                "attribute",
                "sort",
                "exactness",
                "release_date:desc",
                "rank:desc",
            ])
            .with_searchable_attributes(["title", "description"])
            .with_displayed_attributes(["title", "description", "link", "pub_date"])
            .with_sortable_attributes(["title", "pub_date"]);
        client
            .index("podcasts")
            .set_settings(&settings)
            .await?
            .wait_for_completion(&client, None, None)
            .await?;
    }

    // adding documents
    let task: TaskInfo = client
        .index("podcasts2")
        .add_documents(&episodes[..], Some("id")).await?;
    let result = task.wait_for_completion(&client, None, None)
        .await?;

    println!("done! {:?}", result);

    Ok(())
}
