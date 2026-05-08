import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { opencodePluginsApi } from "@/lib/api/opencodePlugins";

export function useOpencodePlugins() {
  return useQuery({
    queryKey: ["opencodePlugins", "all"],
    queryFn: () => opencodePluginsApi.getAll(),
  });
}

export function useAddOpencodePlugin() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (name: string) => opencodePluginsApi.add(name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["opencodePlugins", "all"] });
    },
  });
}

export function useRemoveOpencodePlugin() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (normalizedName: string) =>
      opencodePluginsApi.remove(normalizedName),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["opencodePlugins", "all"] });
    },
  });
}

export function useReorderOpencodePlugins() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (names: string[]) => opencodePluginsApi.reorder(names),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["opencodePlugins", "all"] });
    },
  });
}

export function useImportOpencodePlugins() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => opencodePluginsApi.importFromLive(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["opencodePlugins", "all"] });
    },
  });
}
