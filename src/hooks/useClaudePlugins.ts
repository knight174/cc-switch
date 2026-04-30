import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { claudePluginsApi } from "@/lib/api/claudePlugins";

export function useClaudePlugins() {
  return useQuery({
    queryKey: ["claudePlugins", "all"],
    queryFn: () => claudePluginsApi.getAll(),
  });
}

export function useClaudeInstalledPlugins() {
  return useQuery({
    queryKey: ["claudePlugins", "installed"],
    queryFn: () => claudePluginsApi.getInstalled(),
  });
}

export function useSetClaudePlugin() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({
      pluginId,
      enabled,
    }: {
      pluginId: string;
      enabled: boolean;
    }) => claudePluginsApi.set(pluginId, enabled),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["claudePlugins", "all"] });
    },
  });
}

export function useRemoveClaudePlugin() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (pluginId: string) => claudePluginsApi.remove(pluginId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["claudePlugins", "all"] });
    },
  });
}

export function useApplyClaudePluginSelection() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (enabledIds: string[]) =>
      claudePluginsApi.applySelection(enabledIds),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["claudePlugins", "all"] });
    },
  });
}

export function useImportClaudePlugins() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => claudePluginsApi.importFromLive(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["claudePlugins", "all"] });
    },
  });
}
