use std::collections::HashMap;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct Parameters {
    /// 返回数据的格式。推荐优先设置为"message"
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub result_format: Option<String>,

    /// 当您使用翻译模型时需要配置的翻译参数。
    #[builder(setter(strip_option))]
    #[builder(default=None)]
    pub translation_options: Option<TranslationOptions>,
    // 增量式流式输出
    #[deprecated(
        since = "0.5.0",
        note = "Stream control is now unified under the top-level `stream` parameter in request objects. This parameter will be ignored."
    )]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub incremental_output: Option<bool>,

    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    // function call
    pub tools: Option<Vec<FunctionCall>>,

    /// 是否开启并行工具调用。参数为true时开启，为false时不开启。并行工具调用详情请参见：[并行工具调用](https://help.aliyun.com/zh/model-studio/qwen-function-calling#cb6b5c484bt4x)。
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub parallel_tool_calls: Option<bool>,

    // 限制思考长度
    // 该参数仅支持Qwen3 模型设定。
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub thinking_budget: Option<usize>,

    // 联网搜索
    // 仅 Qwen3 商业版模型、QwQ 商业版模型（除了qwq-plus-2025-03-05）支持联网搜索。
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub enable_search: Option<bool>,

    /// 联网搜索的策略。仅当enable_search为true时生效。
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub search_options: Option<SearchOptions>,

    /// 只支持 qwen3, 对 QwQ 与 DeepSeek-R1 模型无效。
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub enable_thinking: Option<bool>,

    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub response_format: Option<ResponseFormat>,

    /// 模型生成时连续序列中的重复度。提高repetition_penalty时可以降低模型生成的重复度，1.0表示不做惩罚。没有严格的取值范围，只要大于0即可。
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    repetition_penalty: Option<f64>,

    /// 控制模型生成文本时的内容重复度。
    /// 
    /// 取值范围：[-2.0, 2.0]。正数会减少重复度，负数会增加重复度。
    /// 
    /// 适用场景：
    /// - 较高的presence_penalty适用于要求多样性、趣味性或创造性的场景，如创意写作或头脑风暴。
    /// - 较低的presence_penalty适用于要求一致性或专业术语的场景，如技术文档或其他正式文档。
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    presence_penalty: Option<f64>,

    /// 是否提高输入图片的默认Token上限。输入图片的默认Token上限为1280，配置为true时输入图片的Token上限为16384。
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    vl_high_resolution_images: Option<bool>,

    /// 是否返回图像缩放后的尺寸。模型会对输入的图像进行缩放处理，配置为 True 时会返回图像缩放后的高度和宽度，开启流式输出时，该信息在最后一个数据块（chunk）中返回。支持Qwen-VL模型。
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    vl_enable_image_hw_output: Option<bool>,
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct ResponseFormat {
    #[builder(setter(into, strip_option))]
    #[serde(rename = "type")]
    pub type_: String,
}

impl ParametersBuilder {
    pub fn functions<V>(&mut self, value: V) -> &mut Self
    where
        V: Into<Vec<FunctionCall>>,
    {
        self.tools(value)
    }
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct FunctionCall {
    #[builder(setter(into, strip_option))]
    #[serde(rename = "type")]
    pub typ: Option<String>,

    #[builder(setter(into, strip_option))]
    #[serde(rename = "function")]
    pub function: Option<Function>,
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
#[builder(setter(into, strip_option))]
pub struct Function {
    name: String,
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    description: Option<String>,
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    parameters: Option<FunctionParameters>,
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct FunctionParameters {
    #[serde(rename = "type")]
    pub typ: String,
    properties: HashMap<String, Value>,
    required: Option<Vec<String>>,
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
#[builder(setter(into, strip_option))]
pub struct SearchOptions {
    #[builder(default=None)]
    pub forced_search: Option<bool>,
    #[builder(default=None)]
    pub enable_source: Option<bool>,
    #[builder(default=None)]
    pub enable_citation: Option<bool>,
    #[builder(default=None)]
    pub citation_format: Option<String>,
    #[builder(default=None)]
    pub search_strategy: Option<String>,
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct StreamOptions {
    pub include_usage: bool,
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct TranslationOptions {
    #[builder(setter(into))]
    pub source_lang: String,
    #[builder(setter(into))]
    pub target_lang: String,
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub terms: Option<Vec<Term>>,
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct Term {
    pub source: String,
    pub target: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Usage {
    /// 用户输入内容转换成token后的长度。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<i32>,

    /// chat请求返回内容转换成token后的长度。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<i32>,

    /// 当输入为纯文本时返回该字段，为input_tokens与output_tokens之和。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<i32>,

    /// 输入内容包含image时返回该字段。为用户输入图片内容转换成token后的长度。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_tokens: Option<i32>,

    /// 输入内容包含video时返回该字段。为用户输入视频内容转换成token后的长度。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_tokens: Option<i32>,

    /// 输入内容包含audio时返回该字段。为用户输入音频内容转换成token后的长度。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<i32>,

    /// 输入 Token 的细粒度分类。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details: Option<PromptTokensDetails>,

    /// Audio 输入的 Token 消耗信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens_details: Option<InputTokensDetails>,
    /// Audio 输出的 Token 消耗信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens_details: Option<OutputTokensDetails>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InputTokensDetails {
    text_tokens: Option<i32>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OutputTokensDetails {
    audio_tokens: Option<i32>,
    text_tokens: Option<i32>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PromptTokensDetails {
    /// 命中 Cache 的 Token 数。Context Cache 详情请参见上下文缓存[（Context Cache）](https://help.aliyun.com/zh/model-studio/user-guide/context-cache?spm=a2c4g.11186623.0.0.37a0453aeh9s1L)。
    pub prompt_tokens: Option<i32>,
}
