import { invoke } from "@tauri-apps/api/core";

export interface HermesInstalledPlugin {
  id: string;
  enabled: boolean;
}

export const hermesPluginsApi = {
  async getInstalled(): Promise<HermesInstalledPlugin[]> {
    return await invoke("get_hermes_installed_plugins");
  },

  async applySelection(enabledIds: string[]): Promise<void> {
    return await invoke("apply_hermes_plugin_selection", { enabledIds });
  },

  async importFromLive(): Promise<string[]> {
    return await invoke("import_hermes_plugins_from_live");
  },
};
