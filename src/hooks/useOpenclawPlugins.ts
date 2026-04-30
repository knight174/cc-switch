import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { openclawPluginsApi } from "@/lib/api/openclawPlugins";

export function useOpenclawInstalledPlugins() {
  return useQuery({
    queryKey: ["openclawPlugins", "installed"],
    queryFn: () => openclawPluginsApi.getInstalled(),
  });
}

export function useApplyOpenclawPluginSelection() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (enabledIds: string[]) =>
      openclawPluginsApi.applySelection(enabledIds),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["openclawPlugins", "installed"] });
    },
  });
}

export function useImportOpenclawPlugins() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => openclawPluginsApi.importFromLive(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["openclawPlugins", "installed"] });
    },
  });
}
