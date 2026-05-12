import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { claudeMarketplaceApi } from "@/lib/api/claudeMarketplace";

export function useClaudeMarketplaceList() {
  return useQuery({
    queryKey: ["claudePlugins", "marketplace"],
    queryFn: () => claudeMarketplaceApi.getAll(),
  });
}

export function useInstallClaudePlugin() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (pluginId: string) => claudeMarketplaceApi.install(pluginId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["claudePlugins"] });
    },
  });
}

export function useUninstallClaudePlugin() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (pluginId: string) => claudeMarketplaceApi.uninstall(pluginId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["claudePlugins"] });
    },
  });
}

export function useUpdateClaudePlugin() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (pluginId: string) => claudeMarketplaceApi.update(pluginId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["claudePlugins"] });
    },
  });
}

export function useRefreshClaudeMarketplace() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => claudeMarketplaceApi.refresh(),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["claudePlugins", "marketplace"],
      });
    },
  });
}
