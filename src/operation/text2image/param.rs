use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct Text2imageParam {
    #[builder(setter(into, strip_option))]
    pub model: String,

    pub input: Input,

    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    pub parameters: Option<Parameters>
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct Input {
    /// 正向提示词，用来描述生成图像中期望包含的元素和视觉特点。
    ///
    /// 支持中英文，每个汉字/字母占一个字符，超过部分会自动截断。长度限制因模型版本而异：
    ///
    /// - wan2.5-t2i-preview：长度不超过2000个字符。
    /// - wan2.2及以下版本模型：长度不超过800个字符。
    #[builder(setter(into, strip_option))]
    pub prompt: String,

    /// 支持中英文，长度不超过500个字符，超过部分会自动截断。
    ///
    /// 示例值：低分辨率、错误、最差质量、低质量、残缺、多余的手指、比例不良等。
    #[builder(setter(into, strip_option))]
    pub negative_prompt: Option<String>,
}


#[derive(Debug, Clone, Builder, Serialize, Deserialize, PartialEq)]
pub struct Parameters {
    /// 输出图像的分辨率，格式为宽*高。默认值和约束因模型版本而异
    /// 
    /// - wan2.5-t2i-preview：默认值为 1280*1280。总像素在 [768*768, 1440*1440] 之间且宽高比范围为 [1:4, 4:1]。例如，768*2700符合要求。
    #[builder(setter(into, strip_option))]
    size: Option<String>,

    /// 生成图片的数量。取值范围为1~4张，默认为4。测试阶段建议设置为1，便于低成本验证。
    #[builder(setter(into, strip_option))]
    n:Option<i32>,

    /// 是否开启prompt智能改写。开启后使用大模型对输入prompt进行智能改写。对于较短的prompt生成效果提升明显，但会增加耗时。
    #[builder(setter(into, strip_option))]
    prompt_extend:Option<bool>,

    /// 是否添加水印标识，水印位于图片右下角
    #[builder(setter(into, strip_option))]
    watermark:Option<bool>,

    /// 随机数种子，取值范围是[0, 2147483647]。
    #[builder(setter(into, strip_option))]
    #[builder(default=None)]
    seed:Option<i32>
}