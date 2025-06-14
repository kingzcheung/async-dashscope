use async_dashscope::{
    operation::{
        common::{ParametersBuilder, ResponseFormatBuilder},
        generation::{GenerationParamBuilder, InputBuilder, MessageBuilder},
    },
    Client,
};
use serde::Deserialize;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    let request = GenerationParamBuilder::default()
        .model("qwen-turbo".to_string())
        .input(
            InputBuilder::default()
                .messages(vec![MessageBuilder::default()
                    .role("system")
                    .content("你需要提取出name（名字，为string类型）、age（年龄，为string类型）与email（邮箱，为string类型），请输出JSON 字符串，不要输出其它无关内容。\n示例：\nQ：我叫张三，今年25岁，邮箱是zhangsan@example.com\nA：{\"name\":\"张三\",\"age\":\"25岁\",\"email\":\"zhangsan@example.com\"}\nQ：我叫李四，今年30岁，我的邮箱是lisi@example.com\nA：{\"name\":\"李四\",\"age\":\"30岁\",\"email\":\"lisi@example.com\"}\nQ：我叫王五，我的邮箱是wangwu@example.com，今年40岁\nA：{\"name\":\"王五\",\"age\":\"40岁\",\"email\":\"wangwu@example.com\"")
                    .build()?,
                    MessageBuilder::default()
                    .role("user").content("大家好，我叫刘五，今年34岁，邮箱是liuwu@example.com").build()?])
                .build()?,
        )
        .parameters(
            ParametersBuilder::default()
                .result_format("message")
                .response_format(ResponseFormatBuilder::default().type_("json_object")
            .build()?)
                .build()?,
        )
        .build()?;

    let client = Client::default();

    let response = client.generation().call(request).await?;
    if let Some(choices) =  response.output.choices {
        
        for choice in choices {

            #[derive(Deserialize,Debug,Clone)]
            struct Person {
                pub name: String,
                pub age: String,
                pub email: String,
            }

            let person: Person = serde_json::from_str(&choice.message.content)?;

            println!("{:?}", person);

            assert_eq!(person.name, "刘五");
            assert_eq!(person.age, "34岁");
            assert_eq!(person.email, "liuwu@example.com");
        }
    }
    Ok(())
}
