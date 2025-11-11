use crate::{Client, error::Result};
pub use output::*;
pub use param::*;
use secrecy::ExposeSecret;

mod output;
mod param;

const IMAGE2IMAGE_PATH: &str = "/services/aigc/image2image/image-synthesis";

pub struct Image2Image<'a> {
    client: &'a Client,
}

impl<'a> Image2Image<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// 调用图像到图像转换服务
    ///
    /// 此方法会将输入图像上传到OSS，然后发送异步请求到DashScope的图像转换服务。
    ///
    /// # 参数
    /// * `request` - 图像转换请求参数，包含输入图像和转换参数
    ///
    /// # 返回值
    /// 返回 `Result<Image2ImageOutput>`，包含任务ID和初始状态信息
    ///
    /// # 错误
    /// 可能返回以下错误：
    /// - 文件上传失败
    /// - API请求失败
    /// - 参数验证失败
    ///
    /// # 注意事项
    /// - 此方法会启用异步模式（X-DashScope-Async头）
    /// - 上传的文件会自动清理，无需手动处理
    pub async fn call(&self, request: Image2imageParam) -> Result<Image2ImageOutput> {
        // 检查参数
        // let validators = check_model_parameters(&request.model);
        // for valid in validators {
        //     valid.validate(&request)?;
        // }
        let request = request
            .upload_file_to_oss(self.client.config().api_key().expose_secret())
            .await?;

        let mut headers = self.client.config().headers();
        headers.insert("X-DashScope-Async", "enable".parse().unwrap());

        // 发送POST请求到生成服务，并等待结果
        self.client
            .post_with_headers(IMAGE2IMAGE_PATH, request, headers)
            .await
    }
}
