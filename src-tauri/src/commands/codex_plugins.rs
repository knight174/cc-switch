#![allow(non_snake_case)]

use crate::codex_config::{get_codex_config_path, read_codex_config_text};
use serde::Serialize;
use std::collections::HashSet;

/// Codex 已安装插件信息
#[derive(Debug, Clone, Serialize)]
pub struct CodexInstalledPlugin {
    pub id: String,
    pub enabled: bool,
}

/// 读取 Codex 已安装插件列表（从 config.toml 的 [plugins."id"]）
#[tauri::command]
pub fn get_codex_installed_plugins() -> Result<Vec<CodexInstalledPlugin>, String> {
    let path = get_codex_config_path();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let text = read_codex_config_text().map_err(|e| e.to_string())?;
    if text.trim().is_empty() {
        return Ok(Vec::new());
    }

    let doc: toml::Table = toml::from_str(&text).map_err(|e| e.to_string())?;

    let mut plugins = Vec::new();
    if let Some(plugins_table) = doc.get("plugins").and_then(|v| v.as_table()) {
        for (plugin_id, value) in plugins_table {
            let enabled = value
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            plugins.push(CodexInstalledPlugin {
                id: plugin_id.clone(),
                enabled,
            });
        }
    }

    plugins.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(plugins)
}

/// 批量应用 Codex 插件启用选择
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

/// 从现有 Codex config.toml 导入插件
#[tauri::command]
pub fn import_codex_plugins_from_live() -> Result<Vec<String>, String> {
    let path = get_codex_config_path();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let text = read_codex_config_text().map_err(|e| e.to_string())?;
    if text.trim().is_empty() {
        return Ok(Vec::new());
    }

    let doc: toml::Table = toml::from_str(&text).map_err(|e| e.to_string())?;

    let mut imported = Vec::new();
    if let Some(plugins_table) = doc.get("plugins").and_then(|v| v.as_table()) {
        for (plugin_id, value) in plugins_table {
            let enabled = value
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            if enabled {
                imported.push(plugin_id.clone());
            }
        }
    }

    Ok(imported)
}
