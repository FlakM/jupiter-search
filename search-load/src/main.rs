use std::convert::TryFrom;

use anyhow::Result;
use common::AllEpisodes;
use common::Episode;
use meilisearch_sdk::{settings::Settings, task_info::TaskInfo, Client};

#[tokio::main]
async fn main() -> Result<()> {
    let content = include_str!("../all_shows.xml").to_string();

    let episodes: Vec<Episode> = AllEpisodes::try_from(content)?.episodes;
    let client = Client::new("http://139.144.189.54:7700", "MASTER_KEY");

    let indexes = client.list_all_indexes().await?;

    if indexes.results.iter().any(|i| i.uid == "podcasts") {
        // https://docs.meilisearch.com/learn/core_concepts/relevancy.html#built-in-rules
        let settings = Settings::new()
            .with_ranking_rules([
                "words",
                "typo",
                "proximity",
                "attribute",
                "sort",
                "exactness",
                "pub_date:desc",
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
        .add_documents(&episodes[..], Some("id"))
        .await?;
    let result = task.wait_for_completion(&client, None, None).await?;

    println!("done! {:?}", result);

    Ok(())
}
