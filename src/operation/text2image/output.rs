use serde::{Deserialize, Serialize};

use crate::operation::common::TaskStatus;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Text2ImageOutput {
    pub request_id: String,
    pub output: TextTaskSubmitOutput,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextTaskSubmitOutput {
    pub task_status: TaskStatus,
    pub task_id: String,
}
