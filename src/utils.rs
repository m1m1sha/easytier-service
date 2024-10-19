use std::{collections::BTreeSet, iter::repeat_with, path::Path};

use anyhow::Result;
use tokio::{
    fs::OpenOptions,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
};

use crate::{
    constant::{self},
    model::*,
};

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
        // 定义常量来存储硬编码字符串
        const CORE: &str = "easytier-core";
        const CLI: &str = "easytier-cli";
        #[cfg(windows)]
        const PACKET: &str = "packet";
        #[cfg(windows)]
        const WINTUN: &str = "wintun";

        // 处理字符串，转换为小写并移除扩展名
        let processed_value = value.to_lowercase().replace(".exe", "").replace(".dll", "");

        // 检查空字符串和特殊字符
        if processed_value.is_empty()
            || !processed_value
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-')
        {
            return EFile::Other;
        }

        #[cfg(windows)]
        if processed_value == PACKET {
            return EFile::Packet;
        } else if processed_value == WINTUN {
            return EFile::Wintun;
        }

        // 处理其他条件
        if processed_value == CORE {
            EFile::Core
        } else if processed_value == CLI {
            EFile::Cli
        } else {
            EFile::Other
        }
    }
}

pub fn unzip<P>(fname: P, need_file: Vec<EFile>) -> Result<()>
where
    P: AsRef<Path>,
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
    #[cfg(windows)]
    match try_utf8_or_gbk_decode(s) {
        Ok(decoded) => decoded,
        Err(_e) => String::from_utf8_lossy(s).to_string(),
    }

    #[cfg(not(windows))]
    String::from_utf8_lossy(s).to_string()
}

#[cfg(windows)]
fn try_utf8_or_gbk_decode(s: &[u8]) -> Result<String, String> {
    use encoding::{all::GBK, DecoderTrap, Encoding};
    match String::from_utf8(s.to_vec()) {
        Ok(utf8_str) => Ok(utf8_str),
        Err(_) => match GBK.decode(s, DecoderTrap::Replace) {
            Ok(gbk_str) => Ok(gbk_str),
            Err(e) => Err(format!("GBK decoding failed: {}", e)),
        },
    }
}

pub fn random_string(num: usize) -> String {
    repeat_with(fastrand::alphanumeric)
        .take(num)
        .collect::<String>()
}

pub async fn get_auth_token() -> Result<Vec<String>> {
    let mut tokens = BTreeSet::new();
    read_tokens_from_file(&mut tokens).await?;

    if tokens.len() == 0 {
        tokens.insert(random_string(32));
        set_auto_token(&mut tokens).await?;
    }

    Ok(tokens.into_iter().collect())
}

pub async fn read_tokens_from_file(tokens: &mut BTreeSet<String>) -> Result<()> {
    let file = OpenOptions::new()
        .read(true)
        .open(constant::AUTH_FILE_NAME)
        .await?;
    let buf = BufReader::new(file);
    let mut lines = buf.lines();
    while let Some(line) = lines.next_line().await? {
        if !line.is_empty() {
            tokens.insert(line);
        }
    }
    Ok(())
}

pub async fn set_auto_token(tokens: &mut BTreeSet<String>) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .open(constant::AUTH_FILE_NAME)
        .await?;

    let content = tokens
        .clone()
        .into_iter()
        .collect::<Vec<String>>()
        .join("\n");
    file.write_all(content.as_bytes()).await?;

    Ok(())
}
