# Plugin Management v3 设计方案

> 状态：已完成 | 范围：Claude / Codex / Gemini / OpenCode / OpenClaw / Hermes
>
> 本文档基于对 6 个 Provider 官方文档和配置格式的完整调研，重新设计 PluginManager 的架构和 UI。

---

## 1. 调研结论：6 个 Provider 的 Plugin 配置总览

| Provider | 有插件？ | 配置文件 | 配置字段 | 安装方式 |
|---|---|---|---|---|
| **Claude** | ✅ | `~/.claude/settings.json` | `enabledPlugins: {id: bool}` | `claude plugin install` |
| **Codex** | ✅ (2026.3 新增) | `~/.codex/config.toml` | `[plugins."id"] enabled = false` | `codex /plugins` |
| **Gemini** | ✅ (叫 extensions) | `~/.gemini/extensions/` + `extension-enablement.json` | per-extension enabled 状态 | `gemini extensions install <url>` |
| **OpenCode** | ✅ | `~/.config/opencode/opencode.json` | `plugin: ["id1", "id2"]` | `npm install` 或写配置 |
| **OpenClaw** | ✅ | `~/.openclaw/openclaw.json` | `plugins.allow`, `plugins.entries` | `openclaw plugins install <pkg>` |
| **Hermes** | ✅ | `~/.hermes/config.yaml` | `plugins.disabled: []` | `hermes plugins install <git>` |

### 关键发现

**数据所有权**：只有 Claude 和 OpenCode 的插件配置是"CC Switch 可以安全修改"的。其他 Provider（Codex/Gemini/OpenClaw/Hermes）的插件系统较新或实验性，CC Switch 修改其配置的风险较高。

**标准化程度**：6 个 Provider 的插件格式、ID 体系、安装命令**完全不兼容**。不存在像 MCP Protocol 或 SKILL.md 那样的跨 Provider 标准。

---

## 2. Claude 当前 UI 模式分析

当前 `ClaudePluginTab` 的 UI 结构：

```
┌─────────────────────────────────────────────┐
│ [Description: 以下插件对所有 Claude 供应商默认启用]  │
│                                             │
│ 已安装插件 (N)              [✓ Apply] [↓ 导入] │
│ ┌─────────────────────────────────────────┐ │
│ │ ☑ plugin-id        v1.0 · scope         │ │
│ │ ☐ plugin-id        v2.0 · scope         │ │
│ └─────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
```

**核心交互**：
1. 读取 Provider 的安装注册表 → 渲染"已安装"列表
2. 每个插件一个 Checkbox（启用 / 禁用）
3. 用户勾选后点击 Apply → 批量写入 Provider 配置
4. 导入按钮：从现有配置导入到 CC Switch 管理

**这个模式隐含的前提**：
- Provider 有一个"已安装插件注册表"（和"配置"分离）
- 插件的启用状态是布尔值（开/关）
- 安装和启用是两个独立操作

---

## 3. Claude UI 模式对其他 Provider 的适配度

### 3.1 Codex — ✅ 高度适配

| 维度 | 适配情况 |
|---|---|
| 安装注册表 | Codex 插件安装在 `~/.codex/plugins/`，配置在 `config.toml` |
| 开关模式 | TOML `[plugins."id"] enabled = false` → 布尔开关 |
| 配置结构 | 每个插件是一个 table，可能有 `config` 子字段 |
| 结论 | **模式 A 适用**，已完整实现 |

### 3.2 Hermes — ✅ 高度适配

| 维度 | 适配情况 |
|---|---|
| 安装注册表 | `~/.hermes/plugins/` 目录；`hermes plugins list` 可列出 |
| 开关模式 | `plugins.disabled: []` → **逻辑反转**：默认全启用，加入列表 = 禁用 |
| 配置结构 | 简单数组 |
| 结论 | **模式 A 适用**，Checkbox 逻辑反转：默认勾选，取消勾选 = 加入 disabled。已完整实现 |

### 3.3 OpenClaw — ✅ 高度适配

