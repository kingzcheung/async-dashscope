use std::{path::PathBuf, str::FromStr};

use reqwest::{Client, header::HeaderMap};
use serde::Deserialize;
use serde_json::json;
use tokio::{fs::File, io::AsyncReadExt};
use url::Url;

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct UploadPolicy {
    pub(crate) data: PolicyData,
    pub(crate) request_id: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PolicyData {
    pub(crate) policy: String,
    pub(crate) signature: String,
    pub(crate) upload_dir: String,
    pub(crate) upload_host: String,
    pub(crate) expire_in_seconds: i64,
    pub(crate) max_file_size_mb: i32,
    pub(crate) capacity_limit_mb: i32,
    pub(crate) oss_access_key_id: String,
    pub(crate) x_oss_object_acl: String,
    pub(crate) x_oss_forbid_overwrite: String,
}

/// 获取文件上传凭证
pub(crate) async fn get_upload_policy(
    api_key: &str,
    model_name: &str,
) -> Result<UploadPolicy, reqwest::Error> {
    let url = "https://dashscope.aliyuncs.com/api/v1/uploads";
    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        format!("Bearer {api_key}").parse().unwrap(),
    );
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let params = json!({
        "action": "getPolicy",
        "model": model_name
    });
    let response = reqwest::Client::new()
        .get(url)
        .headers(headers)
        .query(&params)
        .send()
        .await?
        .json::<UploadPolicy>()
        .await?;
    // todo: handle error
    Ok(response)
}

/// 将文件上传到临时存储OSS
pub(crate) async fn upload_file_to_oss(
    policy_data: PolicyData,
    mut file: File,
    file_name:&str,
) -> Result<String, crate::error::DashScopeError> {
    
    let key = format!("{}/{}", policy_data.upload_dir, file_name);

    let mut buffer = Vec::new();
    let _ = file.read_to_end(&mut buffer).await;

    let form = reqwest::multipart::Form::new()
        .text("OSSAccessKeyId", policy_data.oss_access_key_id.clone())
        .text("Signature", policy_data.signature.clone())
        .text("policy", policy_data.policy.clone())
        .text("x-oss-object-acl", policy_data.x_oss_object_acl.clone())
        .text(
            "x-oss-forbid-overwrite",
            policy_data.x_oss_forbid_overwrite.clone(),
        )
        .text("key", key.clone())
        .text("success_action_status", "200".to_string())
        .part(
            "file",
            reqwest::multipart::Part::bytes(buffer).file_name(file_name.to_string()),
        );

    let response = Client::new()
        .post(&policy_data.upload_host)
        .multipart(form)
        .send()
        .await
        .map_err(|e| crate::error::DashScopeError::UploadError(e.to_string()))?;

    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(crate::error::DashScopeError::UploadError(text));
    }

    Ok(format!("oss://{key}"))
}

pub(crate) async fn upload_file_and_get_url(
    api_key: &str,
    model_name: &str,
    file_path: &str,
) -> Result<String, crate::error::DashScopeError> {
    let p = PathBuf::from_str(file_path)
        .map_err(|e| crate::error::DashScopeError::UploadError(e.to_string()))?;

    let file_name = p
        .file_name()
        .ok_or_else(|| crate::error::DashScopeError::UploadError("file name is empty".to_string()))?
        .to_str()
        .ok_or_else(|| {
            crate::error::DashScopeError::UploadError("file name is empty".to_string())
        })?;


    let file = tokio::fs::OpenOptions::new()
        .read(true)
        .open(file_path)
        .await
        .map_err(|e| crate::error::DashScopeError::UploadError(e.to_string()))?;
    let meta = file
        .metadata()
        .await
        .map_err(|e| crate::error::DashScopeError::UploadError(e.to_string()))?;

    if !meta.is_file() {
        return Err(crate::error::DashScopeError::UploadError(
            "file is not a file".into(),
        ));
    }

    let policy_data = get_upload_policy(api_key, model_name).await?;

    let url = upload_file_to_oss(policy_data.data, file,file_name).await?;

    Ok(url)
}

/// 检查字符串是否为有效的URL
pub(crate) fn is_valid_url(s: &str) -> bool {
    Url::parse(s).is_ok()
}




#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_get_upload_policy() -> Result<(), Box<dyn std::error::Error>> {
        let _ = dotenvy::dotenv();
        let Ok(api_key) = std::env::var("DASHSCOPE_API_KEY") else {
            println!("DASHSCOPE_API_KEY not set, skipping test");
            return Ok(());
        };
        let model_name = "qwen-vl-max";
        let result = get_upload_policy(&api_key, model_name).await;
        assert!(result.is_ok());

        Ok(())
    }
}
