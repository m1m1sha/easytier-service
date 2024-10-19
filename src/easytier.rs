use std::{os::windows::process::CommandExt, path::Path};

use anyhow::{anyhow, Result};
use tokio::process::Command;

use crate::{
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

    if cfg!(target_os = "windows") {
        Path::new(&format!("{}/easytier-core.exe", prefix)).exists()
            && Path::new(&format!("{}/easytier-cli.exe", prefix)).exists()
            && Path::new(&format!("{}/Packet.dll", prefix)).exists()
            && Path::new(&format!("{}/wintun.dll", prefix)).exists()
    } else {
        Path::new(&format!("{}/easytier-core", prefix)).exists()
            && Path::new(&format!("{}/easytier-cli", prefix)).exists()
    }
}

pub async fn version() -> Result<EVersion> {
    if exists() {
        let core = if let Ok(output) = Command::new(format!(
            "./easytier-{}-{}/{}{}",
            std::env::consts::OS,
            std::env::consts::ARCH,
            "easytier-core",
            if cfg!(target_os = "windows") {
                ".exe"
            } else {
                ""
            }
        ))
        .arg("--version")
        .creation_flags(0x08000000)
        .output()
        .await
        {
            Some(
                utils::utf8_or_gbk_to_string(&output.stdout)
                    .replace("\n", "")
                    .replace("easytier-core ", ""),
            )
        } else {
            None
        };

        let cli = if let Ok(output) = Command::new(format!(
            "./easytier-{}-{}/{}{}",
            std::env::consts::OS,
            std::env::consts::ARCH,
            "easytier-cli",
            if cfg!(target_os = "windows") {
                ".exe"
            } else {
                ""
            }
        ))
        .arg("--version")
        .creation_flags(0x08000000)
        .output()
        .await
        {
            Some(
                utils::utf8_or_gbk_to_string(&output.stdout)
                    .replace("\n", "")
                    .replace("easytier-cli ", ""),
            )
        } else {
            None
        };

        return Ok(EVersion { core, cli });
    }

    Err(anyhow!("easytier not exists"))
}

pub async fn check_update() -> Result<Option<GRelease>> {
    let releases = http::easytier_releases(true).await?;

    let latest_release = releases.first().unwrap();

    check_exists(false).await?;

    let current_version = version().await?;

    if !exists() || current_version.core.is_none() || current_version.cli.is_none() {
        return Ok(Some(latest_release.clone()));
    }
    Ok(None)
}

pub async fn check_exists(replace: bool) -> Result<()> {
    let prefix = format!(
        "./easytier-{}-{}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );

    let mut need_file = vec![];
    if replace {
        need_file.push(utils::EFile::All);
    } else {
        if cfg!(target_os = "windows") {
            if !Path::new(&format!("{}/easytier-core.exe", prefix)).exists() {
                need_file.push(utils::EFile::Core);
            }
            if !Path::new(&format!("{}/easytier-cli.exe", prefix)).exists() {
                need_file.push(utils::EFile::Cli);
            }
            if !Path::new(&format!("{}/Packet.dll", prefix)).exists() {
                need_file.push(utils::EFile::Packet);
            }
            if !Path::new(&format!("{}/wintun.dll", prefix)).exists() {
                need_file.push(utils::EFile::Wintun);
            }
        } else {
            if !Path::new(&format!("{}/easytier-core", prefix)).exists() {
                need_file.push(utils::EFile::Core);
            }
            if !Path::new(&format!("{}/easytier-cli", prefix)).exists() {
                need_file.push(utils::EFile::Cli);
            }
        }
    }

    tracing::info!("check need file: {:?}", need_file);

    if need_file.len() > 0 {
        let releases = http::easytier_releases(true).await?;

        let latest_release = releases.first().unwrap().assets.first().unwrap();
        download_file(
            &latest_release.browser_download_url,
            &format!("./{}", latest_release.name),
            need_file,
        )
        .await?;
    }

    Ok(())
}
