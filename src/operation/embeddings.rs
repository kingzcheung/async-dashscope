use crate::error::Result;
use crate::Client;
pub use output::*;
pub use param::*;

mod output;
mod param;

pub struct Embeddings<'a> {
    client: &'a Client,
}

impl<'a> Embeddings<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// 异步调用文本嵌入服务
    ///
    /// 该函数通过POST请求向指定的服务端点发送文本嵌入请求，并返回处理结果
    /// 主要用途是将输入的文本数据转换为高维向量表示，以便于后续的自然语言处理任务使用
    ///
    /// # 参数
    ///
    /// * `request` - 包含文本嵌入请求所有必要信息的结构体，包括但不限于文本内容和嵌入模型的选择
    ///
    /// # 返回值
    ///
    /// 返回一个结果类型，包含文本嵌入操作的成功与否
    /// 如果操作成功，返回一个包含嵌入向量和其他相关信息的结构体
    /// 如果操作失败，返回一个错误类型，便于错误处理和调试
    pub async fn call(&self, request: param::EmbeddingsParam) -> Result<output::EmbeddingsOutput> {
        // 发送POST请求到指定的服务端点，并传递请求参数
        // 该行代码是异步执行的，允许在等待网络操作时继续执行其他任务，提高程序效率
        self.client
            .post(
                "/services/embeddings/text-embedding/text-embedding",
                request,
            )
            .await
    }
}
