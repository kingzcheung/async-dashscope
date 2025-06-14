use crate::error::Result;
use crate::{error::DashScopeError, operation::generation::GenerationParam};
pub trait ModelValidation {
    fn validate(&self, params: &GenerationParam) -> Result<()>;
}

pub struct DefaultValidation;

impl ModelValidation for DefaultValidation {
    fn validate(&self, params: &GenerationParam) -> Result<()> {
        Ok(())
    }
}

pub struct DeepSeekV1;

impl ModelValidation for DeepSeekV1 {
    fn validate(&self, params: &GenerationParam) -> Result<()> {
        if let Some(param) = &params.parameters {
            if let Some(ref format) = param.result_format {
                if format == "text" {
                    return Err(DashScopeError::InvalidArgument(
                        "deepseek-r1 does not support result_format = text".into(),
                    ));
                }
            }
        }
        Ok(())
    }
}


pub(crate) fn check_model_parameters(model: &str) -> Box<dyn ModelValidation> { 

    match model {
        "deepseek-r1" => Box::new(DeepSeekV1),

        _ =>  Box::new(DefaultValidation),
    }
}