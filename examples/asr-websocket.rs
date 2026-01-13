//! WebSocket ASR 示例
//! 
//! 此示例演示如何使用 DashScope WebSocket API 进行实时语音识别
//! 
//! 运行示例：
//! ```
//! cargo run --example asr-websocket --features asr
//! ```

use async_dashscope::Client;
use async_dashscope::operation::audio::asr::{
    AutomaticSpeechRecognitionParam, AutomaticSpeechRecognitionParamBuilder, AsrParametersBuilder,
    AsrInput, AsrInputBuilder, EventType
};
use std::pin::Pin;
use tokio_stream::Stream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量获取 API Key
    let api_key = std::env::var("DASHSCOPE_API_KEY")
        .expect("请设置 DASHSCOPE_API_KEY 环境变量");

    // 创建客户端
    let client = Client::new();
    let audio = client.audio();
    
    // 创建 ASR WebSocket 客户端
    let asr_client = audio.asr_websocket(api_key);

    // 构建 ASR 参数
    let param = AutomaticSpeechRecognitionParamBuilder::default()
        .model("paraformer-realtime-v2".to_string())
        .input(AsrInputBuilder::default().build()?)
        .parameters(
            AsrParametersBuilder::default()
                .format("pcm".to_string())
                .sample_rate(16000)
                .punctuation_prediction_enabled(Some(true))
                .disfluency_removal_enabled(Some(false))
                .build()?
        )
        .build()?;

    println!("开始语音识别...");
    
    // 创建模拟音频流（这里使用空的音频流作为示例）
    // 在实际使用中，您需要提供真实的音频数据流
    let audio_stream: Pin<Box<dyn Stream<Item = Result<Vec<u8>, async_dashscope::error::DashScopeError>> + Send>> = 
        Box::pin(async_stream::try_stream! {
            // 模拟音频数据（在实际应用中，这里应该是从麦克风或文件读取的音频数据）
            for i in 0..10 {
                // 生成模拟的 PCM 音频数据（1600字节 ≈ 100ms 16kHz 16bit 单声道音频）
                let mut chunk = vec![0u8; 1600];
                // 添加一些模拟数据（在实际应用中应该是真实的音频数据）
                for j in 0..chunk.len() {
                    chunk[j] = ((i * 100 + j) % 256) as u8;
                }
                yield chunk;
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        });

    // 开始语音识别
    let mut event_stream = asr_client.recognize(param, audio_stream).await?;

    // 处理事件流
    while let Some(event) = event_stream.next().await {
        match event {
            Ok(event) => {
                match event.event_type() {
                    Some(EventType::TaskStarted) => {
                        println!("✅ 任务已开始: task_id = {}", event.header.task_id);
                    }
                    Some(EventType::ResultGenerated) => {
                        if let Some(sentence) = event.get_recognition_result() {
                            if sentence.is_intermediate() {
                                println!("🔄 中间结果: {} (开始时间: {}ms)", 
                                    sentence.text, sentence.begin_time);
                            } else {
                                println!("✅ 最终结果: {} (时长: {}ms)", 
                                    sentence.text, 
                                    sentence.duration().unwrap_or(0));
                                
                                // 显示字时间戳信息
                                if !sentence.words.is_empty() {
                                    println!("   字时间戳:");
                                    for word in &sentence.words {
                                        println!("     '{}' {}ms-{}ms 标点: '{}'", 
                                            word.text, word.begin_time, word.end_time, word.punctuation);
                                    }
                                }
                            }
                        }
                    }
                    Some(EventType::TaskFinished) => {
                        println!("✅ 任务已完成: task_id = {}", event.header.task_id);
                    }
                    Some(EventType::TaskFailed) => {
                        println!("❌ 任务失败: {} - {}", 
                            event.header.error_code.unwrap_or_default(),
                            event.header.error_message.unwrap_or_default());
                    }
                    None => {
                        println!("⚠️  未知事件类型: {}", event.header.event);
                    }
                }
            }
            Err(e) => {
                println!("❌ 处理事件时发生错误: {}", e);
                break;
            }
        }
    }

    println!("语音识别完成");
    Ok(())
}

/// 从文件读取音频数据的辅助函数
/// 
/// 在实际应用中，您可以使用此函数从 WAV 或其他音频文件读取数据
/// 并将其转换为适合 WebSocket ASR 的格式
#[allow(dead_code)]
async fn read_audio_from_file(file_path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // 这里应该是读取音频文件并转换为 PCM 格式的代码
    // 例如使用 hound crate 读取 WAV 文件
    
    // 示例：返回空的音频数据
    Ok(vec![])
}

/// 创建实时音频流的辅助函数
/// 
/// 在实际应用中，您可以使用此函数从麦克风捕获实时音频
#[allow(dead_code)]
fn create_realtime_audio_stream() -> Pin<Box<dyn Stream<Item = Result<Vec<u8>, async_dashscope::error::DashScopeError>> + Send>> {
    Box::pin(async_stream::try_stream! {
        // 这里应该是从麦克风捕获音频的代码
        // 例如使用 cpal crate 进行音频捕获
        
        // 示例：返回空的音频流
        yield vec![];
    })
}