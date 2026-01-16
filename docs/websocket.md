
## WebSocket 

WebSocket 支持音频合成(TTS)和 音频识别(ASR)任务。

TTS 任务支持的模型有： `CosyVoice`、`Sambert`。
ASR 任务支持的模型有： `ParaFormer`、` Fun-ASR`、`Gummy`。

无论是 `tts` 还是 `asr` 任务，按时间顺序，客户端与服务端的交互流程如下：

1. 建立连接：客户端与服务端建立WebSocket连接。
2. 开启任务：
   - 客户端发送`run-task`指令以开启任务。
   - 客户端收到服务端返回的`task-started`事件，标志着任务已成功开启。
3. 客户端接收服务端持续返回的音频流和`result-generated`事件
4. 客户端收到服务端返回的`task-finished`事件，标志着任务结束。
5. 关闭连接：客户端关闭WebSocket连接。

对于 tts，调用 api 如下：
```rust
client
    .audio()
    .tts_ws()
    .await?
    .call(callback)
    .await?;
```
对于 asr，调用 api 如下：
```rust
client
    .audio()
    .asr_ws()
    .await?
    .call(callback)
    .await?;
```

### Callback
WebSocket 回调函数需要实现 `WebsocketCallback` trait。

下面是一个简单的回调函数示例:
```rust
pub struct FunAsrCallback {
        task_id: String,
    }

impl WebsocketCallback for FunAsrCallback {
    async fn on_open(&self, tx: &mut SplitSink<WebSocket, Message>) {
        // 连接打开时发送run-task指令
        println!("WebSocket connection opened");
        
        let item = create_asr_run_task(&self.task_id, "fun-asr-realtime", "wav", Some(16000))
            .try_into()
            .unwrap();
        let item = Message::Text(item);

        tx.send(item).await.unwrap();
    }

    async fn on_event(&self, tx: &mut SplitSink<WebSocket, Message>, event: WebSocketEvent) {
        match event {
            WebSocketEvent::TaskStarted { header: _ } => {
                // send audio data
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

                println!(
                    "ResultGenerated result: {:?}",
                    payload.output.unwrap().sentence.unwrap().text
                );
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
```

更详细的调用参考 `examples`。