// https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation - text-generation
// https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation - image-generation
// https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation - 音频理解、视觉理解
// https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation  - 录音文件识别
// https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation  - 语音合成
// https://dashscope.aliyuncs.com/api/v1/services/aigc/text2image/image-synthesis - 创意海报生成API参考

// https://dashscope-intl.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation  新加坡
// https://dashscope.aliyuncs.com/api/v1/services/aigc/image2image/image-synthesis 图像翻译

// https://dashscope.aliyuncs.com/api/v1/files 上传文件

// todo: Qwen3-Coder 暂不支持 dashscope 的基于 Partial Mode 的代码补全功能

use std::path::{Path, PathBuf};

use derive_builder::Builder;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderValue, USER_AGENT};
use secrecy::{ExposeSecret as _, SecretString};

use crate::error::DashScopeError;

pub const DASHSCOPE_API_BASE: &str = "https://dashscope.aliyuncs.com/api/v1";

/// WebSocket 默认接入地址
pub const DASHSCOPE_WEBSOCKET_API_BASE: &str =
    "wss://dashscope.aliyuncs.com/api-ws/v1/inference";

/// API Key 环境变量名
pub const DASHSCOPE_API_KEY_ENV: &str = "DASHSCOPE_API_KEY";

/// API Key 文件路径环境变量名
pub const DASHSCOPE_API_KEY_FILE_PATH_ENV: &str = "DASHSCOPE_API_KEY_FILE_PATH";

/// HTTP 接入地址环境变量名
pub const DASHSCOPE_HTTP_BASE_URL_ENV: &str = "DASHSCOPE_HTTP_BASE_URL";

/// WebSocket 接入地址环境变量名
pub const DASHSCOPE_WEBSOCKET_BASE_URL_ENV: &str = "DASHSCOPE_WEBSOCKET_BASE_URL";

/// 地域环境变量名
pub const DASHSCOPE_API_REGION_ENV: &str = "DASHSCOPE_API_REGION";

/// API 版本环境变量名
pub const DASHSCOPE_API_VERSION_ENV: &str = "DASHSCOPE_API_VERSION";

/// 业务空间 ID 的环境变量名，未显式设置 `workspace` 时作为默认值
pub const DASHSCOPE_WORKSPACE_ID_ENV: &str = "DASHSCOPE_WORKSPACE_ID";

/// 设置任意值可禁用 SDK 标识请求头
pub const DASHSCOPE_DISABLE_SDK_HEADERS_ENV: &str = "DASHSCOPE_DISABLE_SDK_HEADERS";

/// 业务空间请求头
pub const WORKSPACE_HEADER: &str = "X-DashScope-WorkSpace";

/// 异步任务请求头
pub const ASYNC_HEADER: &str = "X-DashScope-Async";

/// OSS 资源解析请求头
pub const OSS_RESOURCE_RESOLVE_HEADER: &str = "X-DashScope-OssResourceResolve";

/// 业务空间 ID 占位符
pub const WORKSPACE_ID_PLACEHOLDER: &str = "{workspace_id}";

/// 支持 MaaS 域名（`{workspace_id}.{region}.maas.aliyuncs.com`）的地域，
/// 与官方 SDK 保持一致；`cn-beijing` 继续使用旧域名
const MAAS_REGIONS: [&str; 5] = [
    "ap-southeast-1",
    "us-east-1",
    "cn-hongkong",
    "eu-central-1",
    "ap-northeast-1",
];

fn api_version() -> String {
    std::env::var(DASHSCOPE_API_VERSION_ENV).unwrap_or_else(|_| "v1".to_string())
}

/// 根据 `DASHSCOPE_API_REGION` 推导 HTTP 接入地址
fn maas_http_base(region: &str) -> Option<String> {
    MAAS_REGIONS.contains(&region).then(|| {
        format!(
            "https://{WORKSPACE_ID_PLACEHOLDER}.{region}.maas.aliyuncs.com/api/{}",
            api_version()
        )
    })
}

/// 根据 `DASHSCOPE_API_REGION` 推导 WebSocket 接入地址
fn maas_websocket_base(region: &str) -> Option<String> {
    MAAS_REGIONS.contains(&region).then(|| {
        format!(
            "wss://{WORKSPACE_ID_PLACEHOLDER}.{region}.maas.aliyuncs.com/api-ws/{}/inference",
            api_version()
        )
    })
}

