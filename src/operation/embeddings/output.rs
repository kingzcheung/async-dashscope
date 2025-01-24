use serde::{Deserialize, Serialize};

use crate::operation::common::Usage;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Embeddings {
    #[serde(rename = "embedding")]
    pub embedding: Option<Vec<f64>>,

    #[serde(rename = "text_index")]
    pub text_index: Option<i32>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Output {
    #[serde(rename = "embeddings")]
    pub embeddings: Vec<Embeddings>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmbeddingsOutput {
    #[serde(rename = "code")]
    pub code: Option<String>,

    #[serde(rename = "message")]
    pub message: Option<String>,

    #[serde(rename = "output")]
    pub output: Output,

    #[serde(rename = "request_id")]
    pub request_id: String,

    #[serde(rename = "status_code")]
    pub status_code: Option<i32>,

    #[serde(rename = "usage")]
    pub usage: Option<Usage>,
}
