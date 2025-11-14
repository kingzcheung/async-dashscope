use crate::{Client, error::Result};
pub use output::*;
pub use param::*;

mod output;
mod param;

const TEXT2IMAGE_PATH: &str = "/services/aigc/text2image/image-synthesis";

pub struct Text2Image<'a> {
    client: &'a Client,
}

impl<'a> Text2Image<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn call(&self, request: Text2imageParam) -> Result<Text2ImageOutput> {
        // 检查参数
        // let validators = check_model_parameters(&request.model);
        // for valid in validators {
        //     valid.validate(&request)?;
        // }

        let mut headers = self.client.config().headers();
        headers.insert("X-DashScope-Async", "enable".parse().unwrap());

        // 发送POST请求到生成服务，并等待结果
        self.client
            .post_with_headers(TEXT2IMAGE_PATH, request, headers)
            .await
    }
}
