use once_cell::sync::Lazy;
use parking_lot::Mutex;

pub const VERSION: &str = git_version::git_version!(
    args = ["--abbrev=8", "--always", "--dirty=~"],
    prefix = concat!(env!("CARGO_PKG_VERSION"), "-"),
    suffix = "",
    fallback = env!("CARGO_PKG_VERSION")
);

pub static GITHUB_PROXY: &str = "https://ghp.ci";
pub static AUTH_FILE_NAME: &str = ".auth_token";
pub static AUTH_FILE_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
