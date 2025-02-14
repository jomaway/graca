use std::{fs, path::PathBuf};

use color_eyre::eyre;
use directories::{ProjectDirs, UserDirs};
use serde::Deserialize;

use crate::grade::GradeScale;

#[derive(Debug, Deserialize)]
pub struct AppConfig
{
    export_path: PathBuf,
    default_scale: GradeScale
}

impl AppConfig
{
    pub fn new() -> AppConfig {
        Self {
            export_path: get_document_dir().expect("Document dir not found."),
            default_scale: GradeScale::IHK,
        }
    }

    pub fn read_config() -> eyre::Result<AppConfig> {
        let config_path = if let Ok(config_dir) = get_config_dir() {
            config_dir.join("config.toml")
        } else {
            return Err(eyre::eyre!("Unable to find config dir."));
        };

        if !config_path.exists() {
            return Ok(AppConfig::new());
        }

        let content = fs::read_to_string(config_path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn get_export_path(&self) -> &PathBuf {
        &self.export_path
    }

    pub fn get_default_scale(&self) -> GradeScale {
        self.default_scale.clone()
    }

}

pub fn get_data_dir() -> eyre::Result<PathBuf> {
  let directory = if let Ok(s) = std::env::var("GRACA_DATA") {
    PathBuf::from(s)
  } else if let Some(proj_dirs) = ProjectDirs::from("de", "jomaway", "graca") {
    proj_dirs.data_local_dir().to_path_buf()
  } else {
    return Err(eyre::eyre!("Unable to find data directory for graca."));
  };
  Ok(directory)
}

pub fn get_config_dir() -> eyre::Result<PathBuf> {
  let directory = if let Ok(s) = std::env::var("GRACA_CONFIG") {
    PathBuf::from(s)
  } else if let Some(proj_dirs) = ProjectDirs::from("de", "jomaway", "graca") {
    proj_dirs.config_local_dir().to_path_buf()
  } else {
    return Err(eyre::eyre!("Unable to find config directory for graca."));
  };
  Ok(directory)
}

pub fn get_document_dir() -> eyre::Result<PathBuf> {
    let directory = if let Some(user_dirs) = UserDirs::new() {
        if let Some(documents_dir) = user_dirs.document_dir() {
            documents_dir.to_path_buf()
        } else {
            return Err(eyre::eyre!("Unable to find document directory."));
        }
    } else {
        return Err(eyre::eyre!("Unable to find user directory."));
    };

    Ok(directory)
}