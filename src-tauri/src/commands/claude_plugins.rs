#![allow(non_snake_case)]

use crate::config::{get_claude_settings_path, read_json_file, write_json_file};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

/// 已安装的 Claude 插件信息
#[derive(Debug, Clone, Serialize)]
pub struct ClaudeInstalledPlugin {
    pub id: String,
    pub version: String,
    pub scope: String,
    pub installed_at: String,
}

/// 读取 ~/.claude/settings.json 的 enabledPlugins
fn read_claude_enabled_plugins() -> Result<HashMap<String, bool>, String> {
    let path = get_claude_settings_path();
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let json: Value = read_json_file(&path).map_err(|e| e.to_string())?;
    let enabled = json
        .get("enabledPlugins")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| (k.clone(), v.as_bool().unwrap_or(true)))
                .collect()
        })
        .unwrap_or_default();

    Ok(enabled)
}

/// 写入 ~/.claude/settings.json 的 enabledPlugins（保留其他字段）
fn write_claude_enabled_plugins(enabled: &HashMap<String, bool>) -> Result<(), String> {
    let path = get_claude_settings_path();

    let mut settings: Value = if path.exists() {
        read_json_file(&path).map_err(|e| e.to_string())?
    } else {
        json!({})
    };

    if enabled.is_empty() {
        if let Some(obj) = settings.as_object_mut() {
            obj.remove("enabledPlugins");
        }
    } else {
        let enabled_obj: serde_json::Map<String, Value> = enabled
            .iter()
            .map(|(k, v)| (k.clone(), Value::Bool(*v)))
            .collect();
        if let Some(obj) = settings.as_object_mut() {
            obj.insert("enabledPlugins".to_string(), Value::Object(enabled_obj));
        }
    }

    write_json_file(&path, &settings).map_err(|e| e.to_string())?;
    Ok(())
}

/// 获取所有 Claude 全局插件（从 live config 直接读取）
#[tauri::command]
pub fn get_claude_global_plugins() -> Result<HashMap<String, bool>, String> {
    read_claude_enabled_plugins()
}

/// 设置 Claude 全局插件（直接写入 settings.json）
#[tauri::command]
pub fn set_claude_global_plugin(pluginId: String, enabled: bool) -> Result<(), String> {
    let mut plugins = read_claude_enabled_plugins()?;
    plugins.insert(pluginId, enabled);
    write_claude_enabled_plugins(&plugins)
}

/// 删除 Claude 全局插件（直接从 settings.json 移除）
#[tauri::command]
pub fn remove_claude_global_plugin(pluginId: String) -> Result<(), String> {
    let mut plugins = read_claude_enabled_plugins()?;
    plugins.remove(&pluginId);
    write_claude_enabled_plugins(&plugins)
}

/// 从 Claude Code 安装目录读取已安装插件列表
#[tauri::command]
pub fn get_claude_installed_plugins() -> Result<Vec<ClaudeInstalledPlugin>, String> {
    let home = dirs::home_dir().ok_or_else(|| "Failed to get user home directory".to_string())?;
    let path = home
        .join(".claude")
        .join("plugins")
        .join("installed_plugins.json");

    if !path.exists() {
        return Ok(Vec::new());
    }

    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let json: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;

    let Some(plugins_obj) = json.get("plugins").and_then(|v| v.as_object()) else {
        return Ok(Vec::new());
    };

    let mut result = Vec::new();
    for (plugin_id, entries) in plugins_obj {
        let Some(arr) = entries.as_array() else {
            continue;
        };
        let Some(first) = arr.first() else { continue };
        let Some(entry) = first.as_object() else {
            continue;
        };

        result.push(ClaudeInstalledPlugin {
            id: plugin_id.clone(),
            version: entry
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            scope: entry
                .get("scope")
                .and_then(|v| v.as_str())
                .unwrap_or("user")
                .to_string(),
            installed_at: entry
                .get("installedAt")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        });
    }

    Ok(result)
}

/// 批量应用 Claude 插件启用选择（直接写入 settings.json）
///
/// 勾选逻辑：用户勾选 → 将插件 ID 以 `{id: true}` 形式写入 settings.json 的 enabledPlugins。
/// 取消勾选 → 从 enabledPlugins 中移除该插件 ID（不写 false，直接移除）。
/// 只保留已安装插件（以 installed_plugins.json 为准），未安装但被勾选的 ID 会被过滤掉。
#[tauri::command]
pub fn apply_claude_plugin_selection(enabledIds: Vec<String>) -> Result<(), String> {
    let installed = get_installed_claude_plugin_ids()?;

    // 构建新的 enabledPlugins：只保留选中的已安装插件
    let mut new_enabled: HashMap<String, bool> = HashMap::new();
    for id in enabledIds {
        if installed.contains(&id) {
            new_enabled.insert(id, true);
        }
    }

    write_claude_enabled_plugins(&new_enabled)
}

