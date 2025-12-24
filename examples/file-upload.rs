use async_dashscope::{Client, operation::file::FilePurpose::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量加载API密钥
     dotenvy::dotenv()?;

    let client = Client::new();

    // 上传文件示例
    let result = client.file()
        .create(
            vec!["./examples/fine-tune.jsonl"],  // 文件路径列表
            Batch,                 // 用途
            Some(vec!["A test file"])    // 描述（可选）
        )
        .await;

    match result {
        Ok(file_info) => {
            println!("File uploaded successfully!");
            println!("Request ID: {}", file_info.request_id);
            for file in &file_info.data.uploaded_files {
                println!("File ID: {}", file.file_id);
                println!("File Name: {}", file.name);
                println!("---");
            }
            
            if !file_info.data.failed_uploads.is_empty() {
                println!("Some files failed to upload:");
                for file in &file_info.data.failed_uploads {
                    println!("File Name: {}, Error: {} - {}", file.name, file.code, file.message);
                    println!("---");
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to upload file: {}", e);
        }
    }

    Ok(())
}