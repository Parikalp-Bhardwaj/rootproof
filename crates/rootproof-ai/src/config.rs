use std::{fs, path::PathBuf};
use directories::ProjectDirs;
use serde::{Serialize, Deserialize};

use crate::AiError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: String,
    pub model: String,
    pub api_key_env: String
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootProofConfig{
    pub ai: AiConfig
}

impl AiConfig {
    pub fn openrouter(model: impl Into<String>) -> Self {
        Self {
            provider: "openrouter".to_owned(),
            model: model.into(),
            api_key_env: "OPENROUTER_API_KEY".to_owned()
        }
    }
}

impl RootProofConfig{
    pub fn openrouter(model: impl Into<String>) -> Self{
        Self { ai: AiConfig::openrouter(model) }
    }
}

pub fn config_path() -> Result<PathBuf, AiError>{
    let project_dirs = ProjectDirs::from("dev", "RootProof", "RootProof")
            .ok_or(AiError::ConfigDirectoryUnavailable)?;

    Ok(project_dirs.config_dir().join("config.toml"))
}

pub fn save_config(config: &RootProofConfig) -> Result<PathBuf, AiError>{
    let path = config_path()?;

    let parent = path.parent().ok_or(AiError::ConfigDirectoryUnavailable)?;

    fs::create_dir_all(parent).map_err(|source|{
        AiError::ConfigIo { path: parent.to_path_buf(), source }
    })?;

    let contents = toml::to_string_pretty(config)
            .map_err(|error|{
                AiError::ConfigSerialize(error.to_string())
            })?;

    fs::write(&path, contents)
            .map_err(|source|{
                AiError::ConfigIo { path: path.clone(), source }
            })?;
    
    Ok(path)
}

pub fn load_config() -> Result<RootProofConfig, AiError>{
    let path = config_path()?;

    let content = fs::read_to_string(&path)
            .map_err(|source|{
                AiError::ConfigIo { path: path.clone(), source }
            })?;
    
    toml::from_str(&content)
            .map_err(|error|{
                AiError::ConfigParse(
                    error.to_string()
                )
            })
}