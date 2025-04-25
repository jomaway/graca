use std::{fs, path::PathBuf};

use color_eyre::eyre;
use directories::{ProjectDirs, UserDirs};
use lazy_static::lazy_static;
use serde::Deserialize;
use tracing::warn;

use crate::model::scale::GradeScaleType;

lazy_static! {
    pub static ref PROJECT_NAME: String = env!("CARGO_CRATE_NAME").to_uppercase().to_string();
    pub static ref CONFIG_FOLDER: Option<PathBuf> =
        std::env::var(format!("{}_CONFIG", PROJECT_NAME.clone()))
            .ok()
            .map(PathBuf::from);
    pub static ref DATA_FOLDER: Option<PathBuf> =
        std::env::var(format!("{}_DATA", PROJECT_NAME.clone()))
            .ok()
            .map(PathBuf::from);
    pub static ref LOG_ENV: String = format!("{}_LOGLEVEL", PROJECT_NAME.clone());
    pub static ref LOG_FILE: String = format!("{}.log", env!("CARGO_PKG_NAME"));
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    export_path: Option<PathBuf>,
    default_scale: GradeScaleType,
}

impl AppConfig {
    pub fn new() -> AppConfig {
        Self {
            export_path: if let Ok(path) = get_document_dir() {
                Some(path)
            } else {
                None
            },
            default_scale: GradeScaleType::IHK,
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

    pub fn get_export_path(&self) -> &Option<PathBuf> {
        &self.export_path
    }

    pub fn get_default_scale(&self) -> GradeScaleType {
        self.default_scale.clone()
    }
}

fn project_directory() -> Option<ProjectDirs> {
    ProjectDirs::from("de", "jomaway", env!("CARGO_PKG_NAME"))
}

pub fn get_data_dir() -> PathBuf {
    let directory = if let Some(s) = DATA_FOLDER.clone() {
        s
    } else if let Some(proj_dirs) = project_directory() {
        proj_dirs.data_local_dir().to_path_buf()
    } else {
        warn!("Could not find local data dir.");
        PathBuf::from(".").join(".data")
    };
    directory
}

pub fn get_config_dir() -> eyre::Result<PathBuf> {
    let directory = if let Some(s) = CONFIG_FOLDER.clone() {
        s
    } else if let Some(proj_dirs) = project_directory() {
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
