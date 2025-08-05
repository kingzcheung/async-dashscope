use async_dashscope::{
    Client,
    operation::embeddings::{
        EmbeddingsInputBuilder, EmbeddingsParamBuilder, EmbeddingsParametersBuilder,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    let client = Client::new();
    let input = EmbeddingsParamBuilder::default()
        .model("text-embedding-v2")
        .input(
            EmbeddingsInputBuilder::default()
                .texts(vec![
                    "风急天高猿啸哀".into(),
                    "渚清沙白鸟飞回".into(),
                    "无边落木萧萧下".into(),
                    "不尽长江滚滚来".into(),
                ])
                .build()?,
        )
        .parameters(
            EmbeddingsParametersBuilder::default()
                .dimension(1536u16)
                .build()?,
        )
        .build()?;

    let output = client.text_embeddings().call(input).await?;

    dbg!(output);

    Ok(())
}
