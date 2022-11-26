use reqwest::{header::HeaderMap, Client};
use serde::{de::DeserializeOwned, Deserialize};

use crate::encryption;

pub struct ReqClient {
    secret_id: String,
    secret_key: String,
    host: String,
    service: String,
    region: String,
    version: String,
    client: Client,
}

/// 通用基础响应
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct TcResponse<T> {
    pub response: T,
}

impl ReqClient {
    pub fn new(secret_id: String, secret_key: String, host: String, service: String) -> Self {
        let client = reqwest::Client::new();
        return Self {
            secret_id,
            secret_key,
            client,
            host,
            service,
            region: "".to_string(),
            version: "2021-03-23".to_string(),
        };
    }

    pub async fn send<T, R>(
        &self,
        action: String,
        payload: T,
    ) -> Result<TcResponse<R>, Box<dyn std::error::Error>>
    where
        T: serde::ser::Serialize,
        R: DeserializeOwned,
    {
        let now = chrono::Utc::now();

        let payload = serde_json::to_string(&payload)?;
        let timestamp = now.timestamp();
        let date = now.format("%Y-%m-%d").to_string();

        // 计算临时认证
        let authorization = self.make_post_authorization(timestamp, date, &payload)?;

        // 配置请求头
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", authorization.parse()?);
        headers.insert("Content-Type", "application/json; charset=utf-8".parse()?);
        headers.insert("Host", self.host.parse()?);
        headers.insert("X-TC-Action", action.parse()?);
        headers.insert("X-TC-Timestamp", timestamp.to_string().parse()?);
        headers.insert("X-TC-Version", self.version.parse()?);
        headers.insert("X-TC-Region", self.region.parse()?);

        let url = format!("https://{}", self.host);
        let res = self
            .client
            .post(&url)
            .headers(headers)
            .body(payload)
            .send()
            .await?;
        let json_resp = res.json::<TcResponse<R>>().await?;
        Ok(json_resp)
    }

    fn make_post_authorization(
        &self,
        timestamp: i64,
        date: String,
        payload: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        /* first */
        let httprequest_method = "POST";
        let canonical_uri = "/";
        let canonical_query_string = "";
        let canonical_headers = format!(
            "content-type:application/json; charset=utf-8\nhost:{}\n",
            self.host
        );
        let signed_headers = "content-type;host";
        // let hashed_request_payload = encryption::sha256_hex(payload);
        let hashed_request_payload = sha256::digest(payload);

        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            httprequest_method,
            canonical_uri,
            canonical_query_string,
            canonical_headers,
            signed_headers,
            hashed_request_payload
        );
        // println!("{}",canonical_request);

        /* second */

        let algorithm = "TC3-HMAC-SHA256";

        let credential_scope = format!("{}/{}/tc3_request", date, self.service);
        let hashed_canonical_request = sha256::digest(canonical_request);

        let string_to_sign = format!(
            "{}\n{}\n{}\n{}",
            algorithm, timestamp, credential_scope, hashed_canonical_request
        );
        // println!("{}", string_to_sign);
        /* third */

        let secret_date = encryption::hmac_sha256(
            &date.as_bytes(),
            format!("TC3{}", self.secret_key).as_str().as_bytes(),
        );
        let secret_service = encryption::hmac_sha256(&self.service.as_bytes(), &secret_date);
        let secret_signing = encryption::hmac_sha256("tc3_request".as_bytes(), &secret_service);

        let signature = encryption::hmac_sha256_hex(&string_to_sign.as_bytes(), &secret_signing);

        /* forth */
        let authorization = format!(
            "TC3-HMAC-SHA256 Credential={}/{},SignedHeaders={},Signature={}",
            self.secret_id, credential_scope, signed_headers, signature
        );
        Ok(authorization)
    }
}
