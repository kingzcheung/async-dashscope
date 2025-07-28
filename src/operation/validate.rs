use crate::error::{DashScopeError, Result};
use crate::operation::request::RequestTrait;

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
}

impl ModelValidator {
    /// Validates the request parameters based on the selected model strategy.
    ///
    /// # Arguments
    ///
    /// * `params` - A type that implements `RequestTrait`, providing access to the model name and parameters.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the parameters are valid.
    /// * `Err(DashScopeError)` if the parameters are invalid for the given model.
    pub fn validate<R: RequestTrait + ?Sized>(&self, params: &R) -> Result<()> {
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
        _ => vec![ModelValidator::Default],
    }
}
