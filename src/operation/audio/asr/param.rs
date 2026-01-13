use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::operation::{common::Parameters, request::RequestTrait};

/// ASR 参数结构体，用于 WebSocket 语音识别
#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct AutomaticSpeechRecognitionParam {
    #[builder(setter(into))]
    pub model: String,
    
    #[builder(setter(into))]
    pub input: AsrInput,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub parameters: Option<AsrParameters>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub resources: Option<Vec<AsrResource>>,
}

/// ASR 输入结构体
#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct AsrInput {
    // WebSocket ASR 输入为空对象
}

/// ASR 参数配置结构体
#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct AsrParameters {
    /// 音频格式：pcm、wav、mp3、opus、speex、aac、amr
    #[builder(setter(into))]
    pub format: String,
    
    /// 采样率（单位Hz）
    pub sample_rate: u32,
    
    /// 热词ID
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub vocabulary_id: Option<String>,
    
    /// 是否过滤语气词
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub disfluency_removal_enabled: Option<bool>,
    
    /// 语言提示
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub language_hints: Option<Vec<String>>,
    
    /// 是否开启语义断句
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub semantic_punctuation_enabled: Option<bool>,
    
    /// VAD 静音时长阈值（单位ms）
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub max_sentence_silence: Option<u32>,
    
    /// 多阈值模式开关
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub multi_threshold_mode_enabled: Option<bool>,
    
    /// 是否添加标点
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub punctuation_prediction_enabled: Option<bool>,
    
    /// 心跳开关
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub heartbeat: Option<bool>,
    
    /// 逆文本正则化开关
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub inverse_text_normalization_enabled: Option<bool>,
}

/// ASR 资源结构体（用于热词功能）
#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct AsrResource {
    /// 热词ID
    #[builder(setter(into))]
    pub resource_id: String,
    
    /// 资源类型
    #[builder(setter(into))]
    pub resource_type: String,
}

impl RequestTrait for AutomaticSpeechRecognitionParam {
    fn model(&self) -> &str {
        &self.model
    }

    fn parameters(&self) -> Option<&Parameters> {
        None
    }
    
    type P = Parameters;
}

/// WebSocket 指令头结构体
#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct WebSocketHeader {
    /// 指令类型：run-task 或 finish-task
    #[builder(setter(into))]
    pub action: String,
    
    /// 任务ID（32位UUID）
    #[builder(setter(into))]
    pub task_id: String,
    
    /// 流式模式
    #[builder(setter(into))]
    pub streaming: String,
}

/// WebSocket 运行任务指令结构体
#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct RunTaskCommand {
    pub header: WebSocketHeader,
    pub payload: RunTaskPayload,
}

/// WebSocket 运行任务负载结构体
#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct RunTaskPayload {
    /// 任务组
    #[builder(setter(into))]
    pub task_group: String,
    
    /// 任务类型
    #[builder(setter(into))]
    pub task: String,
    
    /// 功能
    #[builder(setter(into))]
    pub function: String,
    
    /// 模型名称
    #[builder(setter(into))]
    pub model: String,
    
    /// 参数
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub parameters: Option<AsrParameters>,
    
    /// 资源
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub resources: Option<Vec<AsrResource>>,
    
    /// 输入
    pub input: AsrInput,
}

/// WebSocket 结束任务指令结构体
#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct FinishTaskCommand {
    pub header: WebSocketHeader,
    pub payload: FinishTaskPayload,
}

/// WebSocket 结束任务负载结构体
#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct FinishTaskPayload {
    /// 输入
    pub input: AsrInput,
}