use serde::{Deserialize, Serialize};

use crate::operation::common::TaskStatus;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Image2ImageOutput {
    pub request_id: String,
    pub output: ImageTaskSubmitOutput,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageTaskSubmitOutput {
    pub task_status: TaskStatus,
    pub task_id: String,
}
