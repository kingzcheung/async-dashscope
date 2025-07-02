use async_dashscope::{
    operation::audio::{TextToSpeechInputBuilder, TextToSpeechParamBuilder}, Client
};
use tokio_stream::StreamExt as _;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    let request = TextToSpeechParamBuilder::default()
        .model("qwen-tts")
        .input(
            TextToSpeechInputBuilder::default()
                .text("那我来给大家推荐一款T恤，这款呢真的是超级好看，这个颜色呢很显气质，而且呢也是搭配的绝佳单品，大家可以闭眼入，真的是非常好看，对身材的包容性也很好，不管啥身材的宝宝呢，穿上去都是很好看的。推荐宝宝们下单哦。")
                .voice("Cherry")
                .build()?,
        )
        .stream(true)
        .build()?;

    let client = Client::new();

    let mut stream = client.audio().tts_stream(request).await?;
    
    while let Some(response) = stream.next().await {
        match response {
            Ok(go) => {
                // println!("{}",go.output.audio.data);
                println!("{:?}", go.output.audio.data);
                // 这是 pcm 数据，并不是 wav 数据
                go.output.audio.bytes()?;
                if go.is_finished() {
                    go.download("output.wav").await?;
                    break;
                }
            },
            Err(e) => eprintln!("{e}"),
        }
    }
    Ok(())
}
