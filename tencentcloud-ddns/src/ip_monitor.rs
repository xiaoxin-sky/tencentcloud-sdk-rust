use std::{
    net::Ipv6Addr,
    str::FromStr,
    time::{Duration, SystemTime},
};

use crossbeam_channel::{select, tick};

use crate::ddns::DDNS;

async fn get_ip() -> Result<String, Box<dyn std::error::Error>> {
    let resp = reqwest::get("https://6.ipw.cn").await?.text().await?;
    Ok(resp)
}

pub struct IpMonitor {
    current_ip: Ipv6Addr,
    last_time: SystemTime,
    check_frequency: Duration,
    ddns: DDNS,
}

impl IpMonitor {
    pub fn new(ddns: DDNS) -> Self {
        return IpMonitor {
            current_ip: Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1),
            last_time: SystemTime::now(),
            check_frequency: Duration::from_secs(5),
            ddns,
        };
    }

    pub async fn main_loop(&mut self) {
        let ticker = tick(self.check_frequency);
        loop {
            select! {
                recv(ticker) -> _ => {
                    let _ = &self.check_ip().await;
                    let _ = self.ddns.change_record(self.current_ip.to_string()).await;
                },
            }
        }
    }

    async fn check_ip(&mut self) {
        if let Ok(ip) = get_ip().await {
            if ip != self.current_ip.to_string() {
                println!("检测到ip不一致开始更新,原ip{}", self.current_ip.to_string());
                self.current_ip = Ipv6Addr::from_str(&ip).unwrap();
                self.last_time = SystemTime::now();
            }
        }
    }
}
