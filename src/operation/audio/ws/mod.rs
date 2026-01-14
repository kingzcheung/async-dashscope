pub mod output;
pub mod param;

use bytes::Bytes;
use futures_util::{SinkExt, StreamExt, TryStreamExt, stream::SplitSink};
pub use output::*;
pub use param::*;
use reqwest_websocket::{CloseCode, Message, WebSocket};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, interval};

use crate::{error::DashScopeError, operation::ws_client::WsClient};

pub trait WebsocketCallback {
    /// 当和服务端建立连接完成后，该方法立刻被回调。
    fn on_open(
        &self,
        tx: &mut SplitSink<WebSocket, Message>,
    ) -> impl std::future::Future<Output = ()> + Send;
    /// 当服务有回复时会被回调。
    fn on_event(
        &self,
        tx: &mut SplitSink<WebSocket, Message>,
        event: WebSocketEvent,
    ) -> impl std::future::Future<Output = ()> + Send;
    fn on_data(
        &self,
        _tx: &mut SplitSink<WebSocket, Message>,
        _data: Bytes,
    ) -> impl std::future::Future<Output = ()> + Send {
        async {}
    }
    /// 当所有识别结果全部返回后进行回调。
    fn on_complete(&self) -> impl std::future::Future<Output = ()> + Send {
        async {}
    }

    fn on_pong(&self, _bytes: Bytes) -> impl std::future::Future<Output = ()> + Send {
        async {}
    }
    /// 发生异常时该方法被回调。
    fn on_error(&self, error: DashScopeError) -> impl std::future::Future<Output = ()> + Send;
    /// 当服务已经关闭连接后进行回调。
    fn on_close(
        &self,
        code: CloseCode,
        reason: String,
    ) -> impl std::future::Future<Output = ()> + Send;

    /// 获取心跳间隔时间，如果不返回时间则不发送心跳
    fn heartbeat_interval(&self) -> Option<Duration> {
        None
    }
}

pub struct WebsocketInference {
    ws_client: WsClient,
}

impl WebsocketInference {
    pub fn new(ws_client: WsClient) -> Self {
        Self { ws_client }
    }

    pub async fn call(self, callback: impl WebsocketCallback) -> Result<(), DashScopeError> {
        let (tx, mut rx) = self.ws_client.0.split();
        let tx = Arc::new(Mutex::new(tx));
        let tx_for_open = Arc::clone(&tx);

        {
            let mut tx_guard = tx_for_open.lock().await;
            callback.on_open(&mut tx_guard).await;
        }

        // 启动心跳任务（如果设置了心跳间隔）
        let heartbeat_handle = if let Some(interval_duration) = callback.heartbeat_interval() {
            let tx_for_heartbeat = Arc::clone(&tx);
            let heartbeat_task = tokio::spawn(async move {
                let mut interval_timer = interval(interval_duration);

                loop {
                    interval_timer.tick().await;
                    {
                        let mut tx_guard = tx_for_heartbeat.lock().await;
                        if tx_guard.send(Message::Ping(Bytes::new())).await.is_err() {
                            // 如果发送ping失败，说明连接可能已断开
                            break;
                        }
                    }
                }
            });

            Some(heartbeat_task)
        } else {
            None
        };

        while let Some(message) = rx.try_next().await? {
            match message {
                Message::Text(t) => {
                    let mut tx_guard = tx.lock().await;
                    match WebSocketEvent::try_from(t) {
                        Ok(event) => callback.on_event(&mut tx_guard, event).await,
                        Err(e) => callback.on_error(e).await,
                    }
                }
                Message::Binary(b) => {
                    let mut tx_guard = tx.lock().await;
                    callback.on_data(&mut tx_guard, b).await; // no use
                }
                Message::Ping(bytes) => {
                    // 回应服务器的ping
                    {
                        let mut tx_guard = tx.lock().await;
                        let _ = tx_guard.send(Message::Pong(bytes)).await;
                    }
                }
                Message::Pong(bytes) => {
                    // 服务器回应pong，说明连接正常
                    // println!("Received pong: {:?}", bytes);
                    callback.on_pong(bytes).await;
                }
                Message::Close { code, reason } => {
                    // 结束心跳任务
                    if let Some(handle) = &heartbeat_handle {
                        handle.abort();
                    }
                    callback.on_close(code, reason).await;
                    break;
                }
            }
        }

        // 确保心跳任务结束
        if let Some(handle) = heartbeat_handle {
            handle.abort();
        }

        Ok(())
    }
}
