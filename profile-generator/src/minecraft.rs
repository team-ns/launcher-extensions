use crate::minecraft::fabric::FabricLoaderManifest;
use crate::minecraft::forge::ForgeManifest;

pub mod assets;
pub mod fabric;
pub mod forge;
pub mod libraries;
pub mod version;

pub enum GameType {
    Vanilla,
    Forge(ForgeManifest),
    Fabric(FabricLoaderManifest),
}
