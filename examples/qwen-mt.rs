use async_dashscope::{
    operation::{common::{ParametersBuilder, TranslationOptionsBuilder}, generation::{ GenerationParamBuilder, InputBuilder, MessageBuilder}}, Client
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let request = GenerationParamBuilder::default()
        .model("qwen-mt-turbo".to_string())
        .input(
            InputBuilder::default()
                .messages(vec![MessageBuilder::default()
                    .role("user")
                    .content("我看到这个视频后没有笑")
                    .build()
                    .unwrap()])
                .build()?,
        )
        .parameters(
            ParametersBuilder::default()
                .translation_options(
                    TranslationOptionsBuilder::default()
                        .source_lang("Chinese")
                        .target_lang("English")
                        .build()?,
                )
                .build()?,
        )
        .build()?;

    let client = Client::new();

    let response = client.generation().call(request).await?;
    dbg!(response);
    Ok(())
}
