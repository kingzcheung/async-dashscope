// https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation - text-generation
// https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation - image-generation
// https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation - 音频理解、视觉理解
// https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation  - 录音文件识别
// https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation  - 语音合成
// https://dashscope.aliyuncs.com/api/v1/services/aigc/text2image/image-synthesis - 创意海报生成API参考

// https://dashscope-intl.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation  新加坡
// https://dashscope.aliyuncs.com/api/v1/services/aigc/image2image/image-synthesis 图像翻译

// todo: Qwen3-Coder 暂不支持 dashscope 的基于 Partial Mode 的代码补全功能

use derive_builder::Builder;
use reqwest::header::AUTHORIZATION;
use secrecy::{ExposeSecret as _, SecretString};

pub const DASHSCOPE_API_BASE: &str = "https://dashscope.aliyuncs.com/api/v1";

/// # Config
///
/// ```rust
/// use async_dashscope::config::ConfigBuilder;
/// use async_dashscope::Client;
/// 
/// let conf = ConfigBuilder::default()
///         // optional, default is: https://dashscope.aliyuncs.com/api/v1
///         .api_base("http://localhost:8080")
///         .api_key("test")
///         .build()
///         .unwrap();
/// let  client = Client::with_config(conf);
/// ```
#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct Config {
    #[builder(setter(into, strip_option))]
    #[builder(default = "self.default_base_url()")]
    api_base: Option<String>,
    api_key: SecretString,
}

impl ConfigBuilder {
    fn default_base_url(&self) -> Option<String> {
        Some(DASHSCOPE_API_BASE.to_string())
    }
}

impl Config {
    pub fn url(&self, path: &str) -> String {
        let n_url = format!(
            "{}/{}",
            self.api_base.clone()
                .unwrap_or(DASHSCOPE_API_BASE.to_string())
                .trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        n_url.trim_end_matches('/').to_string()
    }
    pub fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert(
            "X-DashScope-OssResourceResolve",
            "enable".parse().unwrap(),
        );
        headers.insert(
            AUTHORIZATION,
            format!("Bearer {}", self.api_key.expose_secret())
                .parse()
                .unwrap(),
        );
        headers
    }

    pub fn set_api_key(&mut self, api_key: SecretString) {
        self.api_key = api_key;
    }
    
    pub fn api_key(&self) -> &SecretString {
        &self.api_key
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_base: Some(DASHSCOPE_API_BASE.to_string()),
            api_key: std::env::var("DASHSCOPE_API_KEY")
                .unwrap_or_else(|_| "".to_string())
                .into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_normal_case() {
        let instance = ConfigBuilder::default()
            .api_base("https://example.com")
            .api_key("test")
            .build()
            .unwrap();
        assert_eq!(instance.url("/v1"), "https://example.com/v1");
    }

    #[test]
    fn test_url_empty_path() {
        let instance = ConfigBuilder::default()
            .api_base("http://localhost:8080")
            .api_key("test")
            .build()
            .unwrap();
        assert_eq!(instance.url(""), "http://localhost:8080");
    }

    #[test]
    fn test_url_empty_api_base() {
        let instance = ConfigBuilder::default().api_key("test").build().unwrap();
        assert_eq!(
            instance.url("/test"),
            format!("{DASHSCOPE_API_BASE}/test").as_str()
        );
    }

    #[test]
    fn test_url_slash_in_both_parts() {
        let instance = ConfigBuilder::default()
            .api_base("https://a.com/")
            .api_key("test")
            .build()
            .unwrap(); //Config {
        assert_eq!(instance.url("/b"), "https://a.com/b");
    }

    #[test]
    fn test_url_no_slash_in_path() {
        let instance = ConfigBuilder::default()
            .api_base("https://a.com")
            .api_key("test")
            .build()
            .unwrap();
        assert_eq!(instance.url("b"), "https://a.com/b");
    }

    #[test]
    fn test_api_key() {
        let instance = ConfigBuilder::default()
            .api_base("https://example.com")
            .api_key("test")
            .build()
            .unwrap();
        assert_eq!(
            instance.headers().get("Authorization").unwrap(),
            "Bearer test"
        );
    }
}
