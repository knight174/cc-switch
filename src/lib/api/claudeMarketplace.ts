import { invoke } from "@tauri-apps/api/core";

export interface ClaudeMarketplaceInstalled {
  id: string;
  version: string;
  scope: string;
  enabled: boolean;
  installPath: string;
  installedAt: string;
  lastUpdated: string;
}

export interface ClaudeMarketplacePlugin {
  pluginId: string;
  name: string;
  description: string;
  marketplaceName: string;
  installCount: number;
}

export interface ClaudePluginUpdateInfo {
  id: string;
  current_version: string;
  has_update: boolean;
}

export interface ClaudeMarketplaceListOutput {
  installed: ClaudeMarketplaceInstalled[];
  available: ClaudeMarketplacePlugin[];
}

export const claudeMarketplaceApi = {
  async getAll(): Promise<ClaudeMarketplaceListOutput> {
    return await invoke("get_claude_marketplace_plugins");
  },

  async install(pluginId: string): Promise<void> {
    return await invoke("install_claude_marketplace_plugin", { pluginId });
  },

  async uninstall(pluginId: string): Promise<void> {
    return await invoke("uninstall_claude_marketplace_plugin", { pluginId });
  },

  async update(pluginId: string): Promise<void> {
    return await invoke("update_claude_marketplace_plugin", { pluginId });
  },

  async refresh(): Promise<void> {
    return await invoke("refresh_claude_marketplace");
  },

  async checkUpdates(): Promise<ClaudePluginUpdateInfo[]> {
    return await invoke("check_claude_plugin_updates");
  },
};
