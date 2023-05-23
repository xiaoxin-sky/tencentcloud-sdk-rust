use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use tencentcloud_sdk_rs::client::ReqClient;
use tokio::time::sleep;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct ModifyRecordRequest {
    /**
     * 域名
     */
    Domain: String,

    /**
     * 记录类型，通过 API 记录类型获得，大写英文，比如：A 。
     */
    RecordType: String,

    /**
     * 记录线路，通过 API 记录线路获得，中文，比如：默认。
     */
    RecordLine: String,

    /**
     * 记录值，如 IP : 200.200.200.200， CNAME : cname.dnspod.com.， MX : mail.dnspod.com.。
     */
    Value: String,

    /**
     * 记录 ID 。可以通过接口DescribeRecordList查到所有的解析记录列表以及对应的RecordId
     */
    RecordId: i64,

    /**
     * 域名 ID 。参数 DomainId 优先级比参数 Domain 高，如果传递参数 DomainId 将忽略参数 Domain 。可以通过接口DescribeDomainList查到所有的Domain以及DomainId
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    DomainId: Option<i64>,

    /**
     * 主机记录，如 www，如果不传，默认为 @。
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    SubDomain: Option<String>,

    /**
     * 线路的 ID，通过 API 记录线路获得，英文字符串，比如：10=1。参数RecordLineId优先级高于RecordLine，如果同时传递二者，优先使用RecordLineId参数。
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    RecordLineId: Option<String>,

    /**
     * MX 优先级，当记录类型是 MX 时有效，范围1-20，MX 记录时必选。
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    MX: Option<i64>,

    /**
     * TTL，范围1-604800，不同等级域名最小值不同。
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    TTL: Option<i64>,

    /**
     * 权重信息，0到100的整数。仅企业 VIP 域名可用，0 表示关闭，不传该参数，表示不设置权重信息。
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    Weight: Option<i64>,

    /**
     * 记录初始状态，取值范围为 ENABLE 和 DISABLE 。默认为 ENABLE ，如果传入 DISABLE，解析不会生效，也不会验证负载均衡的限制。
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    Status: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct DescribeRecordList {
    domain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    record_type: Option<String>,
}

/// 记录列表元素
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct RecordListItem {
    /// 记录Id
    pub record_id: i64,
    /// 记录值
    pub value: String,
    /// 主机名
    name: String,
    /// 记录类型
    Type: String,
    /// 记录线路
    line: String,
}

/// 通用错误
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct TcErrorResponse {
    code: String,
    message: String,
}

/// 获取域名的解析记录列表响应
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DescribeRecordListResponse {
    error: Option<TcErrorResponse>,
    record_list: Option<Vec<RecordListItem>>,
}

/// 修改域名的解析记录响应
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct ModifyRecordResponse {
    error: Option<TcErrorResponse>,
    record_id: Option<i64>,
}

pub struct DDNS {
    domain: String,
    sdk_client: ReqClient,
    subdomain: String,
}

impl DDNS {
    pub fn new(secret_id: String, secret_key: String, domain: String, subdomain: String) -> Self {
        let host = "dnspod.tencentcloudapi.com".to_string();
        let service = "dnspod".to_string();

        let sdk_client = ReqClient::new(secret_id, secret_key, host.clone(), service);

        return Self {
            sdk_client,
            domain,
            subdomain,
        };
    }

    pub async fn query_record_list(
        &self,
        record_type: Option<String>,
    ) -> Result<DescribeRecordListResponse, Box<dyn std::error::Error>> {
        let res = self
            .sdk_client
            .send::<DescribeRecordList, DescribeRecordListResponse>(
                "DescribeRecordList".to_string(),
                DescribeRecordList {
                    domain: self.domain.clone(),
                    record_type: record_type,
                },
            )
            .await?;

        let record_list_resp = res;
        if let Some(e) = record_list_resp.response.error {
            println!("{}", e.message);
            Err(e.message.into())
        } else {
            Ok(record_list_resp.response)
        }
    }

    /// 获取当前域名对应的第一个AAAA解析记录
    pub async fn get_current_record(&self) -> Option<RecordListItem> {
        match self.query_record_list(Some(String::from("AAAA"))).await {
            Ok(res) => {
                if let Some(mut record_list) = res.record_list {
                    record_list.pop()
                } else {
                    println!("获取失败");
                    None
                }
            }
            Err(_) => None,
        }
    }

    /// 通过二级域名查找解析记录
    pub async fn query_record_by_name(&self) -> Result<RecordListItem, Box<dyn std::error::Error>> {
        let res = self.query_record_list(None).await?;
        let record = res.record_list.expect("查询列表失败");
        let res = record
            .iter()
            .find(|&x| x.name == self.subdomain)
            .expect("未找到，先去手动绑定!");
        return Ok(res.clone());
    }

    pub async fn change_record(
        &self,
        record_item: RecordListItem,
        value: String,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let RecordListItem {
            line,
            Type,
            record_id,
            name,
            ..
        } = record_item;
        let res = self
            .sdk_client
            .send::<ModifyRecordRequest, ModifyRecordResponse>(
                "ModifyRecord".to_string(),
                ModifyRecordRequest {
                    Value: value,
                    Domain: self.domain.clone(),
                    RecordType: Type,
                    RecordLine: line,
                    RecordId: record_id,
                    SubDomain: Some(name),
                    DomainId: None,
                    RecordLineId: None,
                    MX: None,
                    TTL: None,
                    Weight: None,
                    Status: None,
                },
            )
            .await?;

        if let Some(e) = res.response.error {
            return Err(e.message.into());
        } else {
            println!("修改 ddns 成功");
            return Ok(true);
        }
    }

    pub async fn change_record_loop(&self, current_ip: &str, tx: tokio::sync::mpsc::Sender<bool>) {
        let mut count = 10;
        loop {
            match self.query_record_by_name().await {
                Ok(item) => {
                    if item.value != current_ip {
                        let res = self.change_record(item, current_ip.to_owned()).await;
                        match res {
                            Ok(res) => {
                                println!(
                                    "发现ip变化，解析时间：current_time:{:?}",
                                    SystemTime::now()
                                );
                                // tx.send(true).await;
                            }
                            Err(e) => {
                                println!("更新 ip 状态失败");

                                // tx.send(false).await;
                            }
                        }
                    } else {
                        println!("ip未发生变化");
                    }
                    break;
                }
                Err(_) => {
                    count -= 1;
                    sleep(Duration::from_secs(10)).await;
                    println!("检查失败,正在重试第{}次", count);
                }
            };
        }

        // tx.send(true).await;
    }
}
