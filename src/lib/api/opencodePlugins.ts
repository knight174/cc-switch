import { invoke } from "@tauri-apps/api/core";

export interface OpenCodePluginEntry {
  normalized_name: string;
  display_name: string;
  created_at: number;
}

export const opencodePluginsApi = {
  async getAll(): Promise<OpenCodePluginEntry[]> {
    return await invoke("get_opencode_plugins");
  },

  async add(name: string): Promise<void> {
    return await invoke("add_opencode_plugin", { name });
  },

  async remove(normalizedName: string): Promise<void> {
    return await invoke("remove_opencode_plugin", { normalizedName });
  },

  async reorder(names: string[]): Promise<void> {
    return await invoke("reorder_opencode_plugins", { names });
  },

  async importFromLive(): Promise<string[]> {
    return await invoke("import_opencode_plugins_from_live");
  },
};
