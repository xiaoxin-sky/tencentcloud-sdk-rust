use std::{
    env,
    fs::{self, File},
    path::{Path, PathBuf},
};

use clap::Parser;
use daemonize::Daemonize;
use ddns::DDNS;
use ip_monitor::IpMonitor;
use serde::Deserialize;

mod ddns;
mod ip_monitor;

#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    /// 腾讯云 secret_id
    #[arg(default_value_t = String::from(""))]
    secret_id: String,
    /// 腾讯云 secret_key
    #[arg(default_value_t = String::from(""))]
    secret_key: String,
    /// 需要绑定的域名
    #[arg(short, long, default_value_t = String::from(""))]
    domain: String,
    /// 配置文件地址
    #[arg(short, long, default_value_t = String::from("config.toml"))]
    config: String,
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
    let args = Args::parse();

    let config_path = PathBuf::from(args.config);

    let config = get_config(&config_path);
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
    match config_string {
        Ok(config) => match toml::from_str(&config) {
            Ok(config) => Some(config),
            Err(e) => {
                println!("{}", e);
                None
            }
        },
        Err(e) => {
            println!("readFile error-{}", e);
            return None;
        }
    }
}

/// 创建进程守护
fn create_daemonize() {
    let stdout = File::create("/tmp/daemon.out").unwrap();
    let stderr = File::create("/tmp/daemon.err").unwrap();

    let daemonize = Daemonize::new()
        .pid_file("/tmp/test.pid") // Every method except `new` and `start`
        .chown_pid_file(true) // is optional, see `Daemonize` documentation
        .working_directory("/tmp") // for default behaviour.
        .user("nobody")
        .group("daemon") // Group name
        .group(2) // or group id.
        .umask(0o777) // Set umask, `0o027` by default.
        .stdout(stdout) // Redirect stdout to `/tmp/daemon.out`.
        .stderr(stderr) // Redirect stderr to `/tmp/daemon.err`.
        .privileged_action(|| "Executed before drop privileges");

    match daemonize.start() {
        Ok(_) => println!("start success"),
        Err(e) => eprintln!("Error, {}", e),
    }
}
