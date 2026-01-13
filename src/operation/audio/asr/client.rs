use std::pin::Pin;
use std::time::Duration;

use async_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt, TryStreamExt};
use reqwest_websocket::RequestBuilderExt;
use tokio::sync::mpsc;
use tokio_stream::Stream;

use crate::error::DashScopeError;
use crate::operation::audio::asr::param::{
    AutomaticSpeechRecognitionParam, AsrInput, FinishTaskCommand, FinishTaskPayload, 
    RunTaskCommand, RunTaskPayload, WebSocketHeader,
};
use crate::operation::audio::asr::output::{
    AutomaticSpeechRecognitionOutputStream, EventType, WebSocketEvent,
};

/// ASR WebSocket 客户端
pub struct AsrClient {
    client: reqwest::Client,
    api_key: String,
    workspace_id: Option<String>,
    data_inspection: Option<String>,
}

impl AsrClient {
    /// 创建新的 ASR 客户端
    pub fn new(client: reqwest::Client, api_key: String) -> Self {
        Self {
            client,
            api_key,
            workspace_id: None,
            data_inspection: None,
        }
    }

    /// 设置工作空间ID
    pub fn with_workspace_id(mut self, workspace_id: String) -> Self {
        self.workspace_id = Some(workspace_id);
        self
    }

    /// 设置数据合规检测
    pub fn with_data_inspection(mut self, data_inspection: String) -> Self {
        self.data_inspection = Some(data_inspection);
        self
    }

