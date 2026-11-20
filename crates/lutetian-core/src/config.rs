use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UiThemeMode {
    #[default]
    Dark,
    Light,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct AppConfig {
    pub theme: UiThemeMode,
    pub bridge_port: u16,
    pub auto_check_updates: bool,
    pub include_prereleases: bool,
    pub last_project_path: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: UiThemeMode::Dark,
            bridge_port: 47821,
            auto_check_updates: true,
            include_prereleases: false,
            last_project_path: None,
        }
    }
}

impl AppConfig {
    pub fn ensure_valid(&mut self) {
        if self.bridge_port < 1024 {
            self.bridge_port = 47821;
        }
    }
}