/// 校验业务空间 ID 是否可作为 URL 的 DNS label（与官方 SDK 规则一致）
fn is_valid_workspace_id(workspace_id: &str) -> bool {
    let mut chars = workspace_id.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphanumeric() => {}
        _ => return false,
    }
    workspace_id.len() <= 64
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// 默认 API Key 文件路径：`DASHSCOPE_API_KEY_FILE_PATH` 或 `~/.dashscope/api_key`
fn default_api_key_file_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var(DASHSCOPE_API_KEY_FILE_PATH_ENV) {
        return Some(PathBuf::from(path));
    }
    std::env::home_dir().map(|home| home.join(".dashscope").join("api_key"))
}

fn read_api_key_file(path: &Path) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|key| key.trim().to_string())
        .filter(|key| !key.is_empty())
}

/// 解析默认 API Key：环境变量优先，其次 API Key 文件
fn resolve_default_api_key() -> Option<String> {
    if let Ok(api_key) = std::env::var(DASHSCOPE_API_KEY_ENV) {
        return Some(api_key);
    }
    let path = default_api_key_file_path()?;
    read_api_key_file(&path)
}

fn sdk_headers_disabled() -> bool {
    std::env::var_os(DASHSCOPE_DISABLE_SDK_HEADERS_ENV).is_some()
}

fn user_agent() -> String {
    format!(
        "async-dashscope/{}; platform/{}/{}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH
    )
}

fn sdk_session_id() -> &'static str {
    static SESSION_ID: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    SESSION_ID.get_or_init(|| uuid::Uuid::new_v4().simple().to_string())
}

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
///         // optional, 指定归属业务空间（也可用 DASHSCOPE_WORKSPACE_ID 环境变量）
///         .workspace("ws_xxxxxxxx")
///         .build()
///         .unwrap();
/// let  client = Client::with_config(conf);
/// ```
#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct Config {
    /// 接入地址，默认读取 `DASHSCOPE_HTTP_BASE_URL`，否则为
    /// `https://dashscope.aliyuncs.com/api/v1`
    #[builder(setter(into, strip_option))]
    #[builder(default = "self.default_base_url()")]
    api_base: Option<String>,
    /// API Key，默认读取 `DASHSCOPE_API_KEY`，其次读取
    /// `DASHSCOPE_API_KEY_FILE_PATH` 或 `~/.dashscope/api_key`
    #[builder(default = "self.default_api_key()")]
    api_key: SecretString,
    /// 归属业务空间 ID。
    ///
    /// 设置后所有请求会自动携带 `X-DashScope-WorkSpace` 请求头；
    /// 若 `api_base` 中含有 `{workspace_id}` 占位符（例如新加坡地域的
    /// `https://{workspace_id}.ap-southeast-1.maas.aliyuncs.com/api/v1`），
    /// 也会自动替换为实际的业务空间 ID。
    #[builder(setter(into, strip_option))]
    #[builder(default = "self.default_workspace()")]
    workspace: Option<String>,
    /// WebSocket 接入地址，默认读取 `DASHSCOPE_WEBSOCKET_BASE_URL`，
    /// 否则为 `wss://dashscope.aliyuncs.com/api-ws/v1/inference`，
    /// 同样支持 `{workspace_id}` 占位符
    #[builder(setter(into, strip_option))]
    #[builder(default = "self.default_websocket_base()")]
    websocket_base: Option<String>,
}

impl ConfigBuilder {
    fn default_base_url(&self) -> Option<String> {
        Some(resolve_default_base_url())
    }

    fn default_api_key(&self) -> SecretString {
        resolve_default_api_key().unwrap_or_default().into()
    }

    fn default_workspace(&self) -> Option<String> {
        std::env::var(DASHSCOPE_WORKSPACE_ID_ENV).ok()
    }

    fn default_websocket_base(&self) -> Option<String> {
        Some(resolve_default_websocket_base())
    }
}

