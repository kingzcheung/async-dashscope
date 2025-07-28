use crate::{client::Client, error::DashScopeError};
use crate::{error::Result, operation::validate::check_model_parameters};
pub use output::*;
pub use param::{
    AssistantMessageBuilder, GenerationParam, GenerationParamBuilder, InputBuilder, MessageBuilder,
    SystemMessageBuilder, ToolMessageBuilder, UserMessageBuilder,
};

mod output;
mod param;

const GENERATION_PATH: &str = "/services/aigc/text-generation/generation";

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
        let c = check_model_parameters(&request.model);
        c.validate(&request)?;

        // 发送POST请求到生成服务，并等待结果
        self.client.post(GENERATION_PATH, request).await
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
        let c = check_model_parameters(&request.model);
        c.validate(&request)?;

        // 通过客户端发起 POST 请求，使用修改后的 `request` 对象，并等待异步响应
        self.client.post_stream(GENERATION_PATH, request).await
    }
}
