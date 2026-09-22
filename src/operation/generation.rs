use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;
use tokio_stream::StreamExt as _;

use crate::{client::Client, error::DashScopeError, operation::validate::Validator};
use crate::{error::Result, operation::validate::check_model_parameters};
pub use output::*;
pub use param::{
    AssistantMessageBuilder, GenerationParam, GenerationParamBuilder, InputBuilder, MessageBuilder,
    SystemMessageBuilder, ToolMessageBuilder, UserMessageBuilder,
};

mod merge;
mod output;
mod param;

use merge::IncrementalMergeState;

const GENERATION_PATH: &str = "/services/aigc/text-generation/generation";

/// 插件请求头，官方将 `plugins` 参数放在请求头中传递
const PLUGIN_HEADER: &str = "X-DashScope-Plugin";

/// 不支持客户端合并增量输出的模型（与官方 `should_modify_incremental_output` 一致）
fn supports_incremental_merge(model: &str) -> bool {
    let model = model.to_lowercase();
    !(model.contains("tts") || model.contains("omni") || model.contains("qwen-deep-research"))
}

/// 是否需要将 `incremental_output=false` 转换为客户端合并
fn should_merge_incremental_output(request: &GenerationParam) -> bool {
    supports_incremental_merge(&request.model)
        && request
            .parameters
            .as_ref()
            .and_then(|parameters| parameters.incremental_output)
            == Some(false)
}

/// 将 `plugins` 写入 `X-DashScope-Plugin` 请求头
fn apply_plugin_header(
    headers: &mut HeaderMap,
    plugins: &Option<Value>,
) -> crate::error::Result<()> {
    let Some(plugins) = plugins else {
        return Ok(());
    };
    let raw = match plugins {
        Value::String(value) => value.clone(),
        value => serde_json::to_string(value).map_err(|err| {
            DashScopeError::InvalidArgument(format!("invalid plugins value: {err}"))
        })?,
    };
    let value = HeaderValue::from_str(&raw).map_err(|_| {
        DashScopeError::InvalidArgument("invalid X-DashScope-Plugin header value".into())
    })?;
    headers.insert(PLUGIN_HEADER, value);
    Ok(())
}

pub struct Generation<'a> {
    client: &'a Client,
}

