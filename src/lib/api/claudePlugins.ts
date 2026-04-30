import { invoke } from "@tauri-apps/api/core";

export interface ClaudeInstalledPlugin {
  id: string;
  version: string;
  scope: string;
  installed_at: string;
}

export const claudePluginsApi = {
  async getAll(): Promise<Record<string, boolean>> {
    return await invoke("get_claude_global_plugins");
  },

  async getInstalled(): Promise<ClaudeInstalledPlugin[]> {
    return await invoke("get_claude_installed_plugins");
  },

  async set(pluginId: string, enabled: boolean): Promise<void> {
    return await invoke("set_claude_global_plugin", { pluginId, enabled });
  },

  async remove(pluginId: string): Promise<void> {
    return await invoke("remove_claude_global_plugin", { pluginId });
  },

  async applySelection(enabledIds: string[]): Promise<void> {
    return await invoke("apply_claude_plugin_selection", { enabledIds });
  },

  async importFromLive(): Promise<string[]> {
    return await invoke("import_claude_plugins_from_live");
  },
};
