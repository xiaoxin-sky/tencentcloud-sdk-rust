use std::{fs, path::Path};

use clap::{builder::Str, Parser};
use ddns::DDNS;
use ip_monitor::IpMonitor;
use serde::{Deserialize, Serialize};

mod ddns;
mod ip_monitor;

#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    /// 腾讯云 secret_id
    secret_id: String,
    /// 腾讯云 secret_key
    secret_key: String,
    /// 需要绑定的域名
    domain: String,
}

#[derive(Deserialize, Debug)]
struct Config {
    /// 腾讯云 secret_id
    secret_id: String,
    /// 腾讯云 secret_key
    secret_key: String,
    /// 需要绑定的域名
    domain: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let args = Args::parse();

    let path = Path::new("config.toml");

    let config = get_config(&path);
    if let Some(config) = config {
        let ddns = DDNS::new(config.secret_id, config.secret_key, config.domain);

        let mut ip_monitor = IpMonitor::new(ddns).await;

        ip_monitor.main_loop().await;
    } else {
        println!("没有找到配置文件 config.toml");
    }

    Ok(())
}

/// 查询配置文件
fn get_config(path: &Path) -> Option<Config> {
    let config_string = fs::read_to_string(path);
    if let Ok(config) = config_string {
        match toml::from_str(&config) {
            Ok(config) => Some(config),
            Err(e) => None,
        }
    } else {
        return None;
    }
}
