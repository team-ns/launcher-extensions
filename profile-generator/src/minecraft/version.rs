use crate::minecraft::libraries::{AssetIndex, Downloads, Library};
use launcher_extension_api::{Error, Result};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize)]
pub struct VersionManifest {
    pub latest: Latest,
    pub versions: Vec<Version>,
}

#[derive(Serialize, Deserialize)]
pub struct Latest {
    release: String,
    snapshot: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub id: String,
    #[serde(alias = "type")]
    pub v_type: String,
    pub url: String,
    pub time: String,
    pub release_time: String,
}

#[derive(Serialize, Deserialize)]
pub struct Libraries {
    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndex,
    pub downloads: Downloads,
    pub libraries: Vec<Library>,
}

#[derive(Deserialize, Serialize)]
pub struct NameLibrary {
    pub name: String,
    #[serde(default = "default_lib")]
    pub url: String,
}

fn default_lib() -> String {
    "https://libraries.minecraft.net/".to_string()
}

impl FromStr for Libraries {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let manifest = reqwest::blocking::get(
            "https://launchermeta.mojang.com/mc/game/version_manifest.json",
        )?
        .json::<VersionManifest>()?;
        let version = manifest
            .versions
            .iter()
            .find(|v| v.id.eq(s))
            .ok_or(launcher_extension_api::anyhow!("Incorrect minecraft version"))?;
        let libs = reqwest::blocking::get(&version.url)?.json::<Libraries>()?;
        Ok(libs)
    }
}
