use std::collections::HashMap;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct Parameters {
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub result_format: Option<String>,
    #[builder(setter(strip_option))]
    #[builder(default=None)]
    pub translation_options: Option<TranslationOptions>,
    //增量式流式输出
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub incremental_output: Option<bool>,

    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    // function call
    pub tools: Option<Vec<FunctionCall>>,

    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub parallel_tool_calls:Option<bool>,

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

    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub search_options: Option<SearchOptions>,

    // 只支持 qwen3, 对 QwQ 与 DeepSeek-R1 模型无效。
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub enable_thinking: Option<bool>,

    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub response_format: Option<ResponseFormat>
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct ResponseFormat{
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PromptTokensDetails {
    /// 命中 Cache 的 Token 数。Context Cache 详情请参见上下文缓存[（Context Cache）](https://help.aliyun.com/zh/model-studio/user-guide/context-cache?spm=a2c4g.11186623.0.0.37a0453aeh9s1L)。
    pub prompt_tokens: Option<i32>,
}
