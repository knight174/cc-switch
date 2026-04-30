import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { hermesPluginsApi } from "@/lib/api/hermesPlugins";

export function useHermesInstalledPlugins() {
  return useQuery({
    queryKey: ["hermesPlugins", "installed"],
    queryFn: () => hermesPluginsApi.getInstalled(),
  });
}

export function useApplyHermesPluginSelection() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (enabledIds: string[]) =>
      hermesPluginsApi.applySelection(enabledIds),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["hermesPlugins", "installed"] });
    },
  });
}

export function useImportHermesPlugins() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => hermesPluginsApi.importFromLive(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["hermesPlugins", "installed"] });
    },
  });
}
