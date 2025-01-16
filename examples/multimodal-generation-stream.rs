use async_dashscope::{operation::multi_modal_conversation::{InputBuilder, MessageBuilder,  MultiModalConversationParamBuilder}, Client};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let request = MultiModalConversationParamBuilder::default()
        .model("qwen-vl-max")
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

    let response = client.multi_modal_conversation().call(request).await?;

    dbg!(response);

    Ok(())
}
