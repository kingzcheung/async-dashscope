#![allow(unused_imports)]
use std::path::Path;

use anyhow::Result;
use async_dashscope::{
    Client,
    error::DashScopeError,
    operation::audio::ws::{
        ContinueTaskInputBuilder, ContinueTaskParametersBuilder, ContinueTaskPayloadBuilder,
        FinishTaskParameters, RunTaskFunction, RunTaskParametersBuilder, RunTaskPayloadBuilder,
        RunTaskType, TaskAction, TaskHeaderBuilder, TaskParametersBuilder, WebSocketEvent,
        WebsocketCallback, create_continue_task, create_finish_task, create_tts_run_task,
    },
};
use futures_util::{SinkExt, stream::SplitSink};
use reqwest_websocket::{CloseCode, Message, WebSocket};
use tokio::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
pub async fn main() -> Result<()> {
    dotenvy::dotenv()?;
    let client = Client::default();
    let task_id = uuid::Uuid::new_v4().to_string();
    let audio_filename = format!("output_{}.wav", task_id);
    
    // 创建音频文件用于保存接收到的数据
    let audio_file = Arc::new(Mutex::new(Some(File::create(&audio_filename).await?)));
    let shared_task_id = task_id.clone();

    pub struct CosyVoiceCallback {
        task_id: String,
        audio_file: Arc<Mutex<Option<File>>>,
    }

    impl CosyVoiceCallback {
        fn new(task_id: String, audio_file: Arc<Mutex<Option<File>>>) -> Self {
            CosyVoiceCallback { 
                task_id, 
                audio_file 
            }
        }
    }

    impl WebsocketCallback for CosyVoiceCallback {
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
            //             .task(RunTaskType::Tts)
            //             .function(RunTaskFunction::SpeechSynthesizer)
            //             .model("cosyvoice-v3-flash")
            //             .parameters(
            //                 TaskParametersBuilder::default()
            //                     .format("wav".to_string())
            //                     .voice("longanyang")
            //                     .sample_rate(22050)
            //                     .volume(50)
            //                     .rate(1.0)
            //                     .pitch(1.0)
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

            // or use create_tts_run_task
            let item =
                create_tts_run_task(&self.task_id, "cosyvoice-v3-flash", "longanyang", "wav")
                    .try_into()
                    .unwrap();

            let item = Message::Text(item);

            tx.send(item).await.unwrap();
        }

        async fn on_event(&self, tx: &mut SplitSink<WebSocket, Message>, event: WebSocketEvent) {
            // println!("Received event: {:?}", event);
            match event {
                WebSocketEvent::TaskStarted { header: _ } => {
                    // send continue task
                    // let item = ContinueTaskParametersBuilder::default()
                    // .header(
                    //     TaskHeaderBuilder::default()
                    //     .action(async_dashscope::operation::audio::ws::TaskAction::ContinueTask)
                    //     .task_id(self.task_id.clone())
                    //     .build()
                    //     .unwrap()
                    // ).payload(
                    //     ContinueTaskPayloadBuilder::default()
                    //     .input(
                    //         ContinueTaskInputBuilder::default()
                    //         .text("我是一个平平无奇的小学生")
                    //         .build()
                    //         .unwrap()
                    //     )
                    //     .build()
                    //     .unwrap()
                    // ).build().unwrap().try_into().unwrap();

                    // or
                    let item =
                        create_continue_task(self.task_id.clone(), "我是一个平平无奇的小学生")
                            .try_into()
                            .unwrap();

                    let item = Message::Text(item);
                    tx.send(item).await.unwrap();

                    // send finish task
                    // let item = FinishTaskParameters::new(self.task_id.clone())
                    //     .try_into()
                    //     .unwrap();
                    // or
                    let item = create_finish_task(&self.task_id).try_into().unwrap();
                    let item = Message::Text(item);
                    tx.send(item).await.unwrap();
                }
                WebSocketEvent::ResultGenerated {
                    header: _,
                    payload: _,
                } => {
                    // 应该忽略 ResultGenerated 事件
                    // we should ignore this event
                    // println!(
                    //     "ResultGenerated result: {:?}",
                    //     payload.output.unwrap().sentence.unwrap().text
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

        async fn on_data(&self, tx: &mut SplitSink<WebSocket, Message>, data: bytes::Bytes) {
            // 保存接收到的音频数据到文件
            println!("Received audio data chunk, length: {}", data.len());
            
            // 将音频数据写入文件
            if let Ok(mut file_guard) = self.audio_file.try_lock() {
                if let Some(ref mut file) = *file_guard {
                    if let Err(e) = file.write_all(&data).await {
                        eprintln!("Failed to write audio data to file: {}", e);
                    } else {
                        println!("Successfully wrote {} bytes to audio file", data.len());
                    }
                }
            } else {
                eprintln!("Failed to acquire file lock");
            }
        }

        async fn on_complete(&self) {
            println!("TTS process completed. Audio saved to output_{}.wav", self.task_id);
        }
        
        async fn on_close(&self, code: CloseCode, reason: String) {
            println!("WebSocket connection closed: {:?}", (code, reason));
        }

        async fn on_error(&self, error: DashScopeError) {
            println!("WebSocket connection error: {:?}", error);
        }

        fn heartbeat_interval(&self) -> Option<Duration> {
            // 添加心跳间隔，每10秒发送一次心跳
            Some(Duration::from_secs(10))
        }
    }

    let callback = CosyVoiceCallback::new(shared_task_id, audio_file);
    
    client
        .audio()
        .tts_ws()
        .await?
        .call(callback)
        .await?;

    println!("Audio synthesis completed. Output saved to {}", audio_filename);
    Ok(())
}