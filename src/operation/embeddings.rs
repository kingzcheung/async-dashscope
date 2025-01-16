use crate::Client;
use crate::error::Result;
pub use param::*;
pub use output::*;


mod param;
mod output;

pub struct Embeddings<'a> {
    client: &'a Client,
}

impl<'a> Embeddings<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn call(&self, request: param::EmbeddingsParam) -> Result<output::EmbeddingsOutput> {
        self.client
            .post("/services/embeddings/text-embedding/text-embedding", request)
            .await
    }
}

