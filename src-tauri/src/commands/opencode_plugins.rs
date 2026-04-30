#![allow(non_snake_case)]

use crate::database::OpenCodePluginEntry;
use crate::opencode_config::{canonicalize_plugin_name, sync_opencode_plugins};
use crate::store::AppState;
use tauri::State;

/// 获取所有 OpenCode 插件
#[tauri::command]
pub fn get_opencode_plugins(
    state: State<'_, AppState>,
) -> Result<Vec<OpenCodePluginEntry>, String> {
    state.db.get_opencode_plugins().map_err(|e| e.to_string())
}

/// 添加 OpenCode 插件
#[tauri::command]
pub fn add_opencode_plugin(state: State<'_, AppState>, name: String) -> Result<(), String> {
    let normalized = canonicalize_plugin_name(&name);
    state
        .db
        .add_opencode_plugin(&normalized, &name)
        .map_err(|e| e.to_string())?;

    // 同步到 live config
    let names = state
        .db
        .get_opencode_plugin_names()
        .map_err(|e| e.to_string())?;
    sync_opencode_plugins(&names).map_err(|e| e.to_string())?;

    Ok(())
}

/// 删除 OpenCode 插件
#[tauri::command]
pub fn remove_opencode_plugin(
    state: State<'_, AppState>,
    normalizedName: String,
) -> Result<(), String> {
    state
        .db
        .remove_opencode_plugin(&normalizedName)
        .map_err(|e| e.to_string())?;

    let names = state
        .db
        .get_opencode_plugin_names()
        .map_err(|e| e.to_string())?;
    sync_opencode_plugins(&names).map_err(|e| e.to_string())?;

    Ok(())
}

/// 重新排序 OpenCode 插件
#[tauri::command]
pub fn reorder_opencode_plugins(
    state: State<'_, AppState>,
    names: Vec<String>,
) -> Result<(), String> {
    state
        .db
        .reorder_opencode_plugins(&names)
        .map_err(|e| e.to_string())?;

    sync_opencode_plugins(&names).map_err(|e| e.to_string())?;

    Ok(())
}

/// 从现有 opencode.json 导入插件
#[tauri::command]
pub fn import_opencode_plugins_from_live(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    let path = home.join(".config").join("opencode").join("opencode.json");

    if !path.exists() {
        return Ok(Vec::new());
    }

    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let json: serde_json::Value =
        json5::from_str(&text).map_err(|e| format!("解析 opencode.json 失败: {e}"))?;

    let Some(arr) = json.get("plugin").and_then(|v| v.as_array()) else {
        return Ok(Vec::new());
    };

    let mut imported = Vec::new();
    for plugin in arr {
        if let Some(name) = plugin.as_str() {
            let normalized = canonicalize_plugin_name(name);
            state
                .db
                .add_opencode_plugin(&normalized, name)
                .map_err(|e| e.to_string())?;
            imported.push(normalized);
        }
    }

    // 导入后同步到 opencode.json
    let names = state
        .db
        .get_opencode_plugin_names()
        .map_err(|e| e.to_string())?;
    sync_opencode_plugins(&names).map_err(|e| e.to_string())?;

    Ok(imported)
}
