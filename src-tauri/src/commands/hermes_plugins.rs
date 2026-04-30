#![allow(non_snake_case)]

use crate::hermes_config::{get_hermes_config_path, get_hermes_dir, read_hermes_config};
use serde::Serialize;
use std::collections::HashSet;

/// Hermes 已安装插件信息
#[derive(Debug, Clone, Serialize)]
pub struct HermesInstalledPlugin {
    pub id: String,
    pub enabled: bool,
}

/// 读取 Hermes 已安装插件列表（扫描 ~/.hermes/plugins/ 目录）
/// 并合并 config.yaml 中的 plugins.disabled 状态
#[tauri::command]
pub fn get_hermes_installed_plugins() -> Result<Vec<HermesInstalledPlugin>, String> {
    let plugins_dir = get_hermes_dir().join("plugins");
    let config = read_hermes_config().map_err(|e| e.to_string())?;

    // 读取禁用列表
    let disabled: HashSet<String> = config
        .get("plugins")
        .and_then(|v| v.as_mapping())
        .and_then(|m| m.get(&serde_yaml::Value::String("disabled".to_string())))
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    // 扫描插件目录
    let mut plugins = Vec::new();
    if plugins_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&plugins_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        plugins.push(HermesInstalledPlugin {
                            id: name.to_string(),
                            enabled: !disabled.contains(name),
                        });
                    }
                }
            }
        }
    }

    // 按名称排序
    plugins.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(plugins)
}

/// 批量应用 Hermes 插件启用选择
/// enabledIds 为应启用的插件列表，不在其中的将被加入 plugins.disabled
#[tauri::command]
pub fn apply_hermes_plugin_selection(enabledIds: Vec<String>) -> Result<(), String> {
    let config_path = get_hermes_config_path();
    let raw = if config_path.exists() {
        std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?
    } else {
        String::new()
    };

    let config = read_hermes_config().map_err(|e| e.to_string())?;

    // 获取所有已安装插件
    let plugins_dir = get_hermes_dir().join("plugins");
    let mut all_installed: Vec<String> = Vec::new();
    if plugins_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&plugins_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        all_installed.push(name.to_string());
                    }
                }
            }
        }
    }

    let enabled_set: HashSet<String> = enabledIds.into_iter().collect();

    // 计算新的 disabled 列表：已安装但未启用的
    let new_disabled: Vec<serde_yaml::Value> = all_installed
        .into_iter()
        .filter(|id| !enabled_set.contains(id))
        .map(|id| serde_yaml::Value::String(id))
        .collect();

    // 构建 plugins section
    let mut plugins_mapping = config
        .get("plugins")
        .and_then(|v| v.as_mapping())
        .cloned()
        .unwrap_or_default();

    if new_disabled.is_empty() {
        plugins_mapping.remove(&serde_yaml::Value::String("disabled".to_string()));
    } else {
        plugins_mapping.insert(
            serde_yaml::Value::String("disabled".to_string()),
            serde_yaml::Value::Sequence(new_disabled),
        );
    }

    // 写入 config.yaml 的 plugins section
    let plugins_value = serde_yaml::Value::Mapping(plugins_mapping);
    let new_raw = crate::hermes_config::replace_yaml_section(&raw, "plugins", &plugins_value)
        .map_err(|e| e.to_string())?;

    crate::config::atomic_write(&config_path, new_raw.as_bytes()).map_err(|e| e.to_string())?;

    Ok(())
}

/// 从现有 Hermes config.yaml 导入插件禁用状态
#[tauri::command]
pub fn import_hermes_plugins_from_live() -> Result<Vec<String>, String> {
    let config = read_hermes_config().map_err(|e| e.to_string())?;

    let disabled: Vec<String> = config
        .get("plugins")
        .and_then(|v| v.as_mapping())
        .and_then(|m| m.get(&serde_yaml::Value::String("disabled".to_string())))
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    Ok(disabled)
}
