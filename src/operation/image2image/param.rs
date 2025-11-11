use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use crate::oss_util;
#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct Image2imageParam {
    #[builder(setter(into, strip_option))]
    pub model: String,

    pub input: Input,
}

impl Image2imageParam {
    pub(crate) async fn upload_file_to_oss(
        mut self,
        api_key: &str,
    ) -> Result<Self, crate::error::DashScopeError> {
        let oss_url =
            oss_util::upload_file_and_get_url(api_key, &self.model, &self.input.image_url).await?;

        self.input.image_url = oss_url;

        Ok(self)
    }
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct Input {
    /// 图像的公网可访问的URL，支持 HTTP 或 HTTPS 协议。
    ///
    /// - 格式限制：JPG、JPEG、PNG、BMP、PNM、PPM、TIFF、WEBP
    /// - 尺寸限制：图像的宽度和高度均需在15-8192像素范围内，宽高比在1:10至10:1范围内。
    /// - 大小限制：不超过10MB
    #[builder(setter(into))]
    pub image_url: String,
    /// 源语种。
    ///
    /// - 支持值：语种全称、语种编码或auto（自动检测），对大小写不敏感
    /// - 限制：与target_lang不同，且至少有一项为中文或英文
    /// - 示例：Chinese、en或auto
    #[builder(setter(into))]
    pub source_lang: String,
    /// 目标语种。
    ///
    /// - 支持值：语种全称或语种编码，对大小写不敏感
    /// - 限制：与source_lang不同，且至少有一项为中文或英文
    /// - 示例：Chinese、en
    #[builder(setter(into))]
    pub target_lang: String,

    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub ext: Option<Ext>,
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct Ext {
    /// 领域提示，为使译文风格更贴合特定领域，可以使用英文描述使用场景、译文风格等需求。
    /// 为确保翻译效果，建议不超过200个英文单词。
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    domain_hint: Option<String>,

    /// 配置敏感词，以在翻译前过滤图片中完全匹配的文本，对大小写敏感。
    /// 敏感词的语种可与源语种不一致，支持全部的源语种和目标语种。为确保翻译效果，建议单次请求添加的敏感词不超过50个。
    ///
    /// 示例：["全场9折", "七天无理由退换"]
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    sensitives: Option<Vec<String>>,

    /// 术语干预，为特定术语设定译文，以满足特定领域的翻译需求，术语对的语种需要与source_lang和target_lang对应。
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    terminologies: Option<Vec<Terminology>>,
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    config: Option<Config>,
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct Terminology {
    src: String,
    tgt: String,
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct Config {
    /// 用于控制是否跳过主体检测，翻译图像中主体（如人物、商品、Logo）上的文字。
    skip_img_segment: bool,
}

// impl Validator<Image2imageParam> for ModelValidator {
//     fn validate<R: crate::operation::request::RequestTrait<P = Image2imageParam> + ?Sized>(
//         &self,
//         params: &R,
//     ) -> crate::error::Result<()> {
//         match self {
//             ModelValidator::NotSupportResultFormatText => todo!(),
//             ModelValidator::NotSupportEnableThinking => todo!(),
//             ModelValidator::NotSupportToolCall => todo!(),
//             ModelValidator::NotSupportJsonOutput => todo!(),
//             ModelValidator::DimensionNotMatch => todo!(),
//             _ => Ok(()),
//         }
//     }
// }
