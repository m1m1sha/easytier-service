use std::{
    fs::{self, File},
    io::Write,
    path,
};

use anyhow::Result;

use crate::{model::*, utils};
pub async fn github_releases(org: &str, repo: &str) -> Result<Vec<GRelease>> {
    let resp = reqwest::Client::new()
        .get(format!(
            "https://api.github.com/repos/{}/{}/releases",
            org, repo
        ))
        .header("User-Agent", "Easytier-service")
        .send()
        .await?;

    Ok(resp.json::<Vec<GRelease>>().await?)
}

pub async fn download_file(url: &str, file_path: &str) -> Result<()> {
    let target = format!("https://ghp.ci/{}", url);
    let response = reqwest::get(target)
        .await
        .expect("error to download easytier url");
    let path = path::Path::new(&file_path);

    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}", why),
        Ok(file) => file,
    };

    let content = response.bytes().await.expect("error to bytes easytier");
    println!("下载完成，开始写入");
    file.write_all(&content).expect("error to write easytier");
    println!("写入完成");
    utils::unzip(path)?;
    match fs::remove_file(path) {
        Ok(_) => println!("删除zip文件成功"),
        Err(_) => println!("删除zip文件失败"),
    }

    Ok(())
}
