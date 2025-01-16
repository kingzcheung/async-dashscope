use derive_builder::Builder;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Builder,Serialize,Deserialize, PartialEq)]
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
}

#[derive(Debug, Clone, Builder,Serialize,Deserialize, PartialEq)]
pub struct StreamOptions {
    pub include_usage: bool,
}

#[derive(Debug, Clone, Builder,Serialize,Deserialize, PartialEq)]
pub struct TranslationOptions {
    #[builder(setter(into))]
    pub source_lang: String,
    #[builder(setter(into))]
    pub target_lang: String,
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub terms: Option<Vec<Term>>,
}

#[derive(Debug, Clone, Builder,Serialize,Deserialize, PartialEq)]
pub struct Term {
    pub source: String,
    pub target: String,
}
