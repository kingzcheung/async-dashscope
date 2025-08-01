use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::operation::common::{Parameters, StreamOptions};
use crate::operation::request::RequestTrait;

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct GenerationParam {
    #[builder(setter(into, strip_option))]
    pub model: String,

    pub input: Input,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub parameters: Option<Parameters>,

    #[builder(setter(into, strip_option))]
    #[builder(default=Some(false))]
    pub stream: Option<bool>,

    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub stream_options: Option<StreamOptions>,
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct Input {
    #[builder(setter(custom))]
    pub messages: Vec<Message>,
}
impl InputBuilder {
    pub fn messages(&mut self, value: Vec<Message>) -> &mut Self {
        self.messages = Some(value);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
#[derive(Default)]
pub enum Message {
    #[default]
    None,
    System(SystemMessage),
    User(UserMessage),
    Assistant(AssistantMessage),
    Tool(ToolMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MessageBuilder {
    pub role: String,
    pub content: String,
    pub partial: Option<bool>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Error)]
pub enum MessageBuilderError {
    #[error("Invalid role")]
    InvalidRole,
}

impl MessageBuilder {
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
            partial: Some(false),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn role(&mut self, value: impl Into<String>) -> &mut Self {
        self.role = value.into();
        self
    }
    pub fn system(&mut self) -> &mut Self {
        self.role("system")
    }

    pub fn user(&mut self) -> &mut Self {
        self.role("user")
    }
    pub fn assistant(&mut self) -> &mut Self {
        self.role("assistant")
    }
    pub fn tool(&mut self) -> &mut Self {
        self.role("tool")
    }

    pub fn content(&mut self, value: impl Into<String>) -> &mut Self {
        self.content = value.into();
        self
    }

    pub fn partial(&mut self, value: bool) -> &mut Self {
        self.partial = Some(value);
        self
    }

    pub fn tool_call_id(&mut self, value: impl Into<String>) -> &mut Self {
        self.tool_call_id = Some(value.into());
        self
    }

    pub fn tool_calls(&mut self, value: Vec<ToolCall>) -> &mut Self {
        self.tool_calls = Some(value);
        self
    }

    pub fn build(&self) -> Result<Message, MessageBuilderError> {
        match self.role.as_ref() {
            "system" => Ok(Message::System(SystemMessage {
                role: self.role.clone(),
                content: self.clone().content,
            })),
            "user" => Ok(Message::User(UserMessage {
                role: self.role.clone(),
                content: self.clone().content,
            })),
            "assistant" => Ok(Message::Assistant(AssistantMessage {
                role: self.role.clone(),
                content: self.content.clone(),
                partial: self.partial,
                tool_calls: self.tool_calls.clone(),
            })),
            "tool" => Ok(Message::Tool(ToolMessage {
                role: self.role.clone(),
                content: self.content.clone(),
                tool_call_id: self.tool_call_id.clone(),
            })),
            // 不可用的角色
            _ => Err(MessageBuilderError::InvalidRole),
        }
    }
}

/// 模型的目标或角色。如果设置系统消息，请放在messages列表的第一位。
#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct SystemMessage {
    #[builder(setter(into), default = "\"system\".to_string()")]
    pub role: String,
    #[builder(setter(into))]
    pub content: String,
}

impl From<SystemMessage> for Message {
    fn from(value: SystemMessage) -> Self {
        Self::System(value)
    }
}

/// 用户发送给模型的消息
#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct UserMessage {
    #[builder(setter(into), default = "\"user\".to_string()")]
    pub role: String,
    #[builder(setter(into))]
    pub content: String,
}

impl From<UserMessage> for Message {
    fn from(value: UserMessage) -> Self {
        Self::User(value)
    }
}

/// 模型对用户消息的回复
#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct AssistantMessage {
    #[builder(setter(into), default = "\"assistant\".to_string()")]
    pub role: String,
    #[builder(setter(into))]
    pub content: String,

    /// 是否开启Partial Mode，参考: [前缀续写](https://help.aliyun.com/zh/model-studio/partial-mode)
    #[builder(setter(into, strip_option))]
    #[builder(default=Some(false))]
    pub partial: Option<bool>,

    /// 在发起 Function Calling后，模型回复的要调用的工具和调用工具时需要的参数。包含一个或多个对象。由上一轮模型响应的tool_calls字段获得。
    #[builder(setter(into, strip_option))]
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl From<AssistantMessage> for Message {
    fn from(value: AssistantMessage) -> Self {
        Self::Assistant(value)
    }
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    /// 本次工具响应的ID。
    #[builder(setter(into))]
    pub id: String,
    /// 工具的类型，当前只支持function。
    #[builder(setter(into))]
    #[serde(rename = "type")]
    pub type_: String,

    /// 需要被调用的函数。
    pub function: Function,

    /// 工具信息在tool_calls列表中的索引。
    pub index: i32,
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct Function {
    /// 需要被调用的函数名。
    #[builder(setter(into))]
    pub name: String,
    /// 需要输入到工具中的参数，为JSON字符串。
    #[builder(setter(into))]
    pub arguments: String,
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct ToolMessage {
    #[builder(setter(into), default = "\"tool\".to_string()")]
    pub role: String,
    #[builder(setter(into))]
    pub content: String,
    #[builder(setter(into))]
    pub tool_call_id: Option<String>,
}

impl From<ToolMessage> for Message {
    fn from(value: ToolMessage) -> Self {
        Self::Tool(value)
    }
}

impl RequestTrait for GenerationParam {

    type P = Parameters;
    fn model(&self) -> &str {
        &self.model
    }

    fn parameters(&self) -> Option<&Self::P> {
        self.parameters.as_ref()
    }
}
