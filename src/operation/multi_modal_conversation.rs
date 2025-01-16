pub use input::MultiModalConversationInput;
pub use output::MultiModalConversationOutput;
pub use output::MultiModalConversationOutputStream;

use crate::{error::DashScopeError, Client};
use crate::error::Result;

use super::common::ParametersBuilder;
pub mod input;
pub mod output;

pub struct MultiModalConversation<'a> {
    client: &'a Client,
}

impl<'a> MultiModalConversation<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn call(&self, request: MultiModalConversationInput) -> Result<MultiModalConversationOutput> {
        if request.stream == Some(true) {
            return Err(DashScopeError::InvalidArgument(
                "When stream is true, use MultiModalGeneration::call_stream".into(),
            ));
        }

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
            .post("/services/aigc/multimodal-generation/generation", request)
            .await
    }

    pub async fn call_stream(
        &self,
        mut request: MultiModalConversationInput,
    ) -> Result<MultiModalConversationOutputStream> {
        if request.stream != Some(true) {
            return Err(DashScopeError::InvalidArgument(
                "When stream is false, use MultiModalGeneration::call".into(),
            ));
        }

        if request.parameters.is_some() {
            if let Some(ref parameters) = request.parameters  {
                if parameters.incremental_output == Some(false) {
                    return Err(DashScopeError::InvalidArgument(
                        "When stream is false, use MultiModalGeneration::call".into(),
                    ));
                }
                
            }
        }

        request.parameters = Some(ParametersBuilder::default().incremental_output(true).build()?);

        Ok(self
            .client
            .post_stream("/services/aigc/multimodal-generation/generation", request)
            .await)
    }
}
