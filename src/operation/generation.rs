use _generation_input::GenerationInput;
use _generation_output::GenerationOutput;

use crate::{client::Client, error::DashScopeError};

pub(crate) mod _generation_input;
pub(crate) mod _generation_output;

pub struct Generation<'a>{
    client: &'a Client,
}

impl<'a> Generation<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn call(&self, request: GenerationInput) ->Result<GenerationOutput, DashScopeError> {
        if request.stream.is_some() && request.stream.unwrap() {
            return Err(DashScopeError::InvalidArgument(
                "When stream is true, use Chat::create_stream".into(),
            ));
        }
        self.client.post("/text-generation/generation", request).await
    }
}

