use async_dashscope::{
    operation::{
        common::ParametersBuilder,
        generation::{GenerationParamBuilder, InputBuilder, MessageBuilder, UserMessageBuilder},
    },
    Client,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    // let s = UserMessageBuilder::default().content("请将这句话翻译成英文：你好，世界！").build()?.into();
    let request = GenerationParamBuilder::default()
        .model("qwen-turbo".to_string())
        .input(
            InputBuilder::default()
                .messages(vec![
                    MessageBuilder::default().role("system").content("你是一个出色的翻译专家").build()?,
                    // MessageBuilder::default().user().content("请将这句话翻译成英文：你好，世界！").build()?, 
                    // 和上面等效
                    UserMessageBuilder::default().content("请将这句话翻译成英文：你好，世界！").build()?.into(),
                ])
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
    dbg!(response);
    Ok(())
}
