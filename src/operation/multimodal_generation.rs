pub use input::MultiModalGenerationInput;
pub use output::MultiModalGenerationOutput;

use crate::{error::DashScopeError, Client};
use crate::error::Result;
pub mod input;
pub mod output;

pub struct MultiModalGeneration<'a> {
    client: &'a Client,
}

impl<'a> MultiModalGeneration<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn call(&self, request: MultiModalGenerationInput) -> Result<MultiModalGenerationOutput> {
        if request.parameters.is_some() {
            if let Some(ref parameters) = request.parameters  {
                if parameters.incremental_output == Some(true) {
                    return Err(DashScopeError::InvalidArgument(
                        "When stream is true, use MultiModalGeneration::call_stream".into(),
                    ));
                }
                
            }
        }
        self.client
            .post("/multimodal-generation/generation", request)
            .await
    }
}