| 维度 | 适配情况 |
|---|---|
| 安装注册表 | `openclaw plugins list --json` 可以获取已安装列表 |
| 开关模式 | `plugins.entries.id.enabled` → 布尔开关 |
| 配置结构 | 双层：`plugins.allow`（白名单数组）+ `plugins.entries`（配置对象） |
| 额外复杂性 | 插件有独立配置（如 `config.apiKey`）；`plugins.allow` 是加载白名单 |
| 结论 | **模式 A 基本适用**，已完整实现，只管理 `plugins.entries.id.enabled` |

### 3.4 Gemini — ✅ 高度适配（已重新评估）

| 维度 | 适配情况 |
|---|---|
| 安装注册表 | `~/.gemini/extensions/` 目录，每个扩展含 `gemini-extension.json` |
| 开关模式 | `~/.gemini/extensions/extension-enablement.json` 存储 per-extension 启用状态 |
| 列出命令 | `gemini extensions list --output-format=json`（v0.27.0+） |
| 管理命令 | `gemini extensions enable/disable <name> --scope user` |
| 配置结构 | 扩展独立目录 + 集中式 enablement JSON |
| 结论 | **模式 A 适用**，与 Claude 语义最接近 |

> **调研更新**：Gemini extensions 在 v0.27.0+ 已经成熟，不再是实验性功能。`extension-enablement.json` 提供了 per-extension 的启用状态管理，完全支持 Checkbox + Apply 模式。

### 3.5 OpenCode — ❌ 不适配

| 维度 | 适配情况 |
|---|---|
| 安装注册表 | ❌ 没有。`plugin` 数组本身既是安装列表又是启用列表 |
| 开关模式 | 不是布尔开关。在数组里 = 启用，不在 = 禁用 |
| 配置结构 | 简单字符串数组，有序 |
| 结论 | **需要模式 B**：配置数组 + 添加/删除/排序，不能用 Checkbox 列表。已完整实现 |

---

## 4. 两种 UI 模式

基于以上分析，PluginManager 需要支持**两种 UI 模式**：

### 模式 A：已安装列表 + Checkbox（Install Registry + Toggle）

适用 Provider：**Claude、Codex、Hermes、OpenClaw、Gemini**

```
┌─────────────────────────────────────────────┐
│ [Description]                                │
│                                             │
│ 已安装插件 (N)              [✓ Apply] [↓ 导入] │
│ ┌─────────────────────────────────────────┐ │
│ │ ☑ plugin-id        v1.0 · scope    [⚙]  │ │
│ │ ☐ plugin-id        v2.0 · scope    [⚙]  │ │
│ └─────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
```

**特点**：
- 列表来源 = Provider 的安装注册表
- Checkbox = 启用 / 禁用
- Apply = 批量同步启用状态到 Provider 配置
- 可选：齿轮图标打开插件独立配置（如 OpenClaw 的 `config.apiKey`）

### 模式 B：配置数组 + 增删排序（Config Array + CRUD）

适用 Provider：**OpenCode**

```
┌─────────────────────────────────────────────┐
│ [Description]                                │
│                                             │
│ [输入框 + 添加按钮]              [↓ 导入]     │
│ ┌─────────────────────────────────────────┐ │
│ │ oh-my-openagent                    ↑↓ ✕  │ │
│ │ another-plugin                     ↑↓ ✕  │ │
│ └─────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
```

**特点**：
- 列表来源 = CC Switch 数据库表（或 Provider 配置本身）
- 每个条目可删除、上下排序
- 添加 = 输入 ID 并插入
- 无"Apply"按钮，每个操作即时生效

---

## 5. 通用 UI 框架建议

虽然底层模式不同，但**Tab 级 UI 可以统一**：

```
┌──────────────────────────────────────────────────────┐
│ 插件管理                                              │
├────────┬────────┬──────────┬──────────┬──────────────┤
│ Claude │ Codex  │ OpenCode │ OpenClaw │ Hermes │ Gemini │
├────────┴────────┴──────────┴──────────┴──────────────┤
│                                                      │
│ [Provider-specific Description]                      │
│                                                      │
│ [模式 A: 已安装列表 + Checkbox + Apply]              │
│ [模式 B: 输入框 + 列表 + 增删排序]                    │
│                                                      │
└──────────────────────────────────────────────────────┘
```

