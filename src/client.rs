use std::{fmt::Debug, pin::Pin};

use bytes::Bytes;
use reqwest_eventsource::{Event, EventSource, RequestBuilderExt as _};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio_stream::{Stream, StreamExt as _};

use crate::{
    config::Config,
    error::{map_deserialization_error, ApiError, DashScopeError},
};

#[derive(Debug, Default)]
pub struct Client {
    http_client: reqwest::Client,
    config: Config,
    backoff: backoff::ExponentialBackoff,
}

impl Client {
    pub fn new() -> Self {
        Self::default()
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

    pub fn generation(&self) -> crate::operation::generation::Generation<'_> {
        crate::operation::generation::Generation::new(self)
    }

    pub fn multi_modal_conversation(
        &self,
    ) -> crate::operation::multi_modal_conversation::MultiModalConversation<'_> {
        crate::operation::multi_modal_conversation::MultiModalConversation::new(self)
    }

    pub fn text_embeddings(&self)->crate::operation::embeddings::Embeddings<'_>{
        crate::operation::embeddings::Embeddings::new(self)
    }

    pub(crate) async fn post_stream<I, O>(
        &self,
        path: &str,
        request: I,
    ) -> Pin<Box<dyn Stream<Item = Result<O, DashScopeError>> + Send>>
    where
        I: Serialize,
        O: DeserializeOwned + std::marker::Send + 'static,
    {
        let event_source = self
            .http_client
            .post(self.config.url(path))
            .headers(self.config.headers())
            .json(&request)
            .eventsource()
            .unwrap();

        stream(event_source).await
    }

    pub(crate) async fn post<I, O>(&self, path: &str, request: I) -> Result<O, DashScopeError>
    where
        I: Serialize + Debug,
        O: DeserializeOwned,
    {
        dbg!(&request);
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
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(async move {
        while let Some(ev) = event_source.next().await {
            match ev {
                Err(e) => {
                    if let Err(_e) = tx.send(Err(DashScopeError::StreamError(e.to_string()))) {
                        // rx dropped
                        break;
                    }
                }
                Ok(event) => match event {
                    Event::Message(message) => {
                        #[derive(Deserialize, Debug)]
                        struct Result {
                            output: Output,
                        }
                        #[derive(Deserialize, Debug)]
                        struct Output {
                            choices: Vec<Choices>,
                        }
                        #[derive(Deserialize, Debug)]
                        struct Choices {
                            finish_reason: Option<String>,
                        }

                        let r = match serde_json::from_str::<Result>(&message.data) {
                            Ok(r) => r,
                            Err(e) => {
                                if let Err(_e) = tx.send(Err(map_deserialization_error(
                                    e,
                                    message.data.as_bytes(),
                                ))) {
                                    break;
                                }
                                continue;
                            }
                        };
                        if let Some(finish_reason) = r.output.choices[0].finish_reason.clone() {
                            if finish_reason == "stop" {
                                break;
                            }
                        }

                        let response = match serde_json::from_str::<O>(&message.data) {
                            Err(e) => Err(map_deserialization_error(e, message.data.as_bytes())),
                            Ok(output) => Ok(output),
                        };

                        if let Err(_e) = tx.send(response) {
                            // rx dropped
                            break;
                        }
                    }
                    Event::Open => continue,
                },
            }
        }

        event_source.close();
    });

    Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx))
}
