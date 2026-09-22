# async-dashscope

[![Crates.io MSRV](https://img.shields.io/crates/msrv/async-dashscope?style=flat-square)](https://github.com/kingzcheung/async-dashscope) [![Crates.io License](https://img.shields.io/crates/l/async-dashscope?style=flat-square)](https://github.com/kingzcheung/async-dashscope) [![Crates.io Version](https://img.shields.io/crates/v/async-dashscope?style=flat-square)](https://crates.io/crates/async-dashscope) [![Crates.io Downloads (recent)](https://img.shields.io/crates/dr/async-dashscope?style=flat-square)](https://crates.io/crates/async-dashscope) [![docs.rs](https://img.shields.io/docsrs/async-dashscope?style=flat-square&label=docs.rs&link=https%3A%2F%2Fdocs.rs%2Fasync-dashscope%2Flatest%2Fasync_dashscope%2F)](https://docs.rs/async-dashscope)

#### 项目简介

`async-dashscope` 是为通义千问百炼平台实现的（非官方）异步 SDK，支持文本生成、多模态生成以及 embedding 功能。通过该 SDK，开发者可以方便地调用通义千问百炼平台提供的各种 API，进行高效的异步操作。

#### 主要功能

- **文本生成**：支持多种文本生成任务，如文本补全、对话生成等。
- **多模态生成**：支持图像、音频等多种模态的数据生成任务。
- **Embedding**：提供文本 `embedding` 功能，用于将文本转换为向量表示，便于后续的语义分析和相似度计算。
- **DeepSeek**:  支持百炼平台的 `deepseek` 模型的调用
- **Kimi**: 支持 `Moonshot-Kimi-K2-Instruct`
- **GLM**: 支持 `glm-4.6`、`glm-4.5`、`glm-4.5-air`
- **深度思考**: 支持 `qwen`/`deepseek` 深度思考
- **工具调用**: 支持 `qwen` 系列的工具调用(deepseek 不支持)
- **音频合成**： 支持 `qwen-tts`、`qwen3-tts-flash` 音频合成
- 图像编辑： 支持 `qwen-image-edit`,见 [qwen-image-edit](docs/qwen-image-edit.md)
- **支持 `websocket` 调用**： 支持 `CosyVoice`、`Fun-ASR` 等 tts 或者 asr 的 `websocket` 调用。
- **结构化输出**

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

如果你需要使用 `websocket` 相关功能，请添加 `websocket` feature ：

```toml
[dependencies]
async-dashscope = { version = "*", features = ["websocket"] }
```

或者

```bash
cargo add async-dashscope --features websocket
```

#### 使用示例

> 更多的示例请看 [examples](./examples)

api_key 通过环境变量传入：

```shell
export DASHSCOPE_API_KEY=xxxxxxxxxxxxxxxxxxxxxxxx
```

或者

```rust

let client = Client::new().with_api_key(std::env::var("DASHSCOPE_API_KEY").unwrap());

```

##### 归属业务空间（Workspace）

API Key 可以归属到某个业务空间（子业务空间），调用时需要指定业务空间 ID。支持三种方式：

```shell
# 方式一：环境变量，未显式指定 workspace 时作为默认值
export DASHSCOPE_WORKSPACE_ID=ws_xxxxxxxx
```

```rust
// 方式二：Config
use async_dashscope::config::ConfigBuilder;

let config = ConfigBuilder::default()
    .api_key("sk-xxxxxxxx")
    .workspace("ws_xxxxxxxx")
    .build()?;
let client = async_dashscope::Client::with_config(config);

// 方式三：Client 便捷方法
let client = async_dashscope::Client::new()
    .with_api_key("sk-xxxxxxxx".to_string())
    .with_workspace("ws_xxxxxxxx".to_string());
```

指定后所有请求会自动携带 `X-DashScope-WorkSpace` 请求头。对于非北京地域的 MaaS 域名，可以直接设置 `DASHSCOPE_API_REGION` 自动推导，也可以在 `api_base` / `websocket_base` 中使用 `{workspace_id}` 占位符，例如：

```rust
let config = ConfigBuilder::default()
    .api_key("sk-xxxxxxxx")
    .workspace("ws_xxxxxxxx")
    .api_base("https://{workspace_id}.ap-southeast-1.maas.aliyuncs.com/api/v1")
    .websocket_base("wss://{workspace_id}.ap-southeast-1.maas.aliyuncs.com/api-ws/v1/inference")
    .build()?;
```

占位符存在但未配置（或非法）workspace 时，内部请求会直接返回错误。

##### 环境变量

| 环境变量 | 说明 |
| --- | --- |
| `DASHSCOPE_API_KEY` | API Key |
| `DASHSCOPE_API_KEY_FILE_PATH` | API Key 文件路径，未设置时回退到 `~/.dashscope/api_key` |
| `DASHSCOPE_WORKSPACE_ID` | 业务空间 ID |
| `DASHSCOPE_API_REGION` | 地域（如 `ap-southeast-1`），非北京地域自动使用 `{workspace_id}.{region}.maas.aliyuncs.com` 域名 |
| `DASHSCOPE_API_VERSION` | API 版本，默认 `v1` |
| `DASHSCOPE_HTTP_BASE_URL` | HTTP 接入地址，优先级高于地域推导，默认 `https://dashscope.aliyuncs.com/api/v1` |
| `DASHSCOPE_WEBSOCKET_BASE_URL` | WebSocket 接入地址，优先级高于地域推导，默认 `wss://dashscope.aliyuncs.com/api-ws/v1/inference` |
| `DASHSCOPE_DISABLE_SDK_HEADERS` | 设置任意值可禁用 `user-agent`、`x-dashscope-sdk-client` 等标识请求头 |

> HTTP 请求默认读超时 300 秒（与官方 SDK 一致）；`generation` 接口的 `incremental_output(false)` 会自动在客户端合并为全量输出；`GenerationParamBuilder::plugins(...)` 通过 `X-DashScope-Plugin` 请求头传递。

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
