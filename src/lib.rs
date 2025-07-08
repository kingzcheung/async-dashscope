#![doc = include_str!("../docs/base.md")]

mod client;
pub mod config;
pub mod error;
pub mod operation;

pub use client::Client;
pub(crate) mod oss_util;