use launcher_extension_api::command::ExtensionCommandExecutor;
use clap::{App, AppSettings, Arg};
use crate::{validator, generator};
use crate::minecraft::version::Libraries;
use crate::minecraft::fabric::FabricLoaderManifest;
use crate::minecraft::forge::ForgeManifest;
use crate::minecraft::GameType;

pub struct ProfileGenerationCommand<'a> {
    app: App<'a>
}

impl Default for ProfileGenerationCommand<'_> {
    fn default() -> Self {
        let app = App::new("NSLauncher Profile Generator")
            .setting(AppSettings::NoBinaryName)
            .version("1.0")
            .author("Team NS")
            .about("Generate profile for NSLauncher")
            .arg(
                Arg::new("version")
                    .short('v')
                    .required(true)
                    .long("version")
                    .takes_value(true)
                    .about("Minecraft Version"),
            )
            .arg(
                Arg::new("profileName")
                    .short('n')
                    .required(true)
                    .long("name")
                    .takes_value(true)
                    .about("Profile name"),
            )
            .arg(
                Arg::new("serverName")
                    .short('a')
                    .required(true)
                    .long("address")
                    .takes_value(true)
                    .default_value("localhost")
                    .about("Server address"),
            )
            .arg(
                Arg::new("serverPort")
                    .short('p')
                    .required(true)
                    .long("port")
                    .takes_value(true)
                    .default_value("25565")
                    .about("Server port"),
            )
            .arg(
                Arg::new("forge")
                    .about("Forge Version")
                    .long("forge")
                    .takes_value(true)
                    .conflicts_with("fabric")
                    .validator(validator::correct_forge_version),
            )
            .arg(
                Arg::new("assets")
                    .about("Minecraft assets")
                    .long("assets")
                    .takes_value(true),
            )
            .arg(
                Arg::new("fabric")
                    .about("Fabric Loader Version")
                    .long("fabric")
                    .takes_value(true)
                    .conflicts_with("forge")
                    .validator(validator::correct_fabric_version),
            );
        Self {
            app
        }
    }
}

impl ExtensionCommandExecutor for ProfileGenerationCommand<'_> {
    fn execute(&self, args: &[&str]) {
        let app = self.app.clone();
        let result = app.try_get_matches_from(args);
        let matches = match result {
            Ok(matches) => {
                matches
            }
            Err(e) => {
                println!("Argument {}", e);
                return;
            }
        };
        let profile_name = if let Some(val) = matches
            .value_of("profileName") {
            val
        } else {
            println!("Can't get profileName");
            return;
        };
        let assets = matches.value_of("assets");
        let game_version = if let Some(val) = matches.value_of("version") {
            val
        } else {
            println!("Can't get version");
            return;
        };
        let game_libraries = if let Ok(val) = matches
            .value_of_t::<Libraries>("version")
        {
            val
        } else {
            println!("Can't get libs");
            return;
        };
        let address = if let Some(val) = matches
            .value_of("serverName") {
            val
        } else {
            println!("Can't get server address");
            return;
        };
        let port = if let Ok(val) = matches
            .value_of_t::<u32>("serverPort") {
            val
        } else {
            println!("Can't get server port");
            return;
        };
        let fabric = matches.value_of_t::<FabricLoaderManifest>("fabric");
        let forge = matches.value_of_t::<ForgeManifest>("forge");
        let game_type = if let Ok(manifest) = fabric {
            GameType::Fabric(manifest)
        } else if let Ok(manifest) = forge {
            GameType::Forge(manifest)
        } else {
            GameType::Vanilla
        };
        match generator::generate_profile(
            profile_name,
            game_version,
            game_libraries,
            address,
            port,
            game_type,
            assets
        ) {
            Err(e) => {
                println!("Can't generate profile: {}", e)
            }
            Ok(_) => {
                println!("Profile is generated")
            }
        }
    }
}