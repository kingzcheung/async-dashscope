use crate::operation::request::RequestTrait;
use crate::{
    error::{DashScopeError, Result},
    operation::{common::Parameters, embeddings::EmbeddingsParameters},
};

/// Defines the validation strategy for a given model.
///
/// This enum replaces the previous trait-based approach to resolve `dyn` compatibility issues.
/// It allows for static dispatch, which is more performant and avoids the complexities of
/// object safety.
pub enum ModelValidator {
    /// The default validation, which performs no special checks.
    Default,
    /// Validation specific to the `deepseek-r1` model.
    NotSupportResultFormatText,
    /// validation enable_thinking
    NotSupportEnableThinking,
    NotSupportToolCall,
    NotSupportJsonOutput,
    // dimensions 不匹配
    DimensionNotMatch,
    OnlyStreaming,
}

pub trait Validator<T> {
    /// 验证请求的参数
    #[allow(clippy::result_large_err)]
    fn validate<R: RequestTrait<P = T> + ?Sized>(&self, params: &R) -> Result<()>;
}

impl Validator<EmbeddingsParameters> for ModelValidator {
    fn validate<R: RequestTrait<P = EmbeddingsParameters> + ?Sized>(
        &self,
        params: &R,
    ) -> Result<()> {
        match self {
            ModelValidator::DimensionNotMatch => {
                // text-embedding-v4 向量维度 只能是以下值: 2,048、1,536、1,024（默认）、768、512、256、128、64
                // text-embedding-v3 向量维度 只能是以下值: 1,024（默认）、768、512、256、128或64
                // text-embedding-v2 向量维度 只能是1,536
                // text-embedding-v1 向量维度 只能是1,536

                if let Some(p) = params.parameters() {
                    let valid_dimensions = match params.model() {
                        "text-embedding-v1" | "text-embedding-v2" => {
                            vec![1536]
                        }
                        "text-embedding-v3" => {
                            vec![1024, 768, 512, 256, 128, 64]
                        }
                        "text-embedding-v4" => {
                            vec![2048, 1536, 1024, 768, 512, 256, 128, 64]
                        }
                        _ => vec![], // 未知模型不验证
                    };
                    if let Some(dimension) = p.dimension {
                        if !valid_dimensions.contains(&dimension) {
                            return Err(DashScopeError::InvalidArgument(format!(
                                "Invalid dimension: {} for model: {}",
                                dimension,
                                params.model()
                            )));
                        }
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
impl Validator<Parameters> for ModelValidator {
    fn validate<R: RequestTrait<P = Parameters> + ?Sized>(&self, params: &R) -> Result<()> {
        match self {
            ModelValidator::Default => {
                // No specific validation rules for the default case.
                Ok(())
            }
            ModelValidator::NotSupportResultFormatText => {
                // The deepseek-r1 model does not support `result_format: "text"`.
                if let Some(p) = params.parameters() {
                    if let Some(format) = &p.result_format {
                        if format == "text" {
                            return Err(DashScopeError::InvalidArgument(
                                "deepseek-r1 does not support result_format = text".into(),
                            ));
                        }
                    }
                }
                Ok(())
            }
            ModelValidator::NotSupportEnableThinking => {
                if let Some(p) = params.parameters() {
                    if let Some(thinking) = p.enable_thinking {
                        if thinking {
                            return Err(DashScopeError::InvalidArgument(
                                "The model does not support enable_thinking = true".into(),
                            ));
                        }
                    }
                }
                Ok(())
            }
            ModelValidator::NotSupportJsonOutput => {
                if let Some(p) = params.parameters() {
                    if let Some(response_format) = p.response_format.as_ref() {
                        if response_format.type_ == "json_object" {
                            return Err(DashScopeError::InvalidArgument(
                                "The model does not support response_format=json_object".into(),
                            ));
                        }
                    }
                }
                Ok(())
            }

            ModelValidator::NotSupportToolCall => {
                if let Some(p) = params.parameters() {
                    if p.tools.is_some() {
                        return Err(DashScopeError::InvalidArgument(
                            "The model does not support tool call".into(),
                        ));
                    }
                }
                Ok(())
            }

            ModelValidator::OnlyStreaming => {
                if let Some(p) = params.parameters() {
                    #[allow(deprecated)]
                    if p.incremental_output == Some(false) {
                        return Err(DashScopeError::InvalidArgument(
                            "The model does not support streaming".into(),
                        ));
                    }
                }

                Ok(())
            }

            _ => Ok(()),
        }
    }
}

/// Selects the appropriate validator for the given model name.
///
/// # Arguments
///
/// * `model` - The name of the model as a string slice.
///
/// # Returns
///
/// A `ModelValidator` enum variant corresponding to the required validation strategy.
pub(crate) fn check_model_parameters(model: &str) -> Vec<ModelValidator> {
    match model {
        "deepseek-r1" => vec![
            ModelValidator::NotSupportResultFormatText,
            ModelValidator::NotSupportJsonOutput,
        ],
        "qwen-vl" | "qwen-audio" => vec![ModelValidator::NotSupportToolCall],
        "Moonshot-Kimi-K2-Instruct" => vec![
            ModelValidator::NotSupportEnableThinking,
            ModelValidator::NotSupportResultFormatText,
            ModelValidator::NotSupportJsonOutput,
            ModelValidator::NotSupportToolCall,
        ],
        "text-embedding-v4" | "text-embedding-v3" | "text-embedding-v2" | "text-embedding-v1" => {
            vec![ModelValidator::DimensionNotMatch]
        }
        "qwen-mt-image" => {
            vec![ModelValidator::Default]
        }
        "glm-4.6" | "glm-4.5" | "glm-4.5-air" => {
            vec![ModelValidator::OnlyStreaming]
        }
        _ => vec![ModelValidator::Default],
    }
}
