use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::str::FromStr;
use tokio_stream::Stream;

use crate::error::DashScopeError;

/// ASR WebSocket 事件头结构体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WebSocketEventHeader {
    /// 任务ID
    pub task_id: String,
    /// 事件类型
    pub event: String,
    /// 属性
    pub attributes: serde_json::Value,
    /// 错误码（仅task-failed事件有）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    /// 错误消息（仅task-failed事件有）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// ASR WebSocket 事件负载结构体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WebSocketEventPayload {
    /// 输出（仅result-generated和task-finished事件有）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<AsrOutput>,
    /// 使用情况（仅result-generated事件有）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<AsrUsage>,
}

/// ASR WebSocket 事件枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WebSocketEvent {
    /// 任务开始事件
    TaskStarted { header: WebSocketEventHeader },
    /// 结果生成事件
    ResultGenerated {
        header: WebSocketEventHeader,
        payload: WebSocketEventPayload,
    },
    /// 任务完成事件
    TaskFinished {
        header: WebSocketEventHeader,
        payload: WebSocketEventPayload,
    },
    /// 任务失败事件
    TaskFailed { header: WebSocketEventHeader },
}

impl WebSocketEvent {
    /// 获取事件类型
    pub fn event_type(&self) -> EventType {
        match self {
            WebSocketEvent::TaskStarted { .. } => EventType::TaskStarted,
            WebSocketEvent::ResultGenerated { .. } => EventType::ResultGenerated,
            WebSocketEvent::TaskFinished { .. } => EventType::TaskFinished,
            WebSocketEvent::TaskFailed { .. } => EventType::TaskFailed,
        }
    }

    /// 判断是否为任务开始事件
    pub fn is_task_started(&self) -> bool {
        matches!(self, WebSocketEvent::TaskStarted { .. })
    }

    /// 判断是否为结果生成事件
    pub fn is_result_generated(&self) -> bool {
        matches!(self, WebSocketEvent::ResultGenerated { .. })
    }

    /// 判断是否为任务完成事件
    pub fn is_task_finished(&self) -> bool {
        matches!(self, WebSocketEvent::TaskFinished { .. })
    }

    /// 判断是否为任务失败事件
    pub fn is_task_failed(&self) -> bool {
        matches!(self, WebSocketEvent::TaskFailed { .. })
    }

    /// 获取任务ID
    pub fn task_id(&self) -> &str {
        match self {
            WebSocketEvent::TaskStarted { header } => &header.task_id,
            WebSocketEvent::ResultGenerated { header, .. } => &header.task_id,
            WebSocketEvent::TaskFinished { header, .. } => &header.task_id,
            WebSocketEvent::TaskFailed { header } => &header.task_id,
        }
    }

    /// 获取使用情况（仅部分事件类型有）
    pub fn get_usage(&self) -> Option<&AsrUsage> {
        match self {
            WebSocketEvent::ResultGenerated { payload, .. } => payload.usage.as_ref(),
            WebSocketEvent::TaskFinished { payload, .. } => payload.usage.as_ref(),
            _ => None,
        }
    }

    /// 获取错误信息（仅task-failed事件有）
    pub fn get_error_info(&self) -> Option<(&str, &str)> {
        match self {
            WebSocketEvent::TaskFailed { header } => {
                if let (Some(code), Some(message)) = (&header.error_code, &header.error_message) {
                    Some((code, message))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl TryFrom<String> for WebSocketEvent {
    type Error = DashScopeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // 首先解析JSON得到事件头，以确定事件类型
        let json_value: serde_json::Value =
            serde_json::from_str(&value).map_err(|e| DashScopeError::JSONDeserialize {
                source: e,
                raw_response: value.clone(),
            })?;

        // 提取事件类型
        let event_type = json_value
            .get("header")
            .and_then(|h| h.get("event"))
            .and_then(|e| e.as_str())
            .ok_or_else(|| DashScopeError::UnknownEventType {
                event_type: "unknown".to_string(),
            })?;

        // 根据事件类型决定如何反序列化整个对象
        match event_type {
            "task-started" => {
                let event: WebSocketEventWithHeaderOnly =
                    serde_json::from_str(&value).map_err(|e| DashScopeError::JSONDeserialize {
                        source: e,
                        raw_response: value,
                    })?;
                Ok(WebSocketEvent::TaskStarted {
                    header: event.header,
                })
            }
            "result-generated" => {
                let event: WebSocketEventWithPayload =
                    serde_json::from_str(&value).map_err(|e| DashScopeError::JSONDeserialize {
                        source: e,
                        raw_response: value,
                    })?;
                Ok(WebSocketEvent::ResultGenerated {
                    header: event.header,
                    payload: event.payload,
                })
            }
            "task-finished" => {
                let event: WebSocketEventWithPayload =
                    serde_json::from_str(&value).map_err(|e| DashScopeError::JSONDeserialize {
                        source: e,
                        raw_response: value,
                    })?;
                Ok(WebSocketEvent::TaskFinished {
                    header: event.header,
                    payload: event.payload,
                })
            }
            "task-failed" => {
                let event: WebSocketEventWithHeaderOnly =
                    serde_json::from_str(&value).map_err(|e| DashScopeError::JSONDeserialize {
                        source: e,
                        raw_response: value,
                    })?;
                Ok(WebSocketEvent::TaskFailed {
                    header: event.header,
                })
            }
            _ => Err(DashScopeError::UnknownEventType {
                event_type: event_type.to_string(),
            }),
        }
    }
}

// 辅助结构体，用于解析只有header的事件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct WebSocketEventWithHeaderOnly {
    pub header: WebSocketEventHeader,
}

// 辅助结构体，用于解析带payload的事件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct WebSocketEventWithPayload {
    pub header: WebSocketEventHeader,
    pub payload: WebSocketEventPayload,
}

/// ASR 转录结构体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Asrtranscription {
    /// 句子ID
    pub sentence_id: u32,
    /// 开始时间（单位ms）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub begin_time: Option<u32>,
    /// 结束时间（单位ms）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<u32>,
    /// 识别文本
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// 语言（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    /// 字时间戳信息
    pub words: Vec<AsrWord>,
    /// 句子是否已结束
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "deserialize_bool_from_any")]
    pub sentence_end: Option<bool>,
}