/// 默认 HTTP 接入地址：`DASHSCOPE_HTTP_BASE_URL` > 地域 MaaS 域名 > 旧域名
fn resolve_default_base_url() -> String {
    if let Ok(url) = std::env::var(DASHSCOPE_HTTP_BASE_URL_ENV) {
        return url;
    }
    if let Ok(region) = std::env::var(DASHSCOPE_API_REGION_ENV) {
        if let Some(url) = maas_http_base(&region) {
            return url;
        }
    }
    DASHSCOPE_API_BASE.to_string()
}

/// 默认 WebSocket 接入地址：`DASHSCOPE_WEBSOCKET_BASE_URL` > 地域 MaaS 域名 > 旧域名
fn resolve_default_websocket_base() -> String {
    if let Ok(url) = std::env::var(DASHSCOPE_WEBSOCKET_BASE_URL_ENV) {
        return url;
    }
    if let Ok(region) = std::env::var(DASHSCOPE_API_REGION_ENV) {
        if let Some(url) = maas_websocket_base(&region) {
            return url;
        }
    }
    DASHSCOPE_WEBSOCKET_API_BASE.to_string()
}

impl Config {
    /// 拼接请求地址。
    ///
    /// 如果 `api_base` 中的 `{workspace_id}` 占位符无法解析，会记录警告并
    /// 原样返回；需要严格错误处理请使用 [`Config::try_url`]。
    pub fn url(&self, path: &str) -> String {
        match self.try_url(path) {
            Ok(url) => url,
            Err(err) => {
                tracing::warn!("failed to resolve url: {err}");
                join_url(
                    self.api_base.as_deref().unwrap_or(DASHSCOPE_API_BASE),
                    path,
                )
            }
        }
    }

    /// 拼接请求地址，`api_base` 中的 `{workspace_id}` 占位符会被替换为
    /// 实际的业务空间 ID，未配置或非法时返回错误
    pub fn try_url(&self, path: &str) -> Result<String, DashScopeError> {
        let api_base = self.api_base.as_deref().unwrap_or(DASHSCOPE_API_BASE);
        let api_base = self.resolve_workspace(api_base)?;
        Ok(join_url(&api_base, path))
    }

    /// 拼接 WebSocket 接入地址，同样支持 `{workspace_id}` 占位符
    pub fn try_websocket_url(&self) -> Result<String, DashScopeError> {
        let websocket_base = self
            .websocket_base
            .as_deref()
            .unwrap_or(DASHSCOPE_WEBSOCKET_API_BASE);
        self.resolve_workspace(websocket_base)
    }

    fn resolve_workspace(&self, url: &str) -> Result<String, DashScopeError> {
        if !url.contains(WORKSPACE_ID_PLACEHOLDER) {
            return Ok(url.to_string());
        }
        match self.workspace.as_deref() {
            Some(workspace) if is_valid_workspace_id(workspace) => {
                Ok(url.replace(WORKSPACE_ID_PLACEHOLDER, workspace))
            }
            Some(workspace) => Err(DashScopeError::InvalidArgument(format!(
                "invalid workspace id `{workspace}`: must match ^[A-Za-z0-9][A-Za-z0-9_-]{{0,63}}$"
            ))),
            None => Err(DashScopeError::InvalidArgument(format!(
                "url `{url}` contains `{WORKSPACE_ID_PLACEHOLDER}` but no workspace is configured, \
                 set Config::workspace or the {DASHSCOPE_WORKSPACE_ID_ENV} environment variable"
            ))),
        }
    }

