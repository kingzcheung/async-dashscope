use std::collections::HashMap;

use super::output::{GenerationOutput, ToolCall};

/// 将增量（incremental）流式输出合并为全量输出。
///
/// 对应官方 SDK 在 `incremental_output=false` 时的
/// `incremental_to_full` 行为：请求仍然使用增量模式，客户端逐块累积
/// `content`、`reasoning_content` 与 `tool_calls`。
#[derive(Debug, Default)]
pub(crate) struct IncrementalMergeState {
    choices: HashMap<i32, ChoiceState>,
    text: String,
}

#[derive(Debug, Default)]
struct ChoiceState {
    content: String,
    reasoning_content: String,
    tool_calls: Vec<ToolCall>,
}

impl IncrementalMergeState {
    pub(crate) fn merge(&mut self, mut output: GenerationOutput) -> GenerationOutput {
        if let Some(choices) = output.output.choices.as_mut() {
            for (idx, choice) in choices.iter_mut().enumerate() {
                let index = choice.index.unwrap_or(idx as i32);
                let state = self.choices.entry(index).or_default();

                if !choice.message.content.is_empty() {
                    state.content.push_str(&choice.message.content);
                }
                choice.message.content = state.content.clone();

                if let Some(reasoning) = &choice.message.reasoning_content {
                    if !reasoning.is_empty() {
                        state.reasoning_content.push_str(reasoning);
                    }
                }
                if !state.reasoning_content.is_empty() {
                    choice.message.reasoning_content = Some(state.reasoning_content.clone());
                }

                if let Some(tool_calls) = &choice.message.tool_calls {
                    for tool_call in tool_calls {
                        match state
                            .tool_calls
                            .iter_mut()
                            .find(|existing| existing.index == tool_call.index)
                        {
                            Some(existing) => {
                                if !tool_call.id.is_empty() {
                                    existing.id = tool_call.id.clone();
                                }
                                if !tool_call.type_.is_empty() {
                                    existing.type_ = tool_call.type_.clone();
                                }
                                if !tool_call.function.name.is_empty() {
                                    existing.function.name = tool_call.function.name.clone();
                                }
                                if let Some(arguments) = &tool_call.function.arguments {
                                    existing
                                        .function
                                        .arguments
                                        .get_or_insert_with(String::new)
                                        .push_str(arguments);
                                }
                            }
                            None => state.tool_calls.push(tool_call.clone()),
                        }
                    }
                    choice.message.tool_calls = Some(state.tool_calls.clone());
                }
            }
        } else if let Some(text) = output.output.text.as_mut() {
            self.text.push_str(text);
            *text = self.text.clone();
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation::generation::output::{Choices, Function, Message, Output};

    fn output_with_content(content: &str, reasoning: Option<&str>) -> GenerationOutput {
        GenerationOutput {
            output: Output {
                choices: Some(vec![Choices {
                    index: Some(0),
                    finish_reason: None,
                    message: Message {
                        content: content.to_string(),
                        role: "assistant".to_string(),
                        reasoning_content: reasoning.map(str::to_string),
                        function_call: None,
                        tool_calls: None,
                    },
                }]),
                text: None,
                finish_reason: None,
                search_info: None,
            },
            request_id: None,
            usage: None,
        }
    }

    fn content_of(output: &GenerationOutput) -> &str {
        &output.output.choices.as_ref().unwrap()[0].message.content
    }

    #[test]
    fn merges_incremental_content() {
        let mut state = IncrementalMergeState::default();

        let first = state.merge(output_with_content("你", None));
        assert_eq!(content_of(&first), "你");

        let second = state.merge(output_with_content("好", None));
        assert_eq!(content_of(&second), "你好");

        let third = state.merge(output_with_content("！", None));
        assert_eq!(content_of(&third), "你好！");
    }

    #[test]
    fn merges_reasoning_content() {
        let mut state = IncrementalMergeState::default();

        let first = state.merge(output_with_content("", Some("思考")));
        assert_eq!(
            first.output.choices.as_ref().unwrap()[0]
                .message
                .reasoning_content
                .as_deref(),
            Some("思考")
        );

        let second = state.merge(output_with_content("答案", Some("中")));
        assert_eq!(
            second.output.choices.as_ref().unwrap()[0]
                .message
                .reasoning_content
                .as_deref(),
            Some("思考中")
        );
        assert_eq!(content_of(&second), "答案");
    }

    #[test]
    fn merges_tool_call_arguments() {
        let mut state = IncrementalMergeState::default();

        let chunk = |index: i32, arguments: &str| GenerationOutput {
            output: Output {
                choices: Some(vec![Choices {
                    index: Some(0),
                    finish_reason: None,
                    message: Message {
                        content: String::new(),
                        role: "assistant".to_string(),
                        reasoning_content: None,
                        function_call: None,
                        tool_calls: Some(vec![ToolCall {
                            id: "call_1".to_string(),
                            type_: "function".to_string(),
                            index,
                            function: Function {
                                name: "get_weather".to_string(),
                                arguments: Some(arguments.to_string()),
                            },
                        }]),
                    },
                }]),
                text: None,
                finish_reason: None,
                search_info: None,
            },
            request_id: None,
            usage: None,
        };

        state.merge(chunk(0, "{\"city\":"));
        let merged = state.merge(chunk(0, "\"杭州\"}"));

        let tool_calls = merged.output.choices.as_ref().unwrap()[0]
            .message
            .tool_calls
            .as_ref()
            .unwrap();
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].function.name, "get_weather");
        assert_eq!(
            tool_calls[0].function.arguments.as_deref(),
            Some("{\"city\":\"杭州\"}")
        );
    }

    #[test]
    fn merges_text_output() {
        let mut state = IncrementalMergeState::default();

        let mut first = output_with_content("", None);
        first.output.choices = None;
        first.output.text = Some("Hello".to_string());
        let first = state.merge(first);
        assert_eq!(first.output.text.as_deref(), Some("Hello"));

        let mut second = output_with_content("", None);
        second.output.choices = None;
        second.output.text = Some(" World".to_string());
        let second = state.merge(second);
        assert_eq!(second.output.text.as_deref(), Some("Hello World"));
    }

    #[test]
    fn keeps_choices_separate_by_index() {
        let mut state = IncrementalMergeState::default();

        let make = |index: i32, content: &str| GenerationOutput {
            output: Output {
                choices: Some(vec![Choices {
                    index: Some(index),
                    finish_reason: None,
                    message: Message {
                        content: content.to_string(),
                        role: "assistant".to_string(),
                        reasoning_content: None,
                        function_call: None,
                        tool_calls: None,
                    },
                }]),
                text: None,
                finish_reason: None,
                search_info: None,
            },
            request_id: None,
            usage: None,
        };

        state.merge(make(0, "A"));
        state.merge(make(1, "X"));
        let merged = state.merge(make(1, "Y"));

        assert_eq!(
            &merged.output.choices.as_ref().unwrap()[0].message.content,
            "XY"
        );
        assert_eq!(merged.output.choices.as_ref().unwrap()[0].index, Some(1));

        let zero = state.merge(make(0, ""));
        assert_eq!(
            &zero.output.choices.as_ref().unwrap()[0].message.content,
            "A"
        );
    }
}
