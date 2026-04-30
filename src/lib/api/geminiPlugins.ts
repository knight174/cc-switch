import { invoke } from "@tauri-apps/api/core";

export interface GeminiInstalledPlugin {
  id: string;
  name: string;
  version: string;
  description?: string;
  enabled: boolean;
}

export const geminiPluginsApi = {
  async getInstalled(): Promise<GeminiInstalledPlugin[]> {
    return await invoke("get_gemini_installed_plugins");
  },

  async applySelection(enabledIds: string[]): Promise<void> {
    return await invoke("apply_gemini_plugin_selection", { enabledIds });
  },

  async importFromLive(): Promise<string[]> {
    return await invoke("import_gemini_plugins_from_live");
  },
};
