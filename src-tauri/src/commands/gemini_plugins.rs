#![allow(non_snake_case)]

use crate::config::{get_home_dir, read_json_file, write_json_file};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// Gemini 已安装扩展信息
#[derive(Debug, Clone, Serialize)]
pub struct GeminiInstalledPlugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub enabled: bool,
}

/// gemini-extension.json manifest 结构
#[derive(Debug, Clone, Deserialize)]
struct GeminiExtensionManifest {
    name: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    description: Option<String>,
}

/// extension-enablement.json 中的单个扩展配置
#[derive(Debug, Clone, Deserialize, Serialize)]
struct EnablementEntry {
    #[serde(default)]
    overrides: Vec<String>,
}

/// 获取 Gemini 扩展目录路径
fn get_gemini_extensions_dir() -> PathBuf {
    get_home_dir().join(".gemini").join("extensions")
}

/// 获取 extension-enablement.json 路径
fn get_extension_enablement_path() -> PathBuf {
    get_gemini_extensions_dir().join("extension-enablement.json")
}

/// 读取 extension-enablement.json
fn read_extension_enablement() -> HashMap<String, EnablementEntry> {
    let path = get_extension_enablement_path();
    if !path.exists() {
        return HashMap::new();
    }
    read_json_file::<HashMap<String, EnablementEntry>>(&path).unwrap_or_default()
}

/// 判断扩展是否启用
///
/// 逻辑：
/// - 扩展不在 extension-enablement.json 中 → 默认启用
/// - 扩展在文件中，且 overrides 里有以 `!` 开头的条目 → 禁用
/// - 否则 → 启用
fn is_extension_enabled(
    extension_id: &str,
    enablement: &HashMap<String, EnablementEntry>,
    home_dir: &str,
) -> bool {
    if let Some(entry) = enablement.get(extension_id) {
        // 如果有任何以 ! 开头的 override，认为该扩展被禁用
        // 这是简化模型，覆盖大多数用户场景
        for override_pattern in &entry.overrides {
            if override_pattern.starts_with('!') && override_pattern.contains(home_dir) {
                return false;
            }
        }
    }
    true
}

/// 读取 Gemini 已安装扩展列表
#[tauri::command]
pub fn get_gemini_installed_plugins() -> Result<Vec<GeminiInstalledPlugin>, String> {
    let extensions_dir = get_gemini_extensions_dir();
    let enablement = read_extension_enablement();
    let home_dir = get_home_dir()
        .to_str()
        .unwrap_or("/")
        .to_string();

    let mut plugins = Vec::new();

    if !extensions_dir.exists() {
        return Ok(plugins);
    }

    let entries = fs::read_dir(&extensions_dir).map_err(|e| e.to_string())?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        // 跳过非扩展目录（如 extension-enablement.json 所在的目录本身不是扩展）
        let manifest_path = path.join("gemini-extension.json");
        if !manifest_path.exists() {
            continue;
        }

        let manifest: GeminiExtensionManifest =
            read_json_file(&manifest_path).map_err(|e| e.to_string())?;

        let extension_id = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&manifest.name)
            .to_string();

        let enabled = is_extension_enabled(&extension_id, &enablement, &home_dir);

        plugins.push(GeminiInstalledPlugin {
            id: extension_id,
            name: manifest.name,
            version: manifest.version,
            description: manifest.description,
            enabled,
        });
    }

    plugins.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(plugins)
}

/// 批量应用 Gemini 扩展启用选择
#[tauri::command]
pub fn apply_gemini_plugin_selection(enabledIds: Vec<String>) -> Result<(), String> {
    let mut enablement = read_extension_enablement();
    let home_dir = get_home_dir()
        .to_str()
        .unwrap_or("/")
        .to_string();
    let disable_pattern = format!("!{}/*", home_dir);

    let enabled_set: HashSet<String> = enabledIds.into_iter().collect();

    // 获取所有已安装扩展的 ID
    let extensions_dir = get_gemini_extensions_dir();
    let mut all_installed: Vec<String> = Vec::new();
    if extensions_dir.exists() {
        if let Ok(entries) = fs::read_dir(&extensions_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.join("gemini-extension.json").exists() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        all_installed.push(name.to_string());
                    }
                }
            }
        }
    }

    for id in all_installed {
        if enabled_set.contains(&id) {
            // 启用：从 enablement 中移除该扩展条目（恢复默认启用）
            enablement.remove(&id);
        } else {
            // 禁用：添加禁用规则
            enablement.insert(
                id,
                EnablementEntry {
                    overrides: vec![disable_pattern.clone()],
                },
            );
        }
    }

    // 如果 enablement 为空，删除文件；否则写入
    let path = get_extension_enablement_path();
    if enablement.is_empty() {
        if path.exists() {
            fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
    } else {
        write_json_file(&path, &enablement).map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// 从现有 Gemini 配置导入扩展启用状态
///
/// 返回当前已启用的扩展 ID 列表
#[tauri::command]
pub fn import_gemini_plugins_from_live() -> Result<Vec<String>, String> {
    let plugins = get_gemini_installed_plugins()?;
    let enabled: Vec<String> = plugins
        .into_iter()
        .filter(|p| p.enabled)
        .map(|p| p.id)
        .collect();
    Ok(enabled)
}
