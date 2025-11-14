use serde::{Deserialize, Serialize};

use crate::operation::common::TaskStatus;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskOutput {
    pub task_id: String,
    pub task_status: TaskStatus,
    pub submit_time: String,
    pub scheduled_time: Option<String>,
    pub end_time: Option<String>,
    pub image_url: Option<String>,
    pub code: Option<String>,
    pub message: Option<String>,
    pub task_metrics: Option<TaskMetrics>,
    pub results: Option<Vec<Text2ImageResult>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Text2ImageResult {
    pub orig_prompt: String,
    pub actual_prompt: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskMetrics {
    #[serde(rename = "TOTAL")]
    pub total: u32,
    #[serde(rename = "SUCCEEDED")]
    pub succeeded: u32,
    #[serde(rename = "FAILED")]
    pub failed: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskResult {
    pub request_id: String,
    pub output: TaskOutput,
    pub usage: Option<ImageUsage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageUsage {
    pub image_count: u32,
}
