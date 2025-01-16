// https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation - text-generation
// https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation - image-generation
// https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation - 音频理解
// https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation  - 录音文件识别
// https://dashscope.aliyuncs.com/api/v1/services/aigc/text2image/image-synthesis - 创意海报生成API参考

use derive_builder::Builder;
use reqwest::header::AUTHORIZATION;
use secrecy::{ExposeSecret as _, SecretString};


pub const DASHSCOPE_API_BASE: &str = "https://dashscope.aliyuncs.com/api/v1";

#[derive(Debug,Builder)]
pub struct Config {
    api_base: String,
    api_key: SecretString,
}

impl Config {
    pub fn url(&self,path:&str) ->String {
        format!("{}{}",self.api_base,path)
    }
    pub fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert(
            AUTHORIZATION,
            format!("Bearer {}", self.api_key.expose_secret())
                .parse()
                .unwrap(),
        );
        headers
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_base: DASHSCOPE_API_BASE.to_string(),
            api_key: std::env::var("DASHSCOPE_API_KEY")
                .unwrap_or_else(|_| "".to_string())
                .into(),
        }
    }
}

