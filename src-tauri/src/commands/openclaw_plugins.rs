#![allow(non_snake_case)]

use crate::openclaw_config::read_openclaw_config;
use serde::Serialize;

/// OpenClaw 已安装插件信息
#[derive(Debug, Clone, Serialize)]
pub struct OpenClawInstalledPlugin {
    pub id: String,
    pub enabled: bool,
}

/// 读取 OpenClaw 已安装插件列表（从 openclaw.json 的 plugins.entries）
#[tauri::command]
pub fn get_openclaw_installed_plugins() -> Result<Vec<OpenClawInstalledPlugin>, String> {
    let config = read_openclaw_config().map_err(|e| e.to_string())?;

    let entries = config
        .get("plugins")
        .and_then(|v| v.as_object())
        .and_then(|obj| obj.get("entries"))
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    let mut plugins = Vec::new();
    for (id, entry) in entries {
        let enabled = entry
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        plugins.push(OpenClawInstalledPlugin { id, enabled });
    }

    plugins.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(plugins)
}

/// 批量应用 OpenClaw 插件启用选择
///
/// 勾选逻辑：用户勾选 → 将 `plugins.entries.<id>.enabled` 设为 true。
/// 取消勾选 → 将同一字段设为 false。
/// 仅修改已存在于 plugins.entries 中的插件，不会新增或删除条目。
#[tauri::command]
pub fn apply_openclaw_plugin_selection(
    enabledIds: Vec<String>,
) -> Result<(), String> {
    let config = read_openclaw_config().map_err(|e| e.to_string())?;

    let mut entries = config
        .get("plugins")
        .and_then(|v| v.as_object())
        .and_then(|obj| obj.get("entries"))
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    let enabled_set: std::collections::HashSet<String> = enabledIds.into_iter().collect();

    for (id, entry_val) in &mut entries {
        let enabled = enabled_set.contains(id);
        if let Some(entry_obj) = entry_val.as_object_mut() {
            entry_obj.insert("enabled".to_string(), serde_json::Value::Bool(enabled));
        }
    }

    crate::openclaw_config::set_plugins_entries(&serde_json::Value::Object(entries))
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// 从现有 OpenClaw 配置导入插件
#[tauri::command]
pub fn import_openclaw_plugins_from_live() -> Result<Vec<String>, String> {
    let config = read_openclaw_config().map_err(|e| e.to_string())?;

    let entries = config
        .get("plugins")
        .and_then(|v| v.as_object())
        .and_then(|obj| obj.get("entries"))
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    let mut imported = Vec::new();
    for (id, entry) in entries {
        let enabled = entry
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        if enabled {
            imported.push(id);
        }
    }

    Ok(imported)
}
