use std::{convert::TryFrom, path::Path};

use anyhow::Result;
use lofty::{Accessor, ItemKey, Probe};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Metadata {
    title: String,
    pictures: Vec<String>,
    unique_id: String,
}

impl TryFrom<&Path> for Metadata {
    type Error = anyhow::Error;
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let meta = read_metadata(path)?;
        Ok(meta)
    }
}
pub fn read_metadata(path: &Path) -> Result<Metadata> {
    let tagged_file = Probe::open(path)?.read()?;
    let mut meta = Metadata::default();

    let tag = &tagged_file.tags()[0];

    meta.title = tag.title().unwrap().to_string();
    // pictures are too big for now
    //meta.pictures = tag.pictures().iter().map(|a| base64::encode(a.as_ape_bytes())).collect();

    let mut pguid = tag.get_strings(&ItemKey::PodcastGlobalUniqueID).peekable();

    let mut id = String::new();
    while let Some(p) = pguid.next() {
        id.push_str(p);
        let next = pguid.peek();
        if next.is_some() {
            id.push('/')
        }
    }

    meta.unique_id = id;

    Ok(meta)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn get_meta_data() {
        let path = Path::new("../stt/resources/super_short.mp3");
        let meta = read_metadata(path).unwrap();

        assert_eq!(meta.title, "Linux Action News: Linux Action News 264");
        // commented out since i dont know how to show those images and dont know if we even need them
        // assert_eq!(meta.pictures.len(), 1);
        assert_eq!(meta.unique_id, "http://linuxactionnews.com/264/");
    }
}
