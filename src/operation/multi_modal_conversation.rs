use crate::error::Result;
use crate::{error::DashScopeError, Client};
pub use output::*;
pub use param::{
    InputBuilder, MessageBuilder, MultiModalConversationParam, MultiModalConversationParamBuilder,
    MultiModalConversationParamBuilderError,
};

use super::common::ParametersBuilder;
mod output;
mod param;

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
                "When stream is true, use MultiModalGeneration::call_stream".into(),
            ));
        }

        // 检查请求参数是否设置为流式处理，如果是，则返回错误。
        if request.parameters.is_some() {
            if let Some(ref parameters) = request.parameters {
                if parameters.incremental_output == Some(true) {
                    return Err(DashScopeError::InvalidArgument(
                        "When stream is true, use MultiModalGeneration::call_stream".into(),
                    ));
                }
            }
        }

        // 发起非流式多模态对话请求。
        self.client
            .post("/services/aigc/multimodal-generation/generation", request)
            .await
    }

    /// 异步调用流式多媒体对话功能
    ///
    /// 此函数用于处理流式多媒体对话请求。它要求请求必须是流式请求，
    /// 并且如果请求参数中指定了非流式处理，则会返回错误。
    /// 如果请求参数未设置或未明确指定流式处理，函数将自动设置为流式处理。
    ///
    /// # 参数
    /// * `request`: 多媒体对话参数，包括是否为流式请求和其他配置参数
    ///
    /// # 返回
    /// 返回一个流式输出结果，用于逐步处理和接收对话结果
    ///
    /// # 错误
    /// 如果请求不是流式请求或参数配置与流式请求冲突，则返回无效参数错误
    pub async fn call_stream(
        &self,
        mut request: MultiModalConversationParam,
    ) -> Result<MultiModalConversationOutputStream> {
        // 检查请求是否为流式请求，如果不是，则返回错误提示使用非流式调用
        if request.stream != Some(true) {
            return Err(DashScopeError::InvalidArgument(
                "When stream is false, use MultiModalGeneration::call".into(),
            ));
        }

        // 如果请求中包含参数配置，进一步检查是否与流式请求冲突
        if request.parameters.is_some() {
            if let Some(ref parameters) = request.parameters {
                // 如果参数中指定了非流式处理，则返回错误提示使用非流式调用
                if parameters.incremental_output == Some(false) {
                    return Err(DashScopeError::InvalidArgument(
                        "When stream is false, use MultiModalGeneration::call".into(),
                    ));
                }
            }
        }

        // 确保请求参数配置为流式处理
        request.parameters = Some(
            ParametersBuilder::default()
                .incremental_output(true)
                .build()?,
        );

        // 发起流式请求并返回结果流
        Ok(self
            .client
            .post_stream("/services/aigc/multimodal-generation/generation", request)
            .await)
    }
}
