use std::{os::windows::process::CommandExt, path::Path, process::Command};

use anyhow::Result;

use crate::{model::EVersion, utils};

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

pub fn version() -> EVersion {
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
        {
            Some(
                utils::utf8_or_gbk_to_string(&output.stdout)
                    .replace("\n", "")
                    .replace("easytier-cli ", ""),
            )
        } else {
            None
        };

        EVersion { core, cli }
    } else {
        EVersion::default()
    }
}