    /// 建立 WebSocket 连接并开始语音识别
    pub async fn recognize(
        &self,
        param: AutomaticSpeechRecognitionParam,
        audio_stream: Pin<Box<dyn Stream<Item = Result<Vec<u8>, DashScopeError>> + Send>>,
    ) -> Result<AutomaticSpeechRecognitionOutputStream, DashScopeError> {
        let (event_tx, event_rx) = mpsc::channel(100);

        // 生成任务ID
        let task_id = uuid::Uuid::new_v4().to_string().replace("-", "");

        // 建立 WebSocket 连接
        let mut request_builder = self
            .client
            .get("wss://dashscope.aliyuncs.com/api-ws/v1/inference");

        // 添加认证头
        request_builder = request_builder.header(
            "Authorization",
            format!("Bearer {}", self.api_key),
        );

        // 添加可选头
        if let Some(workspace_id) = &self.workspace_id {
            request_builder = request_builder.header("X-DashScope-WorkSpace", workspace_id);
        }

        if let Some(data_inspection) = &self.data_inspection {
            request_builder = request_builder.header("X-DashScope-DataInspection", data_inspection);
        }

        let websocket = request_builder
            .upgrade()
            .send()
            .await
            .map_err(|e| DashScopeError::WebSocketError(format!("连接失败: {}", e)))?
            .into_websocket()
            .await
            .map_err(|e| DashScopeError::WebSocketError(format!("WebSocket升级失败: {}", e)))?;

        let (mut tx, mut rx) = websocket.split();
        let mut tx: futures_util::stream::SplitSink<async_tungstenite::WebSocketStream<reqwest_websocket::Upgraded>, async_tungstenite::tungstenite::Message> = tx;

        // 发送 run-task 指令
        let run_task_command = RunTaskCommand {
            header: WebSocketHeader {
                action: "run-task".to_string(),
                task_id: task_id.clone(),
                streaming: "duplex".to_string(),
            },
            payload: RunTaskPayload {
                task_group: "audio".to_string(),
                task: "asr".to_string(),
                function: "recognition".to_string(),
                model: param.model.clone(),
                parameters: param.parameters.clone(),
                resources: param.resources.clone(),
                input: param.input.clone(),
            },
        };

        let run_task_json = serde_json::to_string(&run_task_command)
            .map_err(|e| DashScopeError::SerializationError(e.to_string()))?;

        tx.send(Message::Text(run_task_json.into()))
            .await
            .map_err(|e| DashScopeError::WebSocketError(format!("发送run-task指令失败: {}", e)))?;

        // 启动音频发送任务
        let audio_tx = tx.clone();
        let task_id_clone = task_id.clone();
        let audio_task = tokio::spawn(async move {
            let mut audio_stream = audio_stream;
            
            // 等待任务开始事件
            let mut started = false;
            
            while let Some(audio_chunk) = audio_stream.next().await {
                match audio_chunk {
                    Ok(chunk) => {
                        if !started {
                            // 等待任务开始事件
                            tokio::time::sleep(Duration::from_millis(100)).await;
                            started = true;
                        }
                        
                        // 发送音频数据
                        if let Err(e) = audio_tx.send(Message::Binary(chunk.into())).await {
                            eprintln!("发送音频数据失败: {}", e);
                            break;
                        }
                        
                        // 建议间隔100ms
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                    Err(e) => {
                        eprintln!("音频流错误: {}", e);
                        break;
                    }
                }
            }

            // 发送 finish-task 指令
            let finish_task_command = FinishTaskCommand {
                header: WebSocketHeader {
                    action: "finish-task".to_string(),
                    task_id: task_id_clone,
                    streaming: "duplex".to_string(),
                },
                payload: FinishTaskPayload {
                    input: AsrInput {},
                },
            };

            if let Ok(finish_task_json) = serde_json::to_string(&finish_task_command) {
                let _ = audio_tx.send(Message::Text(finish_task_json.into())).await;
            }
        });

        // 启动事件接收任务
        let event_tx_clone: tokio::sync::mpsc::Sender<Result<WebSocketEvent, DashScopeError>> = event_tx.clone();
        let receive_task = tokio::spawn(async move {
            let mut rx: futures_util::stream::SplitStream<async_tungstenite::WebSocketStream<reqwest_websocket::Upgraded>> = rx;
            while let Some(message) = rx.try_next().await {
                match message {
                    Ok(Message::Text(text)) => {
                        // 解析 JSON 事件
                        match serde_json::from_str::<WebSocketEvent>(&text) {
                            Ok(event) => {
                                if let Err(e) = event_tx_clone.send(Ok(event)).await {
                                    eprintln!("发送事件失败: {}", e);
                                    break;
                                }

                                // 如果是任务结束或失败事件，停止接收
                                if event.is_task_finished() || event.is_task_failed() {
                                    break;
                                }
                            }
                            Err(e) => {
                                eprintln!("解析事件失败: {} - {}", text, e);
                            }
                        }
                    }
                    Ok(Message::Binary(_)) => {
                        // 忽略二进制消息（应该是音频数据）
                    }
                    Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {
                        // 处理 Ping/Pong 消息
                    }
                    Ok(Message::Close(_)) => {
                        break;
                    }
                    Err(e) => {
                        eprintln!("接收消息失败: {}", e);
                        break;
                    }
                }
            }
        });

        // 创建输出流
        let output_stream = Box::pin(async_stream::try_stream! {
            let mut event_rx = event_rx;
            
            while let Some(event) = event_rx.recv().await {
                yield event?;
            }
            
            // 等待任务完成
            let _ = tokio::try_join!(audio_task, receive_task);
        });

        Ok(output_stream)
    }

    /// 简单的语音识别方法（用于文件或固定音频数据）
    pub async fn recognize_simple(
        &self,
        param: AutomaticSpeechRecognitionParam,
        audio_data: Vec<u8>,
    ) -> Result<AutomaticSpeechRecognitionOutputStream, DashScopeError> {
        // 将音频数据分割成小块（模拟实时流）
        let chunk_size = 1600; // 100ms 的音频数据（16kHz, 16bit）
        let audio_chunks: Vec<Vec<u8>> = audio_data
            .chunks(chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        let audio_stream = Box::pin(async_stream::try_stream! {
            for chunk in audio_chunks {
                yield chunk;
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });

        self.recognize(param, audio_stream).await
    }
}