**每个 Tab 的配置**：

| Tab | 模式 | 数据源 | 操作 |
|---|---|---|---|
| Claude | A | `installed_plugins.json` | 勾选 → Apply → `settings.json` |
| Codex | A | `~/.codex/plugins/` + `config.toml` | 勾选 → Apply → `config.toml` |
| OpenCode | B | `opencode_plugins` 表 | 增删排序 → `opencode.json` |
| OpenClaw | A | `openclaw plugins list --json` | 勾选 → Apply → `openclaw.json` |
| Hermes | A | `~/.hermes/plugins/` | 勾选 → Apply → `config.yaml` |
| Gemini | A | `~/.gemini/extensions/` + `extension-enablement.json` | 勾选 → Apply → `extension-enablement.json` |

---

## 6. 实现计划（分阶段）

### Phase 1：Claude（✅ 已完成）
- [x] 读取 `installed_plugins.json`
- [x] Checkbox 批量启用/禁用
- [x] 导入功能（从 `settings.json` 导入，以 installed 为准过滤 ghost）
- [x] 验证端到端

### Phase 2：OpenCode（✅ 已完成）
- [x] 模式 B UI：增删排序
- [x] 导入后同步到 `opencode.json`
- [x] OMO 互斥校验
- [x] 前端 description 文案展示

### Phase 3：Hermes（✅ 已完成）
- [x] 后端：读取 `~/.hermes/plugins/` + `config.yaml` 的 `plugins.disabled`
- [x] 后端：写入 `plugins.disabled` 数组
- [x] 前端：模式 A UI，Checkbox 逻辑反转（UI 统一为"勾选 = 启用"）

### Phase 4：OpenClaw（✅ 已完成）
- [x] 后端：读取已安装列表 + `plugins.allow` + `plugins.entries`
- [x] 后端：写入启用状态（更新 `plugins.entries.id.enabled`）
- [x] 前端：`OpenClawPluginTab` — 模式 A UI

### Phase 5：Codex（✅ 已完成）
- [x] 后端：读取 `config.toml` 的 `[plugins]`
- [x] 后端：写入 `enabled = false`
- [x] 前端：`CodexPluginTab` — 模式 A UI

### Phase 6：Gemini（✅ 已完成）

#### 调研结论

Gemini extensions 系统已成熟（v0.27.0+），具备完整的 per-extension 启用/禁用能力：

| 维度 | 详情 |
|---|---|
| 安装目录 | `~/.gemini/extensions/<name>/` |
| Manifest | `gemini-extension.json`（含 `name`, `version`, `description`） |
| 启用状态 | `~/.gemini/extensions/extension-enablement.json` |
| 列出命令 | `gemini extensions list --output-format=json` |
| 开关命令 | `gemini extensions enable/disable <name> --scope user` |

**适配模式 A**，与 Claude 语义最接近。

#### 后端任务

- [ ] `src-tauri/src/commands/gemini_plugins.rs`
  - [ ] `get_gemini_installed_plugins()` — 扫描 `~/.gemini/extensions/` 目录读取各 `gemini-extension.json`，合并 `extension-enablement.json` 的启用状态
  - [ ] `apply_gemini_plugin_selection(enabledIds)` — 写入 `extension-enablement.json`
  - [ ] `import_gemini_plugins_from_live()` — 从现有 `extension-enablement.json` 导入当前启用状态
- [ ] `src-tauri/src/lib.rs` — 注册 Tauri 命令

#### 前端任务

- [ ] `src/lib/api/geminiPlugins.ts` — API 封装（`getInstalled`, `applySelection`, `importFromLive`）
- [ ] `src/hooks/useGeminiPlugins.ts` — React Query hooks
- [ ] `src/components/plugins/GeminiPluginTab.tsx` — 模式 A UI（复用 `CheckboxPluginTab`）
- [ ] `src/components/plugins/PluginManager.tsx` — 添加 "Gemini" Tab

#### i18n 任务