    /// 基础请求头
    pub fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json; charset=utf-8".parse().unwrap());
        headers.insert(ACCEPT, "application/json; charset=utf-8".parse().unwrap());
        headers.insert(
            AUTHORIZATION,
            format!("Bearer {}", self.api_key.expose_secret())
                .parse()
                .unwrap(),
        );
        if let Some(workspace) = self.workspace.as_deref() {
            if let Ok(value) = HeaderValue::from_str(workspace) {
                headers.insert(WORKSPACE_HEADER, value);
            }
        }
        if !sdk_headers_disabled() {
            headers.insert(
                USER_AGENT,
                HeaderValue::from_str(&user_agent())
                    .unwrap_or_else(|_| HeaderValue::from_static("async-dashscope")),
            );
            headers.insert(
                "x-dashscope-sdk-client",
                HeaderValue::from_str(&format!(
                    "async-dashscope/{}",
                    env!("CARGO_PKG_VERSION")
                ))
                .unwrap(),
            );
            headers.insert(
                "x-dashscope-sdk-session-id",
                HeaderValue::from_str(sdk_session_id()).unwrap(),
            );
        }
        headers
    }

    /// 在基础请求头上追加额外请求头
    pub fn headers_with(
        &self,
        extra: &[(&'static str, &'static str)],
    ) -> reqwest::header::HeaderMap {
        let mut headers = self.headers();
        for (name, value) in extra {
            headers.insert(*name, HeaderValue::from_static(value));
        }
        headers
    }

    /// 基础请求头 + `X-DashScope-Async: enable`（异步任务接口使用）
    pub fn async_headers(&self) -> reqwest::header::HeaderMap {
        self.headers_with(&[(ASYNC_HEADER, "enable")])
    }

    /// 基础请求头 + `X-DashScope-OssResourceResolve: enable`
    /// （多模态/图像类接口使用，用于解析 `oss://` 资源）
    pub fn oss_headers(&self) -> reqwest::header::HeaderMap {
        self.headers_with(&[(OSS_RESOURCE_RESOLVE_HEADER, "enable")])
    }

    pub fn set_api_key(&mut self, api_key: SecretString) {
        self.api_key = api_key;
    }
    
    pub fn api_key(&self) -> &SecretString {
        &self.api_key
    }

    /// 设置归属业务空间 ID
    pub fn set_workspace(&mut self, workspace: impl Into<String>) {
        self.workspace = Some(workspace.into());
    }

    /// 获取当前归属业务空间 ID
    pub fn workspace(&self) -> Option<&str> {
        self.workspace.as_deref()
    }
}

