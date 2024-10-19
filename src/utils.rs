use std::{iter::repeat_with, path::Path};

use anyhow::Result;

use crate::model::*;

#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EFile {
    #[default]
    All = 0x0,
    Core = 0x1,
    Cli = 0x2,
    #[cfg(windows)]
    Packet = 0x3,
    #[cfg(windows)]
    Wintun = 0x4,
    Other = 0x20,
}

impl EFile {
    pub fn to_u64(list: Vec<Self>) -> u64 {
        use std::collections::BTreeSet;
        let unique: BTreeSet<_> = list.iter().collect();
        let unique_vec: Vec<_> = unique.into_iter().cloned().collect();
        let mut result = 0u64;

        if !unique_vec.contains(&EFile::All) {
            for item in unique_vec {
                result |= 1 << item as u64;
            }
        }

        result
    }

    pub fn from_u64(value: u64) -> Vec<Self> {
        let mut result = Vec::new();

        for i in 0..64 {
            if (value & (1 << i)) != 0 {
                result.push(Self::from_u8(i as u8));
            } else if i == 0 && value == 0 {
                result.push(EFile::All);
            }
        }

        result
    }

    fn from_u8(value: u8) -> Self {
        match value {
            0x0 => EFile::All,
            0x1 => EFile::Core,
            0x2 => EFile::Cli,
            #[cfg(windows)]
            0x3 => EFile::Packet,
            #[cfg(windows)]
            0x4 => EFile::Wintun,
            _ => EFile::Other,
        }
    }

    pub fn from_str(value: &str) -> Self {
        let value = value.to_lowercase().replace(".exe", "").replace(".dll", "");

        if value == String::from("easytier-core") {
            EFile::Core
        } else if value == String::from("easytier-cli") {
            EFile::Cli
        } else if cfg!(target_os = "windows") && value == String::from("packet") {
            EFile::Packet
        } else if cfg!(target_os = "windows") && value == String::from("wintun") {
            EFile::Wintun
        } else {
            EFile::Other
        }
    }
}

pub fn unzip<P>(fname: P, need_file: Vec<EFile>) -> Result<()>
where
    P: AsRef<std::path::Path>,
{
    let need_file = if need_file.len() == 0 {
        vec![EFile::All]
    } else {
        need_file
    };
    let file = std::fs::File::open(fname)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // 遍历压缩文件中的每个条目
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let out_path = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        if let Some(file_name) = out_path.file_name() {
            let file_name_str = utf8_or_gbk_to_string(file_name.as_encoded_bytes());
            if !need_file.contains(&EFile::from_str(&file_name_str))
                && !need_file.contains(&EFile::All)
            {
                continue;
            }
            tracing::info!("extracting {:?}", file_name_str);
        }

        // 创建输出文件的父目录
        if let Some(parent) = out_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        // 如果是目录，直接创建目录
        if file.is_dir() {
            std::fs::create_dir_all(&out_path)?;
        } else {
            // 解压文件
            let mut out_file = std::fs::File::create(&out_path)?;
            std::io::copy(&mut file, &mut out_file)?;
        }

        // 设置文件权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                std::fs::set_permissions(&out_path, std::fs::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(())
}

pub fn filter_release_with_platform(assets: Vec<GAsset>) -> Vec<GAsset> {
    assets
        .into_iter()
        .filter(|asset| {
            !asset.name.contains("gui")
                && asset.name.contains(std::env::consts::OS)
                && asset.name.contains(std::env::consts::ARCH)
        })
        .collect()
}

pub fn utf8_or_gbk_to_string(s: &[u8]) -> String {
    if cfg!(target_os = "windows") {
        use encoding::{all::GBK, DecoderTrap, Encoding};
        if let Ok(utf8_str) = String::from_utf8(s.to_vec()) {
            utf8_str
        } else {
            // 如果解码失败，则尝试使用GBK解码
            if let Ok(gbk_str) = GBK.decode(&s, DecoderTrap::Strict) {
                gbk_str
            } else {
                String::from_utf8_lossy(s).to_string()
            }
        }
    } else {
        String::from_utf8_lossy(s).to_string()
    }
}

pub fn generate_auth_token(num: usize) -> String {
    let token: String = repeat_with(fastrand::alphanumeric).take(num).collect();
    token
}

pub fn get_or_set_auth_token() -> Result<String> {
    let path = Path::new("./.auth_token");

    if path.exists() {
        let content = std::fs::read_to_string(path)?;
        return Ok(content);
    }

    let token = generate_auth_token(32);
    std::fs::write(path, &token)?;
    Ok(token)
}
