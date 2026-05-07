import { useState, useMemo, useCallback } from "react";
import type { UseMutationResult } from "@tanstack/react-query";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";
import { extractErrorMessage } from "@/utils/errorUtils";

interface PluginItem {
  id: string;
}

interface UseCheckboxPluginTabOptions {
  installedPlugins: PluginItem[] | undefined;
  isLoading: boolean;
  initialEnabledIds: Set<string> | undefined;
  applyMutation: UseMutationResult<void, unknown, string[], unknown>;
  importMutation: UseMutationResult<string[], unknown, void, unknown>;
}

export function useCheckboxPluginTab(options: UseCheckboxPluginTabOptions) {
  const { t } = useTranslation();
  const {
    installedPlugins,
    isLoading,
    initialEnabledIds,
    applyMutation,
    importMutation,
  } = options;

  const [pendingSelection, setPendingSelection] = useState<Set<string>>(
    new Set(),
  );

  const selectedIds = useMemo(() => {
    if (pendingSelection.size > 0) return pendingSelection;
    return initialEnabledIds ?? new Set<string>();
  }, [pendingSelection, initialEnabledIds]);

  const handleToggle = useCallback(
    (pluginId: string, checked: boolean) => {
      setPendingSelection((prev) => {
        const next = new Set(prev);
        if (next.size === 0 && initialEnabledIds) {
          initialEnabledIds.forEach((id) => next.add(id));
        }
        if (checked) {
          next.add(pluginId);
        } else {
          next.delete(pluginId);
        }
        return next;
      });
    },
    [initialEnabledIds],
  );

  const hasChanges = useMemo(() => {
    if (pendingSelection.size > 0) return true;
    if (!installedPlugins || !initialEnabledIds) return false;
    return installedPlugins.some((p) => {
      const initiallyEnabled = initialEnabledIds.has(p.id);
      const currentlySelected = selectedIds.has(p.id);
      return initiallyEnabled !== currentlySelected;
    });
  }, [pendingSelection, installedPlugins, initialEnabledIds, selectedIds]);

  const handleApply = useCallback(async () => {
    try {
      const ids = Array.from(selectedIds);
      await applyMutation.mutateAsync(ids);
      setPendingSelection(new Set());
      toast.success(t("common.success"));
    } catch (error) {
      toast.error(t("plugins.applyFailed"), {
        description: extractErrorMessage(error),
      });
    }
  }, [selectedIds, applyMutation, t]);

  const handleImport = useCallback(async () => {
    try {
      const imported = await importMutation.mutateAsync();
      toast.success(t("plugins.importSuccess", { count: imported.length }));
    } catch (error) {
      toast.error(t("plugins.importFailed"), {
        description: extractErrorMessage(error),
      });
    }
  }, [importMutation, t]);

  return {
    selectedIds,
    hasChanges,
    isLoading,
    isApplying: applyMutation.isPending,
    isImporting: importMutation.isPending,
    handleToggle,
    handleApply,
    handleImport,
  };
}