/// ASR 翻译结构体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AsrTranslation {
    /// 翻译文本
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// ASR 输出结构体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AsrOutput {
    /// 句子识别结果
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sentence: Option<AsrSentence>,

    /// 翻译结果
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translations: Option<Vec<AsrTranslation>>,

    /// 转录结果
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcription: Option<Asrtranscription>,
}

/// ASR 句子结构体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AsrSentence {
    /// 句子索引
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<u32>,
    /// 句子开始时间（单位ms）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub begin_time: Option<u32>,
    /// 句子结束时间（如果为中间识别结果则为null）

    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<u32>,
    /// 识别文本
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// 字时间戳信息
    pub words: Vec<AsrWord>,
    /// 心跳标记（若为true可跳过处理）
    pub heartbeat: Option<bool>,
    /// 句子是否已结束
    /// 提示： Gummy 的 sentence_end 有 bug，出现了一个字符串的 false
    #[serde(deserialize_with = "deserialize_bool_from_any", default)]
    pub sentence_end: Option<bool>,
    /// 情感标签（仅特定条件下显示）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emo_tag: Option<String>,
    /// 情感置信度（仅特定条件下显示）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emo_confidence: Option<f32>,
}

/// ASR 字结构体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AsrWord {
    /// 字开始时间（单位ms）
    pub begin_time: u32,
    /// 字结束时间（单位ms）
    pub end_time: u32,
    /// 字文本
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// 标点
    pub punctuation: String,
}

/// ASR 使用情况结构体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AsrUsage {
    /// 任务计费时长（单位秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
}

/// ASR 识别结果结构体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutomaticSpeechRecognitionOutput {
    /// 请求ID
    pub request_id: String,
    /// 输出结果
    pub output: AsrOutput,
    /// 使用情况
    pub usage: AsrUsage,
}

/// ASR WebSocket 流式输出类型
pub type AutomaticSpeechRecognitionOutputStream =
    Pin<Box<dyn Stream<Item = Result<WebSocketEvent, DashScopeError>> + Send>>;

/// 事件类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    TaskStarted,
    ResultGenerated,
    TaskFinished,
    TaskFailed,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::TaskStarted => "task-started",
            EventType::ResultGenerated => "result-generated",
            EventType::TaskFinished => "task-finished",
            EventType::TaskFailed => "task-failed",
        }
    }
}

impl FromStr for EventType {
    type Err = ();

    fn from_str(event: &str) -> Result<Self, Self::Err> {
        match event {
            "task-started" => Ok(EventType::TaskStarted),
            "result-generated" => Ok(EventType::ResultGenerated),
            "task-finished" => Ok(EventType::TaskFinished),
            "task-failed" => Ok(EventType::TaskFailed),
            _ => Err(()),
        }
    }
}

impl AsrSentence {
    /// 判断是否为中间结果（end_time为null）
    pub fn is_intermediate(&self) -> bool {
        self.end_time.is_none()
    }

    /// 判断是否为最终结果（end_time不为null）
    pub fn is_final(&self) -> bool {
        self.end_time.is_some()
    }
}

use serde::de::{self, Deserializer};
/// 从任何类型反序列化为布尔值的辅助函数
fn deserialize_bool_from_any<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Bool(b) => Ok(Some(b)),
        serde_json::Value::Number(n) => {
            if n.is_u64() || n.is_i64() {
                Ok(Some(n.as_i64().unwrap_or(0) != 0))
            } else {
                Ok(Some(n.as_f64().unwrap_or(0.0) != 0.0))
            }
        }
        serde_json::Value::String(s) => {
            if s == "true" || s == "1" || s == "yes" {
                Ok(Some(true))
            } else if s == "false" || s == "0" || s == "no" {
                Ok(Some(false))
            } else {
                // 尝试解析字符串是否为数字
                match s.parse::<i64>() {
                    Ok(n) => Ok(Some(n != 0)),
                    Err(_) => Err(de::Error::custom(format!(
                        "unable to parse '{}' as bool",
                        s
                    ))),
                }
            }
        }
        serde_json::Value::Null => Ok(Some(false)),
        _ => Err(de::Error::custom(format!(
            "unexpected value type for bool: {:?}",
            value
        ))),
    }
}
