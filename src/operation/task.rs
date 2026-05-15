use crate::error::{DashScopeError, Result};
use crate::{Client, operation::common::TaskStatus};
use output::*;
use std::time::Duration;
use tokio::time::sleep;
const TASK_PATH: &str = "/tasks";

pub mod output;

pub struct Task<'a> {
    client: &'a Client,
}

impl<'a> Task<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub(crate) async fn query(&self, task_id: &str) -> Result<TaskResult> {
        let http_client = self.client.http_client();
        let headers = self.client.config().headers();
        let req = http_client
            .get(
                self.client
                    .config()
                    .url(format!("{}/{}", TASK_PATH, task_id).as_str()),
            )
            .headers(headers)
            .build()?;

        let resp = http_client.execute(req).await?.bytes().await?;

        // 检查响应是否为空
        if resp.is_empty() {
            return Err(DashScopeError::ApiError(crate::error::ApiError {
                message: "API returned empty response".to_string(),
                request_id: None,
                code: Some("EmptyResponse".to_string()),
            }));
        }

        let raw_response_str = String::from_utf8_lossy(resp.as_ref());
        tracing::debug!("Raw API response: {}", raw_response_str);

        let resp_json = serde_json::from_slice::<TaskResult>(resp.as_ref()).map_err(|e| {
            crate::error::DashScopeError::JSONDeserialize {
                source: e,
                raw_response: String::from_utf8_lossy(&resp).to_string(),
            }
        })?;

        Ok(resp_json)
    }

    /// 轮询任务状态
    ///
    /// 该方法会定期查询任务状态，直到任务完成、失败或达到最大轮询次数。
    ///
    /// # Arguments
    /// * `task_id` - 要轮询的任务ID
    /// * `interval` - 每次轮询之间的间隔时间（秒）
    /// * `max_attempts` - 最大轮询尝试次数
    ///
    /// # Returns
    /// 返回 `Result<TaskResult>`，包含最终任务结果或错误
    ///
    /// # Errors
    /// - 当任务在最大轮询次数内未完成时返回 `TimeoutError`
    /// - 当遇到不可重试的错误（如配置错误）时返回相应错误
    /// - 当API返回空响应或格式错误时会继续重试
    ///
    /// # Notes
    /// - 对于可恢复的错误（如网络问题、临时API错误）会自动重试
    /// - 每次轮询会打印当前状态信息到标准输出
    pub async fn poll_task_status(
        &self,
        task_id: &str,
        interval: u64,
        max_attempts: u32,
    ) -> Result<TaskResult> {
        let mut attempts = 0;
        loop {
            attempts += 1;
            if attempts > max_attempts {
                return Err(DashScopeError::TimeoutError(
                    "polling timeout, task did not complete within expected time".to_string(),
                ));
            }

            match self.query(task_id).await {
                Ok(result) => {
                    let task_status = &result.output.task_status;

                    match task_status {
                        TaskStatus::Succeeded => {
                            return Ok(result);
                        }
                        TaskStatus::Failed => {
                            return Ok(result);
                        }
                        TaskStatus::Pending | TaskStatus::Running => {
                            tracing::info!(
                                "Task still in progress, waiting {} seconds before next poll...",
                                interval
                            );
                            sleep(Duration::from_secs(interval)).await;
                        }
                        TaskStatus::Canceled | TaskStatus::Unknown => {
                            return Ok(result);
                        }
                    }
                }
                Err(e) => {
                    match &e {
                        DashScopeError::JSONDeserialize {
                            source: _,
                            raw_response: _,
                        } => {
                            sleep(Duration::from_secs(interval)).await;
                        }
                        DashScopeError::Reqwest(_) => {
                            sleep(Duration::from_secs(interval)).await;
                        }
                        DashScopeError::ApiError(api_error) => {
                            if api_error.code.as_deref() == Some("EmptyResponse") {
                                sleep(Duration::from_secs(interval)).await;
                            } else {
                                return Err(e);
                            }
                        }
                        _ => {
                            return Err(e);
                        }
                    }
                }
            }
        }
    }
}
