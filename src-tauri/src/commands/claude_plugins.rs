#![allow(non_snake_case)]

use crate::app_config::AppType;
use crate::store::AppState;
use serde::Serialize;
use std::collections::HashMap;
use tauri::State;

/// 已安装的 Claude 插件信息
#[derive(Debug, Clone, Serialize)]
pub struct ClaudeInstalledPlugin {
    pub id: String,
    pub version: String,
    pub scope: String,
    pub installed_at: String,
}

/// 同步当前 Claude provider 到 live config（辅助函数）
fn sync_claude_provider_after_plugin_change(state: &State<'_, AppState>) {
    if let Err(e) = crate::services::provider::sync_current_provider_for_app_to_live(
        state,
        &AppType::Claude,
    ) {
        log::warn!("Failed to sync Claude provider after plugin change: {e}");
    }
}

/// 获取所有 Claude 全局插件
#[tauri::command]
pub fn get_claude_global_plugins(
    state: State<'_, AppState>,
) -> Result<HashMap<String, bool>, String> {
    state
        .db
        .get_claude_global_plugins()
        .map_err(|e| e.to_string())
}

/// 设置 Claude 全局插件（插入或更新）
#[tauri::command]
pub fn set_claude_global_plugin(
    state: State<'_, AppState>,
    pluginId: String,
    enabled: bool,
) -> Result<(), String> {
    state
        .db
        .set_claude_global_plugin(&pluginId, enabled)
        .map_err(|e| e.to_string())?;
    sync_claude_provider_after_plugin_change(&state);
    Ok(())
}

/// 删除 Claude 全局插件
#[tauri::command]
pub fn remove_claude_global_plugin(
    state: State<'_, AppState>,
    pluginId: String,
) -> Result<(), String> {
    state
        .db
        .remove_claude_global_plugin(&pluginId)
        .map_err(|e| e.to_string())?;
    sync_claude_provider_after_plugin_change(&state);
    Ok(())
}

/// 从 Claude Code 安装目录读取已安装插件列表
#[tauri::command]
pub fn get_claude_installed_plugins() -> Result<Vec<ClaudeInstalledPlugin>, String> {
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    let path = home.join(".claude").join("plugins").join("installed_plugins.json");

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
        let Some(arr) = entries.as_array() else { continue };
        let Some(first) = arr.first() else { continue };
        let Some(entry) = first.as_object() else { continue };

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

/// 批量应用 Claude 插件启用选择（从已安装列表中选择要启用的插件）
#[tauri::command]
pub fn apply_claude_plugin_selection(
    state: State<'_, AppState>,
    enabledIds: Vec<String>,
) -> Result<(), String> {
    // 1. 清空现有全局插件表
    let current = state
        .db
        .get_claude_global_plugins()
        .map_err(|e| e.to_string())?;
    for (id, _) in current {
        state
            .db
            .remove_claude_global_plugin(&id)
            .map_err(|e| e.to_string())?;
    }

    // 2. 写入新选择的插件（全部 enabled = true）
    for id in enabledIds {
        state
            .db
            .set_claude_global_plugin(&id, true)
            .map_err(|e| e.to_string())?;
    }

    sync_claude_provider_after_plugin_change(&state);
    Ok(())
}

/// 读取已安装的 Claude 插件 ID 集合（以 installed_plugins.json 为准）
pub(crate) fn get_installed_claude_plugin_ids() -> Result<std::collections::HashSet<String>, String> {
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    let path = home.join(".claude").join("plugins").join("installed_plugins.json");

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
#[tauri::command]
pub fn import_claude_plugins_from_live(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;

    // 1. 读取已安装插件（唯一真实来源）
    let installed = get_installed_claude_plugin_ids()?;

    // 2. 读取 settings.json 的 enabledPlugins（参考来源，可能不准）
    let settings_path = home.join(".claude").join("settings.json");
    let enabled_in_settings: HashMap<String, bool> = if settings_path.exists() {
        let text = std::fs::read_to_string(&settings_path).map_err(|e| e.to_string())?;
        let json: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        json.get("enabledPlugins")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.as_bool().unwrap_or(true)))
                    .collect()
            })
            .unwrap_or_default()
    } else {
        HashMap::new()
    };

    // 3. 清理全局表中的 ghost 条目（未安装但存在于全局表）
    let current_global = state
        .db
        .get_claude_global_plugins()
        .map_err(|e| e.to_string())?;
    for (id, _) in &current_global {
        if !installed.contains(id) {
            state
                .db
                .remove_claude_global_plugin(id)
                .map_err(|e| e.to_string())?;
        }
    }

    // 4. 只导入已安装的插件
    let mut imported = Vec::new();
    for plugin_id in &installed {
        let enabled = enabled_in_settings.get(plugin_id).copied().unwrap_or(false);
        state
            .db
            .set_claude_global_plugin(plugin_id, enabled)
            .map_err(|e| e.to_string())?;
        if enabled {
            imported.push(plugin_id.clone());
        }
    }

    sync_claude_provider_after_plugin_change(&state);
    Ok(imported)
}
