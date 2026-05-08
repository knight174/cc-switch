import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { geminiPluginsApi } from "@/lib/api/geminiPlugins";

export function useGeminiInstalledPlugins() {
  return useQuery({
    queryKey: ["geminiPlugins", "installed"],
    queryFn: () => geminiPluginsApi.getInstalled(),
  });
}

export function useApplyGeminiPluginSelection() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (enabledIds: string[]) =>
      geminiPluginsApi.applySelection(enabledIds),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["geminiPlugins", "installed"],
      });
    },
  });
}

export function useImportGeminiPlugins() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => geminiPluginsApi.importFromLive(),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["geminiPlugins", "installed"],
      });
    },
  });
}
