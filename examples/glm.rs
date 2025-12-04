use async_dashscope::{
    Client,
    operation::{
        common::ParametersBuilder,
        generation::{GenerationParamBuilder, InputBuilder, MessageBuilder},
    },
};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    let request = GenerationParamBuilder::default()
        .model("glm-4.6".to_string())
        .input(
            InputBuilder::default()
                .messages(vec![
                    MessageBuilder::default()
                        .role("user")
                        .content("中国四大发明分别是什么？")
                        .build()
                        .unwrap(),
                ])
                .build()?,
        )
        .stream(true)
        .parameters(
            ParametersBuilder::default()
                .result_format("message")
                .incremental_output(true)
                .enable_thinking(true)
                .build()?,
        )
        .build()?;

    let client = Client::default();

    let mut stream = client.generation().call_stream(request).await?;
    // dbg!(&response);
    // 思考过程
    while let Some(response) = stream.next().await {
        match response {
            Ok(go) => go.output.choices.unwrap().iter().for_each(|c| {
                if let Some(reasoning_content) = &c.message.reasoning_content
                    && !reasoning_content.is_empty()
                {
                    print!("{reasoning_content}");
                } else if !c.message.content.is_empty() {
                    print!("{content}", content = c.message.content);
                }
            }),
            Err(e) => eprintln!("{e}"),
        }
    }
    Ok(())
}
