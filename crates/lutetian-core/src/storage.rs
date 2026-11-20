use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::config::AppConfig;
use crate::project::LutetianProject;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActiveBridgeInfo {
    pub host: String,
    pub port: u16,
    pub pid: u32,
}

#[derive(Debug, Clone)]
pub struct AppStorage {
    data_dir: PathBuf,
    projects_dir: PathBuf,
}

impl AppStorage {
    pub fn open_default() -> Self {
        let dir = ProjectDirs::from("io", "hamides-dev", "Lutetian")
            .map(|p| p.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".lutetian"));
        Self::open_in(dir)
    }

    pub fn open_in(data_dir: PathBuf) -> Self {
        let projects_dir = data_dir.join("projects");
        let _ = fs::create_dir_all(&data_dir);
        let _ = fs::create_dir_all(&projects_dir);
        Self {
            data_dir,
            projects_dir,
        }
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn projects_dir(&self) -> &Path {
        &self.projects_dir
    }

    pub fn default_project_path(&self) -> PathBuf {
        self.projects_dir.join("active_project.lutetian.json")
    }

    pub fn bridge_lock_path(&self) -> PathBuf {
        self.data_dir.join("bridge.json")
    }

    pub fn load_config(&self) -> AppConfig {
        let path = self.data_dir.join("config.json");
        let mut cfg = fs::read_to_string(&path)
            .ok()
            .and_then(|raw| serde_json::from_str::<AppConfig>(&raw).ok())
            .unwrap_or_default();
        cfg.ensure_valid();
        cfg
    }

    pub fn save_config(&self, config: &AppConfig) -> Result<(), StorageError> {
        let path = self.data_dir.join("config.json");
        let json = serde_json::to_string_pretty(config)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_project(&self, path: &Path) -> Result<LutetianProject, StorageError> {
        let raw = fs::read_to_string(path)?;
        let mut proj: LutetianProject = serde_json::from_str(&raw)?;
        proj.recompute_validation();
        Ok(proj)
    }

    pub fn save_project(&self, path: &Path, project: &LutetianProject) -> Result<(), StorageError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(project)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn write_bridge_info(&self, info: &ActiveBridgeInfo) -> Result<(), StorageError> {
        let json = serde_json::to_string_pretty(info)?;
        fs::write(self.bridge_lock_path(), json)?;
        Ok(())
    }

    pub fn read_bridge_info(&self) -> Option<ActiveBridgeInfo> {
        fs::read_to_string(self.bridge_lock_path())
            .ok()
            .and_then(|raw| serde_json::from_str::<ActiveBridgeInfo>(&raw).ok())
    }

    pub fn clear_bridge_info(&self) {
        let _ = fs::remove_file(self.bridge_lock_path());
    }
}
