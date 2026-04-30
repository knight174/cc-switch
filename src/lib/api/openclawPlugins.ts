import { invoke } from "@tauri-apps/api/core";

export interface OpenClawInstalledPlugin {
  id: string;
  enabled: boolean;
}

export const openclawPluginsApi = {
  async getInstalled(): Promise<OpenClawInstalledPlugin[]> {
    return await invoke("get_openclaw_installed_plugins");
  },

  async applySelection(enabledIds: string[]): Promise<void> {
    return await invoke("apply_openclaw_plugin_selection", { enabledIds });
  },

  async importFromLive(): Promise<string[]> {
    return await invoke("import_openclaw_plugins_from_live");
  },
};
