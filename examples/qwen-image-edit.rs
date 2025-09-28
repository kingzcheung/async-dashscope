#![allow(unused_imports)]
use async_dashscope::{
    Client,
    operation::{
        common::{Parameters, ParametersBuilder},
        multi_modal_conversation::{
            Element, InputBuilder, MessageBuilder, MultiModalConversationParamBuilder,
        },
    },
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;

    let cargo_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let file_path = format!("{cargo_dir}/test_data/dog_and_girl.jpeg");

    let request = MultiModalConversationParamBuilder::default()
        .model("qwen-image-edit")
        .input(
            InputBuilder::default()
                .messages(vec![
                    MessageBuilder::default()
                        .role("user")
                        .contents(
                            // vec![
                            //      Element::Image("https://help-static-aliyun-doc.aliyuncs.com/file-manage-files/zh-CN/20241022/emyrja/dog_and_girl.jpeg".into()),
                            //      Element::Text("这是什么?".into())
                            // ]
                            vec![
                                json!({"image": file_path}).try_into()?,
                                json!({"text": "将图中的人物改为站立姿势，弯腰握住狗的前爪?"})
                                    .try_into()?,
                            ],
                        )
                        .build()?,
                ])
                .build()?,
        )
        .parameters(
            ParametersBuilder::default()
                .watermark(true)
                .negative_prompt("低分辨率、错误、最差质量、低质量、残缺、多余的手指、比例不良")
                .seed(14748364)
                .build()?,
        )
        .build()?;

    let client = Client::new();

    let response = client.multi_modal_conversation().call(request).await?;

    for chunk in response.output.choices {
        println!("图片地址: {:?}", chunk.message.content[0].image);
    }

    Ok(())
}
