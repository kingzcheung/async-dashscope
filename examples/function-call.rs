use async_dashscope::{
    operation::{
        common::{FunctionBuilder, FunctionCallBuilder, ParametersBuilder},
        generation::{GenerationParamBuilder, InputBuilder, MessageBuilder},
    },
    Client,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;

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
    Ok(())
}

#[allow(dead_code)]
fn get_current_time() -> String {
    "2025-06-05 16:00:00".to_string()
}
