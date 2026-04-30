//! Claude 全局插件数据访问对象
//!
//! 提供跨 Provider 生效的 Claude 插件管理。
//! 这些插件在写入 live config 时合并到 provider settings 的 enabledPlugins 中，
//! 在 backfill 回写时从 live config 中剥离。

use crate::database::{lock_conn, Database};
use crate::error::AppError;
use rusqlite::params;
use std::collections::HashMap;

impl Database {
    /// 获取所有 Claude 全局插件（plugin_id → enabled）
    pub fn get_claude_global_plugins(&self) -> Result<HashMap<String, bool>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT plugin_id, enabled FROM claude_global_plugins ORDER BY plugin_id ASC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let enabled: bool = row.get(1)?;
                Ok((id, enabled))
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut plugins = HashMap::new();
        for row in rows {
            let (id, enabled) = row.map_err(|e| AppError::Database(e.to_string()))?;
            plugins.insert(id, enabled);
        }
        Ok(plugins)
    }

    /// 设置 Claude 全局插件（插入或更新）
    pub fn set_claude_global_plugin(
        &self,
        plugin_id: &str,
        enabled: bool,
    ) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        conn.execute(
            "INSERT OR REPLACE INTO claude_global_plugins (plugin_id, enabled, created_at) VALUES (?1, ?2, ?3)",
            params![plugin_id, enabled, now],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// 删除 Claude 全局插件
    pub fn remove_claude_global_plugin(&self, plugin_id: &str) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute(
            "DELETE FROM claude_global_plugins WHERE plugin_id = ?1",
            params![plugin_id],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }
}
