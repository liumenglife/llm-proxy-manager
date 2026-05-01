#[cfg(target_os = "macos")]
use crate::modules::logger;
use serde::{Deserialize, Serialize};

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_CHECK_INTERVAL_HOURS: u64 = 24;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub has_update: bool,
    pub download_url: String, // previously release_url
    pub release_notes: String,
    pub published_at: String,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSettings {
    pub auto_check: bool,
    pub last_check_time: u64,
    #[serde(default = "default_check_interval")]
    pub check_interval_hours: u64,
}

fn default_check_interval() -> u64 {
    DEFAULT_CHECK_INTERVAL_HOURS
}

impl Default for UpdateSettings {
    fn default() -> Self {
        Self {
            auto_check: false,
            last_check_time: 0,
            check_interval_hours: DEFAULT_CHECK_INTERVAL_HOURS,
        }
    }
}

/// 当前项目暂不发布自动更新，因此更新检查默认返回无更新且不访问网络。
pub async fn check_for_updates() -> Result<UpdateInfo, String> {
    let current_version = CURRENT_VERSION.to_string();

    Ok(UpdateInfo {
        latest_version: current_version.clone(),
        current_version,
        has_update: false,
        download_url: String::new(),
        release_notes: "自动更新检查已禁用。".to_string(),
        published_at: String::new(),
        source: Some("disabled".to_string()),
    })
}

/// Compare two semantic versions (e.g., "3.3.30" vs "3.3.29")
fn compare_versions(latest: &str, current: &str) -> bool {
    let parse_version =
        |v: &str| -> Vec<u32> { v.split('.').filter_map(|s| s.parse::<u32>().ok()).collect() };

    let latest_parts = parse_version(latest);
    let current_parts = parse_version(current);

    for i in 0..latest_parts.len().max(current_parts.len()) {
        let latest_part = latest_parts.get(i).unwrap_or(&0);
        let current_part = current_parts.get(i).unwrap_or(&0);

        if latest_part > current_part {
            return true;
        } else if latest_part < current_part {
            return false; // e.g. local: 3.3.30, remote: 3.3.30 => false
        }
    }

    false
}

/// Load update settings from config file
pub fn load_update_settings() -> Result<UpdateSettings, String> {
    let data_dir = crate::modules::account::get_data_dir()
        .map_err(|e| format!("Failed to get data dir: {}", e))?;
    let settings_path = data_dir.join("update_settings.json");

    if !settings_path.exists() {
        return Ok(UpdateSettings::default());
    }

    let content = std::fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings: {}", e))
}

/// Save update settings to config file
pub fn save_update_settings(settings: &UpdateSettings) -> Result<(), String> {
    let data_dir = crate::modules::account::get_data_dir()
        .map_err(|e| format!("Failed to get data dir: {}", e))?;
    let settings_path = data_dir.join("update_settings.json");

    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    std::fs::write(&settings_path, content)
        .map_err(|e| format!("Failed to write settings file: {}", e))
}

/// Detect if the app was installed via Homebrew Cask (macOS only)
pub fn is_homebrew_installed() -> bool {
    #[cfg(target_os = "macos")]
    {
        let caskroom_paths = [
            "/opt/homebrew/Caskroom/llm-proxy-manager",
            "/usr/local/Caskroom/llm-proxy-manager",
        ];

        for path in &caskroom_paths {
            if std::path::Path::new(path).exists() {
                logger::log_info(&format!("Detected Homebrew Cask installation at: {}", path));
                return true;
            }
        }
    }

    false
}

/// Execute `brew upgrade --cask llm-proxy-manager` with timeout (macOS only)
#[cfg(not(target_os = "macos"))]
pub async fn brew_upgrade_cask() -> Result<String, String> {
    Err("brew_not_supported".to_string())
}

#[cfg(target_os = "macos")]
pub async fn brew_upgrade_cask() -> Result<String, String> {
    logger::log_info("Starting Homebrew Cask upgrade for llm-proxy-manager...");

    // Find brew binary
    let brew_path = if std::path::Path::new("/opt/homebrew/bin/brew").exists() {
        "/opt/homebrew/bin/brew"
    } else if std::path::Path::new("/usr/local/bin/brew").exists() {
        "/usr/local/bin/brew"
    } else {
        return Err("brew_not_found".to_string());
    };

    // 3 min timeout to prevent hanging
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(180),
        tokio::process::Command::new(brew_path)
            .args(["upgrade", "--cask", "llm-proxy-manager"])
            .output(),
    )
    .await;

    let output = match result {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => {
            logger::log_error(&format!("Failed to execute brew upgrade: {}", e));
            return Err("brew_exec_failed".to_string());
        }
        Err(_) => {
            logger::log_error("Homebrew upgrade timed out after 3 minutes");
            return Err("brew_timeout".to_string());
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        logger::log_info(&format!("Homebrew upgrade succeeded: {}", stdout));
        Ok(stdout)
    } else {
        logger::log_error(&format!(
            "brew upgrade failed - stdout: {} stderr: {}",
            stdout, stderr
        ));
        // Return structured error key for frontend i18n
        if stderr.contains("already installed") || stdout.contains("already installed") {
            Err("brew_already_latest".to_string())
        } else {
            Err("brew_upgrade_failed".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_versions() {
        assert!(compare_versions("3.3.36", "3.3.35"));
        assert!(compare_versions("3.4.0", "3.3.35"));
        assert!(compare_versions("4.0.3", "3.3.35"));
        assert!(!compare_versions("3.3.34", "3.3.35"));
        assert!(!compare_versions("3.3.35", "3.3.35"));
    }

    #[tokio::test]
    async fn check_for_updates_returns_no_update_without_network() {
        let info = tokio::time::timeout(std::time::Duration::from_millis(100), check_for_updates())
            .await
            .expect("check_for_updates should return without network access")
            .expect("disabled updater should return a normal no-update response");

        assert!(!info.has_update);
        assert_eq!(info.current_version, CURRENT_VERSION);
        assert_eq!(info.latest_version, CURRENT_VERSION);
        assert!(info.download_url.is_empty());
        let legacy_manifest = concat!("updater", ".json");
        assert_ne!(info.source.as_deref(), Some(legacy_manifest));

        let serialized = serde_json::to_string(&info).unwrap();
        assert!(!serialized.contains("github.com/"));
        assert!(!serialized.contains(legacy_manifest));
    }

    #[test]
    fn app_startup_does_not_trigger_update_notification() {
        let app_source = include_str!("../../../src/App.tsx");
        let commands_source = include_str!("../commands/mod.rs");
        let lib_source = include_str!("../lib.rs");
        let request_source = include_str!("../../../src/utils/request.ts");
        let server_source = include_str!("../proxy/server.rs");
        let should_command = concat!("should_check", "_updates");
        let touch_command = concat!("update_last", "_check_time");
        let status_route = concat!("/system/updates/", "check", "-status");
        let touch_route = concat!("/system/updates/", "touch");

        assert!(!app_source.contains(should_command));
        assert!(!app_source.contains(touch_command));
        assert!(!app_source.contains("UpdateNotification"));
        assert!(!commands_source.contains(should_command));
        assert!(!commands_source.contains(touch_command));
        assert!(!lib_source.contains(should_command));
        assert!(!lib_source.contains(touch_command));
        assert!(!request_source.contains(should_command));
        assert!(!request_source.contains(touch_command));
        assert!(!request_source.contains(status_route));
        assert!(!request_source.contains(touch_route));
        assert!(!server_source.contains(status_route));
        assert!(!server_source.contains(touch_route));
    }
}
