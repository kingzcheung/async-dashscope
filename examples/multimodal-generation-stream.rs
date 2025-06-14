use async_dashscope::{
    operation::multi_modal_conversation::{
        InputBuilder, MessageBuilder, MultiModalConversationParamBuilder,
    },
    Client,
};
use serde_json::json;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    let request = MultiModalConversationParamBuilder::default()
        .model("qwen-vl-max")
        .stream(true)
        .input(InputBuilder::default().messages(vec![
            MessageBuilder::default()
            .role("user")
            .contents(
                vec![
                    json!({"image": "https://help-static-aliyun-doc.aliyuncs.com/file-manage-files/zh-CN/20241022/emyrja/dog_and_girl.jpeg"}),
                    json!({"text": "这是什么?"})
                ]
            ).build()?
        ]).build()?
    )
        .build()?;

    let client = Client::new();

    let mut stream = client.multi_modal_conversation().call_stream(request).await?;

    while let Some(response) = stream.next().await {
        match response {
            Ok(r) => {
                println!("{:?}", r.output.choices[0].message.content);
            },
            Err(e) => println!("{}", e),
        }
    }

    Ok(())
}
