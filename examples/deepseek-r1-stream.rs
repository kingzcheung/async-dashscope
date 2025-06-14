use async_dashscope::{
    operation::{
        common::ParametersBuilder,
        generation::{GenerationParamBuilder, InputBuilder, MessageBuilder},
    },
    Client,
};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    let request = GenerationParamBuilder::default()
        .model("deepseek-r1".to_string())
        .input(
            InputBuilder::default()
                .messages(vec![MessageBuilder::default()
                    .role("user")
                    .content("你是谁? 请用5种语言回答我。")
                    .build()
                    .unwrap()])
                .build()?,
        )
        .stream(true)
        .parameters(
            ParametersBuilder::default()
                .result_format("message")
                .incremental_output(true)
                .build()?,
        )
        .build()?;

    let client = Client::default();

    let mut stream = client.generation().call_stream(request).await?;
    // dbg!(&response);
    // 思考过程
    println!("思考过程:::");
    while let Some(response) = stream.next().await {
        match response {
            Ok(go) => go.output.choices.unwrap().iter().for_each(|c| {
                if let Some(reasoning_content) = &c.message.reasoning_content {
                    print!("{}", reasoning_content);
                } 
            }),
            Err(e) => eprintln!("{}", e),
        }
    }
    Ok(())
}