impl<'a> Generation<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// 异步调用生成服务
    ///
    /// 此函数用于当请求参数中的stream设置为false时，发送一次性生成请求
    /// 如果stream参数为true，则会返回错误，提示用户使用call_stream方法
    ///
    /// # 参数
    /// * `request`: 包含生成参数的请求对象
    ///
    /// # 返回
    /// 返回生成输出的结果，如果请求配置了stream且为true，则返回错误
    pub async fn call(&self, request: GenerationParam) -> Result<GenerationOutput> {
        // 检查请求是否启用了流式生成，如果是，则返回错误
        if request.stream == Some(true) {
            return Err(DashScopeError::InvalidArgument(
                "When stream is true, use Generation::call_stream".into(),
            ));
        }

        // 检查参数
        let validators = check_model_parameters(&request.model);
        for valid in validators {
            valid.validate(&request)?;
        }

        let mut headers = self.client.config().headers();
        apply_plugin_header(&mut headers, &request.plugins)?;

        // 发送POST请求到生成服务，并等待结果
        self.client
            .post_with_headers(GENERATION_PATH, request, headers)
            .await
    }

    /// 异步调用生成流函数
    ///
    /// 此函数用于处理文本生成的流式请求。流式请求意味着响应会随着时间的推移逐步返回，
    /// 而不是一次性返回所有内容。这对于需要实时处理生成内容的场景特别有用。
    ///
    /// # 参数
    /// * `request`: 一个可变的 `GenerationParam` 类型对象，包含了生成文本所需的参数。
    ///
    /// # 返回
    /// 返回一个 `Result` 类型，包含一个 `GenerationOutputStream` 对象，用于接收生成的文本流。
    /// 如果 `request` 中的 `stream` 字段为 `Some(false)`，则返回一个 `DashScopeError::InvalidArgument` 错误，
    /// 提示用户应使用 `Generation::call` 函数而不是 `call_stream`。
    ///
    /// # 错误处理
    /// 如果 `request` 参数中的 `stream` 属性为 `Some(false)`，表示用户不希望使用流式处理，
    /// 函数将返回一个错误，提示用户应使用非流式处理的 `call` 方法。
    ///
    /// # 注意
    /// 该函数自动将 `request` 的 `stream` 属性设置为 `Some(true)`，确保总是以流式处理方式执行生成任务。
    pub async fn call_stream(
        &self,
        mut request: GenerationParam,
    ) -> Result<GenerationOutputStream> {
        // 检查 `request` 中的 `stream` 属性，如果明确为 `false`，则返回错误
        if request.stream == Some(false) {
            return Err(DashScopeError::InvalidArgument(
                "When stream is false, use Generation::call".into(),
            ));
        }

        // 确保 `stream` 属性被设置为 `true`，即使它之前是 `None`
        request.stream = Some(true);

        // 检查参数（保持与 call 方法的一致性）
        let validators = check_model_parameters(&request.model);
        for valid in validators {
            valid.validate(&request)?;
        }

        // 请求非增量输出但模型仅支持增量时，改为增量请求并在客户端合并
        let merge_incremental = should_merge_incremental_output(&request);
        if merge_incremental {
            if let Some(parameters) = request.parameters.as_mut() {
                parameters.incremental_output = Some(true);
            }
        }

        let mut headers = self.client.config().headers();
        apply_plugin_header(&mut headers, &request.plugins)?;

        // 通过客户端发起 POST 请求，使用修改后的 `request` 对象，并等待异步响应
        let stream = self
            .client
            .post_stream_with_headers(GENERATION_PATH, request, headers)
            .await?;

        if merge_incremental {
            let mut state = IncrementalMergeState::default();
            Ok(Box::pin(
                stream.map(move |item| item.map(|output| state.merge(output))),
            ))
        } else {
            Ok(stream)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation::generation::param::{GenerationParamBuilder, InputBuilder, MessageBuilder};
    use serde_json::json;

    #[test]
    fn test_supports_incremental_merge() {
        assert!(supports_incremental_merge("qwen-plus"));
        assert!(supports_incremental_merge("deepseek-r1"));
        assert!(!supports_incremental_merge("qwen3-tts-flash"));
        assert!(!supports_incremental_merge("qwen-omni-turbo"));
        assert!(!supports_incremental_merge("qwen-deep-research"));
    }

    fn param_with_incremental_output(value: Option<bool>) -> GenerationParam {
        let mut parameters = crate::operation::common::ParametersBuilder::default()
            .build()
            .unwrap();
        parameters.incremental_output = value;
        GenerationParamBuilder::default()
            .model("qwen-plus")
            .input(
                InputBuilder::default()
                    .messages(vec![
                        MessageBuilder::default()
                            .user()
                            .content("hi")
                            .build()
                            .unwrap(),
                    ])
                    .build()
                    .unwrap(),
            )
            .parameters(parameters)
            .build()
            .unwrap()
    }

    #[test]
    fn test_should_merge_incremental_output() {
        assert!(should_merge_incremental_output(&param_with_incremental_output(
            Some(false)
        )));
        assert!(!should_merge_incremental_output(&param_with_incremental_output(
            Some(true)
        )));
        assert!(!should_merge_incremental_output(&param_with_incremental_output(
            None
        )));
    }

    #[test]
    fn test_apply_plugin_header_string() {
        let mut headers = HeaderMap::new();
        apply_plugin_header(&mut headers, &Some(json!("search_plus"))).unwrap();
        assert_eq!(headers.get(PLUGIN_HEADER).unwrap(), "search_plus");
    }

    #[test]
    fn test_apply_plugin_header_json() {
        let mut headers = HeaderMap::new();
        apply_plugin_header(
            &mut headers,
            &Some(json!({"search": {"enable": true}})),
        )
        .unwrap();
        assert_eq!(
            headers.get(PLUGIN_HEADER).unwrap(),
            "{\"search\":{\"enable\":true}}"
        );
    }

    #[test]
    fn test_apply_plugin_header_none() {
        let mut headers = HeaderMap::new();
        apply_plugin_header(&mut headers, &None).unwrap();
        assert!(headers.get(PLUGIN_HEADER).is_none());
    }

    #[test]
    fn test_plugins_not_serialized_in_body() {
        let request = GenerationParamBuilder::default()
            .model("qwen-plus")
            .input(
                InputBuilder::default()
                    .messages(vec![
                        MessageBuilder::default()
                            .user()
                            .content("hi")
                            .build()
                            .unwrap(),
                    ])
                    .build()
                    .unwrap(),
            )
            .plugins(json!({"search": {"enable": true}}))
            .build()
            .unwrap();

        let value = serde_json::to_value(&request).unwrap();
        assert!(value.get("plugins").is_none());
    }
}
