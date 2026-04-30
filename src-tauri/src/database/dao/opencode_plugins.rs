//! OpenCode 插件数据访问对象
//!
//! OpenCode 插件直接写入 opencode.json 顶层的 `plugin` 数组，
//! 与 Provider 配置无关。

use crate::database::{lock_conn, Database};
use crate::error::AppError;
use rusqlite::params;

/// OpenCode 插件条目
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct OpenCodePluginEntry {
    pub normalized_name: String,
    pub display_name: String,
    pub created_at: i64,
}

impl Database {
    /// 获取所有 OpenCode 插件（按添加顺序）
    pub fn get_opencode_plugins(&self) -> Result<Vec<OpenCodePluginEntry>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare("SELECT normalized_name, display_name, created_at FROM opencode_plugins ORDER BY created_at ASC, normalized_name ASC")
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(OpenCodePluginEntry {
                    normalized_name: row.get(0)?,
                    display_name: row.get(1)?,
                    created_at: row.get(2)?,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut plugins = Vec::new();
        for row in rows {
            plugins.push(row.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(plugins)
    }

    /// 获取所有 OpenCode 插件名称（有序，用于同步）
    pub fn get_opencode_plugin_names(&self) -> Result<Vec<String>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare("SELECT normalized_name FROM opencode_plugins ORDER BY created_at ASC, normalized_name ASC")
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut names = Vec::new();
        for row in rows {
            names.push(row.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(names)
    }

    /// 添加 OpenCode 插件
    pub fn add_opencode_plugin(
        &self,
        normalized_name: &str,
        display_name: &str,
    ) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        conn.execute(
            "INSERT OR REPLACE INTO opencode_plugins (normalized_name, display_name, created_at) VALUES (?1, ?2, ?3)",
            params![normalized_name, display_name, now],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// 删除 OpenCode 插件
    pub fn remove_opencode_plugin(&self, normalized_name: &str) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute(
            "DELETE FROM opencode_plugins WHERE normalized_name = ?1",
            params![normalized_name],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// 重新排序 OpenCode 插件（删除后按新顺序重建）
    pub fn reorder_opencode_plugins(&self, names: &[String]) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // 使用事务保证原子性
        conn.execute("BEGIN", [])
            .map_err(|e| AppError::Database(e.to_string()))?;

        // 更新所有行的 created_at 为一个不可能的大范围，避免冲突
        conn.execute(
            "UPDATE opencode_plugins SET created_at = created_at + 1000000000",
            [],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        for (index, name) in names.iter().enumerate() {
            let new_created_at = now + index as i64;
            conn.execute(
                "UPDATE opencode_plugins SET created_at = ?1 WHERE normalized_name = ?2",
                params![new_created_at, name],
            )
            .map_err(|e| AppError::Database(e.to_string()))?;
        }

        conn.execute("COMMIT", [])
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }
}
