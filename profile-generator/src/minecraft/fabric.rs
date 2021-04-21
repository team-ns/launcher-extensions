use crate::minecraft::version::NameLibrary;
use launcher_extension_api::{Error, Result};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Deserialize, Serialize)]
pub struct FabricLoaderManifest {
    pub version: i32,
    pub libraries: FabricLibraries,
    #[serde(rename = "mainClass")]
    pub main_class: FabricMainClass,
}

#[derive(Deserialize, Serialize)]
pub struct FabricMainClass {
    pub client: String,
}

#[derive(Deserialize, Serialize)]
pub struct FabricLibraries {
    pub client: Vec<NameLibrary>,
    pub common: Vec<NameLibrary>,
}

impl FromStr for FabricLoaderManifest {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url: String = format!(
            "https://maven.fabricmc.net/net/fabricmc/fabric-loader/{ver}/fabric-loader-{ver}.json",
            ver = s
        );
        let manifest = reqwest::blocking::get(&url)?.json::<FabricLoaderManifest>()?;
        Ok(manifest)
    }
}
