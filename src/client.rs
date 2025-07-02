use std::{fmt::Debug, pin::Pin};

use async_stream::try_stream;
use bytes::Bytes;
use reqwest_eventsource::{Event, EventSource, RequestBuilderExt as _};
use serde::{de::DeserializeOwned, Serialize};
use tokio_stream::{Stream, StreamExt as _};

use crate::{
    config::Config,
    error::{map_deserialization_error, ApiError, DashScopeError},
};

#[derive(Debug, Default, Clone)]
pub struct Client {
    http_client: reqwest::Client,
    config: Config,
    backoff: backoff::ExponentialBackoff,
}

impl Client {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: Config) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            config,
            backoff: backoff::ExponentialBackoff::default(),
        }
    }
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.config.set_api_key(api_key.into());
        self
    }

    pub fn build(
        http_client: reqwest::Client,
        config: Config,
        backoff: backoff::ExponentialBackoff,
    ) -> Self {
        Self {
            http_client,
            config,
            backoff,
        }
    }

    /// 获取当前实例的生成（Generation）信息
    ///
    /// 此方法属于操作级别，用于创建一个`Generation`对象，
    /// 该对象表示当前实例的某一特定生成（代）信息
    ///
    /// # Returns
    ///
    /// 返回一个`Generation`对象，用于表示当前实例的生成信息
    pub fn generation(&self) -> crate::operation::generation::Generation<'_> {
        crate::operation::generation::Generation::new(self)
    }

    /// 启发多模态对话的功能
    ///
    /// 该函数提供了与多模态对话相关的操作入口
    /// 它创建并返回一个MultiModalConversation实例，用于执行多模态对话操作
    ///
    /// 返回一个`MultiModalConversation`实例，用于进行多模态对话操作
    pub fn multi_modal_conversation(
        &self,
    ) -> crate::operation::multi_modal_conversation::MultiModalConversation<'_> {
        crate::operation::multi_modal_conversation::MultiModalConversation::new(self)
    }

    /// 获取音频处理功能
    pub fn audio(&self) -> crate::operation::audio::Audio<'_> {
        crate::operation::audio::Audio::new(self)
    }

    /// 获取文本嵌入表示
    ///
    /// 此函数提供了一个接口，用于将文本转换为嵌入表示
    /// 它利用当前实例的上下文来生成文本的嵌入表示
    ///
    /// 返回一个`Embeddings`实例，该实例封装了文本嵌入相关的操作和数据
    /// `Embeddings`类型提供了进一步处理文本数据的能力，如计算文本相似度或进行文本分类等
    pub fn text_embeddings(&self) -> crate::operation::embeddings::Embeddings<'_> {
        crate::operation::embeddings::Embeddings::new(self)
    }

    pub(crate) async fn post_stream<I, O>(
        &self,
        path: &str,
        request: I,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<O, DashScopeError>> + Send>>, DashScopeError>
    where
        I: Serialize,
        O: DeserializeOwned + std::marker::Send + 'static,
    {
        let event_source = self
            .http_client
            .post(self.config.url(path))
            .headers(self.config.headers())
            .json(&request)
            .eventsource()?;

        Ok(stream(event_source).await)
    }

    pub(crate) async fn post<I, O>(&self, path: &str, request: I) -> Result<O, DashScopeError>
    where
        I: Serialize + Debug,
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
                        api_error,
                    )));
                }
            }

            Ok(bytes)
        })
        .await
    }
}

pub(crate) async fn stream<O>(
    mut event_source: EventSource,
) -> Pin<Box<dyn Stream<Item = Result<O, DashScopeError>> + Send>>
where
    O: DeserializeOwned + std::marker::Send + 'static,
{
    let stream = try_stream! {
        while let Some(ev) = event_source.next().await {
            match ev {
                Err(e) => {
                    Err(DashScopeError::StreamError(e.to_string()))?;
                }
                Ok(Event::Open) => continue,
                Ok(Event::Message(message)) => {
                    // First, deserialize to a generic JSON Value to inspect it without failing.
                    let json_value: serde_json::Value = match serde_json::from_str(&message.data) {
                        Ok(val) => val,
                        Err(e) => {
                            Err(map_deserialization_error(e, message.data.as_bytes()))?;
                            continue;
                        }
                    };

                    // Now, deserialize from the `Value` to the target type `O`.
                    let response = serde_json::from_value::<O>(json_value.clone())
                        .map_err(|e| map_deserialization_error(e, message.data.as_bytes()))?;

                    // Yield the successful message
                    yield response;

                    // Check for finish reason after sending the message.
                    // This ensures the final message with "stop" is delivered.
                    let finish_reason = json_value
                        .pointer("/output/choices/0/finish_reason")
                        .and_then(|v| v.as_str());

                    if let Some("stop") = finish_reason {
                        break;
                    }
                }
            }
        }
        event_source.close();
    };

    Box::pin(stream)
}

#[cfg(test)]
mod tests {
    use crate::config::ConfigBuilder;

    use super::*;

    #[test]
    pub fn test_config() {
        let config = ConfigBuilder::default()
            .api_key("test key")
            .build()
            .unwrap();
        let client = Client::with_config(config);

        for header in client.config.headers().iter() {
            if header.0 == "authorization" {
                assert_eq!(header.1, "Bearer test key");
            }
        }
    }
}
