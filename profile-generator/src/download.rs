use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};
use launcher_extension_api::{Result, Error, Context};
use std::{io, thread};

use reqwest::StatusCode;
use std::thread::JoinHandle;

pub fn download_file<P: AsRef<Path>>(url: &str, path: P) -> Result<()> {
    create_dir_all(&path)?;
    let url_parts: Vec<&str> = url.split('/').collect();
    let output = path.as_ref().join(url_parts.last().context("path is empty")?);
    match reqwest::blocking::get(url) {
        Ok(mut resp) => {
            match resp.status() {
                StatusCode::OK => (),
                _ => {
                    return Err(launcher_extension_api::anyhow!("Could not download this file: {}", url));
                }
            }
            let mut file = File::create(&output)?;
            io::copy(&mut resp, &mut file)?;
        }
        Err(err) => return Err(Error::from(err)),
    };
    Ok(())
}

pub fn download_files_single<P: AsRef<Path>>(download: &[(String, P)]) -> Result<()> {
    for file in download {
        download_file(&file.0, &file.1)?
    }
    Ok(())
}

pub fn download_files_concurrent<P: AsRef<Path>>(download: &[(String, P)]) -> Result<()> {
    let workers: usize = 4;
    let chunks = download.chunks(workers);
    let mut threads: Vec<JoinHandle<Result<()>>> = Vec::new();
    for chunk in chunks {
        let chunk: Vec<(String, PathBuf)> = chunk
            .iter()
            .map(|v| (v.0.to_string(), v.1.as_ref().to_path_buf()))
            .collect();
        threads.push(thread::spawn(move || {
            for file in chunk {
                download_file(&file.0, &file.1)?;
            }
            Ok(())
        }));
    }
    for thread in threads {
        thread.join().map_err(|_| launcher_extension_api::anyhow!("Can't join download thread"))??
    }
    Ok(())
}
