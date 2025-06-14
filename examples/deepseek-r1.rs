use async_dashscope::{
    operation::{
        common::ParametersBuilder,
        generation::{GenerationParamBuilder, InputBuilder, MessageBuilder},
    },
    Client,
};

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
        .parameters(
            ParametersBuilder::default()
                .result_format("message")
                .build()?,
        )
        .build()?;

    let client = Client::default();

    let response = client.generation().call(request).await?;
    // dbg!(&response);

    if let Some(choices) = response.output.choices {
        for choice in choices {
            // 思考过程
            println!(
                "思考过程：{}",
                choice.message.reasoning_content.unwrap_or_default()
            );
            // 最终答案
            println!("最终答案: {}", choice.message.content);
        }
    }

    Ok(())
}
