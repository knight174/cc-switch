#![allow(non_snake_case)]

use crate::opencode_config::{
    canonicalize_plugin_name, read_opencode_config, write_opencode_config,
};
use serde_json::{json, Value};

/// OpenCode 插件条目
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct OpenCodePluginEntry {
    pub normalized_name: String,
    pub display_name: String,
    pub created_at: i64,
}

/// 从 opencode.json 读取 plugin 数组
fn read_opencode_plugins_from_live() -> Result<Vec<OpenCodePluginEntry>, String> {
    let config = read_opencode_config().map_err(|e| e.to_string())?;

    let Some(arr) = config.get("plugin").and_then(|v| v.as_array()) else {
        return Ok(Vec::new());
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let mut plugins = Vec::new();
    for (index, plugin) in arr.iter().enumerate() {
        if let Some(name) = plugin.as_str() {
            let normalized = canonicalize_plugin_name(name);
            plugins.push(OpenCodePluginEntry {
                normalized_name: normalized,
                display_name: name.to_string(),
                created_at: now + index as i64, // 用索引模拟顺序
            });
        }
    }

    Ok(plugins)
}

/// 获取所有 OpenCode 插件（从 live config 直接读取）
#[tauri::command]
pub fn get_opencode_plugins() -> Result<Vec<OpenCodePluginEntry>, String> {
    read_opencode_plugins_from_live()
}

/// 添加 OpenCode 插件（直接写入 opencode.json）
#[tauri::command]
pub fn add_opencode_plugin(name: String) -> Result<(), String> {
    let normalized = canonicalize_plugin_name(&name);

    let mut config = read_opencode_config().map_err(|e| e.to_string())?;
    let plugins = config.get_mut("plugin").and_then(|v| v.as_array_mut());

    match plugins {
        Some(arr) => {
            // OMO 互斥逻辑：standard OMO 和 OMO Slim 不能共存
            const STANDARD_OMO_PREFIXES: [&str; 2] = ["oh-my-openagent", "oh-my-opencode"];
            const SLIM_OMO_PREFIXES: [&str; 1] = ["oh-my-opencode-slim"];

            fn matches_prefix(plugin_name: &str, prefix: &str) -> bool {
                plugin_name == prefix
                    || plugin_name
                        .strip_prefix(prefix)
                        .map(|suffix| suffix.starts_with('@'))
                        .unwrap_or(false)
            }

            fn matches_any_prefix(plugin_name: &str, prefixes: &[&str]) -> bool {
                prefixes.iter().any(|p| matches_prefix(plugin_name, p))
            }

            if matches_any_prefix(&normalized, &STANDARD_OMO_PREFIXES) {
                arr.retain(|v| {
                    v.as_str()
                        .map(|s| {
                            !matches_any_prefix(s, &STANDARD_OMO_PREFIXES)
                                && !matches_any_prefix(s, &SLIM_OMO_PREFIXES)
                        })
                        .unwrap_or(true)
                });
            } else if matches_any_prefix(&normalized, &SLIM_OMO_PREFIXES) {
                arr.retain(|v| {
                    v.as_str()
                        .map(|s| {
                            !matches_any_prefix(s, &STANDARD_OMO_PREFIXES)
                                && !matches_any_prefix(s, &SLIM_OMO_PREFIXES)
                        })
                        .unwrap_or(true)
                });
            }

            let already_exists = arr.iter().any(|v| v.as_str() == Some(normalized.as_str()));
            if !already_exists {
                arr.push(Value::String(normalized));
            }
        }
        None => {
            config["plugin"] = json!([normalized]);
        }
    }

    write_opencode_config(&config).map_err(|e| e.to_string())?;
    Ok(())
}

/// 删除 OpenCode 插件（直接从 opencode.json 移除）
#[tauri::command]
pub fn remove_opencode_plugin(normalizedName: String) -> Result<(), String> {
    let mut config = read_opencode_config().map_err(|e| e.to_string())?;

    if let Some(arr) = config.get_mut("plugin").and_then(|v| v.as_array_mut()) {
        arr.retain(|v| v.as_str() != Some(&normalizedName));

        if arr.is_empty() {
            config.as_object_mut().map(|obj| obj.remove("plugin"));
        }
    }

    write_opencode_config(&config).map_err(|e| e.to_string())?;
    Ok(())
}

/// 重新排序 OpenCode 插件（直接重写 opencode.json 的 plugin 数组）
#[tauri::command]
pub fn reorder_opencode_plugins(names: Vec<String>) -> Result<(), String> {
    let mut config = read_opencode_config().map_err(|e| e.to_string())?;

    if names.is_empty() {
        config.as_object_mut().map(|obj| obj.remove("plugin"));
    } else {
        // 应用 OMO 互斥：如果列表中同时存在 standard 和 slim OMO，只保留第一个遇到的 OMO 类型
        const STANDARD_OMO_PREFIXES: [&str; 2] = ["oh-my-openagent", "oh-my-opencode"];
        const SLIM_OMO_PREFIXES: [&str; 1] = ["oh-my-opencode-slim"];

        fn matches_prefix(plugin_name: &str, prefix: &str) -> bool {
            plugin_name == prefix
                || plugin_name
                    .strip_prefix(prefix)
                    .map(|suffix| suffix.starts_with('@'))
                    .unwrap_or(false)
        }

        fn matches_any_prefix(plugin_name: &str, prefixes: &[&str]) -> bool {
            prefixes.iter().any(|p| matches_prefix(plugin_name, p))
        }

        let mut found_omo = false;
        let filtered: Vec<String> = names
            .into_iter()
            .filter(|name| {
                let is_standard = matches_any_prefix(name, &STANDARD_OMO_PREFIXES);
                let is_slim = matches_any_prefix(name, &SLIM_OMO_PREFIXES);
                if is_standard || is_slim {
                    if found_omo {
                        false
                    } else {
                        found_omo = true;
                        true
                    }
                } else {
                    true
                }
            })
            .collect();

        let arr: Vec<Value> = filtered.into_iter().map(Value::String).collect();
        config["plugin"] = Value::Array(arr);
    }

    write_opencode_config(&config).map_err(|e| e.to_string())?;
    Ok(())
}

/// 从现有 opencode.json 导入插件
#[tauri::command]
pub fn import_opencode_plugins_from_live() -> Result<Vec<String>, String> {
    let plugins = read_opencode_plugins_from_live()?;
    let names: Vec<String> = plugins.into_iter().map(|p| p.normalized_name).collect();
    Ok(names)
}
