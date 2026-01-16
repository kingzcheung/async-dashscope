#![allow(unused_imports)]
use std::path::Path;

use anyhow::Result;
use async_dashscope::{
    Client,
    error::DashScopeError,
    operation::audio::ws::{
        FinishTaskParameters, RunTaskFunction, RunTaskParametersBuilder, RunTaskPayloadBuilder,
        RunTaskType, TaskAction, TaskHeaderBuilder, TaskParametersBuilder, WebSocketEvent,
        WebsocketCallback, create_asr_run_task,
    },
};
use futures_util::{SinkExt, stream::SplitSink};
use reqwest_websocket::{CloseCode, Message, WebSocket};
use tokio::time::Duration;

#[tokio::main]
pub async fn main() -> Result<()> {
    dotenvy::dotenv()?;
    let client = Client::default();
    let task_id = uuid::Uuid::new_v4().to_string();

    pub struct FunAsrCallback {
        task_id: String,
    }

    impl WebsocketCallback for FunAsrCallback {
        async fn on_open(&self, tx: &mut SplitSink<WebSocket, Message>) {
            // 连接打开时发送run-task指令
            println!("WebSocket connection opened");

            // let item = RunTaskParametersBuilder::default()
            //     .header(
            //         TaskHeaderBuilder::default()
            //             .action(TaskAction::RunTask)
            //             .task_id(self.task_id.clone())
            //             .build()
            //             .unwrap(),
            //     )
            //     .payload(
            //         RunTaskPayloadBuilder::default()
            //             .task_group("audio".to_string())
            //             .task(RunTaskType::Asr)
            //             .function(RunTaskFunction::Recognition)
            //             .model("fun-asr-realtime")
            //             // .model("paraformer-realtime-v2")
            //             // .model("gummy-realtime-v1")
            //             .parameters(
            //                 TaskParametersBuilder::default()
            //                     .format("wav".to_string())
            //                     .sample_rate(16000)
            //                     .semantic_punctuation_enabled(false)
            //                     .build()
            //                     .unwrap(),
            //             )
            //             .build()
            //             .unwrap(),
            //     )
            //     .build()
            //     .unwrap()
            //     .try_into()
            //     .unwrap();
            // or use fn
            let item = create_asr_run_task(&self.task_id, "fun-asr-realtime", "wav", Some(16000))
                .try_into()
                .unwrap();
            let item = Message::Text(item);

            tx.send(item).await.unwrap();
        }

        async fn on_event(&self, tx: &mut SplitSink<WebSocket, Message>, event: WebSocketEvent) {
            // println!("Received event: {:?}", event);
            match event {
                WebSocketEvent::TaskStarted { header: _ } => {
                    // send audio data
                    // 音频内容为： 我是一个很有钱的人
                    let wav_file_path =
                        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_data/gdg_16k.WAV");
                    let audio_data = std::fs::read(wav_file_path).unwrap();

                    // 分片
                    let chunk_size = 1024;
                    let chunks = audio_data.chunks(chunk_size);
                    for chunk in chunks {
                        let chunk = chunk.to_vec();
                        let item = Message::Binary(chunk.into());
                        tx.send(item).await.unwrap();
                    }

                    // send finish task
                    let item = FinishTaskParameters::new(self.task_id.clone())
                        .try_into()
                        .unwrap();
                    let item = Message::Text(item);
                    tx.send(item).await.unwrap();
                }
                WebSocketEvent::ResultGenerated { header: _, payload } => {
                    // 打印结果
                    // ResultGenerated result: ""
                    // ResultGenerated result: "我"
                    // ResultGenerated result: "我是一个"
                    // ResultGenerated result: "我是一个很有钱"
                    // ResultGenerated result: "我是一个很有钱的人。"

                    println!(
                        "ResultGenerated result: {:?}",
                        payload.output.unwrap().sentence.unwrap().text
                    );
                    // if model is gummy-realtime-v1,
                    // println!(
                    //     "gummy-realtime-v1 result: {:?}",
                    //     payload.output.unwrap().transcription.unwrap().text.unwrap()
                    // );
                }
                WebSocketEvent::TaskFinished { header: _, payload } => {
                    println!("Task finished: {:?}", payload);
                    tx.close().await.unwrap()
                }
                WebSocketEvent::TaskFailed { header } => {
                    println!("Task failed: {:?}", header.error_message);
                    tx.close().await.unwrap()
                }
            }
        }

        async fn on_complete(&self) {
            println!("ASR process completed");
        }
        async fn on_close(&self, code: CloseCode, reason: String) {
            println!("WebSocket connection closed: {:?}", (code, reason));
        }

        async fn on_error(&self, error: DashScopeError) {
            println!("WebSocket connection closed: {:?}", error);
        }

        fn heartbeat_interval(&self) -> Option<Duration> {
            // 添加心跳间隔，每30秒发送一次心跳
            Some(Duration::from_secs(10))
        }
    }

    client
        .audio()
        .asr_ws()
        .await?
        .call(FunAsrCallback { task_id })
        .await?;

    Ok(())
}
