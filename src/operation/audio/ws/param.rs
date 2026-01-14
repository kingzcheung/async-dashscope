use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

/// ASR 输入结构体
#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct RunTaskParameters {
    header: RunTaskHeader,
    payload: RunTaskPayload,
}

impl TryFrom<RunTaskParameters> for String {
    type Error = crate::error::DashScopeError;

    fn try_from(value: RunTaskParameters) -> Result<Self, Self::Error> {
        serde_json::to_string(&value)
            .map_err(|e| crate::error::DashScopeError::SerializationError(e.to_string()))
    }
}

/// 任务头部信息
#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct RunTaskHeader {
    /// 指令类型，固定为run-task
    #[serde(default = "default_action")]
    #[builder(default = "default_action()")]
    action: String,
    /// 任务的唯一标识
    #[builder(default = "uuid::Uuid::new_v4().to_string()")]
    task_id: String,
    /// 通信模式，固定为duplex
    #[serde(default = "default_streaming")]
    #[builder(default = "default_streaming()")]
    streaming: String,
}

impl Default for RunTaskHeader {
    fn default() -> Self {
        Self {
            action: default_action(),
            task_id: uuid::Uuid::new_v4().to_string(),
            streaming: default_streaming(),
        }
    }
}
fn default_action() -> String {
    "run-task".to_string()
}

fn default_streaming() -> String {
    "duplex".to_string()
}

/// 任务载荷信息
#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct RunTaskPayload {
    /// 任务组，固定为audio
    task_group: String,
    /// 任务类型，固定为asr
    task: String,
    /// 功能类型，固定为recognition
    function: String,
    /// 指定要使用的模型
    model: String,
    /// 输入配置，固定为空对象{}
    #[builder(default)]
    input: HashMap<String, serde_json::Value>,
    /// 识别参数
    #[builder(default)]
    parameters: RecognitionParameters,
}

/// 识别参数
#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq, Default)]
pub struct RecognitionParameters {
    /// 音频格式
    format: String,
    /// 音频采样率
    sample_rate: u32,
    /// 热词ID（可选）暂不支持
    // #[builder(setter(strip_option), default)]
    // vocabulary_id: Option<String>,
    /// 是否开启语义断句（可选，默认false）
    #[builder(setter(strip_option), default)]
    semantic_punctuation_enabled: Option<bool>,
    /// VAD静音时长阈值（可选，默认1300）
    #[builder(setter(strip_option), default)]
    max_sentence_silence: Option<u32>,
    /// 是否开启防止VAD断句过长功能（可选，默认false）
    #[builder(setter(strip_option), default)]
    multi_threshold_mode_enabled: Option<bool>,
    /// 是否开启长连接保持开关（可选，默认false）
    #[builder(setter(strip_option), default)]
    heartbeat: Option<bool>,
    /// 设置待识别语言代码（可选）
    #[builder(setter(strip_option), default)]
    language_hints: Option<Vec<String>>,
}

/// 结束任务参数
#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct FinishTaskParameters {
    header: FinishTaskHeader,
    payload: FinishTaskPayload,
}

impl TryFrom<FinishTaskParameters> for String {
    type Error = crate::error::DashScopeError;

    fn try_from(value: FinishTaskParameters) -> Result<Self, Self::Error> {
        serde_json::to_string(&value)
            .map_err(|e| crate::error::DashScopeError::SerializationError(e.to_string()))
    }
}

impl FinishTaskParameters {
    pub fn new(task_id: String) -> Self {
        Self {
            header: FinishTaskHeaderBuilder::default()
                .task_id(task_id)
                .build()
                .unwrap(),
            payload: FinishTaskPayloadBuilder::default().build().unwrap(),
        }
    }
}

/// 结束任务头部信息
#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct FinishTaskHeader {
    /// 指令类型，固定为finish-task
    #[serde(default = "default_finish_action")]
    #[builder(default = "default_finish_action()")]
    action: String,
    /// 任务的唯一标识
    task_id: String,
    /// 通信模式，固定为duplex
    #[serde(default = "default_streaming")]
    #[builder(default = "default_streaming()")]
    streaming: String,
}

fn default_finish_action() -> String {
    "finish-task".to_string()
}

