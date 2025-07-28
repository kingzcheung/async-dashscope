use crate::error::Result;
use crate::{Client, error::DashScopeError, operation::validate::check_model_parameters};
pub use output::*;
pub use param::{
    Element, InputBuilder, MessageBuilder, MultiModalConversationParam,
    MultiModalConversationParamBuilder, MultiModalConversationParamBuilderError,
};
use secrecy::ExposeSecret;

mod output;
mod param;

const MULTIMODAL_CONVERSATION_PATH: &str = "/services/aigc/multimodal-generation/generation";

pub struct MultiModalConversation<'a> {
    client: &'a Client,
}

impl<'a> MultiModalConversation<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// 异步调用多模态对话功能。
    ///
    /// 此函数用于处理非流式多模态对话请求。如果请求参数中设置了流式处理，将返回错误。
    ///
    /// # 参数
    /// * `request`: 包含多模态对话所需参数的请求对象。
    ///
    /// # 返回
    /// * 成功时返回包含多模态对话输出结果的 `Result`。
    /// * 如果请求参数中设置了 `stream` 为 `true`，将返回 `InvalidArgument` 错误。
    pub async fn call(
        &self,
        request: MultiModalConversationParam,
    ) -> Result<MultiModalConversationOutput> {
        // 检查请求是否为流式处理，如果是，则返回错误。
        if request.stream == Some(true) {
            return Err(DashScopeError::InvalidArgument(
                "When stream is true, use MultiModalConversation::call_stream".into(),
            ));
        }

        // Validate parameters before making the request.
        let validators = check_model_parameters(&request.model);
        for valid in validators {
            valid.validate(&request)?;
        }

        let request = request
            .upload_file_to_oss(self.client.config().api_key().expose_secret())
            .await?;

        // 发起非流式多模态对话请求。
        self.client
            .post(MULTIMODAL_CONVERSATION_PATH, request)
            .await
    }

    /// 异步调用流式多媒体对话功能
    ///
    /// 此函数用于处理流式多媒体对话请求。流式请求意味着响应会随着时间的推移逐步返回。
    ///
    /// # 参数
    /// * `request`: 多媒体对话参数。
    ///
    /// # 返回
    /// 返回一个流式输出结果，用于逐步处理和接收对话结果。
    ///
    /// # 错误
    /// 如果 `request` 参数中的 `stream` 属性为 `Some(false)`，
    /// 函数将返回一个错误，提示用户应使用非流式处理的 `call` 方法。
    pub async fn call_stream(
        &self,
        mut request: MultiModalConversationParam,
    ) -> Result<MultiModalConversationOutputStream> {
        // 检查请求是否明确设置为非流式，如果是，则返回错误。
        if request.stream == Some(false) {
            return Err(DashScopeError::InvalidArgument(
                "When stream is false, use MultiModalConversation::call".into(),
            ));
        }

        // 确保请求参数配置为流式处理
        request.stream = Some(true);

        // Validate parameters before making the request.
        let validators = check_model_parameters(&request.model);
        for valid in validators {
            valid.validate(&request)?;
        }

        // 发起流式请求并返回结果流
        self.client
            .post_stream(MULTIMODAL_CONVERSATION_PATH, request)
            .await
    }
}
