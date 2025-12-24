use serde::{Deserialize, Serialize};

/// 文件上传输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUploadOutput {
    /// 请求ID
    pub request_id: String,
    /// 响应数据
    pub data: FileUploadData,
}

/// 文件上传数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUploadData {
    /// 已上传的文件列表
    #[serde(rename = "uploaded_files")]
    pub uploaded_files: Vec<UploadedFile>,
    /// 上传失败的文件列表
    #[serde(rename = "failed_uploads")]
    pub failed_uploads: Vec<FailedUpload>,
}

/// 已上传的文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadedFile {
    /// 文件ID
    #[serde(rename = "file_id")]
    pub file_id: String,
    /// 文件名
    pub name: String,
}

/// 上传失败的文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedUpload {
    /// 文件名
    pub name: String,
    /// 错误代码
    pub code: String,
    /// 错误消息
    pub message: String,
}

/// 文件列表输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileListOutput {
    /// 请求ID
    pub request_id: String,
    /// 响应数据
    pub data: FileListData,
}

/// 文件列表数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileListData {
    /// 总数
    pub total: u64,
    /// 分页大小
    pub page_size: u64,
    /// 当前页
    pub page_no: u64,
    /// 文件列表
    pub files: Vec<FileInfo>,
}

/// 文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// ID (列表接口有此字段，单个文件查询可能没有)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    /// 文件ID
    #[serde(rename = "file_id")]
    pub file_id: String,
    /// 文件名
    pub name: String,
    /// 描述
    pub description: String,
    /// 文件大小（字节）
    pub size: u64,
    /// MD5
    pub md5: String,
    /// 创建时间
    pub gmt_create: String,
    /// URL
    pub url: String,
    /// 区域 (列表接口有此字段，单个文件查询可能没有)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    /// 用户ID (列表接口有此字段，单个文件查询可能没有)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// API密钥ID (列表接口有此字段，单个文件查询可能没有)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_id: Option<String>,
    /// 用途 (列表接口有此字段，单个文件查询可能没有)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
}

/// 单个文件信息输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRetrieveOutput {
    /// 请求ID
    pub request_id: String,
    /// 文件信息
    pub data: FileInfo,
}

/// 文件删除输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDeleteOutput {
    /// 请求ID
    pub request_id: String,
    /// 错误代码（可选，仅在失败时存在）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// 错误消息（可选，仅在失败时存在）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}