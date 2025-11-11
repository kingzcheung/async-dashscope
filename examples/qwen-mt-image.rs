#![allow(unused_imports)]
use async_dashscope::{
    Client,
    operation::{
        common::{Parameters, ParametersBuilder},
        image2image::{Image2imageParamBuilder, InputBuilder},
    },
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;

    let cargo_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let file_path = format!("{cargo_dir}/test_data/qwen-mt-image.webp");

    let request = Image2imageParamBuilder::default()
        .model("qwen-mt-image")
        .input(
            InputBuilder::default()
                .image_url(file_path)
                .source_lang("auto")
                .target_lang("en")
                .build()?,
        )
        .build()?;

    let client = Client::new();

    let response = client.image2image().call(request).await?;

    println!("response:: {:?}", response);

    let task = client.task().poll_task_status(&response.output.task_id,5,10).await?;

    println!("{:?}", task.output.image_url);

    Ok(())
}
