use crate::minecraft::libraries::Library;
use crate::minecraft::version::NameLibrary;
use launcher_extension_api::{Error, Result};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Deserialize, Serialize)]
pub struct ForgeManifest {
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(rename = "+tweakers")]
    pub tweakers: Option<Vec<String>>,
    #[serde(rename = "mavenFiles")]
    pub maven_files: Option<Vec<Library>>,
    pub libraries: Vec<LibraryType>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum LibraryType {
    PathLibrary(Library),
    NameLibrary(NameLibrary),
}

impl FromStr for ForgeManifest {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url: String = format!(
            "https://meta.multimc.org/v1/net.minecraftforge/{ver}.json",
            ver = s
        );
        let manifest = reqwest::blocking::get(&url)?.json::<ForgeManifest>()?;
        Ok(manifest)
    }
}