fn join_url(base: &str, path: &str) -> String {
    let url = format!(
        "{}/{}",
        base.trim_end_matches('/'),
        path.trim_start_matches('/')
    );
    url.trim_end_matches('/').to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_base: Some(resolve_default_base_url()),
            api_key: resolve_default_api_key().unwrap_or_default().into(),
            workspace: std::env::var(DASHSCOPE_WORKSPACE_ID_ENV).ok(),
            websocket_base: Some(resolve_default_websocket_base()),
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

    #[test]
    fn test_workspace_header() {
        let instance = ConfigBuilder::default()
            .api_key("test")
            .workspace("ws_123456")
            .build()
            .unwrap();
        assert_eq!(instance.workspace(), Some("ws_123456"));
        assert_eq!(
            instance.headers().get(WORKSPACE_HEADER).unwrap(),
            "ws_123456"
        );
    }

    #[test]
    fn test_no_workspace_header_by_default() {
        if std::env::var(DASHSCOPE_WORKSPACE_ID_ENV).is_ok() {
            return;
        }
        let instance = ConfigBuilder::default().api_key("test").build().unwrap();
        assert!(instance.headers().get(WORKSPACE_HEADER).is_none());
    }

    #[test]
    fn test_workspace_placeholder_in_api_base() {
        let instance = ConfigBuilder::default()
            .api_key("test")
            .api_base("https://{workspace_id}.ap-southeast-1.maas.aliyuncs.com/api/v1")
            .workspace("ws_abc")
            .build()
            .unwrap();
        assert_eq!(
            instance.url("/services/aigc/text-generation/generation"),
            "https://ws_abc.ap-southeast-1.maas.aliyuncs.com/api/v1/services/aigc/text-generation/generation"
        );
    }

    #[test]
    fn test_set_workspace() {
        let mut instance = ConfigBuilder::default().api_key("test").build().unwrap();
        instance.set_workspace("ws_setter");
        assert_eq!(instance.workspace(), Some("ws_setter"));
        assert_eq!(
            instance.headers().get(WORKSPACE_HEADER).unwrap(),
            "ws_setter"
        );
    }

    #[test]
    fn test_try_url_unresolved_placeholder_errors() {
        if std::env::var(DASHSCOPE_WORKSPACE_ID_ENV).is_ok() {
            return;
        }
        let instance = ConfigBuilder::default()
            .api_key("test")
            .api_base("https://{workspace_id}.ap-southeast-1.maas.aliyuncs.com/api/v1")
            .build()
            .unwrap();
        assert!(instance.try_url("/test").is_err());
    }

    #[test]
    fn test_try_url_invalid_workspace_errors() {
        let instance = ConfigBuilder::default()
            .api_key("test")
            .api_base("https://{workspace_id}.ap-southeast-1.maas.aliyuncs.com/api/v1")
            .workspace("bad/../workspace")
            .build()
            .unwrap();
        assert!(instance.try_url("/test").is_err());
    }

    #[test]
    fn test_websocket_url_placeholder() {
        let instance = ConfigBuilder::default()
            .api_key("test")
            .websocket_base(
                "wss://{workspace_id}.ap-southeast-1.maas.aliyuncs.com/api-ws/v1/inference",
            )
            .workspace("ws_abc")
            .build()
            .unwrap();
        assert_eq!(
            instance.try_websocket_url().unwrap(),
            "wss://ws_abc.ap-southeast-1.maas.aliyuncs.com/api-ws/v1/inference"
        );
    }

    #[test]
    fn test_default_headers() {
        let instance = ConfigBuilder::default().api_key("test").build().unwrap();
        let headers = instance.headers();
        assert_eq!(
            headers.get(CONTENT_TYPE).unwrap(),
            "application/json; charset=utf-8"
        );
        assert_eq!(
            headers.get(ACCEPT).unwrap(),
            "application/json; charset=utf-8"
        );
        if std::env::var_os(DASHSCOPE_DISABLE_SDK_HEADERS_ENV).is_none() {
            assert!(headers.get(USER_AGENT).is_some());
            assert!(
                headers
                    .get("x-dashscope-sdk-client")
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with("async-dashscope/")
            );
            assert!(headers.get("x-dashscope-sdk-session-id").is_some());
        }
    }

    #[test]
    fn test_base_headers_without_oss_resolve() {
        let instance = ConfigBuilder::default().api_key("test").build().unwrap();
        assert!(
            instance
                .headers()
                .get(OSS_RESOURCE_RESOLVE_HEADER)
                .is_none()
        );
    }

    #[test]
    fn test_oss_headers() {
        let instance = ConfigBuilder::default().api_key("test").build().unwrap();
        assert_eq!(
            instance
                .oss_headers()
                .get(OSS_RESOURCE_RESOLVE_HEADER)
                .unwrap(),
            "enable"
        );
        assert!(
            instance.headers().get(OSS_RESOURCE_RESOLVE_HEADER).is_none()
        );
    }

    #[test]
    fn test_async_headers() {
        let instance = ConfigBuilder::default().api_key("test").build().unwrap();
        assert_eq!(instance.async_headers().get(ASYNC_HEADER).unwrap(), "enable");
    }

    #[test]
    fn test_headers_with_extra() {
        let instance = ConfigBuilder::default().api_key("test").build().unwrap();
        let headers = instance.headers_with(&[
            (OSS_RESOURCE_RESOLVE_HEADER, "enable"),
            (ASYNC_HEADER, "enable"),
        ]);
        assert_eq!(headers.get(OSS_RESOURCE_RESOLVE_HEADER).unwrap(), "enable");
        assert_eq!(headers.get(ASYNC_HEADER).unwrap(), "enable");
        assert!(instance.headers().get(ASYNC_HEADER).is_none());
    }

    #[test]
    fn test_maas_base_urls() {
        if std::env::var(DASHSCOPE_API_VERSION_ENV).is_ok() {
            return;
        }
        assert_eq!(
            maas_http_base("ap-southeast-1").as_deref(),
            Some("https://{workspace_id}.ap-southeast-1.maas.aliyuncs.com/api/v1")
        );
        assert_eq!(
            maas_websocket_base("cn-hongkong").as_deref(),
            Some(
                "wss://{workspace_id}.cn-hongkong.maas.aliyuncs.com/api-ws/v1/inference"
            )
        );
        assert!(maas_http_base("cn-beijing").is_none());
        assert!(maas_websocket_base("unknown-region").is_none());
    }

    #[test]
    fn test_read_api_key_file() {
        let path = std::env::temp_dir().join(format!(
            "async-dashscope-key-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&path, "  sk-from-file\n").unwrap();
        assert_eq!(read_api_key_file(&path).as_deref(), Some("sk-from-file"));
        std::fs::write(&path, "   \n").unwrap();
        assert_eq!(read_api_key_file(&path), None);
        let _ = std::fs::remove_file(&path);
        assert_eq!(read_api_key_file(&path), None);
    }
}
