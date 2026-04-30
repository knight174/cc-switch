import { invoke } from "@tauri-apps/api/core";

export interface CodexInstalledPlugin {
  id: string;
  enabled: boolean;
}

export const codexPluginsApi = {
  async getInstalled(): Promise<CodexInstalledPlugin[]> {
    return await invoke("get_codex_installed_plugins");
  },

  async applySelection(enabledIds: string[]): Promise<void> {
    return await invoke("apply_codex_plugin_selection", { enabledIds });
  },

  async importFromLive(): Promise<string[]> {
    return await invoke("import_codex_plugins_from_live");
  },
};
