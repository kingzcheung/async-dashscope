#![allow(unused_imports)]
use async_dashscope::{
    Client,
    operation::{
        common::{Parameters, ParametersBuilder},
        text2image::{InputBuilder, Text2imageParamBuilder},
    },
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;

    let request = Text2imageParamBuilder::default()
        .model("wan2.2-t2i-flash")
        .input(
            InputBuilder::default()
                .prompt("雪地，白色小教堂，极光，冬日场景，柔和的光线。")
                .negative_prompt("人物")
                .build()?,
        )
        .build()?;

    let client = Client::new();

    let response = client.text2image().call(request).await?;

    println!("response:: {:?}", response);

    let task = client
        .task()
        .poll_task_status(&response.output.task_id, 30, 10)
        .await?;

    println!("{:?}", task.output.image_url);

    Ok(())
}
