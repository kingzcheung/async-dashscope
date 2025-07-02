use crate::{error::DashScopeError, operation::common::Usage};
use base64::prelude::*;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use thiserror::Error;
use tokio_stream::Stream;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextToSpeechOutput {
    pub request_id: String,
    /// 调用结果信息。
    #[serde(rename = "output")]
    pub output: Output,
    /// 本次chat请求使用的token信息。
    #[serde(rename = "usage")]
    pub usage: Option<Usage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Output {
    /// 有两种情况：
    /// - 正在生成时为"null"；
    /// - 因模型输出自然结束，或触发输入参数中的stop条件而结束时为"stop"。
    pub finish_reason: Option<String>,
    /// 模型输出的音频信息。
    pub audio: Audio,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Audio {
    pub id: String,
    /// 模型输出的完整音频文件的URL，有效期24小时。
    pub url: Option<String>,
    /// url 将要过期的时间戳。
    pub expires_at: i64,
    /// 流式输出时的Base64 音频数据。
    pub data: String,
}

pub type TextToSpeechOutputStream =
    Pin<Box<dyn Stream<Item = Result<TextToSpeechOutput, DashScopeError>> + Send>>;

#[derive(Error, Debug)]
pub enum AudioOutputError {
    #[error("Failed to download audio file:{}", 0)]
    DownloadError(#[from] reqwest::Error),
    #[error("Failed to save audio file:{}", 0)]
    SaveError(#[from] std::io::Error),
    #[error("Audio url is null")]
    NullUrl,
    #[error("Failed to decode audio data")]
    DataDecodeError,
}

impl Audio {
    pub fn get_audio_data(&self) -> String {
        self.data.clone()
    }

    pub fn is_finished(&self) -> bool {
        self.url.is_some()
    }

    /// 注意这是一个 pcm 数据，需要解码后才能播放
    pub fn to_vec(&self) -> Result<Vec<u8>, AudioOutputError> {
        BASE64_STANDARD
            .decode(&self.data)
            .map_err(|_| AudioOutputError::DataDecodeError)
    }

    #[cfg(feature = "wav-decoder")]
    pub fn to_wav(&self,sample_rate: u32, num_channels: u16, bits_per_sample: u16) -> Result<Vec<u8>, AudioOutputError> {
        use std::io::Cursor;
        use hound::{WavSpec, WavWriter};

        let pcm_data = self.to_vec()?;
        let mut buffer = Cursor::new(Vec::new());
        let spec = WavSpec {
            channels: num_channels,
            sample_rate,
            bits_per_sample,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = WavWriter::new(&mut buffer, spec).map_err(|e| {
            eprintln!("WAV writer error: {e}");
            AudioOutputError::DataDecodeError
        })?;

        // 根据位深度写入PCM数据
        match bits_per_sample {
            16 => {
                // 将字节转换为i16样本
                for chunk in pcm_data.chunks_exact(2) {
                    let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                    writer.write_sample(sample).map_err(|_| AudioOutputError::DataDecodeError)?;
                }
            }
            8 => {
                // 直接写入u8样本
                for &sample in &pcm_data {
                    writer.write_sample(sample as i8).map_err(|_| AudioOutputError::DataDecodeError)?;
                }
            }
            _ => return Err(AudioOutputError::DataDecodeError),
        }

        // 完成写入并返回WAV数据
        writer.finalize().map_err(|_| AudioOutputError::DataDecodeError)?;
        Ok(buffer.into_inner())
    }

    pub fn bytes(&self) -> Result<Bytes, AudioOutputError> {
        Ok(Bytes::copy_from_slice(&self.to_vec()?))
    }

    pub async fn download(&self, save_path: &str) -> Result<(), AudioOutputError> {
        let Some(url) = &self.url else {
            return Err(AudioOutputError::NullUrl);
        };
        let r = reqwest::get(url).await?.bytes().await?;

        // save file
        tokio::fs::write(save_path, r).await?;

        Ok(())
    }
}

impl TextToSpeechOutput {
    pub async fn download(&self, save_path: &str) -> Result<(), AudioOutputError> {
        self.output.audio.download(save_path).await
    }

    pub fn is_finished(&self) -> bool {
        self.output.audio.is_finished()
    }
}