/// 读取已安装的 Claude 插件 ID 集合（以 installed_plugins.json 为准）
pub(crate) fn get_installed_claude_plugin_ids() -> Result<std::collections::HashSet<String>, String>
{
    let home = dirs::home_dir().ok_or_else(|| "Failed to get user home directory".to_string())?;
    let path = home
        .join(".claude")
        .join("plugins")
        .join("installed_plugins.json");

    if !path.exists() {
        return Ok(std::collections::HashSet::new());
    }

    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let json: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;

    let Some(plugins_obj) = json.get("plugins").and_then(|v| v.as_object()) else {
        return Ok(std::collections::HashSet::new());
    };

    Ok(plugins_obj.keys().cloned().collect())
}

/// 从现有 Claude 配置导入插件
///
/// 以 installed_plugins.json 为基准（管"有没有"），settings.json 的 enabledPlugins 为参考（管"开不开"）。
/// 未安装但存在于 enabledPlugins 中的 ghost 条目会被自动清理。
/// 返回当前启用的插件 ID 列表。
#[tauri::command]
pub fn import_claude_plugins_from_live() -> Result<Vec<String>, String> {
    // 1. 读取已安装插件（唯一真实来源）
    let installed = get_installed_claude_plugin_ids()?;

    // 2. 读取 settings.json 的 enabledPlugins
    let mut enabled_in_settings = read_claude_enabled_plugins()?;

    // 3. 清理 ghost 条目（未安装但存在于 enabledPlugins 中）
    let ghost_ids: Vec<String> = enabled_in_settings
        .keys()
        .filter(|id| !installed.contains(*id))
        .cloned()
        .collect();
    for id in &ghost_ids {
        enabled_in_settings.remove(id);
    }

    // 4. 写回清理后的 enabledPlugins
    if !ghost_ids.is_empty() {
        write_claude_enabled_plugins(&enabled_in_settings)?;
    }

    // 5. 返回当前启用的插件 ID 列表
    let enabled_ids: Vec<String> = enabled_in_settings
        .into_iter()
        .filter(|(_, enabled)| *enabled)
        .map(|(id, _)| id)
        .collect();

    Ok(enabled_ids)
}

// ─── Marketplace Commands ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMarketplacePluginSource {
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub url: String,
}

fn deserialize_source<'de, D>(deserializer: D) -> Result<Option<ClaudeMarketplacePluginSource>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Value = Deserialize::deserialize(deserializer)?;
    match value {
        Value::Object(_) => {
            let src: ClaudeMarketplacePluginSource = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
            Ok(Some(src))
        }
        Value::String(_) => Ok(None), // Ignore string sources (local paths)
        _ => Ok(None),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMarketplacePlugin {
    #[serde(rename = "pluginId")]
    pub plugin_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "marketplaceName", default)]
    pub marketplace_name: String,
    #[serde(rename = "installCount", default)]
    pub install_count: u64,
    #[serde(default, deserialize_with = "deserialize_source")]
    pub source: Option<ClaudeMarketplacePluginSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMarketplaceInstalled {
    pub id: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub scope: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(rename = "installPath", default)]
    pub install_path: String,
    #[serde(rename = "installedAt", default)]
    pub installed_at: String,
    #[serde(rename = "lastUpdated", default)]
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMarketplaceListOutput {
    pub installed: Vec<ClaudeMarketplaceInstalled>,
    pub available: Vec<ClaudeMarketplacePlugin>,
}

