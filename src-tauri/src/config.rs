use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::OnceLock;

const CONFIG_FILE: &str = "os-config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaList {
    pub list_id: String,
    pub label: String,
    pub slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClickupConfig {
    pub workspace_id: String,
    pub daily_list_id: String,
    pub areas: Vec<AreaList>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CalendarConfig {
    pub personal_email: String,
    #[serde(default)]
    pub extra_calendars: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// "clickup_doc" | "local_file" | "none"
    pub mode: String,
    /// Used when mode == "clickup_doc"
    pub clickup_logs_folder_id: String,
    /// Used when mode == "local_file" — absolute path or relative to project root
    pub local_file_path: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            mode: "none".to_string(),
            clickup_logs_folder_id: String::new(),
            local_file_path: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandsConfig {
    /// Name of the Claude Code skill invoked by the "Start Day" button.
    /// Default `"start-day"` resolves to the shipped skill at
    /// `.claude/skills/start-day/`. Override (e.g. `"check-in"`) to call
    /// your own personal skill instead.
    pub start_day_skill: String,
}

impl Default for CommandsConfig {
    fn default() -> Self {
        Self { start_day_skill: "start-day".to_string() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub clickup: ClickupConfig,
    pub calendar: CalendarConfig,
    #[serde(default)]
    pub commands: CommandsConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

fn config_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join(CONFIG_FILE)
}

pub fn load() {
    let path = config_path();
    let cfg = match std::fs::read_to_string(&path) {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_else(|e| {
            eprintln!("[config] {CONFIG_FILE} parse error: {e} — using empty defaults");
            Config::default()
        }),
        Err(_) => {
            eprintln!("[config] {CONFIG_FILE} not found — run `python scripts/setup.py`");
            Config::default()
        }
    };
    let _ = CONFIG.set(cfg);
}

pub fn get() -> &'static Config {
    CONFIG.get_or_init(Config::default)
}

#[tauri::command]
pub fn get_config() -> Config {
    get().clone()
}
