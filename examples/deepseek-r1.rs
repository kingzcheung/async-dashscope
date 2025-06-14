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
    // 思考过程
    println!("{:?}",&response.output.choices[0].message.reasoning_content);
    // 最终答案
    println!("{}", response.output.choices[0].message.content);
    Ok(())
}
