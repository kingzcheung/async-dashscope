use derive_builder::Builder;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Builder,Serialize,Deserialize, PartialEq)]
pub struct EmbeddingsParam {
    #[builder(setter(into))]
    pub model: String,
    pub input: EmbeddingsInput,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub parameters: Option<EmbeddingsParameters>,
}

#[derive(Debug, Clone, Builder,Serialize,Deserialize, PartialEq)]
pub struct EmbeddingsInput {
    #[builder(setter(into))]
    texts: Vec<String>
}

#[derive(Debug, Clone, Builder,Serialize,Deserialize, PartialEq)]

pub struct EmbeddingsParameters{
    dimension: u16,
}