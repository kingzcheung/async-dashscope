//! `async-dashscope` 是为通义千问百炼平台实现的（非官方）异步 SDK，支持文本生成、多模态生成以及 embedding 功能。
//! 通过该 SDK，开发者可以方便地调用通义千问百炼平台提供的各种 API，进行高效的异步操作。
//!
//! ### 安装
//! 可以通过 Cargo 来安装 `async-dashscope`：
//！
//! ### 文本生成
//!
//! ```bash
//! cargo add async-dashscope
//! ```
//! ```rust
//！ use async_dashscope::{
//！     operation::{common::{ParametersBuilder, TranslationOptionsBuilder}, generation::{ GenerationParamBuilder, InputBuilder, MessageBuilder}}, Client
//！ };

//！ #[tokio::main]
//！ async fn main() -> Result<(), Box<dyn std::error::Error>> {
//！     let request = GenerationParamBuilder::default()
//！         .model("qwen-mt-turbo".to_string())
//！         .input(
//！             InputBuilder::default()
//！                 .messages(vec![MessageBuilder::default()
//！                     .role("user")
//！                     .content("我看到这个视频后没有笑")
//！                     .build()
//！                     .unwrap()])
//！                 .build()?,
//！         )
//！         .parameters(
//！             ParametersBuilder::default()
//！                 .translation_options(
//！                     TranslationOptionsBuilder::default()
//！                         .source_lang("Chinese")
//！                         .target_lang("English")
//！                         .build()?,
//！                 )
//！                 .build()?,
//！         )
//！         .build()?;

//！     let client = Client::new();

//！     let response = client.generation().call(request).await?;
//！     dbg!(response);
//！     Ok(())
//！ }

//！ ```

mod client;
pub mod config;
pub mod error;
pub mod operation;

pub use client::Client;
