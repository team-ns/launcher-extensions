use crate::download::{download_file, download_files_concurrent, download_files_single};
use crate::minecraft::forge::LibraryType;
use crate::minecraft::version::Libraries;
use crate::minecraft::GameType;
use crate::minecraft::GameType::{Fabric, Forge};
use crate::util::{generate_download_url, generate_lib_path, get_yarn_path, get_yarn_url, jar_url};
use launcher_extension_api::Result;
use launcher_extension_api::launcher::profile::Profile;
use std::collections::{HashSet, HashMap};
use std::fs::{create_dir_all, remove_dir_all, File};
use std::io;
use std::iter::FromIterator;
use std::path::PathBuf;
use walkdir::WalkDir;
use zip::ZipArchive;
use launcher_extension_api::launcher::optional::{Optional, Action, FileAction, Location, OptionalFiles, Rule as LauncherRule, OsRule, CompareMode};
use regex::Regex;
use launcher_extension_api::launcher::validation::OsType;
use path_slash::PathBufExt;

pub fn generate_profile(
    name: &str,
    version: &str,
    manifest: Libraries,
    address: &str,
    port: u32,
    game_type: GameType,
    assets: Option<&str>,
) -> Result<()> {
    let base = PathBuf::from("static");
    let native_folder = &base.join("natives").join(version);
    let assets_folder = &base.join("assets").join(assets.unwrap_or(name));
    let profile_folder = &base.join("profiles").join(name);
    let libraries_folder = base.join("libraries");
    let mut client_args = Vec::new();
    let mut main_class = "net/minecraft/client/main/Main".to_string();
    let mut classpath = Vec::new();
    let mut optionals: Vec<Optional> = Vec::new();
    std::fs::create_dir_all(&native_folder)?;
    std::fs::create_dir_all(&assets_folder)?;
    std::fs::create_dir_all(&profile_folder)?;
    std::fs::create_dir_all(&libraries_folder)?;
    if assets.is_none() {
        println!("Download assets...");
        let assets = crate::util::get_assets(&manifest.asset_index.url)?;
        let objects_path = assets_folder.join("objects");
        let mut assets_download = Vec::new();
        for (_, object) in assets.objects {
            let path = objects_path.join(&object.hash[0..2]);
            assets_download.push((
                format!(
                    "https://resources.download.minecraft.net/{}/{}",
                    &object.hash[0..2],
                    object.hash
                ),
                path,
            ));
        }
        download_files_single(&assets_download)?;
        download_file(
            &manifest.asset_index.url,
            assets_folder.join("indexes").to_str().unwrap(),
        )?;
    }
    println!("Download client...");
    download_file(
        &manifest.downloads.client.unwrap().url,
        &profile_folder.to_str().unwrap(),
    )?;
    std::fs::rename(
        profile_folder.join("client.jar").as_path(),
        profile_folder.join("minecraft.jar").as_path(),
    )?;
    classpath.push("minecraft.jar".to_string());
    println!("Download libs...");
    let version_regex = Regex::new(r"-\d.\d.\d.+").unwrap();
    let mut profile_lib_paths = HashSet::new();
    let libs = &manifest
        .libraries;
    let mut download_list = Vec::with_capacity(libs.len());
    for lib in libs {
        if let Some(file) = &lib.downloads.artifact {
            let lib_path = PathBuf::from(
                file
                    .path
                    .as_ref()
                    .unwrap()
                    .split("/")
                    .last()
                    .unwrap(),
            );
            if let Some(rules) = &lib.rules {
                for rule in rules {
                    let file_name = version_regex.replace_all(lib_path.file_name().unwrap().to_str().unwrap(), ".jar");
                    let new_lib_path = lib_path.with_file_name(&file_name.to_string());
                    let rename_list = {
                        let mut map = HashMap::new();
                        map.insert(lib_path.clone(), PathBuf::from(new_lib_path.to_slash_lossy()));
                        map
                    };
                    profile_lib_paths.insert(new_lib_path.to_slash_lossy());
                    let action = Action::Files(FileAction {
                        location: Location::Libraries,
                        files: OptionalFiles {
                            original_paths: vec![],
                            rename_paths: rename_list,
                        },
                    });
                    let optional_rule = if rule.action.eq("allow") && rule.os.is_none() {
                        LauncherRule::OsType(OsRule {
                            os_type: OsType::MacOsX64,
                            compare_mode: CompareMode::Unequal,
                        })
                    } else if rule.action.eq("allow") && rule.os.as_ref().map(|os| &os.name).map(|name| name.eq("osx")).unwrap_or(false) {
                        LauncherRule::OsType(OsRule {
                            os_type: OsType::MacOsX64,
                            compare_mode: CompareMode::Equal,
                        })
                    } else {
                        continue;
                    };
                    let optional = Optional {
                        actions: vec![action],
                        rules: vec![optional_rule],
                        enabled: true,
                        visible: false,
                        description: None,
                        name: None,
                    };
                    optionals.push(optional);
                }
            } else {
                profile_lib_paths.insert(lib_path.to_str().unwrap().to_string());
            }
            let url = lib.downloads.artifact.as_ref().unwrap().url.to_string();
            download_list.push((url, libraries_folder.clone()));
        }
    }
    download_files_concurrent(&download_list)?;
    match game_type {
        Fabric(mut fabric_manifest) => {
            fabric_manifest
                .libraries
                .client
                .append(&mut fabric_manifest.libraries.common);
            let mut download_list = Vec::with_capacity(fabric_manifest.libraries.client.len());
            for v in fabric_manifest.libraries.client {
                let mut lib_path = PathBuf::from(generate_lib_path(&v.name).split("/")
                    .last()
                    .unwrap());
                profile_lib_paths.insert(lib_path.to_str().unwrap().to_string());
                lib_path.pop();
                let url = generate_download_url(&v.url, &v.name);
                download_list.push((url, libraries_folder.clone()));
            }
            download_files_concurrent(&download_list)?;
            let mappings_url = get_yarn_url(version);
            let mut lib_path = PathBuf::from(get_yarn_path(&version));
            profile_lib_paths.insert(lib_path.to_str().unwrap().to_string());
            lib_path.pop();
            download_file(&mappings_url, libraries_folder.clone())?;
            main_class = fabric_manifest.main_class.client;
        }
        Forge(forge_manifest) => {
            main_class = forge_manifest.main_class;
            let mut download_list = Vec::with_capacity(forge_manifest.libraries.len());
            for library in &forge_manifest.libraries {
                match library {
                    LibraryType::PathLibrary(v) => {
                        if v.downloads.artifact.as_ref().unwrap().path.is_some() {
                            let lib_path = PathBuf::from(
                                v.downloads
                                    .artifact
                                    .as_ref()
                                    .unwrap()
                                    .path
                                    .as_ref()
                                    .unwrap()
                                    .split("/")
                                    .last()
                                    .unwrap()
                                    .to_string(),
                            );
                            profile_lib_paths.insert(lib_path.to_str().unwrap().to_string());
                            let url = v.downloads.artifact.as_ref().unwrap().url.to_string();
                            download_list.push((url, libraries_folder.clone()));
                        }
                    }
                    LibraryType::NameLibrary(v) => {
                        let mut lib_path = PathBuf::from(generate_lib_path(&v.name)
                            .split("/")
                            .last()
                            .unwrap()
                            .to_string());
                        profile_lib_paths.insert(lib_path.to_str().unwrap().to_string());
                        lib_path.pop();
                        let url = generate_download_url(&v.url, &v.name);
                        download_list.push((url, libraries_folder.clone()));
                    }
                }
            }
            download_files_concurrent(&download_list)?;
            if let Some(files) = forge_manifest.maven_files {
                let mut download_list = Vec::with_capacity(files.len());
                for v in files {
                    if v.downloads.artifact.as_ref().unwrap().path.is_some() {
                        let mut lib_path = PathBuf::from(
                            v.downloads
                                .artifact
                                .as_ref()
                                .unwrap()
                                .path
                                .as_ref()
                                .unwrap()
                                .to_string(),
                        );
                        lib_path.pop();
                        let url = v.downloads.artifact.as_ref().unwrap().url.to_string();
                        let path = libraries_folder.join(lib_path);
                        download_list.push((url, path));
                    }
                }
                download_files_concurrent(&download_list)?;
            }
            if let Some(tweakers) = forge_manifest.tweakers {
                for tweak in tweakers {
                    client_args.push("--tweakClass".to_string());
                    client_args.push(tweak);
                }
            }
        }
        _ => {}
    }
    println!("Download natives...");
    let temp_natives = base.join("natives_temp");
    create_dir_all(&temp_natives)?;
    let natives = manifest
        .libraries
        .iter()
        .filter(|v| v.downloads.classifiers.is_some())
        .flat_map(|v| {
            let mut natives: Vec<(String, PathBuf)> = Vec::new();
            if let Some(f) = v
                .downloads
                .classifiers
                .as_ref()
                .unwrap()
                .natives_osx
                .as_ref()
            {
                natives.push(jar_url(&temp_natives, f));
            }
            if let Some(f) = v
                .downloads
                .classifiers
                .as_ref()
                .unwrap()
                .natives_windows
                .as_ref()
            {
                natives.push(jar_url(&temp_natives, f));
            }
            if let Some(f) = v
                .downloads
                .classifiers
                .as_ref()
                .unwrap()
                .natives_linux
                .as_ref()
            {
                natives.push(jar_url(&temp_natives, f));
            }
            natives
        })
        .collect::<Vec<_>>();
    download_files_concurrent(&natives)?;
    for entry in WalkDir::new(&temp_natives)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
    {
        if let Ok(mut zip) = ZipArchive::new(File::open(entry.path()).unwrap()) {
            for index in 0..zip.len() {
                let mut file = zip.by_index(index).unwrap();
                if file.is_file() {
                    if file.name().ends_with(".so")
                        || file.name().ends_with(".dll")
                        || file.name().ends_with(".dylib")
                    {
                        if let Ok(mut outfile) =
                        File::create(native_folder.join(&file.mangled_name()))
                        {
                            io::copy(&mut file, &mut outfile)?;
                        }
                    }
                }
            }
        }
    }
    remove_dir_all(temp_natives)?;
    println!("Generate json profile...");
    serde_json::to_writer_pretty(
        File::create(profile_folder.join(format!("profile.json"))).unwrap(),
        &Profile {
            name: name.to_string(),
            version: version.to_string(),
            libraries: Vec::from_iter(profile_lib_paths),
            class_path: classpath,
            main_class,
            update_verify: vec![],
            update_exclusion: vec![],
            jvm_args: vec![],
            client_args,
            assets: manifest.asset_index.id,
            assets_dir: format!("assets/{}", assets.unwrap_or(name)),
            server_name: address.to_string(),
            server_port: port,
        },
    )?;
    println!("Generate optionals...");
    serde_json::to_writer_pretty(
        File::create(profile_folder.join(format!("optionals.json"))).unwrap(),
        &optionals,
    )?;
    Ok(())
}