fn run_claude_plugin_cmd(args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("claude")
        .args(args)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "Claude CLI not found. Please ensure 'claude' is in your PATH.".to_string()
            } else {
                format!("Failed to execute claude CLI: {}", e)
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let msg = if stderr.trim().is_empty() {
            stdout.trim().to_string()
        } else {
            stderr.trim().to_string()
        };
        return Err(msg);
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[tauri::command]
pub fn get_claude_marketplace_plugins() -> Result<ClaudeMarketplaceListOutput, String> {
    let output = run_claude_plugin_cmd(&["plugin", "list", "--json", "--available"])?;
    serde_json::from_str(&output).map_err(|e| format!("Failed to parse CLI output: {}", e))
}

#[derive(Debug, Clone, Serialize)]
pub struct ClaudePluginUpdateInfo {
    pub id: String,
    pub current_version: String,
    pub has_update: bool,
}

#[tauri::command]
pub fn check_claude_plugin_updates() -> Result<Vec<ClaudePluginUpdateInfo>, String> {
    let home = dirs::home_dir().ok_or_else(|| "Failed to get user home directory".to_string())?;
    let installed_path = home
        .join(".claude")
        .join("plugins")
        .join("installed_plugins.json");
    let marketplace_path = home
        .join(".claude")
        .join("plugins")
        .join("marketplaces")
        .join("claude-plugins-official")
        .join(".claude-plugin")
        .join("marketplace.json");

    if !installed_path.exists() {
        return Ok(Vec::new());
    }

    let installed_text = std::fs::read_to_string(&installed_path).map_err(|e| e.to_string())?;
    let installed_json: Value =
        serde_json::from_str(&installed_text).map_err(|e| e.to_string())?;
    let Some(plugins_obj) = installed_json.get("plugins").and_then(|v| v.as_object()) else {
        return Ok(Vec::new());
    };

    // Build map: plugin_name -> marketplace SHA
    let mut mkt_shas: HashMap<String, String> = HashMap::new();
    if marketplace_path.exists() {
        let mkt_text = std::fs::read_to_string(&marketplace_path).map_err(|e| e.to_string())?;
        let mkt_json: Value = serde_json::from_str(&mkt_text).map_err(|e| e.to_string())?;
        if let Some(mkt_plugins) = mkt_json.get("plugins").and_then(|v| v.as_array()) {
            for p in mkt_plugins {
                let name = p.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let sha = p
                    .get("source")
                    .and_then(|s| s.as_object())
                    .and_then(|s| s.get("sha"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if !name.is_empty() && !sha.is_empty() {
                    mkt_shas.insert(name.to_string(), sha.to_string());
                }
            }
        }
    }

    // Also check version by reading marketplace plugin.json for internal plugins
    let mkt_plugins_dir = home
        .join(".claude")
        .join("plugins")
        .join("marketplaces")
        .join("claude-plugins-official")
        .join("plugins");
    let mkt_ext_dir = home
        .join(".claude")
        .join("plugins")
        .join("marketplaces")
        .join("claude-plugins-official")
        .join("external_plugins");

    let mut mkt_versions: HashMap<String, String> = HashMap::new();
    for dir in [&mkt_plugins_dir, &mkt_ext_dir] {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let manifest = entry.path().join(".claude-plugin").join("plugin.json");
                if manifest.exists() {
                    if let Ok(text) = std::fs::read_to_string(&manifest) {
                        if let Ok(json) = serde_json::from_str::<Value>(&text) {
                            let name =
                                json.get("name").and_then(|v| v.as_str()).unwrap_or("");
                            let version =
                                json.get("version").and_then(|v| v.as_str()).unwrap_or("");
                            if !name.is_empty() && !version.is_empty() {
                                mkt_versions.insert(name.to_string(), version.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    let mut result = Vec::new();
    for (plugin_id, entries) in plugins_obj {
        let Some(first) = entries.as_array().and_then(|a| a.first()) else {
            continue;
        };
        let current_version = first
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let installed_sha = first
            .get("gitCommitSha")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let name = plugin_id.split('@').next().unwrap_or(plugin_id);

        let has_update = if !installed_sha.is_empty() {
            // SHA-based comparison (external plugins)
            mkt_shas
                .get(name)
                .map(|mkt_sha| mkt_sha != installed_sha)
                .unwrap_or(false)
        } else if current_version != "unknown" {
            // Version-based comparison (internal plugins)
            mkt_versions
                .get(name)
                .map(|mkt_ver| mkt_ver != &current_version)
                .unwrap_or(false)
        } else {
            // unknown version, suggest update
            true
        };

        result.push(ClaudePluginUpdateInfo {
            id: plugin_id.clone(),
            current_version,
            has_update,
        });
    }

    Ok(result)
}

#[tauri::command]
pub fn install_claude_marketplace_plugin(pluginId: String) -> Result<(), String> {
    run_claude_plugin_cmd(&["plugin", "install", &pluginId, "-s", "user"])?;
    Ok(())
}

#[tauri::command]
pub fn uninstall_claude_marketplace_plugin(pluginId: String) -> Result<(), String> {
    run_claude_plugin_cmd(&["plugin", "uninstall", &pluginId, "-s", "user", "-y"])?;
    Ok(())
}

#[tauri::command]
pub fn update_claude_marketplace_plugin(pluginId: String) -> Result<(), String> {
    run_claude_plugin_cmd(&["plugin", "update", &pluginId])?;
    Ok(())
}

#[tauri::command]
pub fn refresh_claude_marketplace() -> Result<(), String> {
    run_claude_plugin_cmd(&["plugin", "marketplace", "update"])?;
    Ok(())
}
