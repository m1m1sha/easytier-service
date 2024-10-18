use std::{iter::repeat_with, path::Path};

use anyhow::Result;

use crate::model::*;

pub fn unzip<P>(fname: P) -> Result<()>
where
    P: AsRef<std::path::Path>,
{
    let file = std::fs::File::open(fname)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // 遍历压缩文件中的每个条目
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let out_path = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };
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
