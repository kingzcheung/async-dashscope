use async_dashscope::{
    operation::{
        common::{ParametersBuilder, TranslationOptionsBuilder},
        generation::{GenerationParamBuilder, InputBuilder, MessageBuilder},
    },
    Client,
};
use tokio_stream::StreamExt as _;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
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
        .stream(true)
        .parameters(
            ParametersBuilder::default()
                // .incremental_output(true) # Qwen-MT模型暂时不支持增量式流式输出,所以此设置无效
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

    let mut stream = client.generation().call_stream(request).await?;
    while let Some(response) = stream.next().await {
        match response {
            Ok(go) => go.output.choices.unwrap().iter().for_each(|c| {
                println!("{}", c.message.content);
            }),
            Err(e) => eprintln!("{}", e),
        }
    }
    Ok(())
}
