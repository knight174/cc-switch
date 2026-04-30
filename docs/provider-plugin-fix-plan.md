# Provider 插件系统修复计划

> 基于 `docs/provider-plugin-research-2026-04.md` 调研结论

---

## P0：Hermes — 修复 opt-in 语义（✅ 已完成）

### 问题

Hermes 已从"默认全部启用"变为"opt-in"模式。现有代码把不在 `plugins.disabled` 中的插件都显示为"启用"，这是**错误**的。

### 修复内容

**后端 `src-tauri/src/commands/hermes_plugins.rs`**：

1. `get_hermes_installed_plugins()`：
   - 同时读取 `plugins.enabled` 和 `plugins.disabled`
   - 插件启用条件：**在 enabled 列表中 且 不在 disabled 列表中**
   - 默认未启用：不在任一列表中的插件 = `not enabled`

2. `apply_hermes_plugin_selection(enabledIds)`：
   - 更新 `plugins.enabled` 数组（勾选 = 加入 enabled）
   - 同时更新 `plugins.disabled` 数组（取消勾选 = 加入 disabled）

3. `import_hermes_plugins_from_live()`：
   - 从 `plugins.enabled` 读取已启用列表

**前端 `src/components/plugins/HermesPluginTab.tsx`**：
- 无需改动，CheckboxPluginTab 模式仍然适用

**i18n**：
- 更新 `plugins.hermes.description` 文案，说明 opt-in 语义

### 验证
- 安装新插件 → 默认显示为"未勾选"
- 勾选 → `config.yaml` 中出现 `plugins.enabled` 条目
- 取消勾选 → 条目进入 `plugins.disabled`

---

## P1：Codex — 确认 marketplace 同步（中优先级）

### 问题

Codex 插件可能通过 `~/.agents/plugins/marketplace.json` 管理，`config.toml` 的 `[plugins]` 可能不完整。

### 行动

1. **用户本地验证**：
   ```bash
   ls ~/.agents/plugins/
   cat ~/.codex/config.toml | grep -A 50 "\[plugins"
   ```

2. **如果确认 marketplace 插件未同步到 `[plugins]`**：
   - 修改 `get_codex_installed_plugins()` 同时读取 `config.toml` 的 `[plugins]` 和 `~/.agents/plugins/marketplace.json`
   - 合并两者去重后返回完整列表

3. **如果确认 marketplace 插件已同步**：
   - 无需改动，关闭此任务

---

## P2：OpenClaw — 补充安装元数据（低优先级）

### 问题

现有代码只读取 `plugins.entries` 的启用状态，没有利用 `plugins/installs.json` 中的安装元数据。

### 行动（可选增强）

1. **读取 `plugins/installs.json`**：
   - 获取插件版本、来源（GitHub/npm/本地）、安装时间
   - 在 UI 中展示版本号（类似 GeminiPluginTab 的 `v{version}`）

2. **提示全局开关状态**：
   - 如果 `plugins.enabled = false`，在 Tab 顶部显示警告："OpenClaw 全局插件开关已关闭，勾选不会生效"

---

## P3：Claude — 无需改动（确认）

### 结论

实现正确。Project scope 和 local scope 的插件管理不在 CC Switch 范围内，这是合理的设计边界。

---

## 实施顺序

```
P0: Hermes 修复（立即）
  → P1: Codex 验证（用户确认后）
    → P2: OpenClaw 增强（可选）
```

---

## 文件变更清单

### Hermes 修复
- [ ] `src-tauri/src/commands/hermes_plugins.rs`
- [ ] `src/i18n/locales/zh.json` — 更新 `plugins.hermes.description`
- [ ] `src/i18n/locales/en.json` — 同上
- [ ] `src/i18n/locales/ja.json` — 同上

### Codex 验证（条件性）
- [ ] 用户本地验证 marketplace.json 与 config.toml 的同步关系
- [ ] `src-tauri/src/commands/codex_plugins.rs`（如需修复）

### OpenClaw 增强（可选）
- [ ] `src-tauri/src/commands/openclaw_plugins.rs`
- [ ] `src/components/plugins/OpenclawPluginTab.tsx`
