use serde::{Deserialize, Serialize};

/// 文件上传参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUploadParam {
    /// 文件路径列表
    pub files: Vec<String>,
    /// 用途
    pub purpose: String,
    /// 描述列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descriptions: Option<Vec<String>>,
}

impl FileUploadParam {
    pub fn builder() -> FileUploadParamBuilder {
        FileUploadParamBuilder::default()
    }
}

#[derive(Default)]
pub struct FileUploadParamBuilder {
    files: Option<Vec<String>>,
    purpose: Option<String>,
    descriptions: Option<Vec<String>>,
}

impl FileUploadParamBuilder {
    pub fn files<T: Into<String>>(mut self, files: Vec<T>) -> Self {
        self.files = Some(files.into_iter().map(|f| f.into()).collect());
        self
    }

    pub fn purpose<T: Into<String>>(mut self, purpose: T) -> Self {
        self.purpose = Some(purpose.into());
        self
    }

    pub fn descriptions<T: Into<String>>(mut self, descriptions: Vec<T>) -> Self {
        self.descriptions = Some(descriptions.into_iter().map(|d| d.into()).collect());
        self
    }

    pub fn build(self) -> Result<FileUploadParam, String> {
        let files = self.files.unwrap_or_default();
        if files.is_empty() {
            return Err("Files cannot be empty".to_string());
        }

        let purpose = self.purpose.ok_or("Purpose is required")?;

        Ok(FileUploadParam {
            files,
            purpose,
            descriptions: self.descriptions,
        })
    }
}