- [ ] `src/i18n/locales/zh.json` — 补充 `plugins.gemini.*` 键值
- [ ] `src/i18n/locales/en.json` — 补充 `plugins.gemini.*` 键值
- [ ] `src/i18n/locales/ja.json` — 补充 `plugins.gemini.*` 键值

#### 技术细节

**`extension-enablement.json` 格式推测**（需实现时验证）：

```json
{
  "extension-name-1": true,
  "extension-name-2": false
}
```

或类似结构。实现时应优先尝试读取实际文件格式，如不存在则默认所有已安装扩展为启用状态。

**数据模型**：

```rust
pub struct GeminiInstalledPlugin {
    pub id: String,           // 扩展目录名 / gemini-extension.json 中的 name
    pub name: String,         // gemini-extension.json 中的 name
    pub version: String,      // gemini-extension.json 中的 version
    pub description: Option<String>, // gemini-extension.json 中的 description
    pub enabled: bool,        // extension-enablement.json 中的状态
}
```

---

## 7. 讨论结论（已确定）

| # | 问题 | 结论 |
|---|---|---|
| 1 | OpenCode 是否适配模式 A？ | **保持模式 B**。OpenCode 没有安装注册表，`plugin` 数组既是安装列表又是启用列表。已完整实现。 |
| 2 | Hermes Checkbox 逻辑反转是否困惑？ | **UI 统一为"勾选 = 启用"**。Hermes 后端自动转换：未勾选 → 加入 `plugins.disabled`。用户无感知。已完整实现。 |
| 3 | OpenClaw `plugins.allow` 是否管理？ | **Phase 1 不碰 `allow`**。只管理 `plugins.entries.id.enabled`，assume `allow` 包含所有已安装插件。已完整实现。 |
| 4 | 是否需要跨 Provider 同步？ | **不自动同步**。ID 不兼容、安装不共享、用户可能有 Provider 偏好。 |
| 5 | Gemini 是否纳入 v3？ | **✅ 纳入**。Extensions 已成熟（v0.27.0+），有 `extension-enablement.json` 管理 per-extension 启用状态，适配模式 A。 |

---

## 8. 待处理事项（TODO）

### Gemini（Phase 6，模式 A，已完成）

所有任务已完成。见提交 `732b82d3`。

### Hermes — P0 修复（✅ 已完成）

**问题**：Hermes 已从"默认全部启用"变为 opt-in 模式（config schema v21+）。现有代码假设"不在 `plugins.disabled` 中 = 启用"是错误的。

**修复**：
- [x] `get_hermes_installed_plugins` — 同时读取 `plugins.enabled` 和 `plugins.disabled`，启用条件 = `enabled.contains(id) && !disabled.contains(id)`
- [x] `apply_hermes_plugin_selection` — 更新 `plugins.enabled` 数组
- [x] `import_hermes_plugins_from_live` — 从 `plugins.enabled` 读取
- [x] i18n 更新 `plugins.hermes.description` 文案

**调研文档**：`docs/provider-plugin-research-2026-04.md` §2.3
**修复计划**：`docs/provider-plugin-fix-plan.md` §P0

### Codex — P1 验证（中优先级）

**问题**：Codex 插件可能通过 `~/.agents/plugins/marketplace.json` 管理，`config.toml` 的 `[plugins]` 可能不完整。

**行动**：
- [ ] 用户本地验证 `~/.agents/plugins/marketplace.json` 与 `~/.codex/config.toml` 的同步关系
- [ ] 如需修复，修改 `get_codex_installed_plugins` 合并 marketplace.json

**调研文档**：`docs/provider-plugin-research-2026-04.md` §2.2
**修复计划**：`docs/provider-plugin-fix-plan.md` §P1

### OpenClaw — P2 增强（低优先级，可选）

**发现**：`plugins/installs.json` 存储安装元数据（版本、来源），`plugins.enabled` 是全局主开关。

**可选增强**：
- [ ] 读取 `installs.json` 获取版本号展示
- [ ] 全局开关关闭时显示警告

**调研文档**：`docs/provider-plugin-research-2026-04.md` §2.4
**修复计划**：`docs/provider-plugin-fix-plan.md` §P2
