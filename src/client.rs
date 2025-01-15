use bytes::Bytes;
use serde::{de::DeserializeOwned, Serialize};

use crate::{config::Config, error::{map_deserialization_error, ApiError, DashScopeError}};

#[derive(Debug,Default)]
pub struct Client {
    http_client: reqwest::Client,
    config: Config,
    backoff: backoff::ExponentialBackoff,
}

impl Client {

    pub fn new() -> Self {
        Self::default()
    }
    pub fn build(http_client: reqwest::Client, config: Config, backoff: backoff::ExponentialBackoff) -> Self {
        Self { http_client, config, backoff }
    }
    
    pub fn generation(&self) -> crate::operation::generation::Generation<'_> {
        crate::operation::generation::Generation::new(self)
    }

    pub(crate) async fn post<I, O>(&self, path: &str, request: I) -> Result<O, DashScopeError>
    where
        I: Serialize,
        O: DeserializeOwned,
    {
        let request_maker = || async {
            Ok(self
                .http_client
                .post(self.config.url(path))
                .headers(self.config.headers())
                .json(&request)
                .build()?)
        };

        self.execute(request_maker).await
    }

    async fn execute<O, M, Fut>(&self, request_maker: M) -> Result<O, DashScopeError>
    where
        O: DeserializeOwned,
        M: Fn() -> Fut,
        Fut: core::future::Future<Output = Result<reqwest::Request, DashScopeError>>,
    {
        let bytes = self.execute_raw(request_maker).await?;

        let response: O = serde_json::from_slice(bytes.as_ref())
            .map_err(|e| map_deserialization_error(e, bytes.as_ref()))?;

        Ok(response)
    }

    async fn execute_raw<M, Fut>(&self, request_maker: M) -> Result<Bytes, DashScopeError>
    where
        M: Fn() -> Fut,
        Fut: core::future::Future<Output = Result<reqwest::Request, DashScopeError>>,
    {
        let client = self.http_client.clone();

        backoff::future::retry(self.backoff.clone(), || async {
            let request = request_maker().await.map_err(backoff::Error::Permanent)?;
            let response = client
                .execute(request)
                .await
                .map_err(DashScopeError::Reqwest)
                .map_err(backoff::Error::Permanent)?;

            let status = response.status();
            let bytes = response
                .bytes()
                .await
                .map_err(DashScopeError::Reqwest)
                .map_err(backoff::Error::Permanent)?;

            // Deserialize response body from either error object or actual response object
            if !status.is_success() {
                // bytes to string
                
                let api_error: ApiError = serde_json::from_slice(bytes.as_ref())
                    .map_err(|e| map_deserialization_error(e, bytes.as_ref()))
                    .map_err(backoff::Error::Permanent)?;

                if status.as_u16() == 429 {
                    // Rate limited retry...
                    tracing::warn!("Rate limited: {}", api_error.message);
                    return Err(backoff::Error::Transient {
                        err: DashScopeError::ApiError(api_error),
                        retry_after: None,
                    });
                } else {
                    return Err(backoff::Error::Permanent(DashScopeError::ApiError(
                        api_error
                    )));
                }
            }
            
            Ok(bytes)
        })
        .await
    }
}

