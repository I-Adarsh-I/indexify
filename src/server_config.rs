use anyhow::Result;
use figment::{
    providers::{Env, Format, Yaml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmbeddingModelKind {
    #[serde(rename(serialize = "all-minilm-l12-v2", deserialize = "all-minilm-l12-v2"))]
    AllMiniLmL12V2,

    #[serde(rename(serialize = "all-minilm-l6-v2", deserialize = "all-minilm-l6-v2"))]
    AllMiniLmL6V2,

    #[serde(rename(serialize = "t5-base", deserialize = "t5-base"))]
    T5Base,

    #[serde(rename(serialize = "all-mpnet-base-v2", deserialize = "all-mpnet-base-v2"))]
    AllMPNetBaseV2,

    #[serde(rename(
        serialize = "text-embedding-ada-002",
        deserialize = "text-embedding-ada-002"
    ))]
    OpenAIAda02,
}

impl fmt::Display for EmbeddingModelKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EmbeddingModelKind::AllMiniLmL12V2 => write!(f, "all-minilm-l12-v2"),
            EmbeddingModelKind::AllMiniLmL6V2 => write!(f, "all-minilm-l6-v2"),
            EmbeddingModelKind::T5Base => write!(f, "t5-base"),
            EmbeddingModelKind::AllMPNetBaseV2 => write!(f, "all-mpnet-base-v2"),
            EmbeddingModelKind::OpenAIAda02 => write!(f, "text-embedding-ada-002"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceKind {
    #[serde(rename(serialize = "cpu", deserialize = "cpu"))]
    Cpu,

    #[serde(rename(serialize = "gpu", deserialize = "gpu"))]
    Gpu,

    #[serde(rename(serialize = "remote", deserialize = "remote"))]
    Remote,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingModel {
    #[serde(rename(serialize = "model", deserialize = "model"))]
    pub model_kind: EmbeddingModelKind,
    #[serde(rename(serialize = "device", deserialize = "device"))]
    pub device_kind: DeviceKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub listen_addr: String,
    pub available_models: Vec<EmbeddingModel>,
    pub openai: Option<OpenAIConfig>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:8900".to_string(),
            available_models: vec![
                EmbeddingModel {
                    model_kind: EmbeddingModelKind::AllMiniLmL12V2,
                    device_kind: DeviceKind::Cpu,
                },
                EmbeddingModel {
                    model_kind: EmbeddingModelKind::OpenAIAda02,
                    device_kind: DeviceKind::Remote,
                },
            ],
            openai: Some(OpenAIConfig {
                api_key: "xxxx".to_string(),
            }),
        }
    }
}

impl ServerConfig {
    pub fn from_path(path: String) -> Result<Self> {
        let config_str: String = fs::read_to_string(path)?;
        let config: ServerConfig = Figment::new()
            .merge(Yaml::string(&config_str))
            .merge(Env::prefixed("INDEXIFY_"))
            .extract()?;
        Ok(config)
    }

    pub fn generate(path: String) -> Result<()> {
        let config = ServerConfig::default();
        let str = serde_yaml::to_string(&config)?;
        std::fs::write(path, str)?;
        Ok(())
    }
}
