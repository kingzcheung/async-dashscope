use serde::{Deserialize, Serialize};
use crate::error::Result;
use crate::Client;

const CUSTOMIZATION_PATH: &str = "/services/audio/asr/customization";


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VocabularyParam {
    model: String,
    input: VocabularyInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VocabularyInput {
    action: String,
    target_model: String,
    prefix: String,
    vocabulary: Vec<VocabularyDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VocabularyItem {
    pub gmt_create: String,
    pub gmt_modified: String,
    pub status: String,
    pub vocabulary_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Output {
    pub vocabulary_list: Vec<VocabularyItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Usage {
    pub count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateVocabularyResponse {
    pub output:  CreateVocabularyOutput,
    pub usage: Usage,
    pub request_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateVocabularyOutput {
    pub vocabulary_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VocabularyDetail {
    pub text: String,
    pub weight: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_lang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<String>,
}

impl From<(&str,i32)> for VocabularyDetail {
    fn from(value: (&str,i32)) -> Self {
        let (text,weight) = value;
        Self {
            text: text.to_string(),
            weight,
            lang: None,
            target_lang: None,
            translation: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QueryVocabularyOutput {
    pub gmt_create: String,
    pub gmt_modified: String,
    pub status: String,
    pub target_model: String,
    pub vocabulary: Vec<VocabularyDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QueryVocabularyResponse {
    pub output: QueryVocabularyOutput,
    pub usage: Usage,
    pub request_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListVocabulariesResponse {
    pub output: ListVocabulariesOutput,
    pub usage: Usage,
    pub request_id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListVocabulariesOutput {
    pub vocabulary_list: Vec<VocabularyItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateVocabularyOutput {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateVocabularyResponse {
    pub output: UpdateVocabularyOutput,
    pub usage: Usage,
    pub request_id: String,
}

pub type DeleteVocabularyResponse = UpdateVocabularyResponse;

pub struct Customization {
    client: Client,
}

impl Customization {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn create_vocabulary(&self,target_model: &str,prefix: &str,vocabulary:&[VocabularyDetail]) ->Result<CreateVocabularyResponse> {
        let param = VocabularyParam {
            model: "speech-biasing".to_string(),
            input: VocabularyInput {
                action: "create_vocabulary".to_string(),
                target_model: target_model.to_string(),
                prefix: prefix.to_string(),
                vocabulary: vocabulary.to_vec(),
            },
        };
        let resp: CreateVocabularyResponse = self.client.post(CUSTOMIZATION_PATH, param).await?;
        Ok(resp)
    }

    pub async fn query_vocabulary(&self,vocabulary_id: &str) ->Result<QueryVocabularyResponse> {

        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct QueryVocabularyParam {
            model: String,
            input: QueryVocabularyInput,
        }
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct QueryVocabularyInput {
            action: String,
            vocabulary_id: String,
        }

        let param = QueryVocabularyParam {
            model: "speech-biasing".to_string(),
            input: QueryVocabularyInput {
                action: "query_vocabulary".to_string(),
                vocabulary_id: vocabulary_id.to_string(),
            },
        };
        let resp: QueryVocabularyResponse = self.client.post(CUSTOMIZATION_PATH, param).await?;
        Ok(resp)
    }

    /// bugs in dashscope ?
    pub async fn list_vocabularies(&self,prefix: Option<&str>,page_index: Option<usize>,page_size: Option<usize>) ->Result<ListVocabulariesResponse> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct ListVocabulariesParam {
            model: String,
            input: ListVocabulariesInput,
        }
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct ListVocabulariesInput {
            action: String,
            prefix: String,
            page_index: usize,
            page_size: usize,
        }
        let param = ListVocabulariesParam {
            model: "speech-biasing".to_string(),
            input: ListVocabulariesInput {
                action: "list_vocabularie".to_string(),
                prefix: prefix.map(|s| s.to_string()).unwrap_or_default(),
                page_index: page_index.unwrap_or(0),
                page_size: page_size.unwrap_or(10),
            },
        };
        let resp: ListVocabulariesResponse = self.client.post(CUSTOMIZATION_PATH, param).await?;
        Ok(resp)
    }

    pub async fn update_vocabulary(&self,vocabulary_id: &str,vocabulary:&[VocabularyDetail]) ->Result<UpdateVocabularyResponse> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct UpdateVocabularyParam {
            model: String,
            input: UpdateVocabularyInput,
        }
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct UpdateVocabularyInput {
            action: String,
            vocabulary_id: String,
            vocabulary: Vec<VocabularyDetail>,
        }

        let param = UpdateVocabularyParam {
            model: "speech-biasing".to_string(),
            input: UpdateVocabularyInput {
                action: "update_vocabulary".to_string(),
                vocabulary_id: vocabulary_id.to_string(),
                vocabulary: vocabulary.to_vec(),
            },
        };
        let resp: UpdateVocabularyResponse = self.client.post(CUSTOMIZATION_PATH, param).await?;
        Ok(resp)
    }

    pub async fn delete_vocabulary(&self,vocabulary_id: &str) ->Result<DeleteVocabularyResponse> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct DeleteVocabularyParam {
            model: String,
            input: DeleteVocabularyInput,
        }
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct DeleteVocabularyInput {
            action: String,
            vocabulary_id: String,
        }

        let param = DeleteVocabularyParam {
            model: "speech-biasing".to_string(),
            input: DeleteVocabularyInput {
                action: "delete_vocabulary".to_string(),
                vocabulary_id: vocabulary_id.to_string(),
            },
        };
        let resp: DeleteVocabularyResponse = self.client.post(CUSTOMIZATION_PATH, param).await?;
        Ok(resp)
    }

}
