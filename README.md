# async-dashscope

[![Crates.io MSRV](https://img.shields.io/crates/msrv/async-dashscope?style=flat-square)](https://github.com/kingzcheung/async-dashscope) [![Crates.io License](https://img.shields.io/crates/l/async-dashscope?style=flat-square)](https://github.com/kingzcheung/async-dashscope) [![Crates.io Version](https://img.shields.io/crates/v/async-dashscope?style=flat-square)](https://crates.io/crates/async-dashscope) [![Crates.io Downloads (recent)](https://img.shields.io/crates/dr/async-dashscope?style=flat-square)](https://crates.io/crates/async-dashscope) [![docs.rs](https://img.shields.io/docsrs/async-dashscope?style=flat-square&label=docs.rs&link=https%3A%2F%2Fdocs.rs%2Fasync-dashscope%2Flatest%2Fasync_dashscope%2F)](https://docs.rs/async-dashscope)

#### 项目简介

`async-dashscope` 是为通义千问百炼平台实现的（非官方）异步 SDK，支持文本生成、多模态生成以及 embedding 功能。通过该 SDK，开发者可以方便地调用通义千问百炼平台提供的各种 API，进行高效的异步操作。

#### 主要功能

- **文本生成**：支持多种文本生成任务，如文本补全、对话生成等。
- **多模态生成**：支持图像、音频等多种模态的数据生成任务。
- **Embedding**：提供文本 embedding 功能，用于将文本转换为向量表示，便于后续的语义分析和相似度计算。
- **DeepSeek**:  支持百炼平台的 deepseek 模型的调用
- **深度思考**: 支持 qwen/deepseek 深度思考
- **工具调用**: 支持qwen 系列的工具调用(deepseek 不支持)

#### 安装

可以通过 Cargo 来安装 `async-dashscope`：

```bash
cargo add async-dashscope
```

或者在 `Cargo.toml` 文件中添加依赖：

```toml
[dependencies]
async-dashscope = "*" 
```

#### 使用示例

api_key 通过环境变量传入：

```shell
export DASHSCOPE_API_KEY=xxxxxxxxxxxxxxxxxxxxxxxx
```

或者

```rust

let client = Client::new().with_api_key(std::env::var("DASHSCOPE_API_KEY").unwrap());

```

##### 文本生成示例

```rust
use async_dashscope::{
    operation::{common::{ParametersBuilder, TranslationOptionsBuilder}, generation::{ GenerationParamBuilder, InputBuilder, MessageBuilder}}, Client
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    Ok(())
}

```

##### 多模态生成示例

```rust
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
```

##### Embedding 示例

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let input = EmbeddingsParamBuilder::default()
        .model("text-embedding-v3")
        .input(
            EmbeddingsInputBuilder::default()
                .texts(vec![
                    "风急天高猿啸哀".into(),
                    "渚清沙白鸟飞回".into(), 
                    "无边落木萧萧下".into(), 
                    "不尽长江滚滚来".into()
                ])
                .build()?,
        )
        .parameters(
            EmbeddingsParametersBuilder::default()
                .dimension(1024)
                .build()?,
        )
        .build()?;
    let output = client.text_embeddings().call(input).await?;

    dbg!(output);

    Ok(())
}
```

#### 贡献指南

欢迎贡献代码！如果你有任何改进建议或发现 bug，请提交 issue 或 pull request。我们非常感谢你的帮助！

#### 许可证

本项目采用 MIT 许可证，详情请参见 [LICENSE](LICENSE-MIT) 文件。