/// 结束任务载荷信息
#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct FinishTaskPayload {
    /// 输入配置，固定为空对象{}
    #[builder(default)]
    input: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_task_parameters_creation() {
        let header = RunTaskHeaderBuilder::default()
            .action("run-task".to_string())
            .streaming("duplex".to_string())
            .build()
            .expect("Failed to build header");

        let recognition_params = RecognitionParametersBuilder::default()
            .format("pcm".to_string())
            .sample_rate(16000)
            .semantic_punctuation_enabled(true)
            .build()
            .expect("Failed to build recognition parameters");

        let payload = RunTaskPayloadBuilder::default()
            .task_group("audio".to_string())
            .task("asr".to_string())
            .function("recognition".to_string())
            .model("fun-asr-realtime".to_string())
            .parameters(recognition_params)
            .build()
            .expect("Failed to build payload");

        let params = RunTaskParametersBuilder::default()
            .header(header)
            .payload(payload)
            .build()
            .expect("Failed to build run task parameters");

        assert_eq!(params.header.action, "run-task");
        assert_eq!(params.payload.task_group, "audio");
        assert_eq!(params.payload.task, "asr");
        assert_eq!(params.payload.function, "recognition");
        assert_eq!(params.payload.model, "fun-asr-realtime");
        assert_eq!(params.payload.parameters.format, "pcm");
        assert_eq!(params.payload.parameters.sample_rate, 16000);
        assert_eq!(
            params.payload.parameters.semantic_punctuation_enabled,
            Some(true)
        );
    }

    #[test]
    fn test_finish_task_parameters_creation() {
        let header = FinishTaskHeaderBuilder::default()
            .action("finish-task".to_string())
            .task_id("test-task-id".to_string())
            .streaming("duplex".to_string())
            .build()
            .expect("Failed to build finish task header");

        let payload = FinishTaskPayloadBuilder::default()
            .build()
            .expect("Failed to build finish task payload");

        let params = FinishTaskParametersBuilder::default()
            .header(header)
            .payload(payload)
            .build()
            .expect("Failed to build finish task parameters");

        assert_eq!(params.header.action, "finish-task");
        assert_eq!(params.header.task_id, "test-task-id");
        assert_eq!(params.header.streaming, "duplex");
    }

    #[test]
    fn test_default_recognition_parameters() {
        let params = RecognitionParameters::default();

        assert_eq!(params.format, "");
        assert_eq!(params.sample_rate, 0);
        assert_eq!(params.semantic_punctuation_enabled, None);
        assert_eq!(params.max_sentence_silence, None);
        assert_eq!(params.multi_threshold_mode_enabled, None);
        assert_eq!(params.heartbeat, None);
        assert_eq!(params.language_hints, None);
    }

    #[test]
    fn test_serialization_deserialization() {
        let header = RunTaskHeaderBuilder::default()
            .action("run-task".to_string())
            .streaming("duplex".to_string())
            .build()
            .expect("Failed to build header");

        let recognition_params = RecognitionParametersBuilder::default()
            .format("wav".to_string())
            .sample_rate(44100)
            .semantic_punctuation_enabled(false)
            .build()
            .expect("Failed to build recognition parameters");

        let payload = RunTaskPayloadBuilder::default()
            .task_group("audio".to_string())
            .task("asr".to_string())
            .function("recognition".to_string())
            .model("fun-asr-realtime".to_string())
            .parameters(recognition_params)
            .build()
            .expect("Failed to build payload");

        let original_params = RunTaskParametersBuilder::default()
            .header(header)
            .payload(payload)
            .build()
            .expect("Failed to build run task parameters");

        let serialized = serde_json::to_string(&original_params).unwrap();
        let deserialized: RunTaskParameters = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original_params, deserialized);
    }

    #[test]
    fn test_finish_task_serialization_deserialization() {
        let header = FinishTaskHeaderBuilder::default()
            .action("finish-task".to_string())
            .task_id("test-task-id".to_string())
            .streaming("duplex".to_string())
            .build()
            .expect("Failed to build finish task header");

        let payload = FinishTaskPayloadBuilder::default()
            .build()
            .expect("Failed to build finish task payload");

        let original_params = FinishTaskParametersBuilder::default()
            .header(header)
            .payload(payload)
            .build()
            .expect("Failed to build finish task parameters");

        let serialized = serde_json::to_string(&original_params).unwrap();
        let deserialized: FinishTaskParameters = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original_params, deserialized);
    }
}
