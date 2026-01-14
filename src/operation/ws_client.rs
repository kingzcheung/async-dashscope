use reqwest_websocket::{RequestBuilderExt, WebSocket};

use crate::error::DashScopeError;

const WS_URL: &str = "wss://dashscope.aliyuncs.com/api-ws/v1/inference";

#[derive(Debug)]
pub struct WsClient(pub(crate) WebSocket);

impl WsClient {
    pub async fn into_ws_client(client: crate::Client) -> Result<Self, DashScopeError> {
        let ws = client
            .http_client
            .get(WS_URL)
            .headers(client.config.headers())
            .upgrade()
            .send()
            .await?
            .into_websocket()
            .await?;

        Ok(Self(ws))
    }
}
