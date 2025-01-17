use derive_builder::Builder;
use serde::{Deserialize, Serialize};

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
    pub text_type:Option<String>,
}


#[derive(Debug, Clone, Serialize,Builder, Deserialize, PartialEq)]
pub struct EmbeddingsInput{
    /// 文本列表
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    texts:Option<Vec<String>>,
    /// 图片地址或者图片 base64
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    image:Option<String>,
    /// 视频地址
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    video:Option<String>,
}


#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]

pub struct EmbeddingsParameters {
    /// 向量维度，可选值：768、1024、1536、2048
    dimension: u16,
}
