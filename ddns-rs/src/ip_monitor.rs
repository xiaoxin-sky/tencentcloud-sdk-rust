use std::{
    future::Future,
    net::{Ipv4Addr, Ipv6Addr},
    pin::Pin,
    str::FromStr,
    sync::Arc,
    thread,
    time::{Duration, SystemTime},
};

use crossbeam_channel::{select, tick};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::{mpsc, Mutex},
    time::sleep,
};

use crate::ddns::DDNS;

/* async fn get_ip() -> Result<String, Box<dyn std::error::Error>> {
    let resp = reqwest::get("https://6.ipw.cn").await?.text().await?;
    Ok(resp)
}
 */

#[derive(Deserialize)]
pub struct IpV4 {
    query: String,
}

pub struct IpMonitor {
    current_ip: Option<String>,
    last_time: SystemTime,
    check_frequency: Duration,
}

impl IpMonitor {
    pub fn new() -> Self {
        // let current_ip = IpMonitor::get_record_ip(&ddns).await;
        return IpMonitor {
            current_ip: None,
            last_time: SystemTime::now(),
            check_frequency: Duration::from_secs(5),
        };
    }

    pub async fn main_loop(
        &mut self,
        ddns: Arc<Mutex<DDNS>>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        // let ticker = tick(self.check_frequency);
        loop {
            let mut count = 10;
            loop {
                let check_res = &self.check_ip().await?;
                if check_res.to_owned() == true {
                    break;
                } else {
                    count = count - 1;
                    sleep(Duration::from_secs(10)).await;
                    println!("检查失败,正在重试第{}次", count);
                }
            }
            if self.current_ip.is_none() {
                // TODO: 应该发送警告通知管理员
                panic!("重复查询 ip 失败");
            }

            let current_ip = self.current_ip.as_ref().unwrap();

            let (tx, mut rx) = mpsc::channel::<bool>(1);

            // let res = f(current_ip, tx).await;
            ddns.lock().await.change_record_loop(current_ip, tx).await;

            // loop {
            //     match rx.recv().await {
            //         Some(_) => {
            //             println!("回调执行修改成功，开始下一次查询");
            //             break;
            //         }
            //         None => {
            //             // println!("回调执行修改失败，等待再次回调");
            //             continue;
            //         }
            //     };
            // }
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

    async fn check_ip(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        let ip = self.query_ip().await?;
        let current_ip = self.current_ip.take();
        if self.current_ip.is_some() && ip == current_ip.unwrap() || self.current_ip.is_none() {
            self.current_ip = Some(ip);
            self.last_time = SystemTime::now();
        }

        return Ok(true);
    }

    async fn query_ip(&self) -> Result<String, Box<dyn std::error::Error>> {
        let v4 = reqwest::get("http://ip-api.com/json/?lang=zh-CN")
            .await?
            .json::<IpV4>()
            .await?;

        return Ok(v4.query.to_owned());
    }
}
