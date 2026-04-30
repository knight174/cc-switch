# Provider 插件系统调研报告（2026-04）

> 调研范围：Claude / Codex / Hermes / OpenClaw / OpenCode
> 目的：验证 CC Switch PluginManager v3 中各 Provider 的实现假设是否与最新现实一致

---

## 1. 执行摘要

| Provider | 状态 | 关键发现 |
|---|---|---|
| Claude | ✅ 基本正确 | 只管理 user scope，project/local scope 不在范围内（合理） |
| **Codex** | ⚠️ 需确认 | 插件可能通过 `~/.agents/plugins/marketplace.json` 管理，`config.toml` 的 `[plugins]` 可能不完整 |
| **Hermes** | ❌ 过时 | 已从"默认启用"变为"opt-in"，新增 `plugins.enabled` 列表 |
| **OpenClaw** | ⚠️ 需补充 | `plugins/installs.json` 存储安装元数据；`plugins.deny` 优先于 `allow` |
| OpenCode | ✅ 基本正确 | 未发现变化 |

---

## 2. 逐项详情

### 2.1 Claude

**现有实现**：
- 读取 `~/.claude/plugins/installed_plugins.json` 获取已安装列表
- 读取 `~/.claude/settings.json` 的 `enabledPlugins` 获取启用状态
- 只管理 user scope 的启用状态

**调研发现**：
- ✅ `installed_plugins.json` 路径和格式确认正确
- ✅ `enabledPlugins: { "plugin@marketplace": bool }` 格式确认正确
- ⚠️ Claude 支持四层 scope：`user` / `project` / `local` / `managed`
- ⚠️ 已知 bug：`settings.local.json` 的 `enabledPlugins` 需要 `settings.json` 中先有同名键才生效

**影响评估**：低。CC Switch 管理全局（user scope）插件的策略是合理边界。Project scope 的插件由项目团队成员共享，不应由 CC Switch 全局管理。

---

### 2.2 Codex

**现有实现**：
- 直接从 `~/.codex/config.toml` 的 `[plugins."id"]` 读取已安装插件和启用状态
- 假设 `config.toml` 就是插件的"安装注册表"

**调研发现**：
- ✅ `config.toml` 确实有 `[plugins]` section，格式为 `[plugins."id"] enabled = false`
- ⚠️ Codex 有 **marketplace 机制**：`~/.agents/plugins/marketplace.json`
  ```json
  {
    "name": "local",
    "interface": { "displayName": "Local Plugins" },
    "plugins": [
      {
        "name": "plugin-eval",
        "source": { "source": "local", "path": "./plugins/plugin-eval" },
        "policy": { "installation": "INSTALLED_BY_DEFAULT" }
      }
    ]
  }
  ```
- ⚠️ `config.toml` 中 `[marketplaces.local]` 引用 marketplace 源：
  ```toml
  [marketplaces.local]
  last_updated = "..."
  source_type = "local"
  source = "/home/USER"
  ```
- ❓ **未确认**：通过 marketplace 安装的插件是否一定同步到 `config.toml` 的 `[plugins]` 中？如果不同步，现有代码会漏掉 marketplace 插件。

**影响评估**：中。需要本地验证 `config.toml` 的 `[plugins]` 是否包含所有已安装插件（包括 marketplace 安装的）。

**建议行动**：
1. 检查本地 `~/.codex/config.toml` 中 `[plugins]` 的完整性
2. 如果 marketplace 插件未同步到 `[plugins]`，需要改为读取 marketplace.json + config.toml 合并

---

### 2.3 Hermes — ⚠️ 重大变化

**现有实现**：
- 扫描 `~/.hermes/plugins/` 获取已安装列表
- 读取 `config.yaml` 的 `plugins.disabled` 数组
- 逻辑：**不在 disabled 中 = 启用**

**调研发现（官方文档 2026-04）**：
- ❌ **默认行为已变**：新插件默认是 `not enabled`（未启用），不会加载
- ❌ **新增启用列表**：`plugins.enabled` 数组控制哪些插件被加载
- ✅ `plugins.disabled` 仍然存在，且 `disabled` 优先于 `enabled`
- 三种状态：
  | 状态 | 含义 | `enabled` | `disabled` |
  |---|---|---|---|
  | `enabled` | 已加载 | Yes | No |
  | `disabled` | 显式关闭 | (irrelevant) | Yes |
  | `not enabled` | 发现但未启用 | No | No |

**现有代码的错误逻辑**：
```rust
// 错误：假设不在 disabled 中就是启用的
enabled: !disabled.contains(name)
```

**正确的逻辑**：
```rust
let enabled_list = ... // plugins.enabled 数组
let disabled_list = ... // plugins.disabled 数组

// 插件启用条件：在 enabled 列表中 且 不在 disabled 列表中
let enabled = enabled_list.contains(name) && !disabled_list.contains(name);
```

**注意**：
- 插件功能尚未上线，无需考虑 grandfather/迁移兼容
- Hermes 新安装的插件默认是 `not enabled`，CC Switch 应如实反映这一状态

**影响评估**：高。现有代码会错误地把所有未显式禁用的插件显示为"启用"，包括那些实际上从未被启用的插件。

---

### 2.4 OpenClaw

**现有实现**：
- 通过 `openclaw plugins list --json` 获取已安装列表（实际是从 `openclaw.json` 的 `plugins.entries` 读取）
- 管理 `plugins.entries.<id>.enabled`
- **不碰** `plugins.allow` / `plugins.deny`

**调研发现**：
- ✅ `plugins.entries.<id>.enabled` 格式确认正确
- ✅ `plugins.allow` 是白名单，`plugins.deny` 是黑名单，deny 优先
- ⚠️ **新增发现**：`plugins.enabled` 是全局主开关（默认 true），设为 false 时跳过所有插件发现/加载
- ⚠️ **新增发现**：安装元数据存储在 `plugins/installs.json`（机器管理的状态文件），不是用户配置
- ⚠️ **新增发现**：`openclaw plugins list --json` 可以获取机器可读的完整清单（含安装来源、版本等）
- ⚠️ **新增发现**：Workspace-origin 插件默认禁用，必须显式启用

**影响评估**：低。现有代码的范围（只管理 `plugins.entries.<id>.enabled`）仍然合理。但可以考虑增强：
1. 读取 `plugins/installs.json` 获取更完整的插件元数据（版本、来源等）
2. 在 UI 中提示 `plugins.enabled` 全局开关的状态

---

### 2.5 OpenCode

**现有实现**：
- 模式 B：增删排序
- 直接管理 `opencode.json` 的 `plugin` 字符串数组

**调研发现**：
- ✅ 未发现明显变化
- ✅ 模式 B 仍然适用

**影响评估**：无。

---

## 3. 待验证事项

| # | 问题 | 验证方法 |
|---|---|---|
| 1 | Codex marketplace 插件是否同步到 `config.toml` 的 `[plugins]`？ | 检查 `~/.codex/config.toml` 和 `~/.agents/plugins/marketplace.json` |
| 2 | Hermes 用户本地的 `config.yaml` 是否已有 `plugins.enabled`？ | `cat ~/.hermes/config.yaml \| grep -A 10 plugins` |
| 3 | OpenClaw `plugins/installs.json` 的格式是什么？ | `cat ~/.openclaw/plugins/installs.json` |
