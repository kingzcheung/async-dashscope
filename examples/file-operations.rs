use async_dashscope::{Client, operation::file::FilePurpose::Batch};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量加载API密钥
    let _ = dotenvy::dotenv(); // 可选加载环境变量，不失败如果文件不存在
    let api_key = std::env::var("DASHSCOPE_API_KEY").expect("DASHSCOPE_API_KEY must be set");

    let client = Client::new().with_api_key(api_key);
    let file = client.file();

    // 上传文件示例
    println!("Uploading file...");
    let upload_result = file
        .create(
            vec!["./examples/test.txt"],  // 文件路径列表
            Batch,                 // 用途
            Some(vec!["A test file"])    // 描述（可选）
        )
        .await;

    match upload_result {
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

    // 列出文件示例
    println!("\nListing files...");
    let list_result = file.list(Some(1), Some(10)).await;
    match list_result {
        Ok(list_output) => {
            println!("Retrieved {} files", list_output.data.files.len());
            for file in &list_output.data.files {
                println!("File ID: {}", file.file_id);
                println!("File Name: {}", file.name);
                println!("Size: {} bytes", file.size);
                println!("Created: {:?}", file.gmt_create);
                println!("---");
            }
        }
        Err(e) => {
            eprintln!("Failed to list files: {}", e);
        }
    }

    // 检查是否有文件可以查询和删除
    let list_result = file.list(Some(1), Some(1)).await;
    if let Ok(list_output) = list_result {
        if let Some(first_file) = list_output.data.files.first() {
            // 查询单个文件信息
            println!("Retrieving file info for: {}", first_file.file_id);
            match file.retrieve(&first_file.file_id).await {
                Ok(file_info) => {
                    println!("File details:");
                    println!("  File ID: {}", file_info.data.file_id);
                    println!("  Name: {}", file_info.data.name);
                    println!("  Size: {} bytes", file_info.data.size);
                    println!("  Created: {:?}", file_info.data.gmt_create);
                }
                Err(e) => {
                    eprintln!("Failed to retrieve file info: {}", e);
                }
            }

            // 删除文件（注意：实际使用时请谨慎操作）
            println!("Deleting file: {}", first_file.file_id);
            match file.delete(&first_file.file_id).await {
                Ok(delete_result) => {
                    println!("File deleted successfully: {}", delete_result.request_id);
                }
                Err(e) => {
                    eprintln!("Failed to delete file: {}", e);
                }
            }
        }
    }

    Ok(())
}