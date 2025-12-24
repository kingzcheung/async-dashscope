pub mod output;
pub mod param;
use crate::{Client, error::DashScopeError};

const FILE_PATH: &str = "files";

pub struct File<'a> {
    client: &'a Client,
}

impl<'a> File<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// 上传文件
    /// 
    /// # 参数
    /// 
    /// * `files` - 要上传的文件路径列表
    /// * `purpose` - 文件用途，如 "fine-tune"
    /// * `descriptions` - 文件描述列表（可选）
    /// 
    /// # 返回
    /// 
    /// 返回上传结果，包含文件信息
    pub async fn create(
        &self, 
        files: Vec<&str>, 
        purpose: &str, 
        descriptions: Option<Vec<&str>>
    ) -> Result<crate::operation::file::output::FileUploadOutput, DashScopeError> {
        use reqwest::multipart;
        use std::path::Path;

        // 将参数转换为可以在闭包中使用的类型
        let purpose = purpose.to_string();
        let file_paths: Vec<String> = files.iter().map(|s| s.to_string()).collect();
        let descriptions: Option<Vec<String>> = descriptions.map(|descs| descs.iter().map(|s| s.to_string()).collect());

        // 使用客户端的post_multipart方法发送请求，自动处理认证和重试
        self.client.post_multipart(FILE_PATH, move || {
            let mut form = multipart::Form::new()
                .text("purpose", purpose.clone());

            // 添加描述信息
            if let Some(descs) = &descriptions {
                for desc in descs {
                    form = form.text("descriptions", desc.clone());
                }
            };

            // 添加文件 - 每次调用闭包时重新读取文件（用于重试）
            let mut form_with_files = form;
            for file_path in &file_paths {
                let path = Path::new(file_path);
                let os_file_name = path.file_name()
                    .unwrap_or_else(|| {
                        std::panic::resume_unwind(Box::new(DashScopeError::UploadError(
                            format!("Invalid file path: {}", file_path)
                        )));
                        // 这里的代码不会被执行，因为上面的 panic 已经中断了执行
                    });
                
                let file_name = os_file_name.to_str()
                    .unwrap_or_else(|| {
                        std::panic::resume_unwind(Box::new(DashScopeError::UploadError(
                            format!("Invalid file name: {}", file_path)
                        )));
                        // 这里的代码不会被执行，因为上面的 panic 已经中断了执行
                    }).to_string();

                let file_data = std::fs::read(file_path)
                    .unwrap_or_else(|e| {
                        std::panic::resume_unwind(Box::new(DashScopeError::UploadError(
                            format!("Failed to read file {}: {}", file_path, e)
                        )));
                        // 这里的代码不会被执行，因为上面的 panic 已经中断了执行
                    });

                let part = multipart::Part::bytes(file_data)
                    .file_name(file_name);

                form_with_files = form_with_files.part("files", part);
            }

            form_with_files
        }).await
    }

    /// 查询文件信息
    pub async fn retrieve(
        &self,
        file_id: &str,
    ) -> Result<crate::operation::file::output::FileRetrieveOutput, DashScopeError> {
        // 构建路径
        let path = format!("files/{}", file_id);

        // 使用客户端的get_with_params方法发送请求，参数为空对象
        self.client.get_with_params(&path, &()).await
    }

    /// 查询文件列表
    pub async fn list(
        &self,
        page_no: Option<u64>,
        page_size: Option<u64>,
    ) -> Result<crate::operation::file::output::FileListOutput, DashScopeError> {
        use serde_json::json;

        // 验证参数
        let validated_page_no = page_no.unwrap_or(1);
        let validated_page_no = if validated_page_no < 1 { 1 } else { validated_page_no };

        let validated_page_size = page_size.unwrap_or(10);
        let validated_page_size = validated_page_size.clamp(1, 100);

        // 构建查询参数
        let params = json!({
            "page_no": validated_page_no,
            "page_size": validated_page_size,
        });

        // 使用客户端的get方法发送请求
        self.client.get_with_params("files", &params).await
    }

    /// 删除文件
    pub async fn delete(
        &self,
        file_id: &str,
    ) -> Result<crate::operation::file::output::FileDeleteOutput, DashScopeError> {
        // 构建路径
        let path = format!("files/{}", file_id);

        // 使用客户端的delete方法发送请求
        self.client.delete(&path).await
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigBuilder;

    #[tokio::test]
    async fn test_file_operations() {
        let _ = dotenvy::dotenv(); // 加载 .env 文件，如果存在的话
        let api_key = std::env::var("DASHSCOPE_API_KEY").expect("DASHSCOPE_API_KEY must be set");
        let config = ConfigBuilder::default()
            .api_key(api_key)
            .build()
            .unwrap();
        let client = Client::with_config(config);
        let file = File::new(&client);

        // 测试文件列表功能
        let result = file.list(Some(1), Some(10)).await;
        match result {
            Ok(list_output) => {
                println!("Retrieved {} files", list_output.data.files.len());
            }
            Err(e) => {
                eprintln!("Error listing files: {:?}", e);
            }
        }
    }
}
