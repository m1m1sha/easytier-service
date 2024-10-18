use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct GRelease {
    pub tag_name: String,
    pub name: String,
    pub prerelease: bool,
    pub draft: bool,
    pub created_at: String,
    pub published_at: String,
    pub id: u64,
    pub assets: Vec<GAsset>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct GAsset {
    pub name: String,
    pub size: usize,
    pub download_count: u64,
    pub browser_download_url: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct EVersion {
    pub core: Option<String>,
    pub cli: Option<String>,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct REasytier {
    pub instance_id: Option<String>,
    pub instance_name: Option<String>,
    pub running: bool,
    pub version: EVersion,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct RInfo {
    pub version: String,
    pub os: String,
    pub arch: String,
    pub list: Vec<REasytier>,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct Resp<T> {
    pub code: Option<i64>,
    pub msg: Option<String>,
    pub data: Option<T>,
}
