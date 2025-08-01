use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::operation::common::Parameters;
use crate::operation::request::RequestTrait;

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct EmbeddingsParam {
    /// 调用模型名称，可以选择text-embedding-v1，text-embedding-v2或者text-embedding-v3
    #[builder(setter(into))]
    pub model: String,
    pub input: EmbeddingsInput,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub parameters: Option<EmbeddingsParameters>,

    /// 文本转换为向量后可以应用于检索、聚类、分类等下游任务，
    /// 对检索这类非对称任务为了达到更好的检索效果建议区分查询文本（query）和底库文本（document）类型,
    ///  聚类、分类等对称任务可以不用特殊指定，采用系统默认值document即可。
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub text_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Builder, Deserialize, PartialEq)]
pub struct EmbeddingsInput {
    /// 文本列表
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub texts: Option<Vec<String>>,
    /// 图片地址或者图片 base64
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub image: Option<String>,
    /// 视频地址
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub video: Option<String>,
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct EmbeddingsParameters {
    /// 向量维度，可选值：768、1024、1536、2048
    /// 用于用户指定输出向量维度，只适用于text-embedding-v3与text-embedding-v4模型。指定的值只能在2048（仅适用于text-embedding-v4）、1536（仅适用于text-embedding-v4）1024、768、512、256、128或64八个值之间选取，默认值为1024。
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub dimension: Option<u16>,
    /// 用户指定输出离散向量表示只适用于text_embedding_v3与text_embedding_v4模型，取值在dense、sparse、dense&sparse之间，默认取dense，只输出连续向量。
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub output_type: Option<String>,
    /// 添加自定义任务说明，仅在使用 text-embedding-v4 模型且 text_type 为 query 时生效。建议使用英文撰写，通常可带来约 1%–5% 的效果提升。
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub instruct: Option<String>,
}

impl RequestTrait for EmbeddingsParam {
    type P = EmbeddingsParameters;
    fn model(&self) -> &str {
        &self.model
    }

    fn parameters(&self) -> Option<&Self::P> {
        self.parameters.as_ref()
    }

    
}
