use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Content {
	#[serde(rename = "text")]
	pub text: Option<String>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
	#[serde(rename = "content")]
	pub content: Vec<Content>,

	#[serde(rename = "role")]
	pub role: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Choices {
	#[serde(rename = "finish_reason")]
	pub finish_reason: Option<String>,

	#[serde(rename = "message")]
	pub message: Message,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Output {
	#[serde(rename = "choices")]
	pub choices: Vec<Choices>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Usage {
	#[serde(rename = "image_tokens")]
	pub image_tokens: Option<i32>,

	#[serde(rename = "input_tokens")]
	pub input_tokens: Option<i32>,

	#[serde(rename = "output_tokens")]
	pub output_tokens: Option<i32>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MultiModalGenerationOutput {
	#[serde(rename = "output")]
	pub output: Output,

	#[serde(rename = "request_id")]
	pub request_id: String,

	#[serde(rename = "usage")]
	pub usage: Option<Usage>,
}
