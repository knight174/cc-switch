import {
  useMutation,
  useQuery,
  useQueryClient,
  keepPreviousData,
} from "@tanstack/react-query";
import {
  claudeMarketplaceApi,
  type ClaudeMarketplaceListOutput,
} from "@/lib/api/claudeMarketplace";

export function useClaudeMarketplaceList() {
  return useQuery({
    queryKey: ["claudePlugins", "marketplace"],
    queryFn: () => claudeMarketplaceApi.getAll(),
    staleTime: Infinity,
    placeholderData: keepPreviousData,
  });
}

export function useInstallClaudePlugin() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (pluginId: string) => claudeMarketplaceApi.install(pluginId),
    onSuccess: (_data, pluginId) => {
      queryClient.setQueryData<ClaudeMarketplaceListOutput>(
        ["claudePlugins", "marketplace"],
        (old) => {
          if (!old) return old;
          const plugin = old.available.find((p) => p.pluginId === pluginId);
          return {
            ...old,
            installed: [
              ...old.installed,
              {
                id: pluginId,
                version: "",
                scope: "user",
                enabled: true,
                installPath: "",
                installedAt: new Date().toISOString(),
                lastUpdated: new Date().toISOString(),
              },
            ],
            available: plugin
              ? old.available
              : old.available.filter((p) => p.pluginId !== pluginId),
          };
        },
      );
      queryClient.invalidateQueries({ queryKey: ["claudePlugins", "all"] });
      queryClient.invalidateQueries({
        queryKey: ["claudePlugins", "installed"],
      });
    },
  });
}

export function useUninstallClaudePlugin() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (pluginId: string) => claudeMarketplaceApi.uninstall(pluginId),
    onSuccess: (_data, pluginId) => {
      queryClient.setQueryData<ClaudeMarketplaceListOutput>(
        ["claudePlugins", "marketplace"],
        (old) => {
          if (!old) return old;
          return {
            ...old,
            installed: old.installed.filter((p) => p.id !== pluginId),
          };
        },
      );
      queryClient.invalidateQueries({ queryKey: ["claudePlugins", "all"] });
      queryClient.invalidateQueries({
        queryKey: ["claudePlugins", "installed"],
      });
    },
  });
}

export function useUpdateClaudePlugin() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (pluginId: string) => claudeMarketplaceApi.update(pluginId),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["claudePlugins", "marketplace"],
      });
      queryClient.invalidateQueries({
        queryKey: ["claudePlugins", "installed"],
      });
      queryClient.invalidateQueries({
        queryKey: ["claudePlugins", "updates"],
      });
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

export function useCheckClaudePluginUpdates() {
  return useQuery({
    queryKey: ["claudePlugins", "updates"],
    queryFn: () => claudeMarketplaceApi.checkUpdates(),
    enabled: false,
  });
}
