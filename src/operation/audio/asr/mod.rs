pub mod output;
pub mod param;

use futures_util::{SinkExt, StreamExt, TryStreamExt, stream::SplitSink};
pub use output::*;
pub use param::*;
use reqwest_websocket::{CloseCode, Message, WebSocket};

use crate::{error::DashScopeError, operation::ws_client::WsClient};

pub trait WsCallback {
    /// 当和服务端建立连接完成后，该方法立刻被回调。
    fn on_open(&self, tx: &mut SplitSink<WebSocket, Message>) -> impl std::future::Future<Output = ()> + Send;
    /// 当服务有回复时会被回调。
    fn on_event(&self, tx: &mut SplitSink<WebSocket, Message>, event: WebSocketEvent) -> impl std::future::Future<Output = ()> + Send;
    /// 当所有识别结果全部返回后进行回调。
    fn on_complete(&self) -> impl std::future::Future<Output = ()> + Send { async {} }
    /// 发生异常时该方法被回调。
    fn on_error(&self, error: DashScopeError) -> impl std::future::Future<Output = ()> + Send;
    /// 当服务已经关闭连接后进行回调。
    fn on_close(&self, code: CloseCode, reason: String) -> impl std::future::Future<Output = ()> + Send;
}

pub struct Asr {
    ws_client: WsClient,
}

impl Asr {
    pub fn new(ws_client: WsClient) -> Self {
        Self { ws_client }
    }

    pub async fn call(self, callback: impl WsCallback) -> Result<(), DashScopeError> {
        let (mut tx, mut rx) = self.ws_client.0.split();
        callback.on_open(&mut tx).await;

        while let Some(message) = rx.try_next().await.unwrap() {
            match message {
                Message::Text(t) => match WebSocketEvent::try_from(t) {
                    Ok(event) => callback.on_event(&mut tx, event).await,
                    Err(e) => callback.on_error(e).await,
                },
                Message::Binary(bytes) => println!("Binary: {:?}", bytes), // no use
                Message::Ping(bytes) => println!("Ping: {:?}", bytes),
                Message::Pong(bytes) => println!("Pong: {:?}", bytes),
                Message::Close { code, reason } => callback.on_close(code, reason).await,
            }
        }

        Ok(())
    }
}
