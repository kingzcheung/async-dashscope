use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Builder,Serialize,Deserialize, PartialEq)]
pub struct GenerationInput {
    pub model: String,
    pub input: Input,
    pub parameters: Parameters,
    #[builder(setter(into, strip_option))]
    #[builder(default=Some(false))]
    pub stream: Option<bool>,
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub stream_options: Option<StreamOptions>,
}

#[derive(Debug, Clone, Builder,Serialize,Deserialize, PartialEq)]
pub struct Input {
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Builder,Serialize,Deserialize, PartialEq)]
pub struct Message {
    #[builder(setter(into))]
    pub role: String,
    #[builder(setter(into))]
    pub content: String,
    #[builder(setter(into, strip_option))]
    #[builder(default=Some(false))]
    pub partial: Option<bool>,
}

#[derive(Debug, Clone, Builder,Serialize,Deserialize, PartialEq)]
pub struct Parameters {
    #[builder(setter(into, strip_option))]
    pub result_format: Option<String>,
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub translation_options: Option<TranslationOptions>,
}

#[derive(Debug, Clone, Builder,Serialize,Deserialize, PartialEq)]
pub struct StreamOptions {
    pub include_usage: bool,
}

#[derive(Debug, Clone, Builder,Serialize,Deserialize, PartialEq)]
pub struct TranslationOptions {
    pub source_lang: String,
    pub target_lang: String,
    #[builder(setter(into, strip_option))]
    pub terms: Option<Vec<Term>>,
}

#[derive(Debug, Clone, Builder,Serialize,Deserialize, PartialEq)]
pub struct Term {
    pub source: String,
    pub target: String,
}
