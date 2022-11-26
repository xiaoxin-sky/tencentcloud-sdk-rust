use ddns::DDNS;
use ip_monitor::IpMonitor;

mod ddns;
mod ip_monitor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let secret_id = "AKIDaw2uWvq1VlFTQ8rpiLiW6Vs12pxpbnQ1".to_string();
    let secret_key = "XpwPc3E991d7nDoY3IYcMHwlLn0xpVgb".to_string();
    let domain = "9cka.cn".to_string();

    let ddns = DDNS::new(secret_id, secret_key, domain);

    let mut ip_monitor = IpMonitor::new(ddns);

    ip_monitor.main_loop().await;

    Ok(())
}
