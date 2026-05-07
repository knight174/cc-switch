#![allow(non_snake_case)]

use crate::codex_config::{get_codex_config_path, read_codex_config_text};
use crate::config::{get_home_dir, read_json_file};
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;

/// Codex 已安装插件信息
#[derive(Debug, Clone, Serialize)]
pub struct CodexInstalledPlugin {
    pub id: String,
    pub enabled: bool,
}

/// 读取 Codex marketplace.json 中的插件 ID 列表
///
/// marketplace.json 格式：
/// ```json
/// {
///   "plugins": [
///     { "name": "plugin-eval", ... }
///   ]
/// }
/// ```
fn read_marketplace_plugins() -> Vec<String> {
    let path = get_home_dir()
        .join(".agents")
        .join("plugins")
        .join("marketplace.json");
    if !path.exists() {
        return Vec::new();
    }

    let json: serde_json::Value = match read_json_file(&path) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let mut plugins = Vec::new();
    if let Some(plugins_arr) = json.get("plugins").and_then(|v| v.as_array()) {
        for plugin in plugins_arr {
            if let Some(name) = plugin.get("name").and_then(|v| v.as_str()) {
                plugins.push(name.to_string());
            }
        }
    }

    plugins
}

/// 读取 Codex 已安装插件列表
///
/// 合并两个来源：
/// 1. config.toml 的 `[plugins."id"]` —— 权威启用状态来源
/// 2. marketplace.json 的 `plugins[].name` —— 补充 config.toml 中未列出的插件
///
/// 若插件只在 marketplace.json 中存在，默认视为启用（true）。
#[tauri::command]
pub fn get_codex_installed_plugins() -> Result<Vec<CodexInstalledPlugin>, String> {
    let path = get_codex_config_path();

    // 1. 读取 config.toml 的 [plugins]
    let mut plugins_map: HashMap<String, bool> = HashMap::new();
    if path.exists() {
        let text = read_codex_config_text().map_err(|e| e.to_string())?;
        if !text.trim().is_empty() {
            let doc: toml::Table = toml::from_str(&text).map_err(|e| e.to_string())?;
            if let Some(plugins_table) = doc.get("plugins").and_then(|v| v.as_table()) {
                for (plugin_id, value) in plugins_table {
                    let enabled = value
                        .get("enabled")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true);
                    plugins_map.insert(plugin_id.clone(), enabled);
                }
            }
        }
    }

    // 2. 读取 marketplace.json，补充 config.toml 中不存在的插件
    let marketplace_plugins = read_marketplace_plugins();
    for plugin_id in marketplace_plugins {
        plugins_map.entry(plugin_id).or_insert(true);
    }

    // 3. 构建返回列表
    let mut plugins: Vec<CodexInstalledPlugin> = plugins_map
        .into_iter()
        .map(|(id, enabled)| CodexInstalledPlugin { id, enabled })
        .collect();

    plugins.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(plugins)
}

/// 批量应用 Codex 插件启用选择
///
/// 勾选逻辑：用户勾选 → 将 `[plugins."id"] enabled = true` 写入 config.toml。
/// 取消勾选 → 将同一 table 的 enabled 设为 false。
/// 仅修改已存在于 config.toml [plugins] 下的插件，不会新增或删除插件条目。
#[tauri::command]
pub fn apply_codex_plugin_selection(enabledIds: Vec<String>) -> Result<(), String> {
    let path = get_codex_config_path();
    let text = if path.exists() {
        read_codex_config_text().map_err(|e| e.to_string())?
    } else {
        String::new()
    };

    let enabled_set: HashSet<String> = enabledIds.into_iter().collect();

    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| e.to_string())?;

    // 遍历 plugins table，更新 enabled
    if let Some(plugins) = doc.get_mut("plugins").and_then(|v| v.as_table_like_mut()) {
        for (plugin_id, item) in plugins.iter_mut() {
            let enabled = enabled_set.contains(plugin_id.get());
            if let Some(table) = item.as_table_like_mut() {
                table.insert("enabled", toml_edit::value(enabled));
            }
        }
    }

    crate::config::atomic_write(&path, doc.to_string().as_bytes())
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// 从现有 Codex 配置导入插件
///
/// 复用 get_codex_installed_plugins() 的合并逻辑，返回当前启用的插件 ID 列表。
#[tauri::command]
pub fn import_codex_plugins_from_live() -> Result<Vec<String>, String> {
    let plugins = get_codex_installed_plugins()?;
    let enabled: Vec<String> = plugins
        .into_iter()
        .filter(|p| p.enabled)
        .map(|p| p.id)
        .collect();
    Ok(enabled)
}
