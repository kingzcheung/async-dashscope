use reqwest_websocket::{RequestBuilderExt, WebSocket};

use crate::error::DashScopeError;

#[derive(Debug)]
pub struct WsClient(pub(crate) WebSocket);

impl WsClient {
    pub async fn into_ws_client(client: crate::Client) -> Result<Self, DashScopeError> {
        let url = client.config().try_websocket_url()?;
        let ws = client
            .http_client
            .get(url)
            .headers(client.config.headers())
            .upgrade()
            .send()
            .await?
            .into_websocket()
            .await?;

        Ok(Self(ws))
    }
}
