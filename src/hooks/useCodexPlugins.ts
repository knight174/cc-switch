import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { codexPluginsApi } from "@/lib/api/codexPlugins";

export function useCodexInstalledPlugins() {
  return useQuery({
    queryKey: ["codexPlugins", "installed"],
    queryFn: () => codexPluginsApi.getInstalled(),
  });
}

export function useApplyCodexPluginSelection() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (enabledIds: string[]) =>
      codexPluginsApi.applySelection(enabledIds),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["codexPlugins", "installed"] });
    },
  });
}

export function useImportCodexPlugins() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => codexPluginsApi.importFromLive(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["codexPlugins", "installed"] });
    },
  });
}
