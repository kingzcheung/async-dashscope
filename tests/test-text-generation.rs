use async_dashscope::operation::{common::ParametersBuilder, generation::{GenerationParamBuilder, InputBuilder, MessageBuilder}};

pub mod common;

#[tokio::test]
async fn test_text_generation()->anyhow::Result<()> { 
    let client = common::init_client();

    let request = GenerationParamBuilder::default()
        .model("qwen-turbo".to_string())
        .input(
            InputBuilder::default()
                .messages(vec![MessageBuilder::default()
                    .role("user")
                    .content("你是谁")
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
    let response = client.generation().call(request).await?;

    assert!(response.output.choices.is_some());

    Ok(())
}