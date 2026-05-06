#![allow(non_snake_case)]

use crate::config::{get_claude_settings_path, read_json_file, write_json_file};
use serde::Serialize;
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

/// 批量应用 Claude 插件启用选择（直接写入 settings.json）
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
