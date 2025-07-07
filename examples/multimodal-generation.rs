use async_dashscope::{
    operation::multi_modal_conversation::{
        Element, InputBuilder, MessageBuilder, MultiModalConversationParamBuilder
    }, Client
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    dotenvy::dotenv()?;


    let request = MultiModalConversationParamBuilder::default()
        .model("qwen-vl-max")
        .input(InputBuilder::default().messages(vec![
            MessageBuilder::default()
            .role("user")
            .contents(
            // [
            //      Element::Image("https://help-static-aliyun-doc.aliyuncs.com/file-manage-files/zh-CN/20241022/emyrja/dog_and_girl.jpeg".into()),
            //      Element::Text("这是什么?".into())
            // ];
                vec![
                    json!({"image": "https://help-static-aliyun-doc.aliyuncs.com/file-manage-files/zh-CN/20241022/emyrja/dog_and_girl.jpeg"}).try_into()?,
                    json!({"text": "这是什么?"}).try_into()?
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
