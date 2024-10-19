use std::{
    fs::{self, File},
    io::Write,
    path,
};

use anyhow::Result;

use crate::{
    model::*,
    utils::{self, EFile},
};
pub async fn easytier_releases(filter_platform: bool) -> Result<Vec<GRelease>> {
    let resp = reqwest::Client::new()
        .get("https://api.github.com/repos/easytier/easytier/releases")
        .header("User-Agent", "EasytierService")
        .send()
        .await?;
    let releases: Vec<GRelease> = resp.json().await?;
    Ok(if filter_platform {
        releases
            .into_iter()
            .map(|r| {
                let mut r = r.clone();
                r.assets = utils::filter_release_with_platform(r.assets);
                r
            })
            .collect()
    } else {
        releases
    })
}

pub async fn download_file(url: &str, file_path: &str, need_file: Vec<EFile>) -> Result<()> {
    let target = format!("https://ghp.ci/{}", url);
    let response = reqwest::get(target).await?;
    let path = path::Path::new(&file_path);

    let mut file = File::create(&path)?;

    let content = response.bytes().await?;

    file.write_all(&content)?;
    utils::unzip(path, need_file)?;

    match fs::remove_file(path) {
        Ok(_) => tracing::info!("delete zip file success"),
        Err(e) => tracing::warn!("delete zip file error: {}", e),
    }

    Ok(())
}
