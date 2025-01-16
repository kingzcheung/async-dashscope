use crate::error::Result;
use crate::{client::Client, error::DashScopeError};
pub use input::GenerationInput;
pub use output::{GenerationOutput, GenerationOutputStream};


pub mod input;
pub mod output;

pub struct Generation<'a> {
    client: &'a Client,
}

impl<'a> Generation<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn call(&self, request: GenerationInput) -> Result<GenerationOutput> {
        if request.stream.is_some() && request.stream.unwrap() {
            return Err(DashScopeError::InvalidArgument(
                "When stream is true, use Generation::call_stream".into(),
            ));
        }
        self.client
            .post("/text-generation/generation", request)
            .await
    }

    pub async fn call_stream(
        &self,
        mut request: GenerationInput,
    ) -> Result<GenerationOutputStream> {
        if request.stream.is_some() && !request.stream.unwrap() {
            return Err(DashScopeError::InvalidArgument(
                "When stream is false, use Generation::call".into(),
            ));
        }

        request.stream = Some(true);

        Ok(self
            .client
            .post_stream("/text-generation/generation", request)
            .await)
    }
}
