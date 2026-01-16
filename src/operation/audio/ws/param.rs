use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use std::collections::{HashMap};

/// ASR 输入结构体
#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct RunTaskParameters {
    header: TaskHeader,
    payload: RunTaskPayload,
}

impl TryFrom<RunTaskParameters> for String {
    type Error = crate::error::DashScopeError;

    fn try_from(value: RunTaskParameters) -> Result<Self, Self::Error> {
        serde_json::to_string(&value)
            .map_err(|e| crate::error::DashScopeError::SerializationError(e.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskAction {
    /// 运行任务
    #[serde(rename = "run-task")]
    RunTask,
    /// 停止任务
    #[serde(rename = "finish-task")]
    FinishTask,
    /// 继续任务
    #[serde(rename = "continue-task")]
    ContinueTask,
}

/// 任务头部信息
#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct TaskHeader {
    /// 指令类型，固定为run-task
    action: TaskAction,
    /// 任务的唯一标识
    #[builder(default = "uuid::Uuid::new_v4().to_string()")]
    task_id: String,
    /// 通信模式，固定为duplex
    #[serde(default = "default_streaming")]
    #[builder(default = "default_streaming()")]
    streaming: String,
}

impl TaskHeader {
    pub fn run_task() -> Self {
        Self {
            action: TaskAction::RunTask,
            task_id: uuid::Uuid::new_v4().to_string(),
            streaming: default_streaming(),
        }
    }
    pub fn finish_task() -> Self {
        Self {
            action: TaskAction::FinishTask,
            task_id: uuid::Uuid::new_v4().to_string(),
            streaming: default_streaming(),
        }
    }
    pub fn continue_task() -> Self {
        Self {
            action: TaskAction::ContinueTask,
            task_id: uuid::Uuid::new_v4().to_string(),
            streaming: default_streaming(),
        }
    }
}

fn default_streaming() -> String {
    "duplex".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RunTaskType {
    /// 识别任务
    #[serde(rename = "asr")]
    Asr,
    /// 合成任务
    #[serde(rename = "tts")]
    Tts,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RunTaskFunction {
    /// 识别任务
    #[serde(rename = "recognition")]
    Recognition,
    /// 合成任务
    #[serde(rename = "SpeechSynthesizer")]
    SpeechSynthesizer,
}

/// 任务载荷信息
#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct RunTaskPayload {
    /// 任务组，固定为audio
    task_group: String,
    /// 任务类型，可选项 asr, tts
    task: RunTaskType,
    /// 功能类型，可选项 recognition,SpeechSynthesizer
    function: RunTaskFunction,
    /// 指定要使用的模型
    #[builder(setter(into))]
    model: String,
    /// 输入配置，固定为空对象{}
    #[builder(default)]
    input: HashMap<String, serde_json::Value>,
    /// 识别参数
    #[builder(default)]
    parameters: TaskParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum TextType {
    #[default]
    #[serde(rename = "PlainText")]
    PlainText,
}

/// 识别参数
#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq, Default)]
pub struct TaskParameters {
    /// 文本类型，固定为plain-text
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    text_type: Option<TextType>,
    /// see https://help.aliyun.com/zh/model-studio/cosyvoice-voice-list?spm=0.0.0.i13
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    voice: Option<String>,
    /// 音频格式
    format: String,
    /// 音频采样率
    /// 取值范围：8000, 16000, 22050, 24000, 44100, 48000。
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    sample_rate: Option<u32>,

    /// 取值范围：[0, 100]。50代表标准音量。音量大小与该值呈线性关系，0为静音，100为最大音量。
    #[builder(setter(strip_option), default)]
    #[serde(default = "default_volume")]
    #[serde(skip_serializing_if = "Option::is_none")]
    volume: Option<u8>,

    /// 语调，取值范围：[0.5, 2.0]。1.0为标准语速，小于1.0则减慢，大于1.0则加快。
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    rate: Option<f32>,

    ///音高。该值作为音高调节的乘数，但其与听感上的音高变化并非严格的线性或对数关系，建议通过测试选择合适的值。
    ///默认值：1.0。
    ///取值范围：[0.5, 2.0]。1.0为音色自然音高。大于1.0则音高变高，小于1.0则音高变低。
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pitch: Option<f32>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_ssml: Option<bool>,

    /// 音频码率，取值范围：取值范围：[6, 510]。
    ///
    /// > ⚠️注意: cosyvoice-v1模型不支持该参数。
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    bit_rate: Option<u32>,

    /// 是否开启单词时间戳（可选，默认false）
    ///
    /// 该功能仅适用于cosyvoice-v3-flash、cosyvoice-v3-plus和cosyvoice-v2模型的复刻音色，以及音色列表中标记为支持的系统音色。
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    word_timestamp_enabled: Option<bool>,

    /// 随机数种子（可选）取值范围：[0, 65535]。
    ///
    /// > ⚠️注意: cosyvoice-v1模型不支持该参数。
    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u32>,

    /// 语指定语音合成的目标语言，提升合成效果。
    ///
    /// 取值范围：
    /// - zh：中文
    /// - en：英文
    /// - fr：法语
    /// - de：德语
    /// - ja：日语
    /// - ko：韩语
    /// - ru：俄语
    /// > ⚠️注意: cosyvoice-v1模型不支持该参数。
    #[builder(setter(strip_option), default)]
    language_hints: Option<Vec<String>>,

    /// 指令（可选）
    /// 设置指令。该功能仅适用于cosyvoice-v3-flash和cosyvoice-v3-plus模型的复刻音色，以及音色列表中标记为支持的系统音色。
    ///
    /// 无默认值，不设置不生效。
    /// 在语音合成中有如下作用：
    /// 1. 指定方言（仅限复刻音色）
    /// - 格式："请用<方言>表达。"（注意，结尾一定不要遗漏句号，使用时将"<方言>"替换为具体的方言，例如替换为广东话）。
    /// - 示例："请用广东话表达。"
    /// - 支持的方言：广东话、东北话、甘肃话、贵州话、河南话、湖北话、江西话、闽南话、宁夏话、山西话、陕西话、山东话、上海话、四川话、天津话、云南话。
    ///
    /// 2. 指定情感、场景、角色或身份等：仅部分系统音色支持该功能，且因音色而异，详情请参见音色列表。
    #[builder(setter(into, strip_option), default)]
    instruction: Option<String>,

    /// 是否开启AIGC标签（可选，默认false）
    #[builder(setter(strip_option), default)]
    enable_aigc_tag: Option<bool>,

    /// AIGC标签传播器（可选）
    #[builder(setter(strip_option), default)]
    aigc_propagator: Option<String>,

    /// AIGC标签传播ID（可选）
    #[builder(setter(strip_option), default)]
    aigc_propagate_id: Option<String>,

    /// 热词ID（可选）
    #[builder(setter(into, strip_option), default)]
    vocabulary_id: Option<String>,
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
}

/// 创建ASR任务参数, 如果想要指定更多的参数，请直接使用 builder 创建
///
/// # Arguments
///
/// * `task_id` - 任务ID
/// * `model` - 模型ID
/// * `format` - 音频格式
/// * `sample_rate` - 采样率（可选）
pub fn create_asr_run_task(
    task_id: &str,
    model: &str,
    format: &str,
    sample_rate: Option<u32>,
) -> RunTaskParameters {
    RunTaskParameters {
        header: TaskHeader {
            action: TaskAction::RunTask,
            task_id: task_id.to_string(),
            streaming: "duplex".into(),
        },
        payload: RunTaskPayload {
            task_group: "audio".into(),
            task: RunTaskType::Asr,
            function: RunTaskFunction::Recognition,
            model: model.into(),
            input: HashMap::new(),
            parameters: TaskParameters {
                format: format.into(),
                sample_rate,
                ..Default::default()
            },
        },
    }
}

/// 创建TTS任务参数, 如果想要指定更多的参数，请直接使用 builder 创建
///
/// # Arguments
///
/// * `task_id` - 任务ID
/// * `model` - 模型ID
/// * `voice` - 语音ID
/// * `format` - 音频格式
pub fn create_tts_run_task(
    task_id: &str,
    model: &str,
    voice: Option< &str>,
    format: &str,
    text: Option<&str>,
) -> RunTaskParameters {
    let mut input = HashMap::new();
    if let Some(t) = text  {
        input.insert("text".to_string(), t.into());
    }
    RunTaskParameters {
        header: TaskHeader {
            action: TaskAction::RunTask,
            task_id: task_id.to_string(),
            streaming: "duplex".into(),
        },
        payload: RunTaskPayload {
            task_group: "audio".into(),
            task: RunTaskType::Tts,
            function: RunTaskFunction::SpeechSynthesizer,
            model: model.into(),
            input,
            parameters: TaskParameters {
                text_type: Some(TextType::PlainText),
                voice: voice.map(|v| v.into()),
                format: format.to_string(),
                ..Default::default()
            },
        },
    }
}

/// 默认音量
fn default_volume() -> Option<u8> {
    Some(50)
}

pub type FinishTaskHeader = TaskHeader;

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

pub fn create_finish_task(task_id: &str) -> FinishTaskParameters {
    FinishTaskParameters {
        header: TaskHeader {
            action: TaskAction::FinishTask,
            task_id: task_id.to_string(),
            streaming: "duplex".into(),
        },
        payload: FinishTaskPayload {
            input: HashMap::new(),
        },
    }
}

impl FinishTaskParameters {
    pub fn new(task_id: String) -> Self {
        Self {
            header: TaskHeaderBuilder::default()
                .action(TaskAction::FinishTask)
                .task_id(task_id)
                .build()
                .unwrap(),
            payload: FinishTaskPayloadBuilder::default().build().unwrap(),
        }
    }
}

/// 结束任务载荷信息
#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct FinishTaskPayload {
    /// 输入配置，固定为空对象{}
    #[builder(default)]
    input: HashMap<String, serde_json::Value>,
}

type ContinueTaskHeader = TaskHeader;

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct ContinueTaskParameters {
    header: ContinueTaskHeader,
    payload: ContinueTaskPayload,
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct ContinueTaskPayload {
    /// 输入配置
    input: ContinueTaskInput,
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct ContinueTaskInput {
    /// 输入文本
    #[builder(setter(into))]
    text: String,
}

impl TryFrom<ContinueTaskParameters> for String {
    type Error = crate::error::DashScopeError;

    fn try_from(value: ContinueTaskParameters) -> Result<Self, Self::Error> {
        serde_json::to_string(&value)
            .map_err(|e| crate::error::DashScopeError::SerializationError(e.to_string()))
    }
}

/// 创建继续任务参数
///
/// # Arguments
///
/// * `task_id` - 任务ID
/// * `text` - 输入文本
///
pub fn create_continue_task<S: ToString>(task_id: String, text: S) -> ContinueTaskParameters {
    ContinueTaskParameters {
        header: TaskHeader {
            action: TaskAction::FinishTask,
            task_id,
            streaming: "duplex".into(),
        },
        payload: ContinueTaskPayload {
            input: ContinueTaskInput {
                text: text.to_string(),
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_task_parameters_creation() {
        let header = TaskHeaderBuilder::default()
            .action(TaskAction::RunTask)
            .streaming("duplex".to_string())
            .build()
            .expect("Failed to build header");

        let task_params = TaskParametersBuilder::default()
            .text_type(TextType::PlainText)
            .format("pcm".to_string())
            .sample_rate(16000)
            .semantic_punctuation_enabled(true)
            .build()
            .expect("Failed to build recognition parameters");

        let payload = RunTaskPayloadBuilder::default()
            .task_group("audio".to_string())
            .task(RunTaskType::Asr)
            .function(RunTaskFunction::Recognition)
            .model("fun-asr-realtime".to_string())
            .parameters(task_params)
            .build()
            .expect("Failed to build payload");

        let params = RunTaskParametersBuilder::default()
            .header(header)
            .payload(payload)
            .build()
            .expect("Failed to build run task parameters");

        assert_eq!(params.header.action, TaskAction::RunTask);
        assert_eq!(params.payload.task_group, "audio");
        assert_eq!(params.payload.task, RunTaskType::Asr);
        assert_eq!(params.payload.function, RunTaskFunction::Recognition);
        assert_eq!(params.payload.model, "fun-asr-realtime");
        assert_eq!(params.payload.parameters.format, "pcm");
        assert_eq!(params.payload.parameters.sample_rate, Some(16000));
        assert_eq!(
            params.payload.parameters.semantic_punctuation_enabled,
            Some(true)
        );
    }

    #[test]
    fn test_finish_task_parameters_creation() {
        let header = TaskHeaderBuilder::default()
            .action(TaskAction::FinishTask)
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

        assert_eq!(params.header.action, TaskAction::FinishTask);
        assert_eq!(params.header.task_id, "test-task-id");
        assert_eq!(params.header.streaming, "duplex");
    }

    #[test]
    fn test_default_recognition_parameters() {
        let params = TaskParameters::default();

        assert_eq!(params.format, "");
        assert_eq!(params.sample_rate, None);
        assert_eq!(params.semantic_punctuation_enabled, None);
        assert_eq!(params.max_sentence_silence, None);
        assert_eq!(params.multi_threshold_mode_enabled, None);
        assert_eq!(params.heartbeat, None);
        assert_eq!(params.language_hints, None);
    }

    #[test]
    fn test_serialization_deserialization() {
        let header = TaskHeaderBuilder::default()
            .action(TaskAction::RunTask)
            .streaming("duplex".to_string())
            .build()
            .expect("Failed to build header");

        let task_params = TaskParametersBuilder::default()
            .text_type(TextType::PlainText)
            .format("wav".to_string())
            .sample_rate(44100)
            .semantic_punctuation_enabled(false)
            .build()
            .expect("Failed to build recognition parameters");

        let payload = RunTaskPayloadBuilder::default()
            .task_group("audio".to_string())
            .task(RunTaskType::Asr)
            .function(RunTaskFunction::Recognition)
            .model("fun-asr-realtime".to_string())
            .parameters(task_params)
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
        let header = TaskHeaderBuilder::default()
            .action(TaskAction::FinishTask)
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
