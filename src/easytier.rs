use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use tokio::process::Command;

use crate::{
    constant,
    http::{self, download_file},
    model::{EVersion, GRelease},
    utils,
};

pub fn exists() -> bool {
    let prefix = format!(
        "./easytier-{}-{}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );

    #[cfg(windows)]
    return Path::new(&format!("{}/easytier-core.exe", prefix)).exists()
        && Path::new(&format!("{}/easytier-cli.exe", prefix)).exists()
        && Path::new(&format!("{}/Packet.dll", prefix)).exists()
        && Path::new(&format!("{}/wintun.dll", prefix)).exists();

    #[cfg(not(windows))]
    return Path::new(&format!("{}/easytier-core", prefix)).exists()
        && Path::new(&format!("{}/easytier-cli", prefix)).exists();
}

async fn get_version(path: &str, prefix: &str) -> Option<String> {
    if Path::new(path).exists() {
        match Command::new(path)
            .arg("--version")
            .creation_flags(0x08000000)
            .output()
            .await
        {
            Ok(output) => Some(
                utils::utf8_or_gbk_to_string(&output.stdout)
                    .replace("\n", "")
                    .replace(prefix, ""),
            ),
            Err(_e) => None,
        }
    } else {
        None
    }
}

pub async fn version() -> Result<EVersion> {
    if exists() {
        let core_path = format!(
            "./easytier-{}-{}/{}{}",
            std::env::consts::OS,
            std::env::consts::ARCH,
            "easytier-core",
            if cfg!(target_os = "windows") {
                ".exe"
            } else {
                ""
            }
        );

        let cli_path = format!(
            "./easytier-{}-{}/{}{}",
            std::env::consts::OS,
            std::env::consts::ARCH,
            "easytier-cli",
            if cfg!(target_os = "windows") {
                ".exe"
            } else {
                ""
            }
        );

        let core = get_version(&core_path, "easytier-core ").await;
        let cli = get_version(&cli_path, "easytier-cli ").await;

        return Ok(EVersion { core, cli });
    }

    Err(anyhow!("easytier not exists"))
}
pub async fn check_update() -> Result<Option<GRelease>> {
    // 获取所有发布的版本
    let releases = match http::easytier_releases(true).await {
        Ok(releases) => releases,
        Err(e) => return Err(e),
    };

    // 获取最新的发布版本
    let latest_release = match releases.first() {
        Some(release) => release,
        None => return Ok(None), // 如果没有发布版本，直接返回 None
    };

    // 获取当前版本
    let current_version = match version().await {
        Ok(version) => version,
        Err(e) => return Err(e),
    };

    // 检查当前版本是否存在，并且 core 和 cli 版本是否都存在
    if current_version.core.is_none() || current_version.cli.is_none() {
        return Ok(Some(latest_release.clone()));
    }

    Ok(None)
}

async fn download_missing_files(prefix: &PathBuf, need_file: Vec<utils::EFile>) -> Result<()> {
    let releases = http::easytier_releases(true).await?;

    if let Some(latest_release) = releases.first() {
        if let Some(asset) = latest_release.assets.first() {
            download_file(
                &format!("{}/{}", constant::GITHUB_PROXY, asset.browser_download_url),
                &prefix.join(asset.name.clone()),
                need_file,
            )
            .await?;
        } else {
            return Err(anyhow::anyhow!("No assets found in the latest release"));
        }
    } else {
        return Err(anyhow::anyhow!("No releases found"));
    }

    Ok(())
}

pub async fn check_exists(force: bool) -> Result<()> {
    let prefix = PathBuf::from(format!(
        "./easytier-{}-{}",
        std::env::consts::OS,
        std::env::consts::ARCH
    ));

    let mut need_file = vec![];
    if force {
        need_file.push(utils::EFile::All);
    } else {
        let files_to_check = if cfg!(target_os = "windows") {
            vec![
                ("easytier-core.exe", utils::EFile::Core),
                ("easytier-cli.exe", utils::EFile::Cli),
                ("Packet.dll", utils::EFile::Packet),
                ("wintun.dll", utils::EFile::Wintun),
            ]
        } else {
            vec![
                ("easytier-core", utils::EFile::Core),
                ("easytier-cli", utils::EFile::Cli),
            ]
        };

        for (file, e) in files_to_check {
            if !prefix.join(file).exists() {
                need_file.push(e);
            }
        }
    }

    tracing::info!("check need file: {:?}", need_file);

    if !need_file.is_empty() {
        download_missing_files(&prefix, need_file).await?;
    }

    Ok(())
}
