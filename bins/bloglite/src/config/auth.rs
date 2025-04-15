use directories::UserDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub token: String,
    pub version: String,
}

// pub fn load() -> Config {
//     let path = get_config_path();
//     let config_str = fs::read_to_string(path).unwrap();
//     let config: Config = toml::from_str(&config_str).unwrap();
//     config
// }

pub fn write_auth_config(token: impl Into<String>) {
    let path = get_config_path();

    let config_str = toml::to_string(&Config {
        token: token.into(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
    .unwrap();

    // 获取父目录路径
    if let Some(parent_dir) = path.parent() {
        // 递归创建所有缺失的目录
        std::fs::create_dir_all(parent_dir).unwrap();
    }

    fs::write(path, config_str).unwrap();
}

fn get_config_path() -> PathBuf {
    let user_dirs = UserDirs::new().expect("Failed to get user directories");

    user_dirs
        .home_dir()
        .join(format!(".{}/auth.toml", env!("CARGO_PKG_NAME")))
}
