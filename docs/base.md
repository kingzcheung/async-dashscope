`async-dashscope` 是为通义千问百炼平台实现的（非官方）异步 SDK，支持文本生成、多模态生成以及 embedding 功能。
通过该 SDK，开发者可以方便地调用通义千问百炼平台提供的各种 API，进行高效的异步操作。

### 安装

可以通过 Cargo 来安装 `async-dashscope`：

```bash
cargo add async-dashscope
```

### 文本生成

```rust
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

    let client = Client::default();

    let response = client.generation().call(request).await?;
    dbg!(response);
```

### 流式生成

```rust
let request = GenerationParamBuilder::default()
        .model("qwen-turbo".to_string())
        .input(
            InputBuilder::default()
                .messages(vec![MessageBuilder::default()
                    .role("user")
                    .content("qwen 大模型系统是谁开发的?")
                    .build()
                    .unwrap()])
                .build()?,
        )
        .stream(true)
        .parameters(
            ParametersBuilder::default()
                .result_format("message")
                .incremental_output(true)
                .build()?,
        )
        .build()?;

    let client = Client::new();

    let mut stream = client.generation().call_stream(request).await?;
    while let Some(response) = stream.next().await {
        match response {
            Ok(go) => go.output.choices.unwrap().iter().for_each(|c| {
                print!("{}", c.message.content);
            }),
            Err(e) => eprintln!("{}", e),
        }
    }
```

### 多模态生成

```rust
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

```

### 文本翻译

```rust
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
```
### 推理模型(qwen and deepseek r1)
```rust
   let request = GenerationParamBuilder::default()
        .model("deepseek-r1".to_string())
        .input(
            InputBuilder::default()
                .messages(vec![MessageBuilder::default()
                    .role("user")
                    .content("你是谁? 请用5种语言回答我。")
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

    let client = Client::default();

    let response = client.generation().call(request).await?;
    // dbg!(&response);

    if let Some(choices) = response.output.choices {
        for choice in choices {
            // 思考过程
            println!(
                "思考过程：{}",
                choice.message.reasoning_content.unwrap_or_default()
            );
            // 最终答案
            println!("最终答案: {}", choice.message.content);
        }
    }
```

### 函数调用

先实现一个函数:

```rust

fn get_current_time() -> String {
    "2025-06-05 16:00:00".to_string()
}
```

然后在函数调用中调用这个函数:

```rust
let mut messages = vec![MessageBuilder::default()
        .role("user")
        .content("现在是什么时间？")
        .build()
        .unwrap()];

    // add function call
    let request = GenerationParamBuilder::default()
        .model("qwen-turbo".to_string())
        .input(InputBuilder::default().messages(messages.clone()).build()?)
        .parameters(
            ParametersBuilder::default()
                .functions([FunctionCallBuilder::default()
                    .typ("function")
                    .function(
                        FunctionBuilder::default()
                            .name("get_current_time")
                            .description("return the current time")
                            .build()?,
                    )
                    .build()?])
                // or call .tools(value)
                .result_format("message")
                .parallel_tool_calls(true)
                .build()?,
        )
        .build()?;

    let client = Client::default();

    let response = client.generation().call(request).await?;
    // dbg!(response);
    let response_message = response.output.choices.unwrap().first().unwrap().message.clone();
    // get  function call arguments
    if let Some(func_calls) = response_message.tool_calls {
        for call in &func_calls {
            if call.function.name == "get_current_time" {
                let func_response = get_current_time();
                messages.push(
                    MessageBuilder::default()
                        .role("user")
                        .content(func_response)
                        .build()?,
                );
                break;
            }
        }

        // 结合函数调用的结果,重新请求
        let request = GenerationParamBuilder::default()
            .model("qwen-turbo".to_string())
            .input(InputBuilder::default().messages(messages.clone()).build()?)
            .build()?;

        let response = client.generation().call(request).await?;

        // 返回最终总结结果
        dbg!(&response.output.text);
        
    }
```

### 音频合成

音频合成调用的是 `qwen-tts` 模型。

合成音频格式只支持两种：

- `wav`
- 流式输出 Base64 编码的 pcm

> 如果您需要解码流式输出 Base64 编码的 pcm，需要添加`wav-decoder` 特性:

```toml
async-dashscope = { version = "*", features = ["wav-decoder"] }
```

```rust

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
    
    let mut i = 0;
    while let Some(response) = stream.next().await {
        match response {
            Ok(go) => {
                // println!("{}",go.output.audio.data);
                println!("{:?}", go.output.audio.data);
                // 这是 pcm 数据，并不是 wav 数据
                // go.output.audio.bytes()?;
                // 这是 wav 数据
                let data = go.output.audio.to_wav(16000, 1, 16)?;
                std::fs::write(format!("{i}.wav"), data)?;
                i += 1;
                if go.is_finished() {
                    go.download("output.wav").await?;
                    break;
                }
            },
            Err(e) => eprintln!("{e}"),
        }
    }
```