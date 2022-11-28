use std::{
    net::Ipv6Addr,
    str::FromStr,
    time::{Duration, SystemTime},
};

use crossbeam_channel::{select, tick};
use tokio::time::sleep;

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
    pub async fn new(ddns: DDNS) -> Self {
        let current_ip = IpMonitor::get_record_ip(&ddns).await;
        return IpMonitor {
            current_ip,
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
                    let mut count = 10;
                    loop{
                        let check_res = &self.check_ip().await;
                        if *check_res {
                            break;
                        }else{
                            count = count-1;
                            sleep(Duration::from_secs(10)).await;
                            println!("检查失败,正在重试第{}次",count);
                        }

                    }

                    let record_item = self.ddns.get_current_record().await;
                    if let Some(item) =record_item{
                        if item.value != self.current_ip.to_string(){
                            let res = self.ddns.change_record(item, self.current_ip.to_string()).await;
                            if let Ok(res) = res{
                                println!("更新 ip 状态:{}",res);
                            }else{
                                println!("更新失败");
                            };
                        }
                    }
                },
            }
        }
    }

    /// 获取域名远程ip
    async fn get_record_ip(ddns: &DDNS) -> Ipv6Addr {
        let record_item = ddns.get_current_record().await;
        if let Some(item) = record_item {
            match Ipv6Addr::from_str(&item.value) {
                Ok(ip) => ip,
                Err(_) => Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1),
            }
        } else {
            Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)
        }
    }

    async fn check_ip(&mut self) -> bool {
        if let Ok(ip) = get_ip().await {
            if ip != self.current_ip.to_string() {
                println!(
                    "检测到ip不一致开始更新,原ip:{}, 现ip:{}",
                    self.current_ip.to_string(),
                    ip
                );
                self.current_ip = Ipv6Addr::from_str(&ip).unwrap();
                self.last_time = SystemTime::now();
            } else {
                println!("ip未发生变动,检测时间");
            }
            true
        } else {
            println!("查询ip失败,请检查当前是否处在 ipv6 环境");
            false
        }
    }
